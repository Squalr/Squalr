use crate::project::project_manager::ProjectManager;
use crate::project::serialization::serializable_project_file::SerializableProjectFile;
use anyhow::{Context, anyhow};
use std::path::Path;

impl ProjectManager {
    pub fn operation_save_project(
        &mut self,
        project_root: &Path,
    ) -> Result<(), anyhow::Error> {
        let opened_project = self.get_opened_project();
        let mut opened_project = opened_project
            .write()
            .map_err(|error| anyhow!("Failed to acquire write lock on opened project: {}", error))?;
        let opened_project = opened_project
            .as_mut()
            .ok_or_else(|| anyhow!("Could not save project, no project is opened"))?;

        opened_project
            .save_to_path(project_root, true)
            .context("Failed to save project to disk")?;

        Ok(())
    }
}
