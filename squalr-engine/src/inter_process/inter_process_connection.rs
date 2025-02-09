use interprocess::local_socket::prelude::LocalSocketStream;

pub struct InterProcessConnection {
    pub socket_stream: Option<LocalSocketStream>,
}

impl InterProcessConnection {
    pub fn new() -> Self {
        Self { socket_stream: None }
    }

    pub fn set_socket_stream(
        &mut self,
        socket_stream: LocalSocketStream,
    ) {
        self.socket_stream = Some(socket_stream);
    }
}
