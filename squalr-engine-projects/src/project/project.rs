use serde::{Deserialize, Serialize};
use squalr_engine_api::structures::projects::{built_in_types::project_item_type_directory::ProjectItemTypeDirectory, project_item_type::ProjectItemType};
use std::{fs::File, path::Path};

#[derive(Serialize, Deserialize)]
pub struct Project {
    root: ProjectItemTypeDirectory,
}

impl Project {
    pub fn open_project(path: &Path) -> anyhow::Result<Self> {
        Ok(Self {
            root: Self::load_directory(path)?,
        })
    }

    fn load_directory(path: &Path) -> anyhow::Result<ProjectItemTypeDirectory> {
        let mut directory = ProjectItemTypeDirectory::new(path);

        for entry in std::fs::read_dir(path)? {
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
        let boxed: Box<dyn ProjectItemType> = serde_json::from_reader(file)?;

        Ok(boxed)
    }
}
