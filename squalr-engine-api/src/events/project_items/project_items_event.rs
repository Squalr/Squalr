use crate::events::project_items::changed::project_items_changed_event::ProjectItemsChangedEvent;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ProjectItemsEvent {
    ProjectItemsChanged { project_items_changed_event: ProjectItemsChangedEvent },
}
