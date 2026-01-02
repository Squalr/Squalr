use anyhow::anyhow;
use squalr_engine_api::structures::projects::project_manager::ProjectManager;

impl ProjectManager {
    /// Closes the currently opened project.
    pub fn operation_close_project(&self) -> Result<(), anyhow::Error> {}
}
