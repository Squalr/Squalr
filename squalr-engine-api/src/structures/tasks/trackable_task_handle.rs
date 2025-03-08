use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TrackableTaskHandle {
    pub name: String,
    pub progress: f32,
    pub task_identifier: String,
}

impl TrackableTaskHandle {
    pub fn cancel(&self) {}
}
