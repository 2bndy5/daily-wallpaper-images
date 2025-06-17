//! The actor model is highly recommended for state management,
//! as it provides modularity and scalability.
//! This module demonstrates how to use actors
//! within the async system in Rust.
//! To build a solid app, do not communicate by sharing memory;
//! instead, share memory by communicating.

use std::time::Duration;
use std::{fs, path::Path, time::Instant};

use crate::common::check_err;
use crate::signals::ImageList;
use crate::{
    common::{condense_duration, DATE_FILE_FMT},
    services::actor::ImageServiceActor,
    signals::{DailyImage, ImageService, NotificationAlert, NotificationSeverity},
};
use anyhow::{anyhow, Context, Result};
use chrono::{Local, NaiveDate};
use messages::prelude::{async_trait, Context as MsgContext, Handler};
use reqwest::{ClientBuilder, Url};
use rinf::{debug_print, RustSignal};
use serde::Deserialize;

// The message types implemented for the actor.
pub struct NasaRefresh;

#[derive(Debug, Deserialize)]
pub struct NasaFeed {
    pub channel: NasaChannel,
}

#[derive(Debug, Deserialize)]
pub struct NasaChannel {
    pub item: Vec<NasaItem>,
}

#[derive(Debug, Deserialize)]
pub struct NasaItem {
    pub description: String,
    #[serde(rename = "pubDate")]
    pub pub_date: String,
    pub enclosure: NasaImgUrl,
}

#[derive(Debug, Deserialize)]
pub struct NasaImgUrl {
    #[serde(rename = "@url")]
    pub url: String,
}

fn parse_date(date: &str) -> Result<NaiveDate> {
    NaiveDate::parse_from_str(date, "%d %b %Y")
        .with_context(|| "Failed to parse NASA picture's date.")
}

#[async_trait]
impl Handler<NasaRefresh> for ImageServiceActor {
    type Result = Result<()>;
    // Handles messages received by the actor.
    async fn handle(&mut self, _msg: NasaRefresh, _context: &MsgContext<Self>) -> Self::Result {
        let debug_title = format!("{} images", ImageService::Nasa.as_str());
        debug_print!("{debug_title}");
        let mut notification = NotificationAlert {
            title: debug_title.to_string(),
            body: "Checking cache".to_string(),
            percent: 0.0,
            severity: NotificationSeverity::Info,
            status_message: String::new(),
        };
        self.check_notify_send_error(notification.clone()).await?;
        let timer = Instant::now();
        let mut total_steps = 1;

        let client = ClientBuilder::new().build()?;

        let now = Local::now().date_naive();
        let today = format!("{}.xml", now.format(DATE_FILE_FMT));

        let app_cache_dir = self.app_cache_dir.join(ImageService::Nasa.as_str());
        let cached_metadata = app_cache_dir.join(Path::new(&today));
        let text = if cached_metadata.exists() {
            fs::read_to_string(&cached_metadata)
                .with_context(|| "Failed to read cached NASA metadata")?
        } else {
            notification.body = "Fetching data from NASA".to_string();
            notification.status_message = condense_duration(timer.elapsed());
            self.check_notify_send_error(notification.clone()).await?;
            total_steps += 1;
            let response = self
                .notify_err(
                    client
                        .get("https://www.nasa.gov/rss/dyn/lg_image_of_the_day.rss")
                        .timeout(Duration::from_secs(15))
                        .send()
                        .await
                        .with_context(|| "Failed to get list of NASA images"),
                    ImageService::Nasa,
                )
                .await?;
            let text = self
                .notify_err(
                    response
                        .text()
                        .await
                        .with_context(|| "Failed to get body as text from NASA images' response"),
                    ImageService::Nasa,
                )
                .await?;
            self.notify_err(
                fs::write(&cached_metadata, &text)
                    .with_context(|| "Failed to write NASA metadata to cache"),
                ImageService::Nasa,
            )
            .await?;
            text
        };
        let images = self
            .notify_err(
                quick_xml::de::from_str::<NasaFeed>(&text)
                    .with_context(|| "Failed to deserialize NASA images' response payload."),
                ImageService::Nasa,
            )
            .await?
            .channel
            .item;
        let mut image_list = ImageList {
            service: ImageService::Nasa,
            images: images
                .iter()
                .map(|i| DailyImage {
                    url: String::new(),
                    date: check_err(parse_date(&i.pub_date[5..16]))
                        .map(|d| d.format(DATE_FILE_FMT).to_string())
                        .unwrap_or(i.pub_date.clone()),
                    description: i.description.clone(),
                })
                .collect(),
        };
        image_list.send_signal_to_dart();
        let mut last_day = None;
        let total_images = images.len();
        for (i, item) in images.into_iter().enumerate() {
            last_day = Some(parse_date(&item.pub_date[5..16])?);
            let date = last_day.unwrap().format(DATE_FILE_FMT);
            let name = format!(
                "{date}.{}",
                self.notify_err(
                    Path::new(Url::parse(&item.enclosure.url)?.path())
                        .extension()
                        .ok_or(anyhow!("Failed to find image MIME type from NASA URL.")),
                    ImageService::Nasa,
                )
                .await?
                .to_string_lossy()
            );
            let file_name = app_cache_dir.join(&name);
            let cache_path = file_name.to_string_lossy().to_string();
            if !file_name.exists() {
                let result = self
                    .download_image(
                        &client,
                        &item.enclosure.url,
                        &cache_path,
                        &name,
                        (total_steps + total_images) as u8,
                        notification.clone(),
                    )
                    .await;
                self.notify_err(result, ImageService::Nasa).await?;
            }
            image_list.images[i].url = file_name.to_string_lossy().to_string();
            image_list.send_signal_to_dart();
            notification.body = format!("Processed {date}");
            notification.percent =
                ((i + total_steps) as f32) / ((total_images + total_steps) as f32);
            self.check_notify_send_error(notification.clone()).await?;
        }

        // dispose outdated cached images
        if let Some(last_day) = last_day {
            for entry in self
                .notify_err(
                    fs::read_dir(&app_cache_dir)
                        .with_context(|| "Failed to read cache folder contents."),
                    ImageService::Nasa,
                )
                .await?
                .flatten()
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
                                .ok_or(anyhow!("Failed to get filename for cached NASA image")),
                            ImageService::Nasa,
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
                                ImageService::Nasa,
                            )
                            .await?
                            .to_string_lossy()
                            .ends_with("xml"))
                {
                    debug_print!("Deleting outdated cache file {:?}", entry.path());
                    self.notify_err(
                        fs::remove_file(entry.path())
                            .with_context(|| "Failed to delete outdated NASA cache file"),
                        ImageService::Nasa,
                    )
                    .await?;
                }
            }
        }
        let elapsed = timer.elapsed();
        notification.percent = 1.0;
        notification.body = "Cache updated".to_string();
        notification.status_message = condense_duration(elapsed);
        self.check_notify_send_error(notification).await
    }
}

#[cfg(test)]
mod tests {
    use super::NasaFeed;
    use anyhow::Context;
    use std::{fs, io::Read};

    #[test]
    fn parse_nasa_rss() {
        let mut text = String::new();
        fs::File::open("tests/nasa-rss-feed.xml")
            .unwrap()
            .read_to_string(&mut text)
            .unwrap();
        let items = quick_xml::de::from_str::<NasaFeed>(&text)
            .with_context(|| "Failed to deserialize NASA images' response payload.")
            .unwrap()
            .channel
            .item;
        assert!(!items.is_empty());
    }
}
