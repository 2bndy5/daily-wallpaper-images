#![cfg(not(target_os = "android"))]

use rinf::{DartSignal, SignalPiece};
use serde::Deserialize;

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
    pub mode: Option<WallpaperMode>,
}

#[derive(Debug, DartSignal, Deserialize)]
pub struct SetWallpaper {
    pub selected: WallpaperSelection,
}
