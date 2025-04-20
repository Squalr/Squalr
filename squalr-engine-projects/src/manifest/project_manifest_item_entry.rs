use squalr_engine_api::structures::settings::project_settings::ProjectSettings;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

pub struct ProjectManifestItemEntry {
    config: Arc<RwLock<ProjectSettings>>,
    config_file: PathBuf,
}
