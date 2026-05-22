use crate::events::{
    engine_event::{EngineEvent, EngineEventRequest},
    project::project_event::ProjectEvent,
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProjectCatalogChangedEvent {
    pub changed_project_directory_path: Option<PathBuf>,
}

impl EngineEventRequest for ProjectCatalogChangedEvent {
    fn to_engine_event(&self) -> EngineEvent {
        EngineEvent::Project(ProjectEvent::ProjectCatalogChanged {
            project_catalog_changed_event: self.clone(),
        })
    }
}
