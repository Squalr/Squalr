use serde::{Deserialize, Serialize};
use std::marker::PhantomData;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum InterprocessIngress<RequestType: ExecutableRequest<ResponseType>, ResponseType> {
    EngineCommand(RequestType),
    _Phantom(PhantomData<ResponseType>),
}

pub trait ExecutableRequest<ResponseType> {
    fn execute(&self) -> ResponseType;
}
