use crate::floating_point_tolerance::FloatingPointTolerance;

use serde::{Deserialize, Serialize};
use serde_json::to_string_pretty;
use squalr_engine_memory::memory_alignment::MemoryAlignment;
use std::fmt;
use std::sync::Once;
use std::sync::{Arc, RwLock};

#[derive(Deserialize, Serialize)]
pub struct Config {
    pub result_read_interval: i32,
    pub table_read_interval: i32,
    pub freeze_interval: i32,
    pub alignment: Option<MemoryAlignment>,
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
            result_read_interval: 2500,
            table_read_interval: 2500,
            freeze_interval: 50,
            alignment: None,
            floating_point_tolerance: FloatingPointTolerance::default(),
        }
    }
}

pub struct ScanSettings {
    config: Arc<RwLock<Config>>,
}

impl ScanSettings {
    fn new() -> Self {
        Self {
            config: Arc::new(RwLock::new(Config::default())),
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

            return INSTANCE.as_ref().unwrap_unchecked();
        }
    }

    pub fn get_full_config(&self) -> &Arc<RwLock<Config>> {
        return &self.config;
    }

    pub fn get_result_read_interval(&self) -> i32 {
        return self.config.read().unwrap().result_read_interval;
    }

    pub fn set_result_read_interval(
        &self,
        value: i32,
    ) {
        self.config.write().unwrap().result_read_interval = value;
    }

    pub fn get_table_read_interval(&self) -> i32 {
        return self.config.read().unwrap().table_read_interval;
    }

    pub fn set_table_read_interval(
        &self,
        value: i32,
    ) {
        self.config.write().unwrap().table_read_interval = value;
    }

    pub fn get_freeze_interval(&self) -> i32 {
        return self.config.read().unwrap().freeze_interval;
    }

    pub fn set_freeze_interval(
        &self,
        value: i32,
    ) {
        self.config.write().unwrap().freeze_interval = value;
    }

    pub fn get_alignment(&self) -> Option<MemoryAlignment> {
        return self.config.read().unwrap().alignment;
    }

    pub fn set_alignment(
        &self,
        value: Option<MemoryAlignment>,
    ) {
        self.config.write().unwrap().alignment = value;
    }

    pub fn get_floating_point_tolerance(&self) -> FloatingPointTolerance {
        return self.config.read().unwrap().floating_point_tolerance;
    }

    pub fn set_floating_point_tolerance(
        &self,
        value: FloatingPointTolerance,
    ) {
        self.config.write().unwrap().floating_point_tolerance = value;
    }
}
