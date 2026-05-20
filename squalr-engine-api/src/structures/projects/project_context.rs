use crate::structures::projects::project::Project;
use std::sync::{Arc, RwLock};

/// Session-owned project state required by app command and view code.
pub trait ProjectContext: Send + Sync {
    /// Gets a reference to the shared lock containing the currently opened project.
    fn get_opened_project(&self) -> Arc<RwLock<Option<Project>>>;

    /// Dispatches an engine event indicating that the project items have changed.
    fn notify_project_items_changed(&self);
}
