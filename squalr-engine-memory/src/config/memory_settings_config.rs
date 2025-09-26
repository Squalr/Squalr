use serde_json::to_string_pretty;
use squalr_engine_api::structures::settings::memory_settings::MemorySettings;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Once;
use std::sync::{Arc, RwLock};

pub struct MemorySettingsConfig {
    config: Arc<RwLock<MemorySettings>>,
    config_file: PathBuf,
}

impl MemorySettingsConfig {
    fn new() -> Self {
        let config_file = Self::default_config_path();
        let config = if config_file.exists() {
            match fs::read_to_string(&config_file) {
                Ok(json) => serde_json::from_str(&json).unwrap_or_default(),
                Err(_) => MemorySettings::default(),
            }
        } else {
            MemorySettings::default()
        };

        Self {
            config: Arc::new(RwLock::new(config)),
            config_file,
        }
    }

    fn get_instance() -> &'static MemorySettingsConfig {
        static mut INSTANCE: Option<MemorySettingsConfig> = None;
        static ONCE: Once = Once::new();

        unsafe {
            ONCE.call_once(|| {
                let instance = MemorySettingsConfig::new();
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
            .join("memory_settings.json")
    }

    fn save_config() {
        if let Ok(config) = Self::get_instance().config.read() {
            if let Ok(json) = to_string_pretty(&*config) {
                let _ = fs::write(&Self::get_instance().config_file, json);
            }
        }
    }

    pub fn get_full_config() -> &'static Arc<RwLock<MemorySettings>> {
        &Self::get_instance().config
    }

    pub fn get_memory_type_none() -> bool {
        if let Ok(config) = Self::get_instance().config.read() {
            config.memory_type_none
        } else {
            MemorySettings::default().memory_type_none
        }
    }

    pub fn set_memory_type_none(value: bool) {
        if let Ok(mut config) = Self::get_instance().config.write() {
            config.memory_type_none = value;
        }

        Self::save_config();
    }

    pub fn get_memory_type_private() -> bool {
        if let Ok(config) = Self::get_instance().config.read() {
            config.memory_type_private
        } else {
            MemorySettings::default().memory_type_private
        }
    }

    pub fn set_memory_type_private(value: bool) {
        if let Ok(mut config) = Self::get_instance().config.write() {
            config.memory_type_private = value;
        }

        Self::save_config();
    }

    pub fn get_memory_type_image() -> bool {
        if let Ok(config) = Self::get_instance().config.read() {
            config.memory_type_image
        } else {
            MemorySettings::default().memory_type_image
        }
    }

    pub fn set_memory_type_image(value: bool) {
        if let Ok(mut config) = Self::get_instance().config.write() {
            config.memory_type_image = value;
        }

        Self::save_config();
    }

    pub fn get_memory_type_mapped() -> bool {
        if let Ok(config) = Self::get_instance().config.read() {
            config.memory_type_mapped
        } else {
            MemorySettings::default().memory_type_mapped
        }
    }

    pub fn set_memory_type_mapped(value: bool) {
        if let Ok(mut config) = Self::get_instance().config.write() {
            config.memory_type_mapped = value;
        }

        Self::save_config();
    }

    pub fn get_required_write() -> bool {
        if let Ok(config) = Self::get_instance().config.read() {
            config.required_write
        } else {
            MemorySettings::default().required_write
        }
    }

    pub fn set_required_write(value: bool) {
        if let Ok(mut config) = Self::get_instance().config.write() {
            config.required_write = value;
        }

        Self::save_config();
    }

    pub fn get_required_execute() -> bool {
        if let Ok(config) = Self::get_instance().config.read() {
            config.required_execute
        } else {
            MemorySettings::default().required_execute
        }
    }

    pub fn set_required_execute(value: bool) {
        if let Ok(mut config) = Self::get_instance().config.write() {
            config.required_execute = value;
        }

        Self::save_config();
    }

    pub fn get_required_copy_on_write() -> bool {
        if let Ok(config) = Self::get_instance().config.read() {
            config.required_copy_on_write
        } else {
            MemorySettings::default().required_copy_on_write
        }
    }

    pub fn set_required_copy_on_write(value: bool) {
        if let Ok(mut config) = Self::get_instance().config.write() {
            config.required_copy_on_write = value;
        }

        Self::save_config();
    }

    pub fn get_excluded_write() -> bool {
        if let Ok(config) = Self::get_instance().config.read() {
            config.excluded_write
        } else {
            MemorySettings::default().excluded_write
        }
    }

    pub fn set_excluded_write(value: bool) {
        if let Ok(mut config) = Self::get_instance().config.write() {
            config.excluded_write = value;
        }

        Self::save_config();
    }

    pub fn get_excluded_execute() -> bool {
        if let Ok(config) = Self::get_instance().config.read() {
            config.excluded_execute
        } else {
            MemorySettings::default().excluded_execute
        }
    }

    pub fn set_excluded_execute(value: bool) {
        if let Ok(mut config) = Self::get_instance().config.write() {
            config.excluded_execute = value;
        }

        Self::save_config();
    }

    pub fn get_excluded_copy_on_write() -> bool {
        if let Ok(config) = Self::get_instance().config.read() {
            config.excluded_copy_on_write
        } else {
            MemorySettings::default().excluded_copy_on_write
        }
    }

    pub fn set_excluded_copy_on_write(value: bool) {
        if let Ok(mut config) = Self::get_instance().config.write() {
            config.excluded_copy_on_write = value;
        }

        Self::save_config();
    }

    pub fn get_start_address() -> u64 {
        if let Ok(config) = Self::get_instance().config.read() {
            config.start_address
        } else {
            MemorySettings::default().start_address
        }
    }

    pub fn set_start_address(value: u64) {
        if let Ok(mut config) = Self::get_instance().config.write() {
            config.start_address = value;
        }

        Self::save_config();
    }

    pub fn get_end_address() -> u64 {
        if let Ok(config) = Self::get_instance().config.read() {
            config.end_address
        } else {
            MemorySettings::default().end_address
        }
    }

    pub fn set_end_address(value: u64) {
        if let Ok(mut config) = Self::get_instance().config.write() {
            config.end_address = value;
        }

        Self::save_config();
    }

    pub fn get_only_query_usermode() -> bool {
        if let Ok(config) = Self::get_instance().config.read() {
            config.only_query_usermode
        } else {
            MemorySettings::default().only_query_usermode
        }
    }

    pub fn set_only_query_usermode(value: bool) {
        if let Ok(mut config) = Self::get_instance().config.write() {
            config.only_query_usermode = value;
        }

        Self::save_config();
    }
}
