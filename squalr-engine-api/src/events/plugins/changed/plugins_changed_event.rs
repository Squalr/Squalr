use crate::events::{
    engine_event::{EngineEvent, EngineEventRequest},
    plugins::plugins_event::PluginsEvent,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PluginsChangedEvent {}

impl EngineEventRequest for PluginsChangedEvent {
    fn to_engine_event(&self) -> EngineEvent {
        EngineEvent::Plugins(PluginsEvent::PluginsChanged {
            plugins_changed_event: self.clone(),
        })
    }
}
