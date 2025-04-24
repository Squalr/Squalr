use crate::events::project::closed::project_closed_event::ProjectClosedEvent;
use crate::events::project::created::project_created_event::ProjectCreatedEvent;
use crate::events::project::deleted::project_deleted_event::ProjectDeletedEvent;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ProjectEvent {
    ProjectClosed { project_closed_event: ProjectClosedEvent },
    ProjectCreated { project_created_event: ProjectCreatedEvent },
    ProjectDeleted { project_deleted_event: ProjectDeletedEvent },
}
