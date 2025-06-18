//! The actor model is highly recommended for state management,
//! as it provides modularity and scalability.
//! This module demonstrates how to use actors
//! within the async system in Rust.
//! To build a solid app, do not communicate by sharing memory;
//! instead, share memory by communicating.

use std::{
    fmt::Debug,
    path::Path,
    time::{Duration, Instant},
};

use crate::common::{check_err, condense_duration, DATE_FILE_FMT};
use crate::services::actor::ImageServiceActor;
use crate::signals::{
    DailyImage, ImageList, ImageService, NotificationAlert, NotificationSeverity,
};
use anyhow::{anyhow, Context, Result};
use chrono::{Local, NaiveDate};
use messages::prelude::{async_trait, Context as MsgContext, Handler};
use reqwest::ClientBuilder;
use rinf::{debug_print, RustSignal};
use serde::Deserialize;
use size::Size;
use tokio::fs;

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

// The message types implemented for the actor.
pub struct BingRefresh;

fn parse_date(date: &str) -> Result<NaiveDate> {
    NaiveDate::parse_from_str(date, "%Y%m%d")
        .with_context(|| "Failed to parse Bing picture's date.")
}

#[async_trait]
impl Handler<BingRefresh> for ImageServiceActor {
    type Result = Result<()>;
    // Handles messages received by the actor.
    async fn handle(&mut self, _msg: BingRefresh, _context: &MsgContext<Self>) -> Self::Result {
        let debug_title = format!("{} images", ImageService::Bing.as_str());
        debug_print!("Getting {debug_title}");
        let mut notification = NotificationAlert {
            title: debug_title.to_string(),
            body: "Checking cache".to_string(),
            percent: 0.0,
            severity: NotificationSeverity::Info,
            status_message: String::new(),
        };
        self.check_notify_send_error(notification.clone()).await?;
        let mut downloaded = 0;

        let timer = Instant::now();
        let mut total_steps = 1;

        let client = ClientBuilder::new().build()?;

        let now = Local::now().date_naive();
        let today = format!("{}.json", now.format(DATE_FILE_FMT));
        let app_cache_dir = self.app_cache_dir.join(ImageService::Bing.as_str());
        let cached_metadata = app_cache_dir.join(Path::new(&today));
        let text = if cached_metadata.exists() {
            check_err(
                fs::read_to_string(&cached_metadata)
                    .await
                    .with_context(|| "Failed to read cached Bing metadata"),
            )?
        } else {
            notification.body = "Fetching data from Bing".to_string();
            notification.status_message = condense_duration(timer.elapsed());
            self.check_notify_send_error(notification.clone()).await?;
            total_steps += 1;
            let response = self
                .notify_err(
                    client
                        .get("https://www.bing.com/HPImageArchive.aspx?format=js&idx=0&n=14")
                        .timeout(Duration::from_secs(15))
                        .send()
                        .await
                        .with_context(|| "Failed to get list of Bing images"),
                    ImageService::Bing,
                )
                .await?;
            let text = self
                .notify_err(
                    response
                        .text()
                        .await
                        .with_context(|| "Failed to get body as text from Bing images' response"),
                    ImageService::Bing,
                )
                .await?;
            downloaded += text.len();
            self.notify_err(
                fs::write(&cached_metadata, &text)
                    .await
                    .with_context(|| "Failed to write Bing metadata to cache"),
                ImageService::Bing,
            )
            .await?;
            text
        };
        let images = self
            .notify_err(
                serde_json::from_str::<BingImages>(&text)
                    .with_context(|| "Failed to deserialize Bing images' response payload."),
                ImageService::Bing,
            )
            .await?
            .images;
        let mut image_list = ImageList {
            service: ImageService::Bing,
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
        let mut updated_images: u8 = 0;
        for (i, img) in images.into_iter().enumerate() {
            last_day = Some(parse_date(&img.start_date)?);
            let date = last_day.unwrap().format(DATE_FILE_FMT);

            let name = format!("{date}.jpg");
            let file_name = app_cache_dir.join(&name);
            let cache_path = file_name.to_string_lossy().to_string();
            if !file_name.exists() {
                updated_images += 1;
                let result = self
                    .download_image(
                        &client,
                        format!("https://bing.com{}", img.url).as_str(),
                        &cache_path,
                        &name,
                        (total_steps + total_images) as u8,
                        notification.clone(),
                    )
                    .await;
                downloaded += self.notify_err(result, ImageService::Bing).await?;
            }
            image_list.images[i].url = file_name.to_string_lossy().to_string();
            image_list.send_signal_to_dart();
            notification.body = format!("Processed {date}");
            notification.percent =
                ((i + total_steps) as f32) / ((total_images + total_steps) as f32);
            self.check_notify_send_error(notification.clone()).await?;
        }

        // dispose outdated cached images
        let mut removed: u8 = 0;
        if let Some(last_day) = last_day {
            let mut entries = self
                .notify_err(
                    fs::read_dir(&app_cache_dir)
                        .await
                        .with_context(|| "Failed to read cache folder contents."),
                    ImageService::Bing,
                )
                .await?;
            while let Some(entry) = entries
                .next_entry()
                .await
                .with_context(|| "Failed to traverse cache dir")?
            {
                if !entry.path().is_file() {
                    continue;
                }
                let date = NaiveDate::parse_from_str(
                    &self
                        .notify_err(
                            entry
                                .path()
                                .file_stem()
                                .ok_or(anyhow!("Failed to get filename of cached Bing image/file")),
                            ImageService::Bing,
                        )
                        .await?
                        .to_string_lossy(),
                    DATE_FILE_FMT,
                )?;
                if date < last_day
                    || (date != now
                        && self
                            .notify_err(
                                entry
                                    .path()
                                    .extension()
                                    .ok_or(anyhow!("Failed to get cached file's extension")),
                                ImageService::Bing,
                            )
                            .await?
                            .to_string_lossy()
                            .ends_with("json"))
                {
                    removed += 1;
                    debug_print!("Deleting outdated cache file {:?}", entry.path());
                    self.notify_err(
                        fs::remove_file(entry.path())
                            .await
                            .with_context(|| "Failed to delete outdated Nasa cache file"),
                        ImageService::Bing,
                    )
                    .await?;
                }
            }
        }
        let elapsed = timer.elapsed();
        notification.percent = 1.0;
        if downloaded > 0 {
            notification.body = format!(
                "Cached {updated_images}/{total_images} images\nDownloaded {}. Deleted {removed} files.",
                Size::from_bytes(downloaded)
                    .format()
                    .with_base(size::Base::Base10)
            );
        } else {
            notification.body = "Cache is already updated".to_string();
        }
        notification.status_message = condense_duration(elapsed);
        self.check_notify_send_error(notification).await
    }
}
