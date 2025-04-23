use crate::events::{
    engine_event::{EngineEvent, EngineEventRequest},
    project::project_event::ProjectEvent,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProjectDeletedEvent {}

impl EngineEventRequest for ProjectDeletedEvent {
    fn to_engine_event(&self) -> EngineEvent {
        EngineEvent::Project(ProjectEvent::ProjectDeleted {
            project_deleted_event: self.clone(),
        })
    }
}
