use crate::events::project::changed::project_changed_event::ProjectChangedEvent;
use crate::events::project::created::project_created_event::ProjectCreatedEvent;
use crate::events::project::deleted::project_deleted_event::ProjectDeletedEvent;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ProjectEvent {
    ProjectChanged { project_changed_event: ProjectChangedEvent },
    ProjectCreated { project_created_event: ProjectCreatedEvent },
    ProjectDeleted { project_deleted_event: ProjectDeletedEvent },
}
