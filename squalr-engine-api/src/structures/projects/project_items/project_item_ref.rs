use serde::{Deserialize, Serialize};
use std::{fmt::Debug, path::PathBuf};

/// Represents a handle to a project item.
#[derive(Clone, Debug, Eq, Hash, Default, Serialize, Deserialize, PartialEq)]
pub struct ProjectItemRef {
    /// The unique path to this project item.
    project_item_path: PathBuf,
}

impl ProjectItemRef {
    pub fn new(project_item_path: PathBuf) -> Self {
        Self { project_item_path }
    }

    pub fn get_project_item_path(&self) -> &PathBuf {
        &self.project_item_path
    }

    pub fn get_file_or_directory_name(&self) -> String {
        self.project_item_path
            .file_name()
            .and_then(|os_str| os_str.to_str())
            .unwrap_or("")
            .to_string()
    }
}
