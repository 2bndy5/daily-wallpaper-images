use super::DailyImage;
use rinf::{DartSignal, RustSignal};
use serde::{Deserialize, Serialize};

#[derive(Debug, DartSignal, Deserialize)]
pub struct SpotlightRefresh;

#[derive(Debug, DartSignal, Deserialize)]
pub struct SpotlightReset;

#[derive(Debug, RustSignal, Serialize)]
pub struct SpotlightImageList {
    pub images: Vec<DailyImage>,
}
