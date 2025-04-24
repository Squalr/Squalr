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
        &self,
        directory: &Path,
        allow_overwrite: bool,
    ) -> anyhow::Result<()> {
        if directory.exists() {
            if !allow_overwrite {
                anyhow::bail!("Failed to save directory item, the directory already exists: {:?}", directory);
            }
        } else {
            fs::create_dir(directory)?;
        }

        if let Ok(children) = self.get_children().read().as_deref() {
            for child in children {
                let child_path = directory.join(child.get_name());

                if let Some(directory_child) = child.as_any().downcast_ref::<ProjectItemTypeDirectory>() {
                    // Recursive call for subdirectory
                    directory_child.save_to_path(&child_path, allow_overwrite)?;
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
