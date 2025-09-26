use crate::events::process::changed::process_changed_event::ProcessChangedEvent;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ProcessEvent {
    ProcessChanged { process_changed_event: ProcessChangedEvent },
}
