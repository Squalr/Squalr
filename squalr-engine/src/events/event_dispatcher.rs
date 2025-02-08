use uuid::Uuid;

use crate::event_handlers::event_handler::EventHandler;
use crate::events::engine_event::EngineEvent;
use crate::inter_process::dispatcher_type::DispatcherType;
use crate::inter_process::inter_process_privileged_shell::InterProcessPrivilegedShell;

pub struct EventDispatcher {
    dispatcher_type: DispatcherType,
}

impl EventDispatcher {
    pub fn new(dispatcher_type: DispatcherType) -> Self {
        Self { dispatcher_type }
    }

    pub fn dispatch_event(
        &self,
        event: EngineEvent,
        uuid: Uuid,
    ) {
        match self.dispatcher_type {
            DispatcherType::Standalone => EventHandler::handle_event(event, uuid),
            DispatcherType::InterProcess => InterProcessPrivilegedShell::get_instance().dispatch_event(event, uuid),
            DispatcherType::None => panic!("Event should not be dispatched from an privileged shell."),
        }
    }
}
