use thiserror::Error;

#[derive(Debug, Error)]
pub enum ProcessQueryError {
    #[error("Failed to acquire process monitor lock during `{operation}`: {details}.")]
    ProcessMonitorLockPoisoned { operation: &'static str, details: String },
    #[error("Failed to open process with id `{process_id}`.")]
    OpenProcessFailed { process_id: u32 },
    #[error("Failed to close process handle `{handle}`.")]
    CloseProcessFailed { handle: u64 },
    #[error("Operation `{operation}` is not implemented on `{platform}`.")]
    NotImplemented { operation: &'static str, platform: &'static str },
    #[error("Process query operation `{operation}` failed: {details}.")]
    Internal { operation: &'static str, details: String },
}

impl ProcessQueryError {
    pub fn process_monitor_lock_poisoned(
        operation: &'static str,
        details: impl Into<String>,
    ) -> Self {
        Self::ProcessMonitorLockPoisoned {
            operation,
            details: details.into(),
        }
    }

    pub fn not_implemented(
        operation: &'static str,
        platform: &'static str,
    ) -> Self {
        Self::NotImplemented { operation, platform }
    }

    pub fn internal(
        operation: &'static str,
        details: impl Into<String>,
    ) -> Self {
        Self::Internal {
            operation,
            details: details.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ProcessQueryError;

    #[test]
    fn process_monitor_lock_poisoned_error_contains_operation_and_details() {
        let error = ProcessQueryError::process_monitor_lock_poisoned("start_monitoring", "rwlock poisoned");

        assert_eq!(
            error.to_string(),
            "Failed to acquire process monitor lock during `start_monitoring`: rwlock poisoned."
        );
    }

    #[test]
    fn not_implemented_error_contains_operation_and_platform() {
        let error = ProcessQueryError::not_implemented("open_process", "linux");

        assert_eq!(error.to_string(), "Operation `open_process` is not implemented on `linux`.");
    }
}
