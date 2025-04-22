use crate::structures::processes::process_icon::ProcessIcon;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProjectInfo {
    project_name: String,
    project_icon_rgba: ProcessIcon,
}

impl ProjectInfo {
    pub fn new(
        project_name: String,
        project_icon_rgba: ProcessIcon,
    ) -> Self {
        Self {
            project_name,
            project_icon_rgba,
        }
    }

    pub fn get_name(&self) -> &str {
        &self.project_name
    }

    pub fn get_icon_rgba(&self) -> &ProcessIcon {
        &self.project_icon_rgba
    }
}
