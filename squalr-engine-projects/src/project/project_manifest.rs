use serde::{Deserialize, Serialize};
use squalr_engine_api::structures::processes::process_icon::ProcessIcon;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProjectManifest {
    project_icon_rgba: Option<ProcessIcon>,
}

impl ProjectManifest {
    pub fn new(project_icon_rgba: Option<ProcessIcon>) -> Self {
        Self { project_icon_rgba }
    }

    pub fn get_icon_rgba(&self) -> &Option<ProcessIcon> {
        &self.project_icon_rgba
    }
}
