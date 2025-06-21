//! The actor model is highly recommended for state management,
//! as it provides modularity and scalability.
//! This module demonstrates how to use actors
//! within the async system in Rust.
//! To build a solid app, do not communicate by sharing memory;
//! instead, share memory by communicating.

use std::{path::PathBuf, time::Duration};

use super::{condense_duration, get_service_metadata_name, get_service_url, UpdateResources};
use crate::{
    common::check_err,
    notification_center::{NotificationActor, NotificationUpdate},
    signals::{ImageService, NotificationAlert, NotificationSeverity, NotificationStatus, Refresh},
};
use anyhow::{anyhow, Context, Result};
use futures_util::{StreamExt, TryFutureExt};
use messages::prelude::{async_trait, Actor, Address, Context as MsgContext, Handler};
use reqwest::{header::CONTENT_LENGTH, Client};
use rinf::{debug_print, DartSignal};
use size::Size;
use tokio::{fs, io::AsyncWriteExt, spawn, time::Instant};

/// The actor that holds the Service state and handles messages.
pub struct ImageServiceActor {
    pub(super) app_cache_dir: PathBuf,
    pub(super) notification_center: Address<NotificationActor>,
}

// This defines our `Actor` as an actor in the async system.
impl Actor for ImageServiceActor {}

impl ImageServiceActor {
    pub fn new(
        service_addr: Address<Self>,
        app_cache_dir: PathBuf,
        notification_center: Address<NotificationActor>,
    ) -> Self {
        spawn(Self::listen_to_refresh(service_addr));
        ImageServiceActor {
            app_cache_dir,
            notification_center,
        }
    }

    async fn listen_to_refresh(mut service_addr: Address<Self>) {
        // Spawn an asynchronous task to listen for signals from Dart.
        let receiver = Refresh::get_dart_signal_receiver();
        // Continuously listen for signals.
        while let Some(dart_signal) = receiver.recv().await {
            let service = dart_signal.message.service;
            debug_print!("resetting image list from {}", service.as_str());

            // Send refresh message to the actor.
            let result = service_addr
                .send(dart_signal.message)
                .await
                .map_err(|e| anyhow!("Failed to refresh {} cache: {e:?}", service.as_str()));
            if let Ok(result) = check_err(result) {
                let _ = check_err(result);
            };
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

    pub(super) async fn download_file(
        &mut self,
        client: &Client,
        url: &str,
        cache_path: &str,
        display_id: &str,
        total_steps: u8,
        mut notification: NotificationAlert,
    ) -> Result<usize> {
        let response = client.get(url).send().await?;
        let total_size = if let Some(v) = response.headers().get(CONTENT_LENGTH) {
            let as_str = v.to_str()?;
            Some(as_str.to_string().parse::<usize>()?)
        } else {
            debug_print!("Failed to get the content length for file {display_id}");
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
                fs::File::create(cache_path)
                    .map_err(|e| anyhow!("Failed to create cache file for {cache_path}: {e:?}"))
                    .await,
            )?
            .write_all(&buf)
            .map_err(|e| anyhow!("Failed to write data to file {cache_path}: {e:?}"))
            .await,
        )?;
        Ok(total_size.unwrap_or(downloaded))
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
                status: NotificationStatus::default(),
            })
            .await?;
        }
        result
    }
}

#[async_trait]
impl Handler<Refresh> for ImageServiceActor {
    /// Result of the message processing.
    type Result = Result<()>;

