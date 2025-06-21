//! Any logic that is specific to Windows Spotlight Images service goes here.

use super::{actor::ImageServiceActor, UpdateResources};
use crate::signals::{DailyImage, ImageList, ImageService};
use anyhow::{Context, Result};
use rinf::RustSignal;
use serde::Deserialize;

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
pub struct SpotlightBatchInfo {
    items: Vec<SpotlightItem>,
}

#[derive(Debug, Deserialize)]
pub struct SpotlightImages {
    #[serde(rename(deserialize = "batchrsp"))]
    pub batch_response: SpotlightBatchInfo,
}

impl ImageServiceActor {
    pub(super) async fn cache_updates_spotlight(
        &mut self,
        res: &mut UpdateResources,
    ) -> Result<Vec<String>> {
        let items = self
            .notify_err(
                serde_json::from_str::<SpotlightImages>(&res.text).with_context(|| {
                    "Failed to deserialize Windows Spotlight images' response payload."
                }),
                ImageService::Spotlight,
            )
            .await?
            .batch_response
            .items;
        res.total_images = items.len();

        let mut image_list = ImageList {
            images: vec![],
            service: ImageService::Spotlight,
        };
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
            let file_name = res.app_cache_dir.join(id);
            let cache_path = file_name.as_os_str().to_string_lossy().to_string();
            if !file_name.exists() {
                res.updated_images += 1;
                let result = self
                    .download_file(
                        &res.client,
                        url,
                        &cache_path,
                        id,
                        (res.total_steps + res.total_images) as u8,
                        res.notification.clone(),
                    )
                    .await;
                res.downloaded += self.notify_err(result, ImageService::Spotlight).await?;
            }
            image_list.images[i].url = cache_path;
            // The send method is generated from a marked Protobuf message.
            image_list.send_signal_to_dart();
            res.notification.body = format!("Processed {id}");
            res.notification.percent =
                ((i + res.total_steps) as f32) / ((res.total_images + res.total_steps) as f32);
            self.check_notify_send_error(res.notification.clone())
                .await?;
        }

        let new_ids = new_item_ids.iter().map(|i| i.0.clone()).collect::<Vec<_>>();
        Ok(new_ids)
    }
}
