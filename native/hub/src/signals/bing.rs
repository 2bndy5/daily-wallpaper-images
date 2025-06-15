use super::DailyImage;
use rinf::{DartSignal, RustSignal};
use serde::{Deserialize, Serialize};

#[derive(Debug, DartSignal, Deserialize)]
pub struct BingRefresh;

#[derive(Debug, RustSignal, Serialize)]
pub struct BingImageList {
    pub images: Vec<DailyImage>,
}
