use std::error::Error;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum EngineBindingError {
    #[error("Engine binding unavailable while {context}.")]
    Unavailable { context: &'static str },
    #[error("Engine binding lock failure while {context}: {details}.")]
    LockFailure { context: &'static str, details: String },
    #[error("Engine binding operation failed while {context}: {source}.")]
    OperationFailed {
        context: &'static str,
        #[source]
        source: Box<dyn Error + Send + Sync>,
    },
}

impl EngineBindingError {
    pub fn unavailable(context: &'static str) -> Self {
        Self::Unavailable { context }
    }

    pub fn lock_failure(
        context: &'static str,
        details: impl Into<String>,
    ) -> Self {
        Self::LockFailure {
            context,
            details: details.into(),
        }
    }

    pub fn operation_failed(
        context: &'static str,
        source: impl Error + Send + Sync + 'static,
    ) -> Self {
        Self::OperationFailed {
            context,
            source: Box::new(source),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::EngineBindingError;

    #[test]
    fn unavailable_error_contains_operation_context() {
        let error = EngineBindingError::unavailable("subscribing to engine events");

        assert_eq!(error.to_string(), "Engine binding unavailable while subscribing to engine events.");
    }

    #[test]
    fn operation_failed_preserves_source_text() {
        let source_error = std::io::Error::other("socket disconnected");
        let error = EngineBindingError::operation_failed("sending IPC response", source_error);
        let rendered_error = error.to_string();

        assert!(rendered_error.contains("sending IPC response"));
        assert!(rendered_error.contains("socket disconnected"));
    }
}
