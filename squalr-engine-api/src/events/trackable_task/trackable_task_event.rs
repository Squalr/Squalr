use crate::events::trackable_task::progress_changed::trackable_task_progress_changed_event::TrackableTaskProgressChangedEvent;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum TrackableTaskEvent {
    ProgressChanged {
        progress_changed_event: TrackableTaskProgressChangedEvent,
    },
}
