use crate::{responses::engine_response::EngineResponse, squalr_engine::SqualrEngine};
use uuid::Uuid;

pub enum ResponseHandlerType {
    Standalone(),
    InterProcess(),
}

pub struct ResponseHandler {}

impl ResponseHandler {
    pub fn handle_response(
        response: EngineResponse,
        uuid: Uuid,
    ) {
        SqualrEngine::handle_response(response, uuid);
    }
}
