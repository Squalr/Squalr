use crate::events::project::changed::project_changed_event::ProjectChangedEvent;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ProjectEvent {
    ProjectChanged { project_changed_event: ProjectChangedEvent },
}
