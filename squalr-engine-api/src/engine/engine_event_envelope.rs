use crate::events::engine_event::EngineEvent;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EngineEventEnvelope {
    registry_generation: u64,
    engine_event: EngineEvent,
}

impl EngineEventEnvelope {
    pub fn new(
        registry_generation: u64,
        engine_event: EngineEvent,
    ) -> Self {
        Self {
            registry_generation,
            engine_event,
        }
    }

    pub fn get_registry_generation(&self) -> u64 {
        self.registry_generation
    }

    pub fn into_engine_event(self) -> EngineEvent {
        self.engine_event
    }
}
