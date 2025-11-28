use interprocess::local_socket::prelude::LocalSocketStream;
use interprocess::local_socket::traits::ListenerExt;
use interprocess::local_socket::traits::Stream;
use interprocess::local_socket::ListenerOptions;
use interprocess::local_socket::Name;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::fs;
use std::io::Read;
use std::io::Write;
use std::path::Path;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
use uuid::Uuid;

#[cfg(not(target_os = "android"))]
use interprocess::local_socket::ToFsName;
#[cfg(target_os = "android")]
use interprocess::local_socket::ToNsName;

#[cfg(all(not(windows), not(target_os = "android")))]
use interprocess::local_socket::GenericFilePath as NamedPipeType;
#[cfg(target_os = "android")]
use interprocess::local_socket::GenericNamespaced as NamedPipeType;
#[cfg(windows)]
use interprocess::os::windows::local_socket::NamedPipe as NamedPipeType;

#[cfg(windows)]
const IPC_SOCKET_PATH_TO_SHELL: &str = "\\\\.\\pipe\\interprocess-shell-to-shell";
#[cfg(all(not(windows), not(target_os = "android")))]
const IPC_SOCKET_PATH_TO_SHELL: &str = "/tmp/interprocess-shell-to-shell.sock";
#[cfg(target_os = "android")]
const IPC_SOCKET_PATH_TO_SHELL: &str = "interprocess-shell-to-shell";

#[cfg(windows)]
const IPC_SOCKET_PATH_OUTBOND: &str = "\\\\.\\pipe\\interprocess-shell-from-shell";
#[cfg(all(not(windows), not(target_os = "android")))]
const IPC_SOCKET_PATH_OUTBOND: &str = "/tmp/interprocess-shell-from-shell.sock";
#[cfg(target_os = "android")]
const IPC_SOCKET_PATH_OUTBOND: &str = "interprocess-shell-from-shell";

pub struct InterprocessPipeUnidirectional {
    socket_stream: Arc<Mutex<Option<LocalSocketStream>>>,
}

impl InterprocessPipeUnidirectional {
    pub fn create(to_shell: bool) -> Result<Self, String> {
        match Self::create_interprocess_pipe(to_shell) {
            Ok(socket_stream) => Ok(Self {
                socket_stream: Arc::new(Mutex::new(Some(socket_stream))),
            }),
            Err(error) => Err(error),
        }
    }

    pub fn bind(to_shell: bool) -> Result<Self, String> {
        match Self::bind_to_interprocess_pipe(to_shell) {
            Ok(socket_stream) => Ok(Self {
                socket_stream: Arc::new(Mutex::new(Some(socket_stream))),
            }),
            Err(error) => Err(error),
        }
    }

    /// Creates a single manager connection: effectively "binds" to the socket
    /// (or named pipe on Windows), listens, and accepts exactly one incoming connection.
    fn create_interprocess_pipe(to_shell: bool) -> Result<LocalSocketStream, String> {
        let ipc_socket_path = if to_shell { IPC_SOCKET_PATH_TO_SHELL } else { IPC_SOCKET_PATH_OUTBOND };

        // On Unix-like non-Android systems, remove any leftover socket file
        #[cfg(all(not(windows), not(target_os = "android")))]
        {
            if Path::new(ipc_socket_path).exists() {
                fs::remove_file(ipc_socket_path).map_err(|e| e.to_string());
            }
        }

        #[cfg(not(target_os = "android"))]
        let name: Name<'_> = match ipc_socket_path.to_fs_name::<NamedPipeType>() {
            Ok(name) => name,
            Err(error) => {
                return Err(error.to_string());
            }
        };

