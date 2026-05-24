use crate::events::{
    engine_event::{EngineEvent, EngineEventRequest},
    registry::registry_event::RegistryEvent,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RegistryChangedEvent {
    pub generation: u64,
}

impl EngineEventRequest for RegistryChangedEvent {
    fn to_engine_event(&self) -> EngineEvent {
        EngineEvent::Registry(RegistryEvent::Changed {
            registry_changed_event: self.clone(),
        })
    }
}
