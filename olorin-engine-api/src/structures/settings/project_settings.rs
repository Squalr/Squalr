use directories::UserDirs;
use serde::{Deserialize, Serialize};
use serde_json::to_string_pretty;
use std::fmt;
use std::path::PathBuf;

#[derive(Clone, Deserialize, Serialize)]
pub struct ProjectSettings {
    pub projects_root: PathBuf,
    pub project_update_interval_ms: u64,
}

impl fmt::Debug for ProjectSettings {
    fn fmt(
        &self,
        formatter: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        match to_string_pretty(&self) {
            Ok(json) => write!(formatter, "Settings for project: {}", json),
            Err(_) => write!(formatter, "Project config {{ could not serialize to JSON }}"),
        }
    }
}

impl Default for ProjectSettings {
    fn default() -> Self {
        let projects_root = UserDirs::new()
            .and_then(|dirs| Some(dirs.document_dir()?.join("Olorin")))
            .unwrap_or_else(|| PathBuf::from("./Olorin"));

        Self {
            projects_root,
            project_update_interval_ms: 200,
        }
    }
}
