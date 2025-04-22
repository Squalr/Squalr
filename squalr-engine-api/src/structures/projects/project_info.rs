use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProjectInfo {
    project_name: String,
}

impl ProjectInfo {
    pub fn new(project_name: String) -> Self {
        Self { project_name }
    }
    pub fn get_name(&self) -> &str {
        &self.project_name
    }
}
