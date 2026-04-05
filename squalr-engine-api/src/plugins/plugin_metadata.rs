use crate::plugins::PluginKind;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PluginMetadata {
    plugin_id: String,
    display_name: String,
    description: String,
    plugin_kind: PluginKind,
    is_built_in: bool,
}

impl PluginMetadata {
    pub fn new(
        plugin_id: impl Into<String>,
        display_name: impl Into<String>,
        description: impl Into<String>,
        plugin_kind: PluginKind,
        is_built_in: bool,
    ) -> Self {
        Self {
            plugin_id: plugin_id.into(),
            display_name: display_name.into(),
            description: description.into(),
            plugin_kind,
            is_built_in,
        }
    }

    pub fn get_plugin_id(&self) -> &str {
        &self.plugin_id
    }

    pub fn get_display_name(&self) -> &str {
        &self.display_name
    }

    pub fn get_description(&self) -> &str {
        &self.description
    }

    pub fn get_plugin_kind(&self) -> PluginKind {
        self.plugin_kind
    }

    pub fn get_is_built_in(&self) -> bool {
        self.is_built_in
    }
}
