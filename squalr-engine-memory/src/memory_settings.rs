use serde::{Deserialize, Serialize};
use serde_json::to_string_pretty;
use squalr_engine_common::config::serialized_config_updater;
use std::path::PathBuf;
use std::sync::Once;
use std::sync::{Arc, RwLock};
use std::{fmt, fs};

#[derive(Deserialize, Serialize)]
pub struct Config {
    pub memory_type_none: bool,
    pub memory_type_private: bool,
    pub memory_type_image: bool,
    pub memory_type_mapped: bool,
    pub required_write: bool,
    pub required_execute: bool,
    pub required_copy_on_write: bool,
    pub excluded_write: bool,
    pub excluded_execute: bool,
    pub excluded_copy_on_write: bool,
    pub start_address: u64,
    pub end_address: u64,
    pub only_scan_usermode: bool,
}

impl fmt::Debug for Config {
    fn fmt(
        &self,
        formatter: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        match to_string_pretty(&self) {
            Ok(json) => write!(formatter, "Settings for memory: {}", json),
            Err(_) => write!(formatter, "Memory config {{ could not serialize to JSON }}"),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            memory_type_none: true,
            memory_type_private: true,
            memory_type_image: true,
            memory_type_mapped: true,

            required_write: false,
            required_execute: false,
            required_copy_on_write: false,

            excluded_write: false,
            excluded_execute: false,
            excluded_copy_on_write: false,

            start_address: 0,
            end_address: u64::MAX,
            only_scan_usermode: true,
        }
    }
}

pub struct MemorySettings {
    config: Arc<RwLock<Config>>,
    config_file: PathBuf,
}

impl MemorySettings {
    fn new() -> Self {
        let config_file = Self::default_config_path();
        let config = if config_file.exists() {
            match fs::read_to_string(&config_file) {
                Ok(json) => serde_json::from_str(&json).unwrap_or_default(),
                Err(_) => Config::default(),
            }
        } else {
            Config::default()
        };

        Self {
            config: Arc::new(RwLock::new(config)),
            config_file,
        }
    }

    pub fn get_instance() -> &'static MemorySettings {
        static mut INSTANCE: Option<MemorySettings> = None;
        static ONCE: Once = Once::new();

        unsafe {
            ONCE.call_once(|| {
                let instance = MemorySettings::new();
                INSTANCE = Some(instance);
            });

            return INSTANCE.as_ref().unwrap_unchecked();
        }
    }

    fn default_config_path() -> PathBuf {
        std::env::current_exe()
            .unwrap()
            .parent()
            .unwrap()
            .join("memory_settings.json")
    }

    fn save_config(&self) {
        let config = self.config.read().unwrap();
        if let Ok(json) = to_string_pretty(&*config) {
            let _ = fs::write(&self.config_file, json);
        }
    }

    pub fn get_full_config(&self) -> &Arc<RwLock<Config>> {
        return &self.config;
    }

    pub fn get_memory_type_none(&self) -> bool {
        return self.config.read().unwrap().memory_type_none;
    }

    pub fn set_memory_type_none(
        &self,
        value: bool,
    ) {
        self.config.write().unwrap().memory_type_none = value;
        self.save_config();
    }

    pub fn get_memory_type_private(&self) -> bool {
        return self.config.read().unwrap().memory_type_private;
    }

    pub fn set_memory_type_private(
        &self,
        value: bool,
    ) {
        self.config.write().unwrap().memory_type_private = value;
        self.save_config();
    }

    pub fn get_memory_type_image(&self) -> bool {
        return self.config.read().unwrap().memory_type_image;
    }

    pub fn set_memory_type_image(
        &self,
        value: bool,
    ) {
        self.config.write().unwrap().memory_type_image = value;
        self.save_config();
    }

    pub fn get_memory_type_mapped(&self) -> bool {
        return self.config.read().unwrap().memory_type_mapped;
    }

    pub fn set_memory_type_mapped(
        &self,
        value: bool,
    ) {
        self.config.write().unwrap().memory_type_mapped = value;
        self.save_config();
    }

    pub fn get_required_write(&self) -> bool {
        return self.config.read().unwrap().required_write;
    }

    pub fn set_required_write(
        &self,
        value: bool,
    ) {
        self.config.write().unwrap().required_write = value;
        self.save_config();
    }

    pub fn get_required_execute(&self) -> bool {
        return self.config.read().unwrap().required_execute;
    }

    pub fn set_required_execute(
        &self,
        value: bool,
    ) {
        self.config.write().unwrap().required_execute = value;
        self.save_config();
    }

    pub fn get_required_copy_on_write(&self) -> bool {
        return self.config.read().unwrap().required_copy_on_write;
    }

    pub fn set_required_copy_on_write(
        &self,
        value: bool,
    ) {
        self.config.write().unwrap().required_copy_on_write = value;
        self.save_config();
    }

    pub fn get_excluded_write(&self) -> bool {
        return self.config.read().unwrap().excluded_write;
    }

    pub fn set_excluded_write(
        &self,
        value: bool,
    ) {
        self.config.write().unwrap().excluded_write = value;
        self.save_config();
    }

    pub fn get_excluded_execute(&self) -> bool {
        return self.config.read().unwrap().excluded_execute;
    }

    pub fn set_excluded_execute(
        &self,
        value: bool,
    ) {
        self.config.write().unwrap().excluded_execute = value;
        self.save_config();
    }

    pub fn get_excluded_copy_on_write(&self) -> bool {
        return self.config.read().unwrap().excluded_copy_on_write;
    }

    pub fn set_excluded_copy_on_write(
        &self,
        value: bool,
    ) {
        self.config.write().unwrap().excluded_copy_on_write = value;
        self.save_config();
    }

    pub fn get_start_address(&self) -> u64 {
        return self.config.read().unwrap().start_address;
    }

    pub fn set_start_address(
        &self,
        value: u64,
    ) {
        self.config.write().unwrap().start_address = value;
        self.save_config();
    }

    pub fn get_end_address(&self) -> u64 {
        return self.config.read().unwrap().end_address;
    }

    pub fn set_end_address(
        &self,
        value: u64,
    ) {
        self.config.write().unwrap().end_address = value;
        self.save_config();
    }

    pub fn get_only_scan_usermode(&self) -> bool {
        return self.config.read().unwrap().only_scan_usermode;
    }

    pub fn set_only_scan_usermode(
        &self,
        value: bool,
    ) {
        self.config.write().unwrap().only_scan_usermode = value;
        self.save_config();
    }

    pub fn update_config_field(
        &self,
        field: &str,
        value: &str,
    ) {
        let mut config = self.config.write().unwrap();
        serialized_config_updater::update_config_field(&mut *config, field, value);
        self.save_config();
    }
}
