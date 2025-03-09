use crate::{
    events::{
        engine_event::{EngineEvent, EngineEventRequest},
        process::process_event::ProcessEvent,
    },
    structures::processes::process_info::OpenedProcessInfo,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProcessChangedEvent {
    pub process_info: Option<OpenedProcessInfo>,
}

impl EngineEventRequest for ProcessChangedEvent {
    fn to_engine_event(&self) -> EngineEvent {
        EngineEvent::Process(ProcessEvent::ProcessChanged {
            process_changed_event: self.clone(),
        })
    }
}
