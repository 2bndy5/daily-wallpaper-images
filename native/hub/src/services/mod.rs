mod actor;
pub mod bing;
pub mod nasa;
pub mod settings;
pub mod spotlight;
use std::{path::PathBuf, time::Duration};

pub use actor::{create_actors, ImageServiceActor};
use anyhow::Result;
use chrono::Local;
use reqwest::{Client, ClientBuilder};

use crate::signals::{ImageService, NotificationAlert};

pub fn get_service_url(service: &ImageService) -> &'static str {
    match service {
        ImageService::Bing => "https://www.bing.com/HPImageArchive.aspx?format=js&idx=0&n=14",
        ImageService::Nasa => "https://www.nasa.gov/feeds/iotd-feed/",
        ImageService::Spotlight => "https://fd.api.iris.microsoft.com/v4/api/selection?&placement=88000820&bcnt=4&country=us&locale=en-us&fmt=json",
    }
}

const DATE_FILE_FMT: &str = "%F";
const GENERIC_FILE_FMT: &str = "info.json";

pub fn get_service_metadata_name(service: &ImageService) -> String {
    if service.is_daily() {
        let now = Local::now().date_naive();
        let ext = if matches!(service, ImageService::Nasa) {
            "xml"
        } else {
            "json"
        };
        format!("{}.{ext}", now.format(DATE_FILE_FMT))
    } else {
        GENERIC_FILE_FMT.to_string()
    }
}

struct UpdateResources {
    pub app_cache_dir: PathBuf,
    pub notification: NotificationAlert,
    pub text: String,
    pub client: Client,
    pub downloaded: usize,
    pub updated_images: u8,
    pub total_images: usize,
    pub total_steps: usize,
}

impl UpdateResources {
    pub fn new(app_cache_dir: PathBuf, notification: NotificationAlert) -> Result<Self> {
        Ok(Self {
            app_cache_dir,
            notification,
            text: Default::default(),
            client: ClientBuilder::new().build()?,
            downloaded: Default::default(),
            updated_images: Default::default(),
            total_images: Default::default(),
            total_steps: Default::default(),
        })
    }
}

pub fn condense_duration(duration: Duration) -> String {
    let seconds = duration.as_secs();
    if seconds == 0 {
        let millis = duration.as_millis();
        format!("{millis} ms")
    } else if seconds > 60 {
        format!("{} m", seconds / 60)
    } else {
        format!("{seconds} s")
    }
}
