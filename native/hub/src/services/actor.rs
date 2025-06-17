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
use crate::common::check_err;
use crate::notification_center::NotificationActor;
use crate::signals::{ImageService, Refresh, Reset};
use anyhow::{anyhow, Result};
use messages::prelude::{Actor, Address, Context as MsgContext};
use rinf::{debug_print, DartSignal};
use tokio::spawn;

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
