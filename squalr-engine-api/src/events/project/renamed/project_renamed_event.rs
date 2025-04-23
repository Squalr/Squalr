use crate::events::{
    engine_event::{EngineEvent, EngineEventRequest},
    project::project_event::ProjectEvent,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProjectRenamedEvent {}

impl EngineEventRequest for ProjectRenamedEvent {
    fn to_engine_event(&self) -> EngineEvent {
        EngineEvent::Project(ProjectEvent::ProjectRenamed {
            project_renamed_event: self.clone(),
        })
    }
}
