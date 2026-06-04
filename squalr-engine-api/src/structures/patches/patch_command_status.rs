use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PatchCommandStatus {
    success: bool,
    message: Option<String>,
}

impl PatchCommandStatus {
    pub fn success() -> Self {
        Self { success: true, message: None }
    }

    pub fn failure(message: impl Into<String>) -> Self {
        Self {
            success: false,
            message: Some(message.into()),
        }
    }

    pub fn get_success(&self) -> bool {
        self.success
    }

    pub fn get_message(&self) -> Option<&str> {
        self.message.as_deref()
    }
}
