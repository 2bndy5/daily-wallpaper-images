//! The actor model is highly recommended for state management,
//! as it provides modularity and scalability.
//! This module demonstrates how to use actors
//! within the async system in Rust.
//! To build a solid app, do not communicate by sharing memory;
//! instead, share memory by communicating.

use std::fmt::Debug;
use std::path::Path;
use std::{fs, time::Instant};

use crate::common::{check_err, condense_duration};
use crate::notification_center::{NotificationActor, NotificationUpdate};
use crate::services::actor::ImageServiceActor;
use crate::services::download_file;
use crate::signals::{
    DailyImage, ImageList, ImageService, NotificationAlert, NotificationSeverity,
};
use anyhow::{anyhow, Context, Result};
use messages::prelude::{async_trait, Address, Context as MsgContext, Handler};
use reqwest::ClientBuilder;
use rinf::{debug_print, RustSignal};
use serde::Deserialize;
use tokio::spawn;

#[derive(Debug, Deserialize)]
pub struct SpotlightLandscape {
    pub asset: String,
}

#[derive(Debug, Deserialize)]
pub struct SpotlightImage {
    #[serde(rename(deserialize = "landscapeImage"))]
    pub landscape_image: SpotlightLandscape,
    #[serde(rename(deserialize = "iconHoverText"))]
    pub icon_hover_text: String,
    // pub copyright: String,
    #[serde(rename(deserialize = "entityId"))]
    pub entity_id: String,
}

