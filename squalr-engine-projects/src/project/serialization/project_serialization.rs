use crate::project::{project::Project, serialization::serializable_project_file::SerializableProjectFile};
use squalr_engine_api::structures::projects::{
    project_info::ProjectInfo, project_items::built_in_types::project_item_type_directory::ProjectItemTypeDirectory,
};
use std::path::Path;

impl SerializableProjectFile for Project {
    fn load_from_path(directory: &Path) -> anyhow::Result<Self> {
        let project_info = ProjectInfo::load_from_path(directory)?;
        let project_root = ProjectItemTypeDirectory::new(&directory.join(Project::TABLE_DIR));

        Ok(Project::new(project_info, project_root))
    }

    fn save_to_path(
        &self,
        directory: &Path,
        allow_overwrite: bool,
    ) -> anyhow::Result<()> {
        self.get_project_info().save_to_path(directory, allow_overwrite)
    }

    /*
    fn load_directory(table_path: &Path) -> anyhow::Result<ProjectItemTypeDirectory> {
        let mut directory = ProjectItemTypeDirectory::new(table_path);

        for entry in fs::read_dir(table_path)? {
            let entry = entry?;
            let entry_path = entry.path();

            if entry_path.is_dir() {
                directory.append_child(Box::new(Self::load_directory(&entry_path)?));
            } else {
                directory.append_child(Self::load_item_file(&entry_path)?);
            }
        }

        Ok(directory)
    }

    fn load_item_file(path: &Path) -> anyhow::Result<Box<dyn ProjectItemType>> {
        let file = File::open(path)?;
        let result = serde_json::from_reader(file)?;

        Ok(result)
    }*/
}
