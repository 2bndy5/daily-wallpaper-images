use super::DailyImage;
use rinf::{DartSignal, RustSignal, SignalPiece};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, SignalPiece, Serialize, Clone, Copy)]
pub enum ImageService {
    Bing,
    Nasa,
    Spotlight,
}

impl ImageService {
    pub fn as_str(&self) -> &'static str {
        match self {
            ImageService::Bing => "Bing",
            ImageService::Nasa => "NASA",
            ImageService::Spotlight => "Windows Spotlight",
        }
    }

    pub fn is_daily(&self) -> bool {
        match self {
            ImageService::Bing => true,
            ImageService::Nasa => true,
            ImageService::Spotlight => false,
        }
    }
}

#[derive(Debug, DartSignal, Deserialize, Serialize)]
pub struct Refresh {
    pub service: ImageService,
    pub reset: bool,
}

#[derive(Debug, RustSignal, Serialize)]
pub struct ImageList {
    pub service: ImageService,
    pub images: Vec<DailyImage>,
}
