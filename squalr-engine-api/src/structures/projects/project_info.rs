use crate::structures::processes::process_icon::ProcessIcon;
use crate::structures::projects::project_manifest::ProjectManifest;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProjectInfo {
    /// The name of this project. This is derived from the folder containing the project json.
    project_name: String,

    /// The path of this project.
    project_path: PathBuf,

    /// The process icon associated with this project.
    project_icon_rgba: Option<ProcessIcon>,

    /// The manifest for this project, containing the sort order of project items.
    project_manifest: ProjectManifest,
}

impl ProjectInfo {
    pub fn new(
        project_path: PathBuf,
        project_icon_rgba: Option<ProcessIcon>,
        project_manifest: ProjectManifest,
    ) -> Self {
        let project_name = project_path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        Self {
            project_name,
            project_path,
            project_icon_rgba,
            project_manifest,
        }
    }

    pub fn get_name(&self) -> &str {
        &self.project_name
    }

    pub fn get_path(&self) -> &PathBuf {
        &self.project_path
    }

    pub fn get_project_icon_rgba(&self) -> &Option<ProcessIcon> {
        &self.project_icon_rgba
    }

    pub fn get_project_manifest(&self) -> &ProjectManifest {
        &self.project_manifest
    }
}
