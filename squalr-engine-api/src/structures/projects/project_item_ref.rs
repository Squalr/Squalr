use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Represents a unique reference to a project item in an opened project.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProjectItemRef {
    /// The unique path to this project item.
    path: PathBuf,

    /// The name of this project item.
    name: String,

    /// An optional displayed memory address for this project item.
    address: Option<String>,

    /// The preview value shown for this project item.
    preview: Option<String>,

    /// The child project items underneath this project item.
    children: Vec<ProjectItemRef>,

    /// A value indicating whether this project item accepts children.
    is_container_type: bool,

    /// A value indicating whether this item has been activated / enabled.
    is_activated: bool,

    /// JIRA: Add support for property fields.

    /// A tooltip explaining this project item.
    tooltip: Option<String>,
}

impl ProjectItemRef {
    pub fn get_name(&self) -> &str {
        &self.name
    }
}
