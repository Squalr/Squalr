use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct TrackableTaskHandle {
    pub name: String,
    pub progress: f32,
    pub task_identifier: String,
}
