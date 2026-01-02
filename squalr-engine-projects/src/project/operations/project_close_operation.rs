use crate::project::project_manager::ProjectManager;
use anyhow::anyhow;

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
