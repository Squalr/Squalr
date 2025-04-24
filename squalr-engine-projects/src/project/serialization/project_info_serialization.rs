use crate::project::{project::Project, serialization::serializable_project_file::SerializableProjectFile};
use serde::{Deserialize, Serialize};
use squalr_engine_api::structures::{
    processes::process_icon::ProcessIcon,
    projects::{project_info::ProjectInfo, project_manifest::ProjectManifest},
};
use std::{
    fs::{File, OpenOptions},
    path::Path,
};

/// Represents a condensed version of project info excluding information that we do not want to serialize.
/// Note that #[serde(skip)] is insufficient, as we still want to serialize across commands,
/// So instead we use a small stub that we augment after deserialization.
#[derive(Clone, Debug, Serialize, Deserialize)]
struct ProjectInfoStub {
    /// The process icon associated with this project.
    project_icon_rgba: Option<ProcessIcon>,

    /// The manifest for this project, containing the sort order of project items.
    project_manifest: ProjectManifest,
}

impl SerializableProjectFile for ProjectInfo {
    fn load_from_path(directory: &Path) -> anyhow::Result<Self> {
        let file = File::open(directory)?;
        let result: ProjectInfoStub = serde_json::from_reader(file)?;

        Ok(ProjectInfo::new(directory.to_path_buf(), result.project_icon_rgba, result.project_manifest))
    }

    fn save_to_path(
        &self,
        directory: &Path,
        allow_overwrite: bool,
    ) -> anyhow::Result<()> {
        let project_file_path = directory.join(Project::PROJECT_FILE);

        if project_file_path.exists() && !allow_overwrite {
            anyhow::bail!("Failed to save project info. A project already exists in this directory.");
        }

        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(allow_overwrite)
            .open(&project_file_path)?;

        let project_info_stub = ProjectInfoStub {
            project_icon_rgba: self.get_project_icon_rgba().clone(),
            project_manifest: self.get_project_manifest().clone(),
        };

        serde_json::to_writer_pretty(file, &project_info_stub)?;

        Ok(())
    }
}
