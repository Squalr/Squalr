use anyhow::anyhow;
use squalr_engine_api::structures::projects::project_manager::ProjectManager;

impl ProjectManager {
    /// Closes the currently opened project.
    pub fn operation_close_project(&self) -> Result<(), anyhow::Error> {
        let opened_project = self.get_opened_project();
        let mut project = opened_project
            .write()
            .map_err(|e| anyhow!("Failed to acquire write lock on opened project: {}", e))?;

        *project = None;

        Ok(())
    }
}
