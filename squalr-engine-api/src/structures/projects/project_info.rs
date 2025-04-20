use crate::structures::projects::manifest::project_manifest::ProjectManifest;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProjectInfo {
    name: String,
    manifest: ProjectManifest,
}

impl ProjectInfo {
    pub fn get_name(&self) -> &str {
        &self.name
    }
}
