//! Any logic that is specific to NASA Image of the Day service goes here.

use std::path::Path;

use super::{actor::ImageServiceActor, UpdateResources, DATE_FILE_FMT};
use crate::{
    common::check_err,
    signals::{DailyImage, ImageList, ImageService},
};
use anyhow::{anyhow, Context, Result};
use chrono::NaiveDate;
use reqwest::Url;
use rinf::RustSignal;
use serde::Deserialize;

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

impl ImageServiceActor {
    pub(super) async fn cache_updates_nasa(
        &mut self,
        res: &mut UpdateResources,
    ) -> Result<Vec<String>> {
        let images = self
            .notify_err(
                quick_xml::de::from_str::<NasaFeed>(&res.text)
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
        res.total_images = images.len();

        let mut image_names = vec![];
        for (i, item) in images.into_iter().enumerate() {
            let date = parse_date(&item.pub_date[5..16])?.format(DATE_FILE_FMT);
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
            image_names.push(name.clone());
            let file_name = res.app_cache_dir.join(&name);
            let cache_path = file_name.to_string_lossy().to_string();
            if !file_name.exists() {
                res.updated_images += 1;
                let result = self
                    .download_file(
                        &res.client,
                        &item.enclosure.url,
                        &cache_path,
                        &name,
                        (res.total_steps + res.total_images) as u8,
                        res.notification.clone(),
                    )
                    .await;
                res.downloaded += self.notify_err(result, ImageService::Nasa).await?;
            }
            image_list.images[i].url = file_name.to_string_lossy().to_string();
            image_list.send_signal_to_dart();
            res.notification.body = format!("Processed {date}");
            res.notification.percent =
                ((i + res.total_steps) as f32) / ((res.total_images + res.total_steps) as f32);
            self.check_notify_send_error(res.notification.clone())
                .await?;
        }
        Ok(image_names)
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
