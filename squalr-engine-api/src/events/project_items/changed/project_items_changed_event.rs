use crate::{
    events::{
        engine_event::{EngineEvent, EngineEventRequest},
        project_items::project_items_event::ProjectItemsEvent,
    },
    structures::projects::project_items::project_item::ProjectItem,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProjectItemsChangedEvent {
    pub project_root: Option<ProjectItem>,
}

impl EngineEventRequest for ProjectItemsChangedEvent {
    fn to_engine_event(&self) -> EngineEvent {
        EngineEvent::ProjectItems(ProjectItemsEvent::ProjectItemsChanged {
            project_items_changed_event: self.clone(),
        })
    }
}
