use thiserror::Error;

#[derive(Debug, Error)]
pub enum InterprocessPipeError {
    #[error("Failed to remove stale IPC socket at '{socket_path}': {source}.")]
    StaleSocketCleanupFailed {
        socket_path: &'static str,
        #[source]
        source: std::io::Error,
    },
    #[error("Failed to construct IPC socket name for '{socket_path}': {details}.")]
    SocketNameCreationFailed { socket_path: &'static str, details: String },
    #[error("Failed to create IPC listener for '{socket_path}': {details}.")]
    ListenerCreationFailed { socket_path: &'static str, details: String },
    #[error("IPC listener for '{socket_path}' did not receive an incoming connection.")]
    MissingIncomingConnection { socket_path: &'static str },
    #[error("Incoming IPC connection failed for '{socket_path}': {details}.")]
    IncomingConnectionFailed { socket_path: &'static str, details: String },
    #[error("Failed to connect to IPC socket '{socket_path}' after {attempt_count} attempts.")]
    ConnectRetriesExhausted { socket_path: &'static str, attempt_count: u32 },
    #[error("Failed to serialize IPC payload: {source}.")]
    PayloadSerializationFailed {
        #[source]
        source: bincode::Error,
    },
    #[error("Failed to deserialize IPC payload: {source}.")]
    PayloadDeserializationFailed {
        #[source]
        source: bincode::Error,
    },
    #[error("Failed to acquire IPC stream lock: {details}.")]
    StreamLockFailed { details: String },
    #[error("IPC stream is not initialized.")]
    StreamUnavailable,
    #[error("Failed while {operation}: {source}.")]
    IoOperationFailed {
        operation: &'static str,
        #[source]
        source: std::io::Error,
    },
    #[error("Received invalid IPC frame length ({frame_length}) smaller than request header length ({header_length}).")]
    InvalidFrameLength { frame_length: u32, header_length: usize },
}
