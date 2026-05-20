use crate::project::serialization::serializable_project_file::SerializableProjectFile;
use squalr_engine_api::structures::projects::project_items::project_item::ProjectItem;
use std::{
    fs::{self, File},
    path::Path,
};

impl SerializableProjectFile for ProjectItem {
    fn save_to_path(
        &mut self,
        project_item_path: &Path,
        save_even_if_unchanged: bool,
    ) -> anyhow::Result<()> {
        if save_even_if_unchanged || self.get_has_unsaved_changes() {
            if !project_item_path.exists() {
                fs::create_dir(&project_item_path)?;
            }

            // Only serialize if this is an actual file. Directories have no serialization logic.
            if project_item_path.is_file() {
                let file = File::create(&project_item_path)?;

                serde_json::to_writer_pretty(file, &self)?;
                self.set_has_unsaved_changes(false);
            }
        }

        Ok(())
    }

    fn load_from_path(project_item_path: &Path) -> anyhow::Result<Self> {
        if project_item_path.exists() {
            if !project_item_path.is_file() {
                Err(anyhow::anyhow!("Unable to load directory item, path is not a file: {:?}", project_item_path))
            } else {
                let file = File::open(project_item_path)?;
                let mut project_item: ProjectItem = serde_json::from_reader(file)?;
                project_item.set_has_unsaved_changes(false);

                Ok(project_item)
            }
        } else {
            Err(anyhow::anyhow!(
                "Unable to load directory item, directory does not exist: {:?}",
                project_item_path
            ))
        }
    }
}
