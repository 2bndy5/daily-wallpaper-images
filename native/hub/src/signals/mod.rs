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

pub(crate) mod wallpaper;
pub use wallpaper::*;

pub(crate) mod notifications;
pub use notifications::*;
