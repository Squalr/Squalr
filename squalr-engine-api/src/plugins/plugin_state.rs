use crate::plugins::PluginMetadata;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PluginState {
    metadata: PluginMetadata,
    is_enabled: bool,
    can_activate_for_current_process: bool,
    is_active_for_current_process: bool,
}

impl PluginState {
    pub fn new(
        metadata: PluginMetadata,
        is_enabled: bool,
        can_activate_for_current_process: bool,
        is_active_for_current_process: bool,
    ) -> Self {
        Self {
            metadata,
            is_enabled,
            can_activate_for_current_process,
            is_active_for_current_process,
        }
    }

    pub fn get_metadata(&self) -> &PluginMetadata {
        &self.metadata
    }

    pub fn get_is_enabled(&self) -> bool {
        self.is_enabled
    }

    pub fn get_can_activate_for_current_process(&self) -> bool {
        self.can_activate_for_current_process
    }

    pub fn get_is_active_for_current_process(&self) -> bool {
        self.is_active_for_current_process
    }
}
