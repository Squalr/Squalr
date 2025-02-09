use super::process::process_event::ProcessEvent;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum EngineEvent {
    ProcessOpened(ProcessEvent),
}
