use crate::project::{project::Project, serialization::serializable_project_file::SerializableProjectFile};
use olorin_engine_api::structures::projects::{project_info::ProjectInfo, project_items::project_item::ProjectItem};
use std::path::Path;

impl SerializableProjectFile for Project {
    fn load_from_path(directory: &Path) -> anyhow::Result<Self> {
        let project_info = ProjectInfo::load_from_path(&directory.join(Project::PROJECT_FILE))?;
        let project_root = ProjectItem::load_from_path(&directory.join(Project::TABLE_DIR))?;

        Ok(Project::new(project_info, project_root))
    }

    fn save_to_path(
        &mut self,
        directory: &Path,
        allow_overwrite: bool,
        save_changed_only: bool,
    ) -> anyhow::Result<()> {
        // Save the main project file.
        self.get_project_info_mut()
            .save_to_path(directory, allow_overwrite, save_changed_only)?;

        // Recursively save all project items.
        self.get_project_root_mut()
            .save_to_path(&directory.join(Project::TABLE_DIR), allow_overwrite, save_changed_only)?;

        Ok(())
    }
}
