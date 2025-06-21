use rinf::SignalPiece;
use serde::Serialize;

#[derive(Debug, SignalPiece, Serialize)]
pub struct DailyImage {
    pub url: String,
    pub date: String,
    pub description: String,
}

pub(crate) mod services;
pub use services::*;

#[cfg(not(target_os = "android"))]
pub(crate) mod wallpaper;
#[cfg(not(target_os = "android"))]
pub use wallpaper::*;

pub(crate) mod notifications;
pub use notifications::*;
