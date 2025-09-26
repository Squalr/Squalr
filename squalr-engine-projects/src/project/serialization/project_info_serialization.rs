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
    #[serde(rename = "icon")]
    project_icon_rgba: Option<ProcessIcon>,

    /// The manifest for this project, containing the sort order of project items.
    #[serde(rename = "manifest")]
    project_manifest: ProjectManifest,
}

impl SerializableProjectFile for ProjectInfo {
    fn load_from_path(project_file_path: &Path) -> anyhow::Result<Self> {
        let project_dir = project_file_path.parent().unwrap_or_else(|| project_file_path);
        let file = File::open(project_file_path)?;
        let result: ProjectInfoStub = serde_json::from_reader(file)?;

        Ok(ProjectInfo::new(project_dir.to_path_buf(), result.project_icon_rgba, result.project_manifest))
    }

    fn save_to_path(
        &mut self,
        directory: &Path,
        save_even_if_unchanged: bool,
    ) -> anyhow::Result<()> {
        if save_even_if_unchanged || self.get_has_unsaved_changes() {
            let project_file_path = directory.join(Project::PROJECT_FILE);

            let file = OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(&project_file_path)?;

            let project_info_stub = ProjectInfoStub {
                project_icon_rgba: self.get_project_icon_rgba().clone(),
                project_manifest: self.get_project_manifest().clone(),
            };

            serde_json::to_writer(file, &project_info_stub)?;

            self.set_has_unsaved_changes(false);
        }

        Ok(())
    }
}
