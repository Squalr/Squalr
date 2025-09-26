use crate::engine_privileged_state::EnginePrivilegedState;
use serde::Serialize;
use serde::de::DeserializeOwned;
use squalr_engine_api::commands::engine_command_request::EngineCommandRequest;
use std::sync::Arc;

pub trait EngineCommandRequestExecutor: EngineCommandRequest + Clone + Serialize + DeserializeOwned {
    type ResponseType;

    fn execute(
        &self,
        engine_privileged_state: &Arc<EnginePrivilegedState>,
    ) -> <Self as EngineCommandRequestExecutor>::ResponseType;
}
