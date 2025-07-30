use olorin_engine_api::structures::settings::project_settings::ProjectSettings;
use serde_json::to_string_pretty;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Once;
use std::sync::{Arc, RwLock};

pub struct ProjectSettingsConfig {
    config: Arc<RwLock<ProjectSettings>>,
    config_file: PathBuf,
}

impl ProjectSettingsConfig {
    fn new() -> Self {
        let config_file = Self::default_config_path();
        let config = if config_file.exists() {
            match fs::read_to_string(&config_file) {
                Ok(json) => serde_json::from_str(&json).unwrap_or_default(),
                Err(_) => ProjectSettings::default(),
            }
        } else {
            ProjectSettings::default()
        };

        Self {
            config: Arc::new(RwLock::new(config)),
            config_file,
        }
    }

    fn get_instance() -> &'static ProjectSettingsConfig {
        static mut INSTANCE: Option<ProjectSettingsConfig> = None;
        static ONCE: Once = Once::new();

        unsafe {
            ONCE.call_once(|| {
                let instance = ProjectSettingsConfig::new();
                INSTANCE = Some(instance);
            });

            #[allow(static_mut_refs)]
            INSTANCE.as_ref().unwrap_unchecked()
        }
    }

    fn default_config_path() -> PathBuf {
        std::env::current_exe()
            .unwrap_or_default()
            .parent()
            .unwrap_or(&Path::new(""))
            .join("project_settings.json")
    }

    fn save_config() {
        if let Ok(config) = Self::get_instance().config.read() {
            if let Ok(json) = to_string_pretty(&*config) {
                let _ = fs::write(&Self::get_instance().config_file, json);
            }
        }
    }

    pub fn get_full_config() -> &'static Arc<RwLock<ProjectSettings>> {
        &Self::get_instance().config
    }

    pub fn get_projects_root() -> PathBuf {
        if let Ok(config) = Self::get_instance().config.read() {
            config.projects_root.clone()
        } else {
            ProjectSettings::default().projects_root
        }
    }

    pub fn set_projects_root(value: PathBuf) {
        if let Ok(mut config) = Self::get_instance().config.write() {
            config.projects_root = value;
        }

        Self::save_config();
    }

    pub fn get_project_update_interval() -> u64 {
        if let Ok(config) = Self::get_instance().config.read() {
            config.project_update_interval_ms
        } else {
            ProjectSettings::default().project_update_interval_ms
        }
    }

    pub fn set_project_update_interval(value: u64) {
        if let Ok(mut config) = Self::get_instance().config.write() {
            config.project_update_interval_ms = value;
        }

        Self::save_config();
    }
}
