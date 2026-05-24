use crate::plugins::{PluginActivationState, PluginMetadata};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PluginState {
    metadata: PluginMetadata,
    is_enabled: bool,
    activation_state: PluginActivationState,
}

impl PluginState {
    pub fn new(
        metadata: PluginMetadata,
        is_enabled: bool,
        activation_state: PluginActivationState,
    ) -> Self {
        Self {
            metadata,
            is_enabled,
            activation_state,
        }
    }

    pub fn get_metadata(&self) -> &PluginMetadata {
        &self.metadata
    }

    pub fn get_is_enabled(&self) -> bool {
        self.is_enabled
    }

    pub fn get_activation_state(&self) -> PluginActivationState {
        self.activation_state
    }

    pub fn get_can_activate_for_current_process(&self) -> bool {
        matches!(
            self.activation_state,
            PluginActivationState::Available | PluginActivationState::Activating | PluginActivationState::Activated
        )
    }

    pub fn get_is_active_for_current_process(&self) -> bool {
        matches!(self.activation_state, PluginActivationState::Activated)
    }
}
