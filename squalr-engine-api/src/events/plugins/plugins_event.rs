use crate::events::plugins::changed::plugins_changed_event::PluginsChangedEvent;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum PluginsEvent {
    PluginsChanged { plugins_changed_event: PluginsChangedEvent },
}
