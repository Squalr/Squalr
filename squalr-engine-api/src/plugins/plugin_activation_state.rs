use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum PluginActivationState {
    Idle,
    Available,
    Activating,
    Activated,
}
