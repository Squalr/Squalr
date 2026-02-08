use crate::engine_bindings::interprocess::pipes::interprocess_pipe_error::InterprocessPipeError;
use crate::engine_bindings::interprocess::pipes::interprocess_pipe_unidirectional::InterprocessPipeUnidirectional;
use serde::Serialize;
use serde::de::DeserializeOwned;
use uuid::Uuid;

pub struct InterprocessPipeBidirectional {
    pub pipe_receive: InterprocessPipeUnidirectional,
    pub pipe_send: InterprocessPipeUnidirectional,
}

impl InterprocessPipeBidirectional {
    pub fn create() -> Result<Self, InterprocessPipeError> {
        let pipe_receive = InterprocessPipeUnidirectional::create(true)?;
        let pipe_send = InterprocessPipeUnidirectional::create(false)?;
        Ok(Self { pipe_receive, pipe_send })
    }

    pub fn bind() -> Result<Self, InterprocessPipeError> {
        let pipe_send = InterprocessPipeUnidirectional::bind(true)?;
        let pipe_receive = InterprocessPipeUnidirectional::bind(false)?;
        Ok(Self { pipe_receive, pipe_send })
    }

    pub fn send<T: Serialize>(
        &self,
        value: T,
        request_id: Uuid,
    ) -> Result<(), InterprocessPipeError> {
        self.pipe_send.ipc_send(value, request_id)
    }

    pub fn receive<T: DeserializeOwned>(&self) -> Result<(T, Uuid), InterprocessPipeError> {
        self.pipe_receive.ipc_receive()
    }
}
