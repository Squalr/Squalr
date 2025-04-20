use crate::structures::projects::manifest::project_manifest_item_entry::ProjectManifestItemEntry;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProjectManifest {
    items: Vec<ProjectManifestItemEntry>,
}

impl ProjectManifest {
    pub fn new() -> Self {
        Self { items: vec![] }
    }
}
