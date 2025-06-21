use std::collections::HashMap;

use rinf::{DartSignal, RustSignal, SignalPiece};
use serde::{Deserialize, Serialize};

#[derive(Debug, DartSignal, Deserialize)]
pub struct NotificationRefresh;

#[derive(Debug, DartSignal, Deserialize)]
pub struct NotificationDismiss {
    pub timestamp: String,
}

#[derive(Debug, DartSignal, Deserialize)]
pub struct NotificationDismissAll;

#[derive(Debug, RustSignal, Serialize, Default)]
pub struct NotificationResults {
    pub notifications: HashMap<String, NotificationAlert>,
    pub pending: Vec<String>,
    pub just_finished: Vec<String>,
}

#[derive(Debug, SignalPiece, Deserialize, Serialize, Clone)]
pub struct NotificationAlert {
    pub title: String,
    pub body: String,
    pub percent: f32,
    pub severity: NotificationSeverity,
    pub status: NotificationStatus,
}

impl NotificationAlert {
    /// Moves everything from `other` (except `title`) into self (overwriting current state).
    pub(crate) fn update(&mut self, other: Self) {
        self.body = other.body;
        self.percent = other.percent;
        self.severity = other.severity;
        self.status = other.status;
    }
}

#[derive(Debug, SignalPiece, Deserialize, Serialize, Clone, Copy)]
pub enum NotificationSeverity {
    Debug,
    Info,
    Warning,
    Error,
}

#[derive(Debug, SignalPiece, Deserialize, Serialize, Clone, Default)]
pub struct NotificationStatus {
    pub downloaded: Option<String>,
    pub removed: Option<u8>,
    pub elapsed: Option<String>,
}
