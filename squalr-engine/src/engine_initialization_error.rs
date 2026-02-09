use squalr_engine_api::engine::engine_binding_error::EngineBindingError;
use squalr_engine_session::os::ProcessQueryError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum EngineInitializationError {
    #[error("Failed to acquire privileged engine bindings write lock while {context}: {details}.")]
    PrivilegedBindingsLockFailed { context: &'static str, details: String },
    #[error("Failed to initialize privileged engine bindings while {context}: {source}.")]
    PrivilegedBindingsInitializeFailed {
        context: &'static str,
        #[source]
        source: EngineBindingError,
    },
    #[error("Failed to start process monitoring during privileged engine bootstrap: {source}.")]
    ProcessMonitoringStartFailed {
        #[source]
        source: ProcessQueryError,
    },
    #[error("Failed to spawn privileged CLI process for unprivileged host startup: {source}.")]
    SpawnPrivilegedCliFailed {
        #[source]
        source: std::io::Error,
    },
    #[error("Failed to bind unprivileged host IPC channel during startup: {source}.")]
    BindUnprivilegedIpcFailed {
        #[source]
        source: EngineBindingError,
    },
}

impl EngineInitializationError {
    pub fn privileged_bindings_lock_failed(
        context: &'static str,
        details: impl Into<String>,
    ) -> Self {
        Self::PrivilegedBindingsLockFailed {
            context,
            details: details.into(),
        }
    }

    pub fn privileged_bindings_initialize_failed(
        context: &'static str,
        source: EngineBindingError,
    ) -> Self {
        Self::PrivilegedBindingsInitializeFailed { context, source }
    }

    pub fn process_monitoring_start_failed(source: ProcessQueryError) -> Self {
        Self::ProcessMonitoringStartFailed { source }
    }

    pub fn spawn_privileged_cli_failed(source: std::io::Error) -> Self {
        Self::SpawnPrivilegedCliFailed { source }
    }

    pub fn bind_unprivileged_ipc_failed(source: EngineBindingError) -> Self {
        Self::BindUnprivilegedIpcFailed { source }
    }
}
