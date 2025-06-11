use rinf::{DartSignal, RustSignal, SignalPiece};
use serde::{Deserialize, Serialize};

#[derive(Debug, DartSignal, Deserialize)]
pub struct BingRefresh;

#[derive(Debug, DartSignal, Deserialize)]
pub struct NasaRefresh;

#[derive(Debug, DartSignal, Deserialize)]
pub struct SpotlightRefresh;

#[derive(Debug, DartSignal, Deserialize)]
pub struct SpotlightReset;

#[derive(Debug, SignalPiece, Serialize)]
pub struct DailyImage {
    pub url: String,
    pub date: String,
    pub description: String,
}

#[derive(Debug, RustSignal, Serialize)]
pub struct NasaImageList {
    pub images: Vec<DailyImage>,
}

#[derive(Debug, RustSignal, Serialize)]
pub struct BingImageList {
    pub images: Vec<DailyImage>,
}

#[derive(Debug, RustSignal, Serialize)]
pub struct SpotlightImageList {
    pub images: Vec<DailyImage>,
}

#[derive(Debug, Deserialize, SignalPiece)]
pub enum WallpaperMode {
    Center = 0,
    Crop = 1,
    Fit = 2,
    Stretch = 3,
    Tile = 4,
}

#[derive(Debug, Deserialize, SignalPiece)]
pub struct WallpaperSelection {
    pub path: String,
    pub mode: WallpaperMode,
}

#[derive(Debug, DartSignal, Deserialize)]
pub struct SetWallpaper {
    pub selected: WallpaperSelection,
}

#[derive(Debug, RustSignal, Serialize)]
pub struct NotificationAlert {
    pub title: String,
    pub body: String,
    pub percent: Option<f32>,
    pub severity: NotificationSeverity,
    pub status_message: String,
}

#[allow(dead_code)]
#[derive(Debug, SignalPiece, Serialize)]
pub enum NotificationSeverity {
    Debug,
    Info,
    Warning,
    Error,
}
