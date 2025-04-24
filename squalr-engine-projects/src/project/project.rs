use serde::{Deserialize, Serialize};
use squalr_engine_api::structures::projects::{
    project_info::ProjectInfo, project_items::built_in_types::project_item_type_directory::ProjectItemTypeDirectory, project_manifest::ProjectManifest,
};
use std::{
    fs::{self},
    path::Path,
};

use super::serialization::serializable_project_item::SerializableProjectItem;

#[derive(Serialize, Deserialize)]
pub struct Project {
    project_info: ProjectInfo,
    root: ProjectItemTypeDirectory,
}

impl Project {
    pub const TABLE_DIR: &'static str = "table";

    pub fn new(
        project_info: ProjectInfo,
        root: ProjectItemTypeDirectory,
    ) -> Self {
        Self { project_info, root }
    }

    /// Creates a new project and writes it to disk.
    pub fn create_new_project(path: &Path) -> anyhow::Result<Self> {
        if path.exists() && path.read_dir()?.next().is_some() {
            anyhow::bail!("Cannot create project: directory already contains files.");
        }

        fs::create_dir_all(path)?;
        fs::create_dir(path.join(Self::TABLE_DIR))?;

        let project_info = ProjectInfo::new(path.to_path_buf(), None, ProjectManifest::default());
        let root: ProjectItemTypeDirectory = ProjectItemTypeDirectory::new(path);

        let project = Self { project_info, root };

        project.save_to_path(path, false)?;

        Ok(project)
    }

    pub fn get_name(&self) -> &str {
        self.project_info.get_name()
    }

    pub fn get_project_manifest(&self) -> &ProjectManifest {
        &self.project_info.get_project_manifest()
    }

    pub fn get_project_info(&self) -> &ProjectInfo {
        &self.project_info
    }
}
