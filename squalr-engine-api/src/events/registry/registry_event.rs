use crate::events::registry::changed::registry_changed_event::RegistryChangedEvent;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum RegistryEvent {
    Changed { registry_changed_event: RegistryChangedEvent },
}
