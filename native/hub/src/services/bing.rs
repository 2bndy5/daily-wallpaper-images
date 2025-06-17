//! The actor model is highly recommended for state management,
//! as it provides modularity and scalability.
//! This module demonstrates how to use actors
//! within the async system in Rust.
//! To build a solid app, do not communicate by sharing memory;
//! instead, share memory by communicating.

use std::fmt::Debug;
use std::fs;
use std::path::Path;
use std::time::Instant;

use crate::common::{check_err, condense_duration, DATE_FILE_FMT};
use crate::notification_center::NotificationUpdate;
use crate::services::actor::ImageServiceActor;
use crate::services::download_file;
use crate::signals::{
    DailyImage, ImageList, ImageService, NotificationAlert, NotificationSeverity,
};
use anyhow::{anyhow, Context, Result};
use chrono::{Local, NaiveDate};
use messages::prelude::{async_trait, Context as MsgContext, Handler};
use reqwest::ClientBuilder;
use rinf::{debug_print, RustSignal};
use serde::Deserialize;

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

        let client = ClientBuilder::new().build()?;

        let now = Local::now().date_naive();
        let today = format!("{}.json", now.format(DATE_FILE_FMT));
        let app_cache_dir = self.app_cache_dir.join(ImageService::Bing.as_str());
        let cached_metadata = app_cache_dir.join(Path::new(&today));
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
                client
                    .get("https://www.bing.com/HPImageArchive.aspx?format=js&idx=0&n=14")
                    .send()
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
        for (i, img) in images.into_iter().enumerate() {
            last_day = Some(parse_date(&img.start_date)?);
            let date = last_day.unwrap().format(DATE_FILE_FMT);

            let name = format!("{date}.jpg");
            let file_name = app_cache_dir.join(&name);
            let cache_path = file_name.to_string_lossy().to_string();
            if !file_name.exists() {
                check_err(
                    download_file(
                        &client,
                        format!("https://bing.com{}", img.url).as_str(),
                        &cache_path,
                        &name,
                        (total_steps + total_images) as u8,
                        &mut self.notification_center,
                        notification.clone(),
                    )
                    .await,
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
                fs::read_dir(&app_cache_dir)
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
