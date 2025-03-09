use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TrackableTaskProgressChangedEvent {
    pub task_id: String,
    pub progress: f32,
}
