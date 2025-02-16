use crate::command_dispatchers::inter_process::inter_process_pipe_unidirectional::InterProcessPipeUnidirectional;
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::io;
use uuid::Uuid;

pub struct InterProcessPipeBidirectional {
    pub pipe_receive: InterProcessPipeUnidirectional,
    pub pipe_send: InterProcessPipeUnidirectional,
}

impl InterProcessPipeBidirectional {
    pub fn create() -> io::Result<Self> {
        let pipe_receive = InterProcessPipeUnidirectional::create(true)?;
        let pipe_send = InterProcessPipeUnidirectional::create(false)?;
        Ok(Self { pipe_receive, pipe_send })
    }

    pub fn bind() -> io::Result<Self> {
        let pipe_send = InterProcessPipeUnidirectional::bind(true)?;
        let pipe_receive = InterProcessPipeUnidirectional::bind(false)?;
        Ok(Self { pipe_receive, pipe_send })
    }

    pub fn send<T: Serialize>(
        &self,
        value: T,
        request_id: Uuid,
    ) -> io::Result<()> {
        self.pipe_send.ipc_send(value, request_id)
    }

    pub fn receive<T: DeserializeOwned>(&self) -> io::Result<(T, Uuid)> {
        self.pipe_receive.ipc_receive()
    }
}
