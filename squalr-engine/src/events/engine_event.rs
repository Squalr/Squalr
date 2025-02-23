use crate::events::process::process_changed_event::ProcessChangedEvent;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum EngineEvent {
    Process(ProcessChangedEvent),
}
