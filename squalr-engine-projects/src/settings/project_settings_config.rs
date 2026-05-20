use directories::UserDirs;
use serde_json::to_string_pretty;
use squalr_engine_api::structures::settings::project_settings::ProjectSettings;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, OnceLock, RwLock};

pub struct ProjectSettingsConfig {
    config: Arc<RwLock<ProjectSettings>>,
    config_file: PathBuf,
}

impl ProjectSettingsConfig {
    fn new() -> Self {
        let config_file = Self::default_config_path();
        let config = if config_file.exists() {
            match fs::read_to_string(&config_file) {
                Ok(json) => serde_json::from_str(&json).unwrap_or_else(|error| {
                    log::warn!("Failed to parse project settings config '{}': {}.", config_file.display(), error);
                    Self::default_project_settings()
                }),
                Err(error) => {
                    log::warn!("Failed to read project settings config '{}': {}.", config_file.display(), error);
                    Self::default_project_settings()
                }
            }
        } else {
            Self::default_project_settings()
        };

        Self {
            config: Arc::new(RwLock::new(config)),
            config_file,
        }
    }

    fn get_instance() -> &'static ProjectSettingsConfig {
        static INSTANCE: OnceLock<ProjectSettingsConfig> = OnceLock::new();

        INSTANCE.get_or_init(ProjectSettingsConfig::new)
    }

    fn default_project_settings() -> ProjectSettings {
        ProjectSettings {
            projects_root: Self::default_projects_root(),
            ..ProjectSettings::default()
        }
    }

    fn default_projects_root() -> PathBuf {
        UserDirs::new()
            .and_then(|dirs| {
                dirs.document_dir()
                    .map(|documents_directory| documents_directory.join("Squalr"))
            })
            .unwrap_or_else(|| PathBuf::from("./Squalr"))
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
            Self::default_projects_root()
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
            Self::default_project_settings().project_update_interval_ms
        }
    }

    pub fn set_project_update_interval(value: u64) {
        if let Ok(mut config) = Self::get_instance().config.write() {
            config.project_update_interval_ms = value;
        }

        Self::save_config();
    }
}

#[cfg(test)]
mod tests {
    use super::ProjectSettingsConfig;

    #[test]
    fn default_project_settings_uses_app_projects_root() {
        let default_project_settings = ProjectSettingsConfig::default_project_settings();

        assert_eq!(default_project_settings.project_update_interval_ms, 200);
        assert_eq!(
            default_project_settings
                .projects_root
                .file_name()
                .and_then(|file_name| file_name.to_str()),
            Some("Squalr")
        );
    }
}
