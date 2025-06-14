//! The actor model is highly recommended for state management,
//! as it provides modularity and scalability.
//! This module demonstrates how to use actors
//! within the async system in Rust.
//! To build a solid app, do not communicate by sharing memory;
//! instead, share memory by communicating.

use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
    time::Instant,
};

use crate::{
    common::{check_err, condense_duration, DATE_FILE_FMT},
    signals::{DailyImage, NasaRefresh, NotificationAlert, NotificationSeverity},
};
use crate::{
    notification_center::{NotificationActor, NotificationUpdate},
    signals::NasaImageList,
};
use anyhow::{anyhow, Context, Result};
use chrono::{Local, NaiveDate};
use messages::prelude::{async_trait, Actor, Address, Context as MsgContext, Handler};
use reqwest::Url;
use rinf::{debug_print, DartSignal, RustSignal};
use serde::Deserialize;
use tokio::spawn;

// The type for instigating the actor.
struct NasaRefreshMsg;

// The actor that holds the counter state and handles messages.
pub struct NasaActor {
    app_cache_dir: PathBuf,
    notification_center: Address<NotificationActor>,
}

impl Actor for NasaActor {}

impl NasaActor {
    pub fn new(
        nasa_addr: Address<Self>,
        app_cache_dir: PathBuf,
        notification_center: Address<NotificationActor>,
    ) -> Self {
        spawn(Self::listen_to_refresh_trigger(nasa_addr));
        NasaActor {
            app_cache_dir,
            notification_center,
        }
    }

    async fn listen_to_refresh_trigger(mut nasa_addr: Address<Self>) {
        // Spawn an asynchronous task to listen for
        // button click signals from Dart.
        let receiver = NasaRefresh::get_dart_signal_receiver();
        // Continuously listen for signals.
        while let Some(_dart_signal) = receiver.recv().await {
            debug_print!("refreshing image list from NASA");
            // Send a message to the actor.
            let _ = nasa_addr.send(NasaRefreshMsg).await;
        }
    }
}

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
impl Handler<NasaRefreshMsg> for NasaActor {
    type Result = Result<()>;
    // Handles messages received by the actor.
    async fn handle(&mut self, _msg: NasaRefreshMsg, _context: &MsgContext<Self>) -> Self::Result {
        let debug_title = "NASA images";
        debug_print!("{debug_title}");
        let mut notification = NotificationAlert {
            title: debug_title.to_string(),
            body: "Checking cache".to_string(),
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

        let now = Local::now().date_naive();
        let today = format!("{}.xml", now.format(DATE_FILE_FMT));
        let cached_metadata = self.app_cache_dir.join(Path::new(&today));
        let text = if cached_metadata.exists() {
            fs::read_to_string(&cached_metadata)
                .with_context(|| "Failed to read cached NASA metadata")?
        } else {
            notification.body = "Fetching data from NASA".to_string();
            notification.status_message = condense_duration(timer.elapsed());
            check_err(check_err(
                self.notification_center
                    .send(NotificationUpdate(notification.clone()))
                    .await
                    .map_err(|e| anyhow!("Failed to send notification update: {e:?}")),
            )?)?;
            total_steps += 1;
            let response = check_err(
                reqwest::get("https://www.nasa.gov/rss/dyn/lg_image_of_the_day.rss")
                    .await
                    .with_context(|| "Failed to get list of NASA images"),
            )?;
            let text = check_err(
                response
                    .text()
                    .await
                    .with_context(|| "Failed to get body as text from NASA images' response"),
            )?;
            check_err(
                fs::write(&cached_metadata, &text)
                    .with_context(|| "Failed to write NASA metadata to cache"),
            )?;
            text
        };
        let images = check_err(
            quick_xml::de::from_str::<NasaFeed>(&text)
                .with_context(|| "Failed to deserialize NASA images' response payload."),
        )?
        .channel
        .item;
        let mut image_list = NasaImageList {
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
            let file_name = self.app_cache_dir.join(format!(
                "{date}.{}",
                Path::new(Url::parse(&item.enclosure.url)?.path())
                    .extension()
                    .ok_or(anyhow!("Failed to find image MIME type from NASA URL."))?
                    .to_string_lossy()
            ));
            if !file_name.exists() {
                let response = check_err(
                    reqwest::get(&item.enclosure.url)
                        .await
                        .with_context(|| "Failed to download image from NASA"),
                )?;
                let item_bin = check_err(
                    response
                        .bytes()
                        .await
                        .with_context(|| "Failed to get bytes of image from response payload"),
                )?;
                let mut file = check_err(
                    fs::File::create(&file_name)
                        .with_context(|| "Failed to create a cache file for NASA image"),
                )?;
                check_err(
                    file.write_all(&item_bin)
                        .with_context(|| "Failed to write NASA image data to cache file"),
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
                            .ok_or(anyhow!("Failed to get filename for cached NASA image")),
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
                        .ends_with("xml"))
                {
                    debug_print!("Deleting outdated cache file {:?}", entry.path());
                    check_err(
                        fs::remove_file(entry.path())
                            .with_context(|| "Failed to delete outdated NASA cache file"),
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

// Creates and spawns the actors in the async system.
pub async fn create_actors(notification_center: Address<NotificationActor>) -> Result<()> {
    // Create actor contexts.
    let nasa_context = MsgContext::new();
    let nasa_addr = nasa_context.address();

    let cache_dir = dirs::cache_dir().ok_or(anyhow!(
        "Failed to detect system cache folder; Is this running on a desktop?"
    ))?;
    let app_cache_dir = cache_dir.join("Daily-Images").join("NASA");
    if !app_cache_dir.exists() {
        fs::create_dir_all(&app_cache_dir)?;
    }

    // Spawn actors.
    let actor = NasaActor::new(nasa_addr, app_cache_dir, notification_center);
    spawn(nasa_context.run(actor));
    Ok(())
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