impl From<SpotlightImage> for DailyImage {
    fn from(value: SpotlightImage) -> Self {
        DailyImage {
            url: String::new(),
            date: String::new(),
            description: value
                .icon_hover_text
                .trim_end_matches("\r\nRight-click to learn more")
                .replace("\r\n", "\n"),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct SpotlightItemContent {
    pub ad: SpotlightImage,
}

#[derive(Debug, Deserialize)]
pub struct SpotlightItem {
    item: String,
}

#[derive(Debug, Deserialize)]
pub struct SpotlightBatchInfo {
    items: Vec<SpotlightItem>,
}
#[derive(Debug, Deserialize)]
pub struct SpotlightImages {
    #[serde(rename(deserialize = "batchrsp"))]
    pub batch_response: SpotlightBatchInfo,
}

/// The type for telling the actor to get new images.
pub struct SpotlightRefresh;

/// The type for telling the actor to reset the cache.
pub struct SpotlightReset;

#[async_trait]
impl Handler<SpotlightReset> for ImageServiceActor {
    type Result = Result<()>;
    // Handles messages received by the actor.
    async fn handle(&mut self, _msg: SpotlightReset, _context: &MsgContext<Self>) -> Self::Result {
        debug_print!("Resetting Windows Spotlight images");
        let app_cache_dir = self.app_cache_dir.join(ImageService::Spotlight.as_str());
        let cached_metadata = app_cache_dir.join(Path::new(Self::INFO_FILE));
        if cached_metadata.exists() {
            fs::remove_file(cached_metadata)
                .with_context(|| "Failed to delete cached info about Windows Spotlight images.")?;
        }
        Ok(())
    }
}

#[async_trait]
impl Handler<SpotlightRefresh> for ImageServiceActor {
    type Result = Result<()>;
    // Handles messages received by the actor.
    async fn handle(
        &mut self,
        _msg: SpotlightRefresh,
        _context: &MsgContext<Self>,
    ) -> Self::Result {
        let debug_title = "Windows Spotlight images";
        debug_print!("{debug_title}");
        let mut notification = NotificationAlert {
            title: debug_title.to_string(),
            body: "checking cache".to_string(),
            percent: 0.0,
            severity: NotificationSeverity::Info,
            status_message: String::new(),
        };
        check_err(check_err(
            self.notification_center
                .send(NotificationUpdate(notification.clone()))
                .await
                .map_err(|e| anyhow!("Failed to send notification update: {e:?}")),
        )?)?;
        let timer = Instant::now();
        let mut total_steps = 1;

        let client = ClientBuilder::new().build()?;

        let app_cache_dir = self.app_cache_dir.join(ImageService::Spotlight.as_str());
        let cached_metadata = app_cache_dir.join(Path::new(Self::INFO_FILE));
        let text = if cached_metadata.exists() {
            check_err(
                fs::read_to_string(&cached_metadata)
                    .with_context(|| "Failed to read cached Windows Spotlight metadata"),
            )?
        } else {
            notification.body = "Fetching data from Windows Spotlight".to_string();
            notification.status_message = condense_duration(timer.elapsed());
            check_err(check_err(
                self.notification_center
                    .send(NotificationUpdate(notification.clone()))
                    .await
                    .map_err(|e| anyhow!("Failed to send notification update: {e:?}")),
            )?)?;
            total_steps += 1;
            let url = "https://fd.api.iris.microsoft.com/v4/api/selection?&placement=88000820&bcnt=4&country=us&locale=en-us&fmt=json";
            let response = check_err(
                client
                    .get(url)
                    .send()
                    .await
                    .with_context(|| "Failed to get list of Windows Spotlight images"),
            )?;
            let text = check_err(response.text().await.with_context(|| {
                "Failed to get body as text from Windows Spotlight images' response"
            }))?;
            check_err(
                fs::write(&cached_metadata, &text)
                    .with_context(|| "Failed to write Windows Spotlight metadata to cache"),
            )?;
            text
        };
        let items = check_err(
            serde_json::from_str::<SpotlightImages>(&text).with_context(|| {
                "Failed to deserialize Windows Spotlight images' response payload."
            }),
        )?
        .batch_response
        .items;
        let total_images = items.len();
        let mut image_list = ImageList {
            images: vec![],
            service: ImageService::Spotlight,
        };
        let mut new_item_ids = vec![];
        for item in items {
            let content = serde_json::from_str::<SpotlightItemContent>(item.item.trim_matches('`'))
                .with_context(|| "Failed to deserialize Windows Spotlight image info")?
                .ad;
            new_item_ids.push((
                format!("{}.jpg", content.entity_id),
                content.landscape_image.asset.clone(),
            ));
            image_list.images.push(content.into());
        }

        for (i, (id, url)) in new_item_ids.iter().enumerate() {
            let file_name = app_cache_dir.join(id);
            let cache_path = file_name.as_os_str().to_string_lossy().to_string();
            if !file_name.exists() {
                check_err(
                    download_file(
                        &client,
                        url,
                        &cache_path,
                        id,
                        (total_steps + total_images) as u8,
                        &mut self.notification_center,
                        notification.clone(),
                    )
                    .await,
                )?;
            }
            image_list.images[i].url = cache_path;
            // The send method is generated from a marked Protobuf message.
            image_list.send_signal_to_dart();
            notification.body = format!("Processed {id}");
            notification.percent =
                ((i + total_steps) as f32) / ((total_images + total_steps) as f32);
            check_err(check_err(
                self.notification_center
                    .send(NotificationUpdate(notification.clone()))
                    .await
                    .map_err(|e| anyhow!("Failed to send notification update: {e:?}")),
            )?)?;
        }

        let new_ids = new_item_ids.iter().map(|i| i.0.clone()).collect::<Vec<_>>();
        // dispose outdated cached images
        for entry in (check_err(
            fs::read_dir(&app_cache_dir).with_context(|| "Failed to read cache folder contents."),
        )?)
        .flatten()
        {
            if !entry.path().is_file() {
                continue;
            }
            if check_err(
                entry
                    .path()
                    .extension()
                    .ok_or(anyhow!("Failed to get cached file's extension")),
            )?
            .to_string_lossy()
            .ends_with("json")
            {
                continue;
            }
            if !new_ids.contains(
                &entry
                    .path()
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string(),
            ) {
                debug_print!("Deleting outdated cache file {:?}", entry.path());
                check_err(
                    fs::remove_file(entry.path())
                        .with_context(|| "Failed to delete outdated Nasa cache file"),
                )?;
            }
        }
        let elapsed = timer.elapsed();
        notification.percent = 1.0;
        notification.body = "Cache updated".to_string();
        notification.status_message = condense_duration(elapsed);
        check_err(check_err(
            self.notification_center
                .send(NotificationUpdate(notification))
                .await
                .map_err(|e| anyhow!("Failed to send notification update: {e:?}")),
        )?)?;
        Ok(())
    }
}

// Creates and spawns the actors in the async system.
pub async fn create_actors(notification_center: Address<NotificationActor>) -> Result<()> {
    // Create actor contexts.
    let spotlight_context = MsgContext::new();
    let spotlight_addr = spotlight_context.address();

    let cache_dir = dirs::cache_dir().ok_or(anyhow!(
        "Failed to detect system cache folder; Is this running on a desktop?"
    ))?;
    let app_cache_dir = cache_dir.join("Daily-Images").join("Windows Spotlight");
    if !app_cache_dir.exists() {
        fs::create_dir_all(&app_cache_dir)?;
    }

    // Spawn actors.
    let actor = ImageServiceActor::new(spotlight_addr, app_cache_dir, notification_center);
    spawn(spotlight_context.run(actor));
    Ok(())
}
