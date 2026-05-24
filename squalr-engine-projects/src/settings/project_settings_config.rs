#[cfg(not(target_os = "android"))]
use directories::UserDirs;
use serde_json::to_string_pretty;
use squalr_engine_api::structures::settings::project_settings::ProjectSettings;
use std::fs;
use std::path::PathBuf;
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

    #[cfg(target_os = "android")]
    fn default_projects_root() -> PathBuf {
        Self::default_android_app_files_root()
            .join("projects")
            .join("Squalr")
    }

    #[cfg(not(target_os = "android"))]
    fn default_projects_root() -> PathBuf {
        UserDirs::new()
            .and_then(|dirs| {
                dirs.document_dir()
                    .map(|documents_directory| documents_directory.join("Squalr"))
            })
            .unwrap_or_else(|| PathBuf::from("./Squalr"))
    }

    #[cfg(target_os = "android")]
    fn default_config_path() -> PathBuf {
        Self::default_android_app_files_root().join("project_settings.json")
    }

    #[cfg(not(target_os = "android"))]
    fn default_config_path() -> PathBuf {
        std::env::current_exe()
            .unwrap_or_default()
            .parent()
            .unwrap_or(std::path::Path::new(""))
            .join("project_settings.json")
    }

    #[cfg(target_os = "android")]
    fn default_android_app_files_root() -> PathBuf {
        std::env::var_os("SQUALR_ANDROID_APP_FILES_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("/data/data/com.squalr.android/files"))
    }

    fn save_config() {
        let config_instance = Self::get_instance();

        if let Ok(config) = config_instance.config.read() {
            if let Ok(json) = to_string_pretty(&*config) {
                if let Some(config_directory_path) = config_instance.config_file.parent() {
                    if let Err(error) = fs::create_dir_all(config_directory_path) {
                        log::warn!(
                            "Failed to create project settings config directory '{}': {}.",
                            config_directory_path.display(),
                            error
                        );

                        return;
                    }
                }

                if let Err(error) = fs::write(&config_instance.config_file, json) {
                    log::warn!(
                        "Failed to write project settings config '{}': {}.",
                        config_instance.config_file.display(),
                        error
                    );
                }
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
