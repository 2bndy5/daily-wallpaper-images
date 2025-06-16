use std::{fs::File, io::Write, time::Duration};

use anyhow::{anyhow, Context, Result};
use futures_util::StreamExt;
use messages::prelude::Address;
use reqwest::{header::CONTENT_LENGTH, Client};
use rinf::debug_print;

use crate::{
    common::check_err,
    notification_center::{NotificationActor, NotificationUpdate},
    signals::NotificationAlert,
};

pub mod bing;
pub mod nasa;
pub mod spotlight;

pub async fn download_file(
    client: &Client,
    url: &str,
    cache_path: &str,
    display_id: &str,
    total_steps: u8,
    notification_center: &mut Address<NotificationActor>,
    mut notification: NotificationAlert,
) -> Result<()> {
    let response = client
        .get(url)
        .timeout(Duration::from_secs(5))
        .send()
        .await?;
    let total_size = if let Some(v) = response.headers().get(CONTENT_LENGTH) {
        let as_str = v.to_str()?;
        Some(as_str.to_string().parse::<usize>()?)
    } else {
        debug_print!("Failed to get the content length for image file {display_id}");
        None
    };

    let completed_steps = notification.percent * (total_steps as f32);
    let mut buf: Vec<u8> = Vec::new();
    let mut downloaded = 0;
    notification.body = format!("Downloading {display_id}");
    let mut stream = response.bytes_stream();
    while let Some(chunk) = stream.next().await {
        let bytes =
            chunk.map_err(|e| anyhow!("Failed to get chunk of bytes from stream: {e:?}"))?;
        downloaded += bytes.len();
        buf.extend(bytes);
        if let Some(total_size) = total_size {
            notification.percent = (completed_steps + ((downloaded as f32) / (total_size as f32)))
                / (total_steps as f32);
        }
        check_err(check_err(
            notification_center
                .send(NotificationUpdate(notification.clone()))
                .await
                .map_err(|e| anyhow!("Failed to send notification update: {e:?}")),
        )?)?;
    }
    check_err(
        check_err(
            File::create(cache_path)
                .with_context(|| format!("Failed to create cache file for image: {cache_path}")),
        )?
        .write_all(&buf)
        .with_context(|| format!("Failed to write data to image file {cache_path}")),
    )?;
    Ok(())
}
