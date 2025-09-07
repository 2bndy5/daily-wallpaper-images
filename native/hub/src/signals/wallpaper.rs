use rinf::{DartSignal, RustSignal, SignalPiece};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, SignalPiece, Serialize, Clone, Copy)]
pub enum WallpaperMode {
    Center = 0,
    Crop = 1,
    Fit = 2,
    Stretch = 3,
    Tile = 4,
}

#[derive(Debug, DartSignal, RustSignal, Deserialize, Serialize)]
pub struct WallpaperModeCache {
    pub mode: Option<WallpaperMode>,
}

#[cfg(not(target_os = "android"))]
#[derive(Debug, Deserialize, SignalPiece)]
pub struct WallpaperSelection {
    pub path: String,
    pub mode: Option<WallpaperMode>,
}

#[cfg(not(target_os = "android"))]
#[derive(Debug, DartSignal, Deserialize)]
pub struct SetWallpaper {
    pub selected: WallpaperSelection,
}
