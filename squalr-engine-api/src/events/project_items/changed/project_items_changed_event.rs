use crate::events::{
    engine_event::{EngineEvent, EngineEventRequest},
    project_items::project_items_event::ProjectItemsEvent,
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProjectItemsChangedEvent {
    pub changed_project_paths: Vec<PathBuf>,
}

impl EngineEventRequest for ProjectItemsChangedEvent {
    fn to_engine_event(&self) -> EngineEvent {
        EngineEvent::ProjectItems(ProjectItemsEvent::ProjectItemsChanged {
            project_items_changed_event: self.clone(),
        })
    }
}
