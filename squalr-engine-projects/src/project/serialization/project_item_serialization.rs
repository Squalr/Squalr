use crate::project::serialization::serializable_project_file::SerializableProjectFile;
use squalr_engine_api::structures::projects::project_items::{
    built_in_types::project_item_type_directory::ProjectItemTypeDirectory, project_item_type::ProjectItemType,
};
use std::{
    fs::{self, File},
    path::Path,
};

impl SerializableProjectFile for ProjectItemTypeDirectory {
    fn load_from_path(directory: &Path) -> anyhow::Result<Self> {
        if directory.exists() {
            let mut directory_item = ProjectItemTypeDirectory::new(directory);

            for entry in fs::read_dir(directory)? {
                let entry_path = entry?.path();

                if entry_path.is_dir() {
                    if let Ok(child_directory) = ProjectItemTypeDirectory::load_from_path(&entry_path) {
                        directory_item.append_child(Box::new(child_directory));
                    }
                } else {
                    let file = File::open(entry_path)?;
                    let result: Box<dyn ProjectItemType> = serde_json::from_reader(file)?;

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
        allow_overwrite: bool,
        save_changed_only: bool,
    ) -> anyhow::Result<()> {
        if save_changed_only && self.get_has_unsaved_changes() {
            if directory.exists() {
                if !allow_overwrite {
                    anyhow::bail!("Failed to save directory item, the directory already exists: {:?}", directory);
                }
            } else {
                fs::create_dir(&directory)?;
            }

            if let Ok(mut children) = self.get_children().write() {
                for child in children.iter_mut() {
                    let child_path = directory.join(child.get_name());

                    if let Some(directory_child) = child.as_any_mut().downcast_mut::<ProjectItemTypeDirectory>() {
                        // Recursive call for subdirectory
                        directory_child.save_to_path(&child_path, allow_overwrite, save_changed_only)?;
                    } else {
                        // Save individual file item
                        let file = File::create(&child_path)?;
                        serde_json::to_writer_pretty(file, &child)?;
                    }
                }
            }
        }

        Ok(())
    }
}
