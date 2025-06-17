//! The actor model is highly recommended for state management,
//! as it provides modularity and scalability.
//! This module demonstrates how to use actors
//! within the async system in Rust.
//! To build a solid app, do not communicate by sharing memory;
//! instead, share memory by communicating.

use std::{fs, path::PathBuf};

use super::{
    bing::BingRefresh,
    nasa::NasaRefresh,
    spotlight::{SpotlightRefresh, SpotlightReset},
};
use crate::signals::{ImageService, Refresh, Reset};
use crate::{
    common::check_err, notification_center::NotificationUpdate, signals::NotificationAlert,
};
use crate::{notification_center::NotificationActor, signals::NotificationSeverity};
use anyhow::{anyhow, Result};
use futures_util::{StreamExt, TryFutureExt};
use messages::prelude::{Actor, Address, Context as MsgContext};
use reqwest::{header::CONTENT_LENGTH, Client};
use rinf::{debug_print, DartSignal};
use size::Size;
use tokio::{fs::File, io::AsyncWriteExt, spawn};

/// The actor that holds the Service state and handles messages.
pub struct ImageServiceActor {
    pub(super) app_cache_dir: PathBuf,
    pub(super) notification_center: Address<NotificationActor>,
}

// This defines our `Actor` as an actor in the async system.
impl Actor for ImageServiceActor {}

impl ImageServiceActor {
    pub(super) const INFO_FILE: &str = "info.json";

    pub fn new(
        service_addr: Address<Self>,
        app_cache_dir: PathBuf,
        notification_center: Address<NotificationActor>,
    ) -> Self {
        spawn(Self::listen_to_refresh(service_addr.clone()));
        spawn(Self::listen_to_reset(service_addr));
        ImageServiceActor {
            app_cache_dir,
            notification_center,
        }
    }

    async fn refresh(service_addr: &mut Address<Self>, service: ImageService) {
        let result = match service {
            ImageService::Bing => service_addr.send(BingRefresh).await,
            ImageService::Nasa => service_addr.send(NasaRefresh).await,
            ImageService::Spotlight => service_addr.send(SpotlightRefresh).await,
        }
        .map_err(|e| anyhow!("Failed to refresh {} cache: {e:?}", service.as_str()));
        if let Ok(result) = check_err(result) {
            let _ = check_err(result);
        }
    }

    async fn listen_to_refresh(mut service_addr: Address<Self>) {
        // Spawn an asynchronous task to listen for signals from Dart.
        let receiver = Refresh::get_dart_signal_receiver();
        // Continuously listen for signals.
        while let Some(dart_signal) = receiver.recv().await {
            debug_print!("refreshing image list from Windows Spotlight");

            // Send refresh message to the actor.
            Self::refresh(&mut service_addr, dart_signal.message.0).await;
        }
    }

    async fn listen_to_reset(mut service_addr: Address<Self>) {
        // Spawn an asynchronous task to listen for signals from Dart.
        let receiver = Reset::get_dart_signal_receiver();
        // Continuously listen for signals.
        while let Some(dart_signal) = receiver.recv().await {
            debug_print!("resetting image list from Windows Spotlight");
            let service = dart_signal.message.0;

            // Send reset message to the actor.
            if matches!(service, ImageService::Spotlight) {
                let result = service_addr
                    .send(SpotlightReset)
                    .await
                    .map_err(|e| anyhow!("Failed to reset {} cache: {e:?}", service.as_str()));
                if let Ok(result) = check_err(result) {
                    let _ = check_err(result);
                }
            }

            // Send refresh message to the actor.
            Self::refresh(&mut service_addr, service).await;
        }
    }

    pub(super) async fn check_notify_send_error(
        &mut self,
        notification: NotificationAlert,
    ) -> Result<()> {
        if let Ok(result) = check_err(
            self.notification_center
                .send(NotificationUpdate(notification))
                .await
                .map_err(|e| anyhow!("Failed to send notification update: {e:?}")),
        ) {
            let _ = check_err(result);
        }
        Ok(())
    }

    pub(super) async fn download_image(
        &mut self,
        client: &Client,
        url: &str,
        cache_path: &str,
        display_id: &str,
        total_steps: u8,
        mut notification: NotificationAlert,
    ) -> Result<()> {
        let response = client.get(url).send().await?;
        let total_size = if let Some(v) = response.headers().get(CONTENT_LENGTH) {
            let as_str = v.to_str()?;
            Some(as_str.to_string().parse::<usize>()?)
        } else {
            debug_print!("Failed to get the content length for image file {display_id}");
            None
        };

        let completed_steps = notification.percent * (total_steps as f32);
        let mut buf: Vec<u8> = Vec::new();
        let mut downloaded = 0;
        let status = format!("Downloading {display_id}");
        let mut stream = response.bytes_stream();
        while let Some(chunk) = stream.next().await {
            let bytes =
                chunk.map_err(|e| anyhow!("Failed to get chunk of bytes from stream: {e:?}"))?;
            downloaded += bytes.len();
            buf.extend(bytes);
            if let Some(total_size) = total_size {
                notification.percent = (completed_steps
                    + ((downloaded as f32) / (total_size as f32)))
                    / (total_steps as f32);
            }
            notification.body = format!(
                "{status} ({})",
                Size::from_bytes(downloaded)
                    .format()
                    .with_base(size::Base::Base10)
            );
            self.check_notify_send_error(notification.clone()).await?;
        }
        check_err(
            check_err(
                File::create(cache_path)
                    .map_err(|e| {
                        anyhow!("Failed to create cache file for image {cache_path}: {e:?}")
                    })
                    .await,
            )?
            .write_all(&buf)
            .map_err(|e| anyhow!("Failed to write data to image file {cache_path}: {e:?}"))
            .await,
        )?;
        Ok(())
    }

    pub(super) async fn notify_err<T>(
        &mut self,
        result: Result<T>,
        service: ImageService,
    ) -> Result<T> {
        if let Err(e) = &result {
            self.check_notify_send_error(NotificationAlert {
                title: format!("{} images", service.as_str()),
                body: [e.to_string(), e.root_cause().to_string()].join("\n"),
                percent: 1.0,
                severity: NotificationSeverity::Error,
                status_message: String::new(),
            })
            .await?;
        }
        result
    }
}

// Creates and spawns the actors in the async system.
pub async fn create_actors(notification_center: Address<NotificationActor>) -> Result<()> {
    // Create actor contexts.
    let spotlight_context = MsgContext::new();
    let spotlight_addr = spotlight_context.address();

    let cache_dir = dirs::cache_dir()
        .ok_or(anyhow!(
            "Failed to detect system cache folder; Is this running on a desktop?"
        ))?
        .join("Daily-Wallpaper-Images");
    for service in [
        ImageService::Bing,
        ImageService::Nasa,
        ImageService::Spotlight,
    ] {
        let app_cache_dir = cache_dir.join(service.as_str());
        if !app_cache_dir.exists() {
            fs::create_dir_all(&app_cache_dir)?;
        }
    }

    // Spawn actors.
    let actor = ImageServiceActor::new(spotlight_addr, cache_dir, notification_center);
    spawn(spotlight_context.run(actor));
    Ok(())
}