        #[cfg(target_os = "android")]
        let name: Name<'_> = match ipc_socket_path.to_ns_name::<NamedPipeType>() {
            Ok(name) => name,
            Err(error) => {
                return Err(error.to_string());
            }
        };

        // Create the listener using ListenerOptions
        let listener = match ListenerOptions::new().name(name).create_sync() {
            Ok(listener) => listener,
            Err(error) => {
                return Err(error.to_string());
            }
        };

        // Accept one connection
        let stream = match listener.incoming().next() {
            Some(Ok(conn)) => conn,
            Some(Err(error)) => return Err(error.to_string()),
            None => return Err("Manager: no connection arrived.".to_string()),
        };

        Ok(stream)
    }

    fn bind_to_interprocess_pipe(to_shell: bool) -> Result<LocalSocketStream, String> {
        const MAX_RETRIES: u32 = 256;
        let ipc_socket_path = if to_shell { IPC_SOCKET_PATH_TO_SHELL } else { IPC_SOCKET_PATH_OUTBOND };
        let retry_delay = std::time::Duration::from_millis(100);

        #[cfg(not(target_os = "android"))]
        let name: Name<'_> = match ipc_socket_path.to_fs_name::<NamedPipeType>() {
            Ok(name) => name,
            Err(error) => {
                return Err(error.to_string());
            }
        };

        #[cfg(target_os = "android")]
        let name: Name<'_> = match ipc_socket_path.to_ns_name::<NamedPipeType>() {
            Ok(name) => name,
            Err(error) => {
                return Err(error.to_string());
            }
        };

        for _attempt in 1..=MAX_RETRIES {
            thread::sleep(retry_delay);
            match LocalSocketStream::connect(name.clone()) {
                Ok(stream) => {
                    return Ok(stream);
                }
                Err(_) => {
                    // Ignore, we will keep retrying up to the max retries.
                }
            }
        }

        Err("Failed to create IPC connection!".to_string())
    }

    /// Sends a value of generic type `T` (which must implement `Serialize`) over the IPC connection.
    /// Returns the serialized bytes on success.
    pub fn ipc_send<T: Serialize>(
        &self,
        value: T,
        request_id: Uuid,
    ) -> Result<(), String> {
        // Serialize the data.
        let serialized_data = bincode::serialize(&value).map_err(|error| format!("Serialize error: {}", error))?;

        // Acquire write lock on the connection to send in a thread-safe manner.
        if let Ok(stream) = self.socket_stream.lock() {
            if let Some(mut stream) = stream.as_ref() {
                let request_id_bytes = request_id.as_bytes();
                let len = (request_id_bytes.len() + serialized_data.len()) as u32;

                // First send length as u32 in little-endian.
                stream
                    .write_all(&len.to_le_bytes())
                    .map_err(|error| format!("Write length error: {}", error))?;

                // Next send identifier as uuid bytes.
                stream
                    .write_all(request_id_bytes)
                    .map_err(|error| format!("Write request id error: {}", error))?;

                // Then send the actual data.
                stream
                    .write_all(&serialized_data)
                    .map_err(|error| format!("Write serialized data error: {}", error))?;

                stream
                    .flush()
                    .map_err(|error| format!("Flush error: {}", error))?;

                Ok(())
            } else {
                Err("No stream set, failed to send data.".to_string())
            }
        } else {
            Err("Failed to acquire connection lock.".to_string())
        }
    }

    /// Receives a value of generic type `T` (which must implement `DeserializeOwned`) from the IPC connection.
    pub fn ipc_receive<T: DeserializeOwned>(&self) -> Result<(T, Uuid), String> {
        // Acquire read lock on the connection
        if let Ok(stream) = self.socket_stream.lock() {
            if let Some(mut stream) = stream.as_ref() {
                // First read the length (4 bytes).
                let mut len_buf = [0u8; size_of::<u32>()];
                stream
                    .read_exact(&mut len_buf)
                    .map_err(|error| format!("Read length error: {}", error))?;
                let total_len = u32::from_le_bytes(len_buf);

                // Next read the uuid (16 bytes).
                let mut request_id_buf = [0u8; size_of::<Uuid>()];
                stream
                    .read_exact(&mut request_id_buf)
                    .map_err(|error| format!("Read request id error: {}", error))?;
                let request_id = Uuid::from_bytes(request_id_buf);

                // Finally read the remaining data (total_len - request_id size).
                let data_len = total_len as usize - size_of::<Uuid>();
                let mut data_buf = vec![0u8; data_len];
                stream
                    .read_exact(&mut data_buf)
                    .map_err(|error| format!("Read data error: {}", error))?;

                // Deserialize the data into T
                let value = bincode::deserialize::<T>(&data_buf).map_err(|error| format!("Deserialize error: {}", error))?;

                Ok((value, request_id))
            } else {
                Err("No stream set, failed to send data.".to_string())
            }
        } else {
            Err("Failed to acquire connection lock".to_string())
        }
    }
}
