//! The actor model is highly recommended for state management,
//! as it provides modularity and scalability.
//! This module demonstrates how to use actors
//! within the async system in Rust.
//! To build a solid app, do not communicate by sharing memory;
//! instead, share memory by communicating.

use std::{fs, io::Write, path::PathBuf};

use crate::common::{check_err, DATE_FILE_FMT};
use crate::messages::*;
use anyhow::{anyhow, Context, Result};
use chrono::NaiveDate;
use messages::prelude::{async_trait, Actor, Address, Context as MsgContext, Handler};
use rinf::debug_print;
use serde::Deserialize;
use tokio::spawn;

// The type for instigating the actor.
pub struct BingRefreshMsg;

// The actor that holds the counter state and handles messages.
pub struct BingActor {
    // The counter number.
    #[allow(dead_code)]
    images: Vec<String>,
    app_cache_dir: PathBuf,
}

// Implementing the `Actor` trait for `CountingActor`.
// This defines `CountingActor` as an actor in the async system.
impl Actor for BingActor {}

impl BingActor {
    pub fn new(bing_addr: Address<Self>, app_cache_dir: PathBuf) -> Self {
        spawn(Self::listen_to_refresh_trigger(bing_addr));
        BingActor {
            images: vec![],
            app_cache_dir,
        }
    }

    async fn listen_to_refresh_trigger(mut bing_addr: Address<Self>) {
        // Spawn an asynchronous task to listen for
        // button click signals from Dart.
        let receiver = BingRefresh::get_dart_signal_receiver();
        // Continuously listen for signals.
        while let Some(_dart_signal) = receiver.recv().await {
            debug_print!("refreshing image list from Bing");
            // Send a message to the actor.
            let _ = bing_addr.send(BingRefreshMsg).await;
        }
    }
}
fn parse_date(date: &str) -> Result<NaiveDate> {
    NaiveDate::parse_from_str(date, "%Y%m%d")
        .with_context(|| "Failed to parse Bing picture's date.")
}

#[derive(Debug, Deserialize)]
pub struct BingImages {
    images: Vec<BingImage>,
}

#[async_trait]
impl Handler<BingRefreshMsg> for BingActor {
    type Result = Result<()>;
    // Handles messages received by the actor.
    async fn handle(&mut self, _msg: BingRefreshMsg, _context: &MsgContext<Self>) -> Self::Result {
        debug_print!("Getting Bing images");
        // get list of images
        let response = check_err(
            reqwest::get("https://www.bing.com/HPImageArchive.aspx?format=js&idx=0&n=14")
                .await
                .with_context(|| "Failed to get list of Bing images"),
        )?;
        let text = check_err(
            response
                .text()
                .await
                .with_context(|| "Failed to get body as text from Bing images' response"),
        )?;
        let mut images = check_err(
            serde_json::from_str::<BingImages>(&text)
                .with_context(|| "Failed to deserialize Bing images' response payload."),
        )?
        .images;
        let mut last_day = None;
        for img in &mut images {
            last_day = Some(parse_date(&img.startdate)?);
            let date = last_day.unwrap().format(DATE_FILE_FMT);

            let file_name = self.app_cache_dir.join(format!("{date}.jpg"));
            img.startdate = date.to_string();
            if file_name.exists() {
                img.url = file_name.to_string_lossy().to_string();
            } else {
                img.url = format!("https://bing.com{}", img.url);
                let response = check_err(
                    reqwest::get(&img.url)
                        .await
                        .with_context(|| "Failed to download image from Bing"),
                )?;
                let img_bin = check_err(
                    response
                        .bytes()
                        .await
                        .with_context(|| "Failed to get bytes of image from response payload"),
                )?;
                let mut file = check_err(
                    fs::File::create(&file_name)
                        .with_context(|| "Failed to create a cache file for Bing image"),
                )?;
                check_err(
                    file.write_all(&img_bin)
                        .with_context(|| "Failed to write Bing image data to cache file"),
                )?;
                img.url = file_name.to_string_lossy().to_string();
            }
        }

        // dispose outdated cached images
        if let Some(last_day) = last_day {
            for entry in (check_err(
                fs::read_dir(&self.app_cache_dir)
                    .with_context(|| "Failed to read cache folder contents."),
            )?)
            .flatten()
            {
                if !entry.path().is_file() {
                    continue;
                }
                let date = NaiveDate::parse_from_str(
                    &entry
                        .path()
                        .file_stem()
                        .ok_or(anyhow!("Failed to get filename of cached Bing image"))?
                        .to_string_lossy(),
                    DATE_FILE_FMT,
                )?;
                if date < last_day {
                    fs::remove_file(entry.path())?;
                }
            }
        }

        // The send method is generated from a marked Protobuf message.
        BingImageList { images }.send_signal_to_dart();
        Ok(())
    }
}

// Creates and spawns the actors in the async system.
pub async fn create_actors() -> Result<()> {
    // Create actor contexts.
    let bing_context = MsgContext::new();
    let bing_addr = bing_context.address();

    let cache_dir = dirs::cache_dir().ok_or(anyhow!(
        "Failed to detect system cache folder; Is this running on a desktop?"
    ))?;
    let app_cache_dir = cache_dir.join("Daily-Images").join("Bing");
    if !app_cache_dir.exists() {
        fs::create_dir_all(&app_cache_dir)?;
    }

    // Spawn actors.
    let actor = BingActor::new(bing_addr, app_cache_dir);
    spawn(bing_context.run(actor));
    Ok(())
}
