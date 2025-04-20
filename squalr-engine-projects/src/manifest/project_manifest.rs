use crate::manifest::project_manifest_item_entry::ProjectManifestItemEntry;
use std::sync::{Arc, RwLock};

pub struct ProjectManifest {
    items: Arc<RwLock<Vec<ProjectManifestItemEntry>>>,
}

impl ProjectManifest {
    pub fn new() -> Self {
        Self {
            items: Arc::new(RwLock::new(vec![])),
        }
    }
}
