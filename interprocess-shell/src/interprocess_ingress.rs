use serde::{Deserialize, Serialize};
use std::{marker::PhantomData, sync::Arc};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum InterprocessIngress<RequestType: ExecutableRequest<ResponseType, ExecutionContextType>, ResponseType, ExecutionContextType> {
    EngineCommand(RequestType),
    _Phantom(PhantomData<(ResponseType, ExecutionContextType)>),
}

pub trait ExecutableRequest<ResponseType, ExecutionContextType> {
    fn execute(
        &self,
        execution_context: &Arc<ExecutionContextType>,
    ) -> ResponseType;
}
