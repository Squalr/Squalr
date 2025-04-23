use crate::events::{
    engine_event::{EngineEvent, EngineEventRequest},
    project::project_event::ProjectEvent,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProjectChangedEvent {}

impl EngineEventRequest for ProjectChangedEvent {
    fn to_engine_event(&self) -> EngineEvent {
        EngineEvent::Project(ProjectEvent::ProjectChanged {
            project_changed_event: self.clone(),
        })
    }
}
