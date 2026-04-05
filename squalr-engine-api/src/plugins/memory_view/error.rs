use thiserror::Error;

#[derive(Debug, Error)]
pub enum MemoryViewPluginError {
    #[error("Memory-view plugin `{plugin_id}` is not yet implemented: {feature_name}.")]
    NotYetImplemented { plugin_id: String, feature_name: String },
    #[error("Memory-view plugin `{plugin_id}` failed: {message}.")]
    Message { plugin_id: String, message: String },
}

impl MemoryViewPluginError {
    pub fn not_yet_implemented(
        plugin_id: impl Into<String>,
        feature_name: impl Into<String>,
    ) -> Self {
        Self::NotYetImplemented {
            plugin_id: plugin_id.into(),
            feature_name: feature_name.into(),
        }
    }

    pub fn message(
        plugin_id: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self::Message {
            plugin_id: plugin_id.into(),
            message: message.into(),
        }
    }
}
