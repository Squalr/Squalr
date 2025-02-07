use crate::inter_process::inter_process_privileged_shell::InterProcessPrivilegedShell;
use crate::responses::engine_response::EngineResponse;
use crate::responses::response_handler::ResponseHandler;

pub trait ResponseDispatcher {
    fn dispatch_response(
        &self,
        response: EngineResponse,
    );
}

pub enum ResponseDispatcherType {
    Standalone(),
    InterProcess(),
}

impl ResponseDispatcher for ResponseDispatcherType {
    fn dispatch_response(
        &self,
        response: EngineResponse,
    ) {
        match self {
            Self::Standalone() => ResponseHandler::handle_response(response),
            Self::InterProcess() => InterProcessPrivilegedShell::get_instance().dispatch_response(response),
        }
    }
}
