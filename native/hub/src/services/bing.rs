//! Any logic that is specific to Bing Image of the Day service goes here.

use super::{ImageServiceActor, UpdateResources, DATE_FILE_FMT};
use crate::{
    common::check_err,
    signals::{DailyImage, ImageList, ImageService},
};
use anyhow::{Context, Result};
use chrono::NaiveDate;
use rinf::RustSignal;
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

fn parse_date(date: &str) -> Result<NaiveDate> {
    NaiveDate::parse_from_str(date, "%Y%m%d")
        .with_context(|| "Failed to parse Bing picture's date.")
}

impl ImageServiceActor {
    pub(super) async fn cache_updates_bing(
        &mut self,
        res: &mut UpdateResources,
    ) -> Result<Vec<String>> {
        let images = self
            .notify_err(
                serde_json::from_str::<BingImages>(&res.text)
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
        res.total_images = images.len();
        let mut image_names = vec![];

        for (i, img) in images.into_iter().enumerate() {
            let date = parse_date(&img.start_date)?.format(DATE_FILE_FMT);

            let name = format!("{date}.jpg");
            image_names.push(name.clone());
            let file_name = res.app_cache_dir.join(&name);
            let cache_path = file_name.to_string_lossy().to_string();
            if !file_name.exists() {
                res.updated_images += 1;
                let result = self
                    .download_file(
                        &res.client,
                        format!("https://bing.com{}", img.url).as_str(),
                        &cache_path,
                        &name,
                        (res.total_steps + res.total_images) as u8,
                        res.notification.clone(),
                    )
                    .await;
                res.downloaded += self.notify_err(result, ImageService::Bing).await?;
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
