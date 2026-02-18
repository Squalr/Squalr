use serde_json::to_string_pretty;
use squalr_engine_api::structures::settings::general_settings::GeneralSettings;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Once;
use std::sync::{Arc, RwLock};

pub struct GeneralSettingsConfig {
    config: Arc<RwLock<GeneralSettings>>,
    config_file: PathBuf,
}

impl GeneralSettingsConfig {
    fn new() -> Self {
        let config_file = Self::default_config_path();
        let config = if config_file.exists() {
            match fs::read_to_string(&config_file) {
                Ok(json) => serde_json::from_str(&json).unwrap_or_default(),
                Err(_) => GeneralSettings::default(),
            }
        } else {
            GeneralSettings::default()
        };

        Self {
            config: Arc::new(RwLock::new(config)),
            config_file,
        }
    }

    fn get_instance() -> &'static GeneralSettingsConfig {
        static mut INSTANCE: Option<GeneralSettingsConfig> = None;
        static ONCE: Once = Once::new();

        unsafe {
            ONCE.call_once(|| {
                let instance = GeneralSettingsConfig::new();
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
            .join("scan_settings.json")
    }

    fn save_config() {
        if let Ok(config) = Self::get_instance().config.read() {
            if let Ok(json) = to_string_pretty(&*config) {
                let _ = fs::write(&Self::get_instance().config_file, json);
            }
        }
    }

    pub fn get_full_config() -> &'static Arc<RwLock<GeneralSettings>> {
        &Self::get_instance().config
    }

    pub fn get_debug_engine_request_delay_ms() -> u64 {
        if let Ok(config) = Self::get_instance().config.read() {
            config.debug_engine_request_delay_ms
        } else {
            GeneralSettings::default().debug_engine_request_delay_ms
        }
    }

    pub fn set_debug_engine_request_delay_ms(value: u64) {
        if let Ok(mut config) = Self::get_instance().config.write() {
            config.debug_engine_request_delay_ms = value;
        }

        Self::save_config();
    }
}
