use std::sync::Once;
use std::sync::{
    Arc,
    RwLock,
};

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
    pub is_usermode: bool,
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
            is_usermode: true,
        }
    }
}

pub struct MemorySettings {
    config: Arc<RwLock<Config>>,
}

impl MemorySettings {
    fn new() -> Self {
        Self {
            config: Arc::new(RwLock::new(Config::default())),
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

    pub fn get_memory_type_none(&self) -> bool {
        return self.config.read().unwrap().memory_type_none;
    }

    pub fn set_memory_type_none(
        &self,
        value: bool,
    ) {
        self.config.write().unwrap().memory_type_none = value;
    }

    pub fn get_memory_type_private(&self) -> bool {
        return self.config.read().unwrap().memory_type_private;
    }

    pub fn set_memory_type_private(
        &self,
        value: bool,
    ) {
        self.config.write().unwrap().memory_type_private = value;
    }

    pub fn get_memory_type_image(&self) -> bool {
        return self.config.read().unwrap().memory_type_image;
    }

    pub fn set_memory_type_image(
        &self,
        value: bool,
    ) {
        self.config.write().unwrap().memory_type_image = value;
    }

    pub fn get_memory_type_mapped(&self) -> bool {
        return self.config.read().unwrap().memory_type_mapped;
    }

    pub fn set_memory_type_mapped(
        &self,
        value: bool,
    ) {
        self.config.write().unwrap().memory_type_mapped = value;
    }

    pub fn get_required_write(&self) -> bool {
        return self.config.read().unwrap().required_write;
    }

    pub fn set_required_write(
        &self,
        value: bool,
    ) {
        self.config.write().unwrap().required_write = value;
    }

    pub fn get_required_execute(&self) -> bool {
        return self.config.read().unwrap().required_execute;
    }

    pub fn set_required_execute(
        &self,
        value: bool,
    ) {
        self.config.write().unwrap().required_execute = value;
    }

    pub fn get_required_copy_on_write(&self) -> bool {
        return self.config.read().unwrap().required_copy_on_write;
    }

    pub fn set_required_copy_on_write(
        &self,
        value: bool,
    ) {
        self.config.write().unwrap().required_copy_on_write = value;
    }

    pub fn get_excluded_write(&self) -> bool {
        return self.config.read().unwrap().excluded_write;
    }

    pub fn set_excluded_write(
        &self,
        value: bool,
    ) {
        self.config.write().unwrap().excluded_write = value;
    }

    pub fn get_excluded_execute(&self) -> bool {
        return self.config.read().unwrap().excluded_execute;
    }

    pub fn set_excluded_execute(
        &self,
        value: bool,
    ) {
        self.config.write().unwrap().excluded_execute = value;
    }

    pub fn get_excluded_copy_on_write(&self) -> bool {
        return self.config.read().unwrap().excluded_copy_on_write;
    }

    pub fn set_excluded_copy_on_write(
        &self,
        value: bool,
    ) {
        self.config.write().unwrap().excluded_copy_on_write = value;
    }

    pub fn get_start_address(&self) -> u64 {
        return self.config.read().unwrap().start_address;
    }

    pub fn set_start_address(
        &self,
        value: u64,
    ) {
        self.config.write().unwrap().start_address = value;
    }

    pub fn get_end_address(&self) -> u64 {
        return self.config.read().unwrap().end_address;
    }

    pub fn set_end_address(
        &self,
        value: u64,
    ) {
        self.config.write().unwrap().end_address = value;
    }

    pub fn is_usermode(&self) -> bool {
        return self.config.read().unwrap().is_usermode;
    }

    pub fn set_is_usermode(
        &self,
        value: bool,
    ) {
        self.config.write().unwrap().is_usermode = value;
    }
}
