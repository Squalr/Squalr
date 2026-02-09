use serde::Serialize;
use serde::de::DeserializeOwned;
use squalr_engine_api::commands::unprivileged_command_request::UnprivilegedCommandRequest;
use squalr_engine_api::engine::engine_execution_context::EngineExecutionContext;
use std::sync::Arc;

pub trait UnprivilegedCommandRequestExecutor: UnprivilegedCommandRequest + Clone + Serialize + DeserializeOwned {
    type ResponseType;

    fn execute(
        &self,
        engine_unprivileged_state: &Arc<dyn EngineExecutionContext>,
    ) -> <Self as UnprivilegedCommandRequestExecutor>::ResponseType;
}
