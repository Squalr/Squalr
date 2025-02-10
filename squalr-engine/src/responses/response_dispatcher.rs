use crate::inter_process::dispatcher_type::DispatcherType;
use crate::inter_process::inter_process_privileged_shell::InterProcessPrivilegedShell;
use crate::responses::engine_response::EngineResponse;
use crate::squalr_engine::SqualrEngine;
use uuid::Uuid;

pub struct ResponseDispatcher {
    dispatcher_type: DispatcherType,
}

impl ResponseDispatcher {
    pub fn new(dispatcher_type: DispatcherType) -> Self {
        Self { dispatcher_type }
    }

    pub fn dispatch_response(
        &self,
        response: EngineResponse,
        uuid: Uuid,
    ) {
        match self.dispatcher_type {
            DispatcherType::Standalone => SqualrEngine::handle_response(response, uuid),
            DispatcherType::InterProcess => InterProcessPrivilegedShell::get_instance().dispatch_response(response, uuid),
            DispatcherType::None => panic!("Response should not be dispatched from an unprivileged host."),
        }
    }
}
