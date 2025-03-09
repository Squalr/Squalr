use crate::engine_bindings::interprocess::pipes::interprocess_pipe_unidirectional::InterProcessPipeUnidirectional;
use serde::Serialize;
use serde::de::DeserializeOwned;
use uuid::Uuid;

pub struct InterProcessPipeBidirectional {
    pub pipe_receive: InterProcessPipeUnidirectional,
    pub pipe_send: InterProcessPipeUnidirectional,
}

impl InterProcessPipeBidirectional {
    pub fn create() -> Result<Self, String> {
        let pipe_receive = InterProcessPipeUnidirectional::create(true)?;
        let pipe_send = InterProcessPipeUnidirectional::create(false)?;
        Ok(Self { pipe_receive, pipe_send })
    }

    pub fn bind() -> Result<Self, String> {
        let pipe_send = InterProcessPipeUnidirectional::bind(true)?;
        let pipe_receive = InterProcessPipeUnidirectional::bind(false)?;
        Ok(Self { pipe_receive, pipe_send })
    }

    pub fn send<T: Serialize>(
        &self,
        value: T,
        request_id: Uuid,
    ) -> Result<(), String> {
        self.pipe_send.ipc_send(value, request_id)
    }

    pub fn receive<T: DeserializeOwned>(&self) -> Result<(T, Uuid), String> {
        self.pipe_receive.ipc_receive()
    }
}
