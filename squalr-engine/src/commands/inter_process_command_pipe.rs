use interprocess::local_socket::ListenerOptions;
use interprocess::local_socket::ToFsName;
use interprocess::local_socket::prelude::LocalSocketStream;
use interprocess::local_socket::traits::ListenerExt;
use interprocess::local_socket::traits::Stream;
use interprocess::os::windows::local_socket::NamedPipe;
use serde::Serialize;
use serde::de::DeserializeOwned;
use squalr_engine_common::logging::log_level::LogLevel;
use squalr_engine_common::logging::logger::Logger;
use std::fs;
use std::io;
use std::io::Read;
use std::io::Write;
use std::path::Path;
use std::sync::Arc;
use std::sync::RwLock;
use std::thread;

const IPC_SOCKET_PATH: &str = if cfg!(windows) { "\\\\.\\pipe\\squalr-ipc" } else { "/tmp/squalr-ipc.sock" };

pub struct InterProcessCommandPipe {}

impl InterProcessCommandPipe {
    /// Creates a single manager connection: effectively "binds" to the socket
    /// (or named pipe on Windows), listens, and accepts exactly one incoming connection.
    pub fn create_manager() -> io::Result<LocalSocketStream> {
        // On Unix-like systems, remove any leftover socket file to avoid "address in use" errors.
        if cfg!(not(windows)) {
            let path = Path::new(IPC_SOCKET_PATH);
            if path.exists() {
                fs::remove_file(path)?;
            }
        }

        // Convert the path to the correct representation (NamedPipe on Windows or filesystem path on Unix).
        let name = IPC_SOCKET_PATH.to_fs_name::<NamedPipe>()?;

        // Create the listener (server) by using ListenerOptions.
        // The older `LocalSocketListener::bind(...)` function does not exist in new versions.
        let listener = ListenerOptions::new().name(name).create_sync()?; // creates a synchronous listener

        Logger::get_instance().log(LogLevel::Info, &format!("Manager: listening on {}", IPC_SOCKET_PATH), None);

        // Accept one connection. The `incoming()` method returns an iterator over incoming connections.
        // We'll simply grab the first one (or return an error if none arrives).
        let stream = match listener.incoming().next() {
            Some(Ok(conn)) => conn,
            Some(Err(e)) => return Err(e),
            None => return Err(io::Error::new(io::ErrorKind::Other, "Manager: no connection arrived.")),
        };

        Logger::get_instance().log(LogLevel::Info, "Manager accepted a connection from worker", None);

        Ok(stream)
    }

    pub fn create_worker() -> io::Result<LocalSocketStream> {
        // Attempt to connect to the new child process in a loop, rather than sleeping once.
        const MAX_RETRIES: u32 = 256;
        let retry_delay = std::time::Duration::from_millis(100);
        let name = IPC_SOCKET_PATH.to_fs_name::<NamedPipe>().unwrap();

        for attempt in 1..=MAX_RETRIES {
            thread::sleep(retry_delay);
            match LocalSocketStream::connect(name.clone()) {
                Ok(stream) => {
                    Logger::get_instance().log(LogLevel::Info, "Squalr successfully connected to privileged local server!", None);
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

    /// Sends a value of generic type `T` (which must implement `Serialize`) over the IPC connection.
    /// Returns the serialized bytes on success.
    pub fn ipc_send<T: Serialize>(
        ipc_connection: &Arc<RwLock<Option<LocalSocketStream>>>,
        value: &T,
    ) -> io::Result<Vec<u8>> {
        // Serialize the data
        let encoded = bincode::serialize(value).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, format!("Serialize error: {}", e)))?;

        // Acquire read lock on the connection
        if let Ok(connection_guard) = ipc_connection.read() {
            if let Some(mut stream) = connection_guard.as_ref() {
                // First send length as u32 in little-endian
                let len = encoded.len() as u32;
                stream.write_all(&len.to_le_bytes())?;

                // Then send the actual data
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

    /// Receives a value of generic type `T` (which must implement `DeserializeOwned`) from the IPC connection.
    pub fn ipc_receive<T: DeserializeOwned>(ipc_connection: &Arc<RwLock<Option<LocalSocketStream>>>) -> io::Result<T> {
        // Acquire read lock on the connection
        if let Ok(connection_guard) = ipc_connection.read() {
            if let Some(mut stream) = connection_guard.as_ref() {
                // Read the length first (4 bytes)
                let mut len_buf = [0u8; 4];
                stream.read_exact(&mut len_buf)?;
                let response_len = u32::from_le_bytes(len_buf);

                // Read the exact number of bytes specified by `response_len`
                let mut response_data = vec![0u8; response_len as usize];
                stream.read_exact(&mut response_data)?;

                // Deserialize the data into T
                let value =
                    bincode::deserialize::<T>(&response_data).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, format!("Deserialize error: {}", e)))?;

                Ok(value)
            } else {
                Err(io::Error::new(io::ErrorKind::NotConnected, "No IPC connection established"))
            }
        } else {
            Err(io::Error::new(io::ErrorKind::Other, "Failed to acquire connection lock"))
        }
    }
}
