use crate::events::process::process_changed_event::ProcessChangedEvent;
use serde::{Deserialize, Serialize};

use super::trackable_task::trackable_task_event::TrackableTaskEvent;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum EngineEvent {
    Process(ProcessChangedEvent),
    TrackableTask(TrackableTaskEvent),
}
