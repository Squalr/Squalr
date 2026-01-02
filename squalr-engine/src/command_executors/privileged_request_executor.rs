use crate::engine_privileged_state::EnginePrivilegedState;
use serde::Serialize;
use serde::de::DeserializeOwned;
use squalr_engine_api::commands::privileged_command_request::PrivilegedCommandRequest;
use std::sync::Arc;

pub trait PrivilegedCommandRequestExecutor: PrivilegedCommandRequest + Clone + Serialize + DeserializeOwned {
    type ResponseType;

    fn execute(
        &self,
        engine_privileged_state: &Arc<EnginePrivilegedState>,
    ) -> <Self as PrivilegedCommandRequestExecutor>::ResponseType;
}
