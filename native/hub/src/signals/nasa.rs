use super::DailyImage;
use rinf::{DartSignal, RustSignal};
use serde::{Deserialize, Serialize};

#[derive(Debug, DartSignal, Deserialize)]
pub struct NasaRefresh;

#[derive(Debug, RustSignal, Serialize)]
pub struct NasaImageList {
    pub images: Vec<DailyImage>,
}
