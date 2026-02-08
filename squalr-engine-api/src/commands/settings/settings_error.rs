use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Clone, Debug, Serialize, Deserialize, Error, Eq, PartialEq)]
pub enum SettingsError {
    #[error("Failed to read {settings_scope} settings.")]
    ReadFailure { settings_scope: String },
}

impl SettingsError {
    pub fn read_failure(settings_scope: impl Into<String>) -> Self {
        Self::ReadFailure {
            settings_scope: settings_scope.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::SettingsError;

    #[test]
    fn read_failure_formats_scope() {
        let error = SettingsError::read_failure("memory");

        assert_eq!(error.to_string(), "Failed to read memory settings.");
    }
}
