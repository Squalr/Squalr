use uuid::Uuid;

use crate::inter_process::dispatcher_type::DispatcherType;
use crate::inter_process::inter_process_privileged_shell::InterProcessPrivilegedShell;
use crate::response_handlers::response_handler::ResponseHandler;
use crate::responses::engine_response::EngineResponse;

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
            DispatcherType::Standalone => ResponseHandler::handle_response(response, uuid),
            DispatcherType::InterProcess => InterProcessPrivilegedShell::get_instance().dispatch_response(response, uuid),
            DispatcherType::None => panic!("Response should not be dispatched from an unprivileged host."),
        }
    }
}
