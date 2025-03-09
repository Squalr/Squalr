use serde::{Deserialize, Serialize};
use squalr_engine_api::commands::engine_command::EngineCommand;
use std::sync::Arc;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum InterprocessIngress {
    EngineCommand(EngineCommand),
}

pub trait ExecutableRequest<ResponseType, ExecutionContextType> {
    fn execute(
        &self,
        execution_context: &Arc<ExecutionContextType>,
    ) -> ResponseType;
}
