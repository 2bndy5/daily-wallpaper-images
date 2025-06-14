//! The actor model is highly recommended for state management,
//! as it provides modularity and scalability.
//! This module demonstrates how to use actors
//! within the async system in Rust.
//! To build a solid app, do not communicate by sharing memory;
//! instead, share memory by communicating.

use std::fmt::Debug;
use std::path::Path;
use std::time::Instant;
use std::{fs, io::Write, path::PathBuf};

use crate::common::{check_err, condense_duration, DATE_FILE_FMT};
use crate::notification_center::{NotificationActor, NotificationUpdate};
use crate::signals::{
    BingImageList, BingRefresh, DailyImage, NotificationAlert, NotificationSeverity,
};
use anyhow::{anyhow, Context, Result};
use chrono::{Local, NaiveDate};
use messages::prelude::{async_trait, Actor, Address, Context as MsgContext, Handler};
use rinf::{debug_print, DartSignal, RustSignal};
use serde::Deserialize;
use tokio::spawn;

#[derive(Debug, Deserialize)]
pub struct BingImage {
    pub url: String,
    #[serde(rename(deserialize = "startdate"))]
    pub start_date: String,
    pub copyright: String,
}

#[derive(Debug, Deserialize)]
pub struct BingImages {
    images: Vec<BingImage>,
}

// The type for instigating the actor.
struct BingRefreshMsg;

// The actor that holds the counter state and handles messages.
pub struct BingActor {
    app_cache_dir: PathBuf,
    notification_center: Address<NotificationActor>,
}

// Implementing the `Actor` trait for `CountingActor`.
// This defines `CountingActor` as an actor in the async system.
impl Actor for BingActor {}

impl BingActor {
    pub fn new(
        bing_addr: Address<Self>,
        app_cache_dir: PathBuf,
        notification_center: Address<NotificationActor>,
    ) -> Self {
        spawn(Self::listen_to_refresh_trigger(bing_addr));
        BingActor {
            app_cache_dir,
            notification_center,
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

#[async_trait]
impl Handler<BingRefreshMsg> for BingActor {
    type Result = Result<()>;
    // Handles messages received by the actor.
    async fn handle(&mut self, _msg: BingRefreshMsg, _context: &MsgContext<Self>) -> Self::Result {
        let debug_title = "Bing images";
        debug_print!("{debug_title}");
        let mut notification = NotificationAlert {
            title: debug_title.to_string(),
            body: "Checking cache".to_string(),
            percent: 0.0,
            severity: NotificationSeverity::Info,
            status_message: String::new(),
        };
        // notification.send_signal_to_dart();
        let timer = Instant::now();
        let mut total_steps = 1;

        let now = Local::now().date_naive();
        let today = format!("{}.json", now.format(DATE_FILE_FMT));
        let cached_metadata = self.app_cache_dir.join(Path::new(&today));
        let text = if cached_metadata.exists() {
            check_err(
                fs::read_to_string(&cached_metadata)
                    .with_context(|| "Failed to read cached Bing metadata"),
            )?
        } else {
            notification.body = "Fetching data from Bing".to_string();
            notification.status_message = condense_duration(timer.elapsed());
            check_err(check_err(
                self.notification_center
                    .send(NotificationUpdate(notification.clone()))
                    .await
                    .map_err(|e| anyhow!("Failed to send notification update: {e:?}")),
            )?)?;
            total_steps += 1;
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
            check_err(
                fs::write(&cached_metadata, &text)
                    .with_context(|| "Failed to write Bing metadata to cache"),
            )?;
            text
        };
        let images = check_err(
            serde_json::from_str::<BingImages>(&text)
                .with_context(|| "Failed to deserialize Bing images' response payload."),
        )?
        .images;
        let mut image_list = BingImageList {
            images: images
                .iter()
                .map(|i| DailyImage {
                    url: String::new(),
                    date: check_err(parse_date(&i.start_date))
                        .map(|d| d.format(DATE_FILE_FMT).to_string())
                        .unwrap_or(i.start_date.clone()),
                    description: i.copyright.clone(),
                })
                .collect(),
        };
        image_list.send_signal_to_dart();
        let mut last_day = None;
        let total_images = images.len();
        for (i, img) in images.into_iter().enumerate() {
            last_day = Some(parse_date(&img.start_date)?);
            let date = last_day.unwrap().format(DATE_FILE_FMT);

            let file_name = self.app_cache_dir.join(format!("{date}.jpg"));
            if !file_name.exists() {
                let response = check_err(
                    reqwest::get(format!("https://bing.com{}", img.url).as_str())
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
            }
            image_list.images[i].url = file_name.to_string_lossy().to_string();
            image_list.send_signal_to_dart();
            notification.body = format!("Processed {date}");
            notification.percent =
                ((i + total_steps) as f32) / ((total_images + total_steps) as f32);
            check_err(check_err(
                self.notification_center
                    .send(NotificationUpdate(notification.clone()))
                    .await
                    .map_err(|e| anyhow!("Failed to send notification update: {e:?}")),
            )?)?;
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
                    &check_err(
                        entry
                            .path()
                            .file_stem()
                            .ok_or(anyhow!("Failed to get filename of cached Bing image/file")),
                    )?
                    .to_string_lossy(),
                    DATE_FILE_FMT,
                )?;
                if date < last_day
                    || (date != now
                        && check_err(
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
        }
        let elapsed = timer.elapsed();
        notification.percent = 100.0;
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
    let actor = BingActor::new(bing_addr, app_cache_dir, notification_center);
    spawn(bing_context.run(actor));
    Ok(())
}
