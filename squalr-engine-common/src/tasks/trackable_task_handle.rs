use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

#[derive(Serialize, Deserialize, Clone)]
pub struct TrackableTaskHandle {
    pub name: String,
    pub progress: f32,
    pub task_identifier: String,
    #[serde(skip)]
    pub cancellation_token: Arc<AtomicBool>,
}

impl TrackableTaskHandle {
    pub fn cancel(&self) {
        self.cancellation_token.store(true, Ordering::SeqCst);
    }
}
