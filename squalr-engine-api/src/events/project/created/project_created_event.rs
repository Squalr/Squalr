use crate::{
    events::{
        engine_event::{EngineEvent, EngineEventRequest},
        project::project_event::ProjectEvent,
    },
    structures::projects::project_info::ProjectInfo,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProjectCreatedEvent {
    pub project_info: ProjectInfo,
}

impl EngineEventRequest for ProjectCreatedEvent {
    fn to_engine_event(&self) -> EngineEvent {
        EngineEvent::Project(ProjectEvent::ProjectCreated {
            project_created_event: self.clone(),
        })
    }
}
