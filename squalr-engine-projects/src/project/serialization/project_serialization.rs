use crate::project::{project::Project, serialization::serializable_project_item::SerializableProjectItem};
use serde::{Deserialize, Serialize};
use squalr_engine_api::structures::projects::{
    project_info::ProjectInfo, project_items::built_in_types::project_item_type_directory::ProjectItemTypeDirectory,
};
use std::{
    fs::{File, OpenOptions},
    path::Path,
};

/// Represents a condensed version of a project excluding information that we do not want to serialize.
/// This is done because project items themselves are serialized separately, not as a single item.
#[derive(Clone, Debug, Serialize, Deserialize)]
struct ProjectStub {
    project_info: ProjectInfo,
}

impl SerializableProjectItem for Project {
    fn load_from_path(directory: &Path) -> anyhow::Result<Self> {
        let file = File::open(directory)?;
        let result: ProjectStub = serde_json::from_reader(file)?;
        let project_root = ProjectItemTypeDirectory::new(&directory.join(Project::TABLE_DIR));

        Ok(Project::new(result.project_info, project_root))
    }

    fn save_to_path(
        &self,
        directory: &Path,
        allow_overwrite: bool,
    ) -> anyhow::Result<()> {
        if directory.exists() && !allow_overwrite {
            anyhow::bail!("Failed to save project. A project already exists in this directory.");
        }

        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(allow_overwrite)
            .open(&directory)?;

        let project_info_stub = ProjectStub {
            project_info: self.get_project_info().clone(),
        };

        serde_json::to_writer_pretty(file, &project_info_stub)?;

        Ok(())
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
