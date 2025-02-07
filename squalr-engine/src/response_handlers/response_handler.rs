use crate::responses::engine_response::EngineResponse;

pub enum ResponseHandlerType {
    Standalone(),
    InterProcess(),
}

pub struct ResponseHandler {}

impl ResponseHandler {
    pub fn handle_response(response: EngineResponse) {
        //
    }
}
