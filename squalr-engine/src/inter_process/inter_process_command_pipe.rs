use interprocess::local_socket::ListenerOptions;
use interprocess::local_socket::Name;
use interprocess::local_socket::prelude::LocalSocketStream;
use interprocess::local_socket::traits::ListenerExt;
use interprocess::local_socket::traits::Stream;
use serde::Serialize;
use serde::de::DeserializeOwned;
use squalr_engine_common::logging::log_level::LogLevel;
use squalr_engine_common::logging::logger::Logger;
use std::io;
use std::io::Read;
use std::io::Write;
use std::sync::Arc;
use std::sync::RwLock;
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

use super::inter_process_data_egress::InterProcessDataEgress;
use super::inter_process_data_ingress::InterProcessDataIngress;

#[cfg(windows)]
const IPC_SOCKET_PATH: &str = "\\\\.\\pipe\\squalr-ipc";
#[cfg(all(not(windows), not(target_os = "android")))]
const IPC_SOCKET_PATH: &str = "/tmp/squalr-ipc.sock";
#[cfg(target_os = "android")]
const IPC_SOCKET_PATH: &str = "squalr-ipc";

pub struct InterProcessCommandPipe {}

impl InterProcessCommandPipe {
    /// Creates a single manager connection: effectively "binds" to the socket
    /// (or named pipe on Windows), listens, and accepts exactly one incoming connection.
    pub fn create_inter_process_pipe() -> io::Result<LocalSocketStream> {
        // On Unix-like non-Android systems, remove any leftover socket file
        #[cfg(all(not(windows), not(target_os = "android")))]
        {
            if Path::new(IPC_SOCKET_PATH).exists() {
                fs::remove_file(IPC_SOCKET_PATH)?;
            }
        }

        #[cfg(not(target_os = "android"))]
        let name: Name<'_> = IPC_SOCKET_PATH.to_fs_name::<NamedPipeType>()?;
        #[cfg(target_os = "android")]
        let name: Name<'_> = IPC_SOCKET_PATH.to_ns_name::<NamedPipeType>()?;

        Logger::get_instance().log(LogLevel::Info, "Creating listener...", None);

        // Create the listener using ListenerOptions
        let listener = ListenerOptions::new().name(name).create_sync()?;

        Logger::get_instance().log(LogLevel::Info, &format!("Manager: listening on {}", IPC_SOCKET_PATH), None);

        // Accept one connection
        let stream = match listener.incoming().next() {
            Some(Ok(conn)) => conn,
            Some(Err(e)) => return Err(e),
            None => return Err(io::Error::new(io::ErrorKind::Other, "Manager: no connection arrived.")),
        };

        Logger::get_instance().log(LogLevel::Info, "Manager accepted a connection from worker", None);

        Ok(stream)
    }

    pub fn bind_to_inter_process_pipe() -> io::Result<LocalSocketStream> {
        const MAX_RETRIES: u32 = 256;
        let retry_delay = std::time::Duration::from_millis(100);

        #[cfg(not(target_os = "android"))]
        let name: Name<'_> = IPC_SOCKET_PATH.to_fs_name::<NamedPipeType>()?;
        #[cfg(target_os = "android")]
        let name: Name<'_> = IPC_SOCKET_PATH.to_ns_name::<NamedPipeType>()?;

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

    pub fn ipc_send_to_shell(
        ipc_connection: &Arc<RwLock<Option<LocalSocketStream>>>,
        value: InterProcessDataIngress,
        uuid: Uuid,
    ) -> io::Result<()> {
        Self::ipc_send(ipc_connection, value, uuid)
    }

    pub fn ipc_send_to_host(
        ipc_connection: &Arc<RwLock<Option<LocalSocketStream>>>,
        value: InterProcessDataEgress,
        uuid: Uuid,
    ) -> io::Result<()> {
        Self::ipc_send(ipc_connection, value, uuid)
    }

    pub fn ipc_receive_from_shell(ipc_connection: &Arc<RwLock<Option<LocalSocketStream>>>) -> io::Result<(InterProcessDataEgress, Uuid)> {
        Self::ipc_receive(ipc_connection)
    }

    pub fn ipc_receive_from_host(ipc_connection: &Arc<RwLock<Option<LocalSocketStream>>>) -> io::Result<(InterProcessDataIngress, Uuid)> {
        Self::ipc_receive(ipc_connection)
    }

    /// Sends a value of generic type `T` (which must implement `Serialize`) over the IPC connection.
    /// Returns the serialized bytes on success.
    fn ipc_send<T: Serialize>(
        ipc_connection: &Arc<RwLock<Option<LocalSocketStream>>>,
        value: T,
        uuid: Uuid,
    ) -> io::Result<()> {
        // Serialize the data.
        let serialized_data = bincode::serialize(&value).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, format!("Serialize error: {}", e)))?;

        // Acquire read lock on the connection.
        if let Ok(connection_guard) = ipc_connection.write() {
            if let Some(mut stream) = connection_guard.as_ref() {
                let uuid_bytes = uuid.as_bytes();
                let len = (uuid_bytes.len() + serialized_data.len()) as u32;

                // First send length as u32 in little-endian.
                stream.write_all(&len.to_le_bytes())?;

                // Next send identifier as uuid bytes.
                stream.write_all(uuid_bytes)?;

                // Then send the actual data.
                stream.write_all(&serialized_data)?;
                stream.flush()?;

                Ok(())
            } else {
                Err(io::Error::new(io::ErrorKind::NotConnected, "No IPC connection established"))
            }
        } else {
            Err(io::Error::new(io::ErrorKind::Other, "Failed to acquire connection lock"))
        }
    }

    /// Receives a value of generic type `T` (which must implement `DeserializeOwned`) from the IPC connection.
    fn ipc_receive<T: DeserializeOwned>(ipc_connection: &Arc<RwLock<Option<LocalSocketStream>>>) -> io::Result<(T, Uuid)> {
        // Acquire read lock on the connection
        if let Ok(connection_guard) = ipc_connection.read() {
            if let Some(mut stream) = connection_guard.as_ref() {
                // First read the length (4 bytes).
                let mut len_buf = [0u8; 4];
                stream.read_exact(&mut len_buf)?;
                let total_len = u32::from_le_bytes(len_buf);

                // Next read the uuid (16 bytes).
                let mut uuid_buf = [0u8; 16];
                stream.read_exact(&mut uuid_buf)?;
                let uuid = Uuid::from_bytes(uuid_buf);

                // Finally read the remaining data (total_len - 16 bytes for UUID).
                let data_len = total_len as usize - 16;
                let mut data_buf = vec![0u8; data_len];
                stream.read_exact(&mut data_buf)?;

                // Deserialize the data into T
                let value =
                    bincode::deserialize::<T>(&data_buf).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, format!("Deserialize error: {}", e)))?;

                Ok((value, uuid))
            } else {
                Err(io::Error::new(io::ErrorKind::NotConnected, "No IPC connection established"))
            }
        } else {
            Err(io::Error::new(io::ErrorKind::Other, "Failed to acquire connection lock"))
        }
    }
}
