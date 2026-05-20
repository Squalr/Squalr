use crate::structures::projects::{project::Project, project_info::ProjectInfo};
use std::sync::{Arc, RwLock};

/// Session-owned project state required by app command and view code.
pub trait ProjectContext: Send + Sync {
    /// Gets a reference to the shared lock containing the currently opened project.
    fn get_opened_project(&self) -> Arc<RwLock<Option<Project>>>;

    /// Dispatches an engine event indicating that the project items have changed.
    fn notify_project_items_changed(&self);

    /// Dispatches an engine event indicating that a project has been created.
    fn notify_project_created(
        &self,
        project_info: ProjectInfo,
    );

    /// Dispatches an engine event indicating that a project has been deleted.
    fn notify_project_deleted(
        &self,
        project_info: ProjectInfo,
    );

    /// Dispatches an engine event indicating that the opened project has been closed.
    fn notify_project_closed(&self);
}
