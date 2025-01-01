use serde::{Deserialize, Serialize};
use serde_json::to_string_pretty;
use squalr_engine_common::config::serialized_config_updater;
use std::path::PathBuf;
use std::sync::Once;
use std::sync::{Arc, RwLock};
use std::{collections::HashMap, fs};

#[derive(Deserialize, Serialize)]
pub struct Config {
    pub windows: HashMap<String, DockedWindowLayout>,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct DockedWindowLayout {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl Default for Config {
    fn default() -> Self {
        Self { windows: HashMap::new() }
    }
}

pub struct DockableWindowSettings {
    config: Arc<RwLock<Config>>,
    config_file: PathBuf,
}

impl DockableWindowSettings {
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

    pub fn get_instance() -> &'static DockableWindowSettings {
        static mut INSTANCE: Option<DockableWindowSettings> = None;
        static ONCE: Once = Once::new();

        unsafe {
            ONCE.call_once(|| {
                let instance = DockableWindowSettings::new();
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
            .join("docking_settings.json")
    }

    fn save_config(&self) {
        let config = self.config.read().unwrap();
        if let Ok(json) = to_string_pretty(&*config) {
            let _ = fs::write(&self.config_file, json);
        }
    }

    pub fn get_full_config(&self) -> &Arc<RwLock<Config>> {
        &self.config
    }

    pub fn get_window_settings(
        &self,
        window_id: &str,
    ) -> Option<DockedWindowLayout> {
        self.config.read().unwrap().windows.get(window_id).cloned()
    }

    pub fn set_window_settings(
        &self,
        window_id: String,
        settings: DockedWindowLayout,
    ) {
        self.config.write().unwrap().windows.insert(window_id, settings);
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
