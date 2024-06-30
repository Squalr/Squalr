use crate::floating_point_tolerance::FloatingPointTolerance;
use squalr_engine_memory::memory_alignment::MemoryAlignment;
use std::sync::{Arc, RwLock};
use std::sync::Once;

#[derive(Default)]
pub struct Config {
    pub result_read_interval: i32,
    pub table_read_interval: i32,
    pub freeze_interval: i32,
    pub memory_type_none: bool,
    pub memory_type_private: bool,
    pub memory_type_image: bool,
    pub memory_type_mapped: bool,
    pub alignment: MemoryAlignment,
    pub floating_point_tolerance: FloatingPointTolerance,
    pub required_write: bool,
    pub required_execute: bool,
    pub required_copy_on_write: bool,
    pub excluded_write: bool,
    pub excluded_execute: bool,
    pub excluded_copy_on_write: bool,
    pub start_address: u64,
    pub end_address: u64,
    pub is_usermode: bool,
    pub use_multi_thread_scans: bool,
}

impl Config {
    fn new() -> Self {
        Self::default()
    }
}

pub struct ScanSettings {
    config: Arc<RwLock<Config>>,
}

impl ScanSettings {
    fn new() -> Self {
        Self {
            config: Arc::new(RwLock::new(Config::new())),
        }
    }
    
    pub fn instance() -> &'static ScanSettings {
        static mut INSTANCE: Option<ScanSettings> = None;
        static ONCE: Once = Once::new();

        unsafe {
            ONCE.call_once(|| {
                let instance = ScanSettings::new();
                INSTANCE = Some(instance);
            });

            INSTANCE.as_ref().unwrap()
        }
    }

    pub fn get_result_read_interval(&self) -> i32 {
        return self.config.read().unwrap().result_read_interval;
    }

    pub fn set_result_read_interval(&self, value: i32) {
        self.config.write().unwrap().result_read_interval = value;
    }

    pub fn get_table_read_interval(&self) -> i32 {
        return self.config.read().unwrap().table_read_interval;
    }

    pub fn set_table_read_interval(&self, value: i32) {
        self.config.write().unwrap().table_read_interval = value;
    }

    pub fn get_freeze_interval(&self) -> i32 {
        return self.config.read().unwrap().freeze_interval;
    }

    pub fn set_freeze_interval(&self, value: i32) {
        self.config.write().unwrap().freeze_interval = value;
    }

    pub fn get_memory_type_none(&self) -> bool {
        return self.config.read().unwrap().memory_type_none;
    }

    pub fn set_memory_type_none(&self, value: bool) {
        self.config.write().unwrap().memory_type_none = value;
    }

    pub fn get_memory_type_private(&self) -> bool {
        return self.config.read().unwrap().memory_type_private;
    }

    pub fn set_memory_type_private(&self, value: bool) {
        self.config.write().unwrap().memory_type_private = value;
    }

    pub fn get_memory_type_image(&self) -> bool {
        return self.config.read().unwrap().memory_type_image;
    }

    pub fn set_memory_type_image(&self, value: bool) {
        self.config.write().unwrap().memory_type_image = value;
    }

    pub fn get_memory_type_mapped(&self) -> bool {
        return self.config.read().unwrap().memory_type_mapped;
    }

    pub fn set_memory_type_mapped(&self, value: bool) {
        self.config.write().unwrap().memory_type_mapped = value;
    }

    pub fn get_alignment(&self) -> MemoryAlignment {
        return self.config.read().unwrap().alignment;
    }

    pub fn set_alignment(&self, value: MemoryAlignment) {
        self.config.write().unwrap().alignment = value;
    }

    pub fn get_floating_point_tolerance(&self) -> FloatingPointTolerance {
        return self.config.read().unwrap().floating_point_tolerance;
    }

    pub fn set_floating_point_tolerance(&self, value: FloatingPointTolerance) {
        self.config.write().unwrap().floating_point_tolerance = value;
    }

    pub fn get_required_write(&self) -> bool {
        return self.config.read().unwrap().required_write;
    }

    pub fn set_required_write(&self, value: bool) {
        self.config.write().unwrap().required_write = value;
    }

    pub fn get_required_execute(&self) -> bool {
        return self.config.read().unwrap().required_execute;
    }

    pub fn set_required_execute(&self, value: bool) {
        self.config.write().unwrap().required_execute = value;
    }

    pub fn get_required_copy_on_write(&self) -> bool {
        return self.config.read().unwrap().required_copy_on_write;
    }

    pub fn set_required_copy_on_write(&self, value: bool) {
        self.config.write().unwrap().required_copy_on_write = value;
    }

    pub fn get_excluded_write(&self) -> bool {
        return self.config.read().unwrap().excluded_write;
    }

    pub fn set_excluded_write(&self, value: bool) {
        self.config.write().unwrap().excluded_write = value;
    }

    pub fn get_excluded_execute(&self) -> bool {
        return self.config.read().unwrap().excluded_execute;
    }

    pub fn set_excluded_execute(&self, value: bool) {
        self.config.write().unwrap().excluded_execute = value;
    }

    pub fn get_excluded_copy_on_write(&self) -> bool {
        return self.config.read().unwrap().excluded_copy_on_write;
    }

    pub fn set_excluded_copy_on_write(&self, value: bool) {
        self.config.write().unwrap().excluded_copy_on_write = value;
    }

    pub fn get_start_address(&self) -> u64 {
        return self.config.read().unwrap().start_address;
    }

    pub fn set_start_address(&self, value: u64) {
        self.config.write().unwrap().start_address = value;
    }

    pub fn get_end_address(&self) -> u64 {
        return self.config.read().unwrap().end_address;
    }

    pub fn set_end_address(&self, value: u64) {
        self.config.write().unwrap().end_address = value;
    }

    pub fn is_usermode(&self) -> bool {
        return self.config.read().unwrap().is_usermode;
    }

    pub fn set_is_usermode(&self, value: bool) {
        self.config.write().unwrap().is_usermode = value;
    }

    pub fn get_use_multi_thread_scans(&self) -> bool {
        return self.config.read().unwrap().use_multi_thread_scans;
    }

    pub fn set_use_multi_thread_scans(&self, value: bool) {
        self.config.write().unwrap().use_multi_thread_scans = value;
    }

    pub fn get_resolve_auto_alignment(alignment: MemoryAlignment, data_type_size: i32) -> MemoryAlignment {
        if alignment == MemoryAlignment::Auto {
            return MemoryAlignment::from(data_type_size);
        } else {
            return alignment;
        }
    }
}
