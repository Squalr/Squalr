use crate::project::project_manifest::ProjectManifest;
use serde::{Deserialize, Serialize};
use squalr_engine_api::structures::projects::{
    built_in_types::project_item_type_directory::ProjectItemTypeDirectory, project_info::ProjectInfo, project_item_type::ProjectItemType,
};
use std::{
    fs::{self, File, OpenOptions},
    path::Path,
};

#[derive(Serialize, Deserialize)]
pub struct Project {
    project_manifest: ProjectManifest,
    project_info: ProjectInfo,
    root: ProjectItemTypeDirectory,
}

impl Project {
    const MANIFEST_FILENAME: &'static str = "manifest.sqlr";
    const TABLE_DIR: &'static str = "table";

    /// Creates a new project and writes it to disk.
    pub fn create_project(path: &Path) -> anyhow::Result<Self> {
        if path.exists() && path.read_dir()?.next().is_some() {
            anyhow::bail!("Cannot create project: directory already contains files.");
        }

        fs::create_dir_all(path)?;
        fs::create_dir(path.join(Self::TABLE_DIR))?;

        let project_info = ProjectInfo::new(path.to_path_buf());
        let project_manifest = ProjectManifest::new(None);
        let root = ProjectItemTypeDirectory::new(path);

        let project = Self {
            project_manifest,
            project_info,
            root,
        };

        project.save(false)?;

        Ok(project)
    }

    /// Loads an existing project from disk.
    pub fn load_project(path: &Path) -> anyhow::Result<Self> {
        let manifest_path = path.join(Self::MANIFEST_FILENAME);
        let table_path = path.join(Self::TABLE_DIR);

        let project_info = ProjectInfo::new(path.to_path_buf());
        let project_manifest = Self::load_project_manifest(&manifest_path)?;
        let root = Self::load_directory(&table_path)?;

        Ok(Self {
            project_info,
            project_manifest,
            root,
        })
    }

    pub fn get_name(&self) -> &str {
        self.project_info.get_name()
    }

    pub fn get_project_manifest(&self) -> &ProjectManifest {
        &self.project_manifest
    }

    pub fn get_project_info(&self) -> &ProjectInfo {
        &self.project_info
    }

    pub fn save(
        &self,
        allow_overwrite: bool,
    ) -> anyhow::Result<()> {
        let manifest_path = self.project_info.get_path().join(Self::MANIFEST_FILENAME);

        if manifest_path.exists() && !allow_overwrite {
            anyhow::bail!("Failed to save project. A project already exists in this directory.");
        }

        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(allow_overwrite)
            .open(&manifest_path)?;

        serde_json::to_writer_pretty(file, &self.project_manifest)?;

        // JIRA: Serialize project items if changed.

        Ok(())
    }

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

    fn load_project_manifest(path: &Path) -> anyhow::Result<ProjectManifest> {
        let file = File::open(path)?;
        let result = serde_json::from_reader(file)?;

        Ok(result)
    }

    fn load_item_file(path: &Path) -> anyhow::Result<Box<dyn ProjectItemType>> {
        let file = File::open(path)?;
        let result = serde_json::from_reader(file)?;

        Ok(result)
    }
}
