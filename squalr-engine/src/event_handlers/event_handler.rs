use uuid::Uuid;

use crate::events::engine_event::EngineEvent;

pub enum EventHandlerType {
    Standalone(),
    InterProcess(),
}

pub struct EventHandler {}

impl EventHandler {
    pub fn handle_event(
        event: EngineEvent,
        uuid: Uuid,
    ) {
        //
    }
}
