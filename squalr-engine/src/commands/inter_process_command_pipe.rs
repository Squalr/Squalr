use crate::commands::engine_command::EngineCommand;
use interprocess::local_socket::ToFsName;
use interprocess::local_socket::prelude::LocalSocketStream;
use interprocess::local_socket::traits::Stream;
use interprocess::os::windows::local_socket::NamedPipe;
use squalr_engine_common::logging::log_level::LogLevel;
use squalr_engine_common::logging::logger::Logger;
use std::io;
use std::io::Read;
use std::io::Write;
use std::sync::Arc;
use std::sync::RwLock;
use std::thread;

const IPC_SOCKET_PATH: &str = if cfg!(windows) { "\\\\.\\pipe\\squalr-ipc" } else { "/tmp/squalr-ipc.sock" };

pub struct InterProcessCommandPipe {}

impl InterProcessCommandPipe {
    pub fn create_connection() -> io::Result<LocalSocketStream> {
        // Attempt to connect to the new child process in a loop, rather than sleeping once.
        const MAX_RETRIES: u32 = 256;
        let retry_delay = std::time::Duration::from_millis(100);
        let name = IPC_SOCKET_PATH.to_fs_name::<NamedPipe>().unwrap();

        for attempt in 1..=MAX_RETRIES {
            thread::sleep(retry_delay);
            match LocalSocketStream::connect(name.clone()) {
                Ok(stream) => {
                    Logger::get_instance().log(LogLevel::Info, &format!("Squalr successfully connected to privileged local server!",), None);
                    return Ok(stream);
                }
                Err(e) => {
                    Logger::get_instance().log(
                        LogLevel::Info,
                        &format!(
                            "Squalr privileged local server connection failed, attempt {}/{}. Error: {}. Retrying...",
                            attempt, MAX_RETRIES, e
                        ),
                        None,
                    );
                }
            }
        }

        Err(io::Error::new(io::ErrorKind::Other, "Failed to create IPC connection!"))
    }

    pub fn ipc_send_command(
        ipc_connection: &Arc<RwLock<Option<LocalSocketStream>>>,
        command: &mut EngineCommand,
    ) -> io::Result<Vec<u8>> {
        let encoded: Vec<u8> = bincode::serialize(&command).unwrap();

        if let Ok(connection) = ipc_connection.read() {
            if let Some(mut stream) = connection.as_ref() {
                // Write message length first (as u32)
                let len = encoded.len() as u32;
                stream.write_all(&len.to_le_bytes())?;

                // Write the actual message
                stream.write_all(&encoded)?;
                stream.flush()?;

                Ok(encoded)
            } else {
                Err(io::Error::new(io::ErrorKind::NotConnected, "No IPC connection established"))
            }
        } else {
            Err(io::Error::new(io::ErrorKind::Other, "Failed to acquire connection lock"))
        }
    }

    pub fn ipc_listen_command(ipc_connection: &Arc<RwLock<Option<LocalSocketStream>>>) -> io::Result<EngineCommand> {
        if let Ok(connection) = ipc_connection.read() {
            if let Some(mut stream) = connection.as_ref() {
                // Read response length
                let mut len_buf = [0u8; 4];
                stream.read_exact(&mut len_buf)?;
                let response_len = u32::from_le_bytes(len_buf);

                // Read response
                let mut response = vec![0u8; response_len as usize];
                stream.read_exact(&mut response)?;

                match bincode::deserialize::<EngineCommand>(&response) {
                    Ok(engine_command) => {
                        return Ok(engine_command);
                    }
                    Err(err) => {
                        return Err(io::Error::new(io::ErrorKind::Other, format!("{}", err)));
                    }
                }
            } else {
                Err(io::Error::new(io::ErrorKind::NotConnected, "No IPC connection established"))
            }
        } else {
            Err(io::Error::new(io::ErrorKind::Other, "Failed to acquire connection lock"))
        }
    }
}
