//! The actor model is highly recommended for state management,
//! as it provides modularity and scalability.
//! This module demonstrates how to use actors
//! within the async system in Rust.
//! To build a solid app, do not communicate by sharing memory;
//! instead, share memory by communicating.

use std::fmt::Debug;
use std::path::Path;
use std::{fs, io::Write, path::PathBuf, time::Instant};

use crate::common::{check_err, condense_duration};
use crate::notification_center::{NotificationActor, NotificationUpdate};
use crate::signals::{
    DailyImage, NotificationAlert, NotificationSeverity, SpotlightImageList, SpotlightRefresh,
    SpotlightReset,
};
use anyhow::{anyhow, Context, Result};
use messages::prelude::{async_trait, Actor, Address, Context as MsgContext, Handler};
use rinf::{debug_print, DartSignal, RustSignal};
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
pub struct SpotlightBathInfo {
    items: Vec<SpotlightItem>,
}
#[derive(Debug, Deserialize)]
pub struct SpotlightImages {
    #[serde(rename(deserialize = "batchrsp"))]
    pub batch_response: SpotlightBathInfo,
}

// The type for telling the actor to get new images.
pub struct SpotlightRefreshMsg;

// The type for telling the actor to set the cache.
pub struct SpotlightResetMsg;

// The actor that holds the counter state and handles messages.
pub struct SpotlightActor {
    app_cache_dir: PathBuf,
    notification_center: Address<NotificationActor>,
}

// Implementing the `Actor` trait for `CountingActor`.
// This defines `CountingActor` as an actor in the async system.
impl Actor for SpotlightActor {}

impl SpotlightActor {
    const INFO_FILE: &str = "info.json";

    pub fn new(
        spotlight_addr: Address<Self>,
        app_cache_dir: PathBuf,
        notification_center: Address<NotificationActor>,
    ) -> Self {
        spawn(Self::listen_to_refresh_trigger(spotlight_addr.clone()));
        spawn(Self::listen_to_reset_trigger(spotlight_addr));
        SpotlightActor {
            app_cache_dir,
            notification_center,
        }
    }

    async fn listen_to_refresh_trigger(mut spotlight_addr: Address<Self>) {
        // Spawn an asynchronous task to listen for
        // button click signals from Dart.
        let receiver = SpotlightRefresh::get_dart_signal_receiver();
        // Continuously listen for signals.
        while let Some(_dart_signal) = receiver.recv().await {
            debug_print!("refreshing image list from Windows Spotlight");
            // Send a message to the actor.
            let _ = spotlight_addr.send(SpotlightRefreshMsg).await;
        }
    }

    async fn listen_to_reset_trigger(mut spotlight_addr: Address<Self>) {
        // Spawn an asynchronous task to listen for
        // button click signals from Dart.
        let receiver = SpotlightReset::get_dart_signal_receiver();
        // Continuously listen for signals.
        while let Some(_dart_signal) = receiver.recv().await {
            debug_print!("resetting image list from Windows Spotlight");
            // send reset message to the actor.
            let _ = spotlight_addr.send(SpotlightResetMsg).await;
            // Send refresh message to the actor.
            let _ = spotlight_addr.send(SpotlightRefreshMsg).await;
        }
    }
}

#[async_trait]
impl Handler<SpotlightResetMsg> for SpotlightActor {
    type Result = Result<()>;
    // Handles messages received by the actor.
    async fn handle(
        &mut self,
        _msg: SpotlightResetMsg,
        _context: &MsgContext<Self>,
    ) -> Self::Result {
        debug_print!("Resetting Windows Spotlight images");
        let cached_metadata = self.app_cache_dir.join(Path::new(Self::INFO_FILE));
        if cached_metadata.exists() {
            fs::remove_file(cached_metadata)
                .with_context(|| "Failed to delete cached info about Windows Spotlight images.")?;
        }
        Ok(())
    }
}

#[async_trait]
impl Handler<SpotlightRefreshMsg> for SpotlightActor {
    type Result = Result<()>;
    // Handles messages received by the actor.
    async fn handle(
        &mut self,
        _msg: SpotlightRefreshMsg,
        _context: &MsgContext<Self>,
    ) -> Self::Result {
        let debug_title = "Getting Windows Spotlight images";
        debug_print!("{debug_title}");
        let mut notification = NotificationAlert {
            title: debug_title.to_string(),
            body: "checking cache".to_string(),
            percent: Some(0.0),
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

        let cached_metadata = self.app_cache_dir.join(Path::new(Self::INFO_FILE));
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
                reqwest::get(url)
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
        let mut image_list = SpotlightImageList { images: vec![] };
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
            let file_name = self.app_cache_dir.join(id);
            if !file_name.exists() {
                let response = check_err(
                    reqwest::get(url)
                        .await
                        .with_context(|| "Failed to download image from Windows Spotlight"),
                )?;
                let img_bin = check_err(
                    response
                        .bytes()
                        .await
                        .with_context(|| "Failed to get bytes of image from response payload"),
                )?;
                let mut file =
                    check_err(fs::File::create(&file_name).with_context(|| {
                        "Failed to create a cache file for Windows Spotlight image"
                    }))?;
                check_err(file.write_all(&img_bin).with_context(|| {
                    "Failed to write Windows Spotlight image data to cache file"
                }))?;
            }
            image_list.images[i].url = file_name.to_string_lossy().to_string();
            // The send method is generated from a marked Protobuf message.
            image_list.send_signal_to_dart();
            notification.body = format!("Processed {id}");
            notification.percent =
                Some(((i + total_steps) as f32) / ((total_images + total_steps) as f32));
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
            fs::read_dir(&self.app_cache_dir)
                .with_context(|| "Failed to read cache folder contents."),
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
        notification.percent = Some(100.0);
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
    let actor = SpotlightActor::new(spotlight_addr, app_cache_dir, notification_center);
    spawn(spotlight_context.run(actor));
    Ok(())
}
