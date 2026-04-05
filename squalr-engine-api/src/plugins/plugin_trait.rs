use crate::plugins::{PluginKind, PluginMetadata};

pub trait Plugin: Send + Sync {
    fn metadata(&self) -> &PluginMetadata;

    fn plugin_kind(&self) -> PluginKind {
        self.metadata().get_plugin_kind()
    }
}
