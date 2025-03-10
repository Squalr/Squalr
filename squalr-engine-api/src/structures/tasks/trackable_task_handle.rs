use serde::{Deserialize, Serialize};

/// An identifier for a task running in the engine. Coupled with engine commands, this handle can be used to cancel tasks.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TrackableTaskHandle {
    pub name: String,
    pub progress: f32,
    pub task_identifier: String,
}

impl TrackableTaskHandle {}