    /// Processes a message.
    async fn handle(&mut self, message: Refresh, _context: &MsgContext<Self>) -> Self::Result {
        // set up output mechanisms (in-app notifications and terminal out)
        let service = message.service;
        let service_name = service.as_str();
        let debug_title = format!("{} images", service_name);
        debug_print!("Getting {debug_title}");
        let mut res = UpdateResources::new(
            self.app_cache_dir.join(service_name),
            NotificationAlert {
                title: debug_title.to_string(),
                body: "Checking cache".to_string(),
                percent: 0.0,
                severity: NotificationSeverity::Info,
                status: NotificationStatus::default(),
            },
        )?;
        self.check_notify_send_error(res.notification.clone())
            .await?;
        let timer = Instant::now();

        // get cache state
        let metadata_file_name = get_service_metadata_name(&service);
        let cached_metadata = res.app_cache_dir.join(&metadata_file_name);

        res.text = if !message.reset && cached_metadata.exists() {
            check_err(
                fs::read_to_string(&cached_metadata)
                    .await
                    .with_context(|| "Failed to read cached metadata"),
            )?
        } else {
            // update cache metadata
            res.notification.body = format!("Fetching data from {}", service_name);
            self.check_notify_send_error(res.notification.clone())
                .await?;
            res.total_steps += 1;
            let response = self
                .notify_err(
                    res.client
                        .get(get_service_url(&service))
                        .timeout(Duration::from_secs(15))
                        .send()
                        .await
                        .with_context(|| "Failed to get list of images"),
                    service,
                )
                .await?;
            let text = self
                .notify_err(
                    response
                        .text()
                        .await
                        .with_context(|| "Failed to get metadata from service response"),
                    service,
                )
                .await?;
            res.downloaded += text.len();
            self.notify_err(
                fs::write(&cached_metadata, &text)
                    .await
                    .with_context(|| "Failed to write metadata to cache"),
                service,
            )
            .await?;
            text
        };

        // process images per service
        let cached_images = match service {
            ImageService::Bing => self.cache_updates_bing(&mut res).await,
            ImageService::Nasa => self.cache_updates_nasa(&mut res).await,
            ImageService::Spotlight => self.cache_updates_spotlight(&mut res).await,
        }?;

        // dispose outdated cached images
        let mut removed: u8 = 0;
        let mut entries = self
            .notify_err(
                fs::read_dir(&res.app_cache_dir)
                    .await
                    .with_context(|| "Failed to read cache folder contents."),
                service,
            )
            .await?;
        while let Some(entry) = entries
            .next_entry()
            .await
            .with_context(|| "Failed to traverse cache dir")?
        {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            let filename = path
                .file_name()
                .ok_or(anyhow!("Failed to get cached file name"))?
                .to_string_lossy()
                .to_string();
            if filename == metadata_file_name {
                continue;
            }
            if !cached_images.contains(&filename) {
                removed += 1;
                debug_print!("Deleting outdated cache file {:?}", path);
                self.notify_err(
                    fs::remove_file(path)
                        .await
                        .with_context(|| "Failed to delete outdated Nasa cache file"),
                    service,
                )
                .await?;
            }
        }

        // finish up
        let elapsed = condense_duration(timer.elapsed());
        res.notification.percent = 1.0;
        if res.downloaded > 0 {
            res.notification.status = NotificationStatus {
                downloaded: Some(
                    Size::from_bytes(res.downloaded)
                        .format()
                        .with_base(size::Base::Base10)
                        .to_string(),
                ),
                removed: Some(removed),
                elapsed: Some(elapsed),
            };
            res.notification.body =
                format!("Cached {}/{} images", res.updated_images, res.total_images,);
        } else {
            res.notification.status = NotificationStatus {
                elapsed: Some(elapsed),
                ..Default::default()
            };
            res.notification.body = "Cache is already updated".to_string();
        }
        self.check_notify_send_error(res.notification).await
    }
}

// Creates and spawns the actors in the async system.
pub async fn create_actors(notification_center: Address<NotificationActor>) -> Result<()> {
    // Create actor contexts.
    let img_service_ctx = MsgContext::new();
    let img_service_addr = img_service_ctx.address();

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
            fs::create_dir_all(&app_cache_dir).await?;
        }
    }

    // Spawn actors.
    let actor = ImageServiceActor::new(img_service_addr, cache_dir, notification_center);
    spawn(img_service_ctx.run(actor));
    Ok(())
}
