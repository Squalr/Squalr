use crate::project::{project::Project, serialization::serializable_project_file::SerializableProjectFile};
use olorin_engine_api::structures::projects::{project_info::ProjectInfo, project_items::project_item::ProjectItem};
use std::path::Path;

impl SerializableProjectFile for Project {
    fn load_from_path(directory: &Path) -> anyhow::Result<Self> {
        let project_info = ProjectInfo::load_from_path(&directory.join(Project::PROJECT_FILE))?;
        let project_root = ProjectItem::load_from_path(&directory.join(Project::PROJECT_DIR))?;

        Ok(Project::new(project_info, project_root))
    }

    fn save_to_path(
        &mut self,
        directory: &Path,
        save_even_if_unchanged: bool,
    ) -> anyhow::Result<()> {
        // Save the main project file.
        self.get_project_info_mut()
            .save_to_path(directory, save_even_if_unchanged)?;

        // Recursively save all project items.
        self.get_project_root_mut()
            .save_to_path(&directory.join(Project::PROJECT_DIR), save_even_if_unchanged)?;

        Ok(())
    }
}
