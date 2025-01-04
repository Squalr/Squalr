use crate::floating_point_tolerance::FloatingPointTolerance;
use serde::{Deserialize, Serialize};
use serde_json::to_string_pretty;
use squalr_engine_common::config::serialized_config_updater;
use squalr_engine_memory::memory_alignment::MemoryAlignment;
use std::path::PathBuf;
use std::sync::Once;
use std::sync::{Arc, RwLock};
use std::{fmt, fs};

#[derive(Deserialize, Serialize)]
pub struct Config {
    pub results_page_size: u32,
    pub results_read_interval: u32,
    pub table_read_interval: u32,
    pub freeze_interval: u32,
    pub memory_alignment: Option<MemoryAlignment>,
    pub floating_point_tolerance: FloatingPointTolerance,
}

impl fmt::Debug for Config {
    fn fmt(
        &self,
        formatter: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        match to_string_pretty(&self) {
            Ok(json) => write!(formatter, "Settings for scan: {}", json),
            Err(_) => write!(formatter, "Scan config {{ could not serialize to JSON }}"),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            results_page_size: 16,
            results_read_interval: 2500,
            table_read_interval: 2500,
            freeze_interval: 50,
            memory_alignment: None,
            floating_point_tolerance: FloatingPointTolerance::default(),
        }
    }
}

pub struct ScanSettings {
    config: Arc<RwLock<Config>>,
    config_file: PathBuf,
}

impl ScanSettings {
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

    pub fn get_instance() -> &'static ScanSettings {
        static mut INSTANCE: Option<ScanSettings> = None;
        static ONCE: Once = Once::new();

        unsafe {
            ONCE.call_once(|| {
                let instance = ScanSettings::new();
                INSTANCE = Some(instance);
            });

            #[allow(static_mut_refs)]
            return INSTANCE.as_ref().unwrap_unchecked();
        }
    }

    fn default_config_path() -> PathBuf {
        std::env::current_exe()
            .unwrap()
            .parent()
            .unwrap()
            .join("scan_settings.json")
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

    pub fn get_results_page_size(&self) -> u32 {
        return self.config.read().unwrap().results_page_size;
    }

    pub fn set_results_page_size(
        &self,
        value: u32,
    ) {
        self.config.write().unwrap().results_page_size = value;
        self.save_config();
    }

    pub fn get_results_read_interval(&self) -> u32 {
        return self.config.read().unwrap().results_read_interval;
    }

    pub fn set_results_read_interval(
        &self,
        value: u32,
    ) {
        self.config.write().unwrap().results_read_interval = value;
        self.save_config();
    }

    pub fn get_table_read_interval(&self) -> u32 {
        return self.config.read().unwrap().table_read_interval;
    }

    pub fn set_table_read_interval(
        &self,
        value: u32,
    ) {
        self.config.write().unwrap().table_read_interval = value;
        self.save_config();
    }

    pub fn get_freeze_interval(&self) -> u32 {
        return self.config.read().unwrap().freeze_interval;
    }

    pub fn set_freeze_interval(
        &self,
        value: u32,
    ) {
        self.config.write().unwrap().freeze_interval = value;
        self.save_config();
    }

    pub fn get_memory_alignment(&self) -> Option<MemoryAlignment> {
        return self.config.read().unwrap().memory_alignment;
    }

    pub fn set_memory_alignment(
        &self,
        value: Option<MemoryAlignment>,
    ) {
        self.config.write().unwrap().memory_alignment = value;
        self.save_config();
    }

    pub fn get_floating_point_tolerance(&self) -> FloatingPointTolerance {
        return self.config.read().unwrap().floating_point_tolerance;
    }

    pub fn set_floating_point_tolerance(
        &self,
        value: FloatingPointTolerance,
    ) {
        self.config.write().unwrap().floating_point_tolerance = value;
        self.save_config();
    }

    pub fn update_config_field(
        &self,
        field: &str,
        value: &str,
    ) {
        // Scope to drop write lock before saving.
        {
            let mut config = self.config.write().unwrap();
            serialized_config_updater::update_config_field(&mut *config, field, value);
        }

        self.save_config();
    }
}
