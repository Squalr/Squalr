use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct DebuggerPluginError {
    plugin_id: String,
    message: String,
}

impl DebuggerPluginError {
    pub fn new(
        plugin_id: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            plugin_id: plugin_id.into(),
            message: message.into(),
        }
    }

    pub fn get_plugin_id(&self) -> &str {
        &self.plugin_id
    }

    pub fn get_message(&self) -> &str {
        &self.message
    }
}

impl Display for DebuggerPluginError {
    fn fmt(
        &self,
        formatter: &mut Formatter<'_>,
    ) -> std::fmt::Result {
        write!(formatter, "{}: {}", self.plugin_id, self.message)
    }
}

impl std::error::Error for DebuggerPluginError {}
