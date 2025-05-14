//! The actor model is highly recommended for state management,
//! as it provides modularity and scalability.
//! This module demonstrates how to use actors
//! within the async system in Rust.
//! To build a solid app, do not communicate by sharing memory;
//! instead, share memory by communicating.

use std::collections::HashMap;
use std::fmt::Debug;
use std::path::Path;
use std::{fs, io::Write, path::PathBuf};

use crate::common::{check_err, DATE_FILE_FMT};
use crate::signals::{DailyImage, SpotlightImageList, SpotlightRefresh};
use anyhow::{anyhow, Context, Result};
use chrono::Local;
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
            date: value.entity_id,
            title: String::new(),
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

// The type for instigating the actor.
pub struct SpotlightRefreshMsg;

// The actor that holds the counter state and handles messages.
pub struct SpotlightActor {
    app_cache_dir: PathBuf,
}

// Implementing the `Actor` trait for `CountingActor`.
// This defines `CountingActor` as an actor in the async system.
impl Actor for SpotlightActor {}

impl SpotlightActor {
    pub fn new(spotlight_addr: Address<Self>, app_cache_dir: PathBuf) -> Self {
        spawn(Self::listen_to_refresh_trigger(spotlight_addr));
        SpotlightActor { app_cache_dir }
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
        debug_print!("Getting Windows Spotlight images");
        let now = Local::now().date_naive();
        let today = format!("{}.json", now.format(DATE_FILE_FMT));
        let cached_metadata = self.app_cache_dir.join(Path::new(&today));
        let text = if cached_metadata.exists() {
            check_err(
                fs::read_to_string(&cached_metadata)
                    .with_context(|| "Failed to read cached Windows Spotlight metadata"),
            )?
        } else {
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
        let mut image_list = SpotlightImageList { images: vec![] };
        let mut new_item_ids = HashMap::new();
        for item in items {
            let content = serde_json::from_str::<SpotlightItemContent>(item.item.trim_matches('`'))
                .with_context(|| "Failed to deserialize Windows Spotlight image info")?
                .ad;
            new_item_ids.insert(
                format!("{}.jpg", content.entity_id),
                content.landscape_image.asset.clone(),
            );
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
        }

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
            if !new_item_ids.contains_key(
                &entry
                    .path()
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string(),
            ) || (check_err(
                entry
                    .path()
                    .extension()
                    .ok_or(anyhow!("Failed to get cached file's extension")),
            )?
            .to_string_lossy()
            .ends_with("json"))
            {
                debug_print!("Deleting outdated cache file {:?}", entry.path());
                check_err(
                    fs::remove_file(entry.path())
                        .with_context(|| "Failed to delete outdated Nasa cache file"),
                )?;
            }
        }
        Ok(())
    }
}

// Creates and spawns the actors in the async system.
pub async fn create_actors() -> Result<()> {
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
    let actor = SpotlightActor::new(spotlight_addr, app_cache_dir);
    spawn(spotlight_context.run(actor));
    Ok(())
}
