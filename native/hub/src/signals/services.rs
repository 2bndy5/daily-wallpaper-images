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
    pub fn as_str(&self) -> &str {
        match self {
            ImageService::Bing => "Bing",
            ImageService::Nasa => "NASA",
            ImageService::Spotlight => "Windows Spotlight",
        }
    }
}

#[derive(Debug, DartSignal, Deserialize, Serialize)]
pub struct Refresh(pub ImageService);

#[derive(Debug, DartSignal, Deserialize)]
pub struct Reset(pub ImageService);

#[derive(Debug, RustSignal, Serialize)]
pub struct ImageList {
    pub service: ImageService,
    pub images: Vec<DailyImage>,
}
