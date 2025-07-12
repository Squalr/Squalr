use crate::events::{
    engine_event::{EngineEvent, EngineEventRequest},
    project::project_event::ProjectEvent,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProjectClosedEvent {}

impl EngineEventRequest for ProjectClosedEvent {
    fn to_engine_event(&self) -> EngineEvent {
        EngineEvent::Project(ProjectEvent::ProjectClosed {
            project_closed_event: self.clone(),
        })
    }
}
