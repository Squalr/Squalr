use crate::events::engine_event::EngineEvent;
use crate::inter_process::dispatcher_type::DispatcherType;
use crate::inter_process::inter_process_privileged_shell::InterProcessPrivilegedShell;
use crate::squalr_engine::SqualrEngine;
use uuid::Uuid;

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
            DispatcherType::Standalone => SqualrEngine::broadcast_engine_event(event),
            DispatcherType::InterProcess => InterProcessPrivilegedShell::get_instance().dispatch_event(event, uuid),
            DispatcherType::None => panic!("Event should not be dispatched from an privileged shell."),
        }
    }
}
