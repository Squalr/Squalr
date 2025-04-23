use crate::structures::processes::process_icon::ProcessIcon;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProjectInfo {
    project_name: String,
    project_path: PathBuf,
    project_icon_rgba: ProcessIcon,
}

impl ProjectInfo {
    pub fn new(
        project_path: PathBuf,
        project_icon_rgba: ProcessIcon,
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
        }
    }

    pub fn get_name(&self) -> &str {
        &self.project_name
    }

    pub fn get_path(&self) -> &PathBuf {
        &self.project_path
    }

    pub fn get_icon_rgba(&self) -> &ProcessIcon {
        &self.project_icon_rgba
    }
}
