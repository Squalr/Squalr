use crate::project::serialization::serializable_project_file::SerializableProjectFile;
use olorin_engine_api::structures::projects::project_items::{
    built_in_types::project_item_type_directory::ProjectItemTypeDirectory, project_item::ProjectItem,
};
use std::{
    fs::{self, File},
    path::Path,
};

impl SerializableProjectFile for ProjectItem {
    fn load_from_path(directory: &Path) -> anyhow::Result<Self> {
        if directory.exists() {
            let mut directory_item = ProjectItemTypeDirectory::new_project_item(directory);

            for entry in fs::read_dir(directory)? {
                let entry_path = entry?.path();

                if entry_path.is_dir() {
                    if let Ok(child_directory) = ProjectItem::load_from_path(&entry_path) {
                        directory_item.append_child(child_directory);
                    }
                } else {
                    let file = File::open(entry_path)?;
                    let result = serde_json::from_reader(file)?;

                    directory_item.append_child(result);
                }
            }

            Ok(directory_item)
        } else {
            Err(anyhow::anyhow!("Unable to load directory item, directory does not exist: {:?}", directory))
        }
    }

    fn save_to_path(
        &mut self,
        directory: &Path,
        save_even_if_unchanged: bool,
    ) -> anyhow::Result<()> {
        if save_even_if_unchanged || self.get_has_unsaved_changes() {
            if !directory.exists() {
                fs::create_dir(&directory)?;
            }

            for child in self.get_children_mut() {
                let child_path = child.get_path();

                if child.get_is_container_type() {
                    fs::create_dir(child_path)?;
                } else {
                    // Save individual file item
                    let file = File::create(&child_path)?;
                    serde_json::to_writer_pretty(file, &child)?;
                }
            }
        }

        Ok(())
    }
}
