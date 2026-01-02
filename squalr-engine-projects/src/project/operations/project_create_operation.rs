use crate::project::{project::Project, project_manager::ProjectManager, serialization::serializable_project_file::SerializableProjectFile};
use anyhow::Context;
use anyhow::anyhow;
use squalr_engine_api::structures::projects::{
    project_info::ProjectInfo,
    project_items::{built_in_types::project_item_type_directory::ProjectItemTypeDirectory, project_item_ref::ProjectItemRef},
    project_manifest::ProjectManifest,
};
use std::{collections::HashMap, fs, path::PathBuf};

impl ProjectManager {
    pub fn operation_create_project(
        &mut self,
        path: &PathBuf,
    ) -> Result<(), anyhow::Error> {
        let opened_project = self.get_opened_project();
        let mut project = opened_project
            .write()
            .map_err(|error| anyhow!("Failed to acquire write lock on opened project: {}", error))?;

        if project.is_some() {
            return Err(anyhow!("Cannot create new project, a project is already opened"));
        }

        if path.exists()
            && path
                .read_dir()
                .with_context(|| format!("Failed to read directory {:?}", path))?
                .next()
                .is_some()
        {
            return Err(anyhow!("Cannot create project: directory already contains files"));
        }

        fs::create_dir_all(path)?;

        let project_info = ProjectInfo::new(path.to_path_buf(), None, ProjectManifest::default());
        let project_root_ref = ProjectItemRef::new(PathBuf::new());
        let mut project_items = HashMap::new();

        project_items.insert(project_root_ref.clone(), ProjectItemTypeDirectory::new_project_item(&project_root_ref));

        let mut new_project = Project::new(project_info, project_items, project_root_ref);

        new_project.save_to_path(path, true)?;

        *project = Some(new_project);

        Ok(())
    }
}
