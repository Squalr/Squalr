use crate::models::docking::builder::dock_builder::DockBuilder;
use crate::models::docking::hierarchy::dock_node::DockNode;
use crate::models::docking::hierarchy::types::dock_split_direction::DockSplitDirection;
use serde::{Deserialize, Serialize};
use serde_json::to_string_pretty;
use squalr_engine_common::config::serialized_config_updater;
use std::fs;
use std::path::PathBuf;
use std::sync::Once;
use std::sync::{Arc, RwLock};

#[derive(Deserialize, Serialize)]
pub struct DockSettingsConfig {
    pub dock_root: DockNode,
}

impl Default for DockSettingsConfig {
    fn default() -> Self {
        Self {
            dock_root: Self::get_default_layout(),
        }
    }
}

impl DockSettingsConfig {
    pub fn get_default_layout() -> DockNode {
        DockBuilder::split_node(DockSplitDirection::VerticalDivider)
            .push_child(
                0.7,
                DockBuilder::split_node(DockSplitDirection::HorizontalDivider)
                    .push_child(
                        0.5,
                        DockBuilder::split_node(DockSplitDirection::VerticalDivider)
                            // Build a tab with two leaves: process-selector & project-explorer
                            .push_child(
                                0.5,
                                DockBuilder::tab_node("project-explorer")
                                    .push_tab(DockBuilder::window("process-selector"))
                                    .push_tab(DockBuilder::window("project-explorer")),
                            )
                            // And a window node for "settings" occupying the other 0.5
                            .push_child(0.5, DockBuilder::window("settings")),
                    )
                    .push_child(0.5, DockBuilder::window("output")),
            )
            .push_child(
                0.3,
                DockBuilder::split_node(DockSplitDirection::HorizontalDivider)
                    .push_child(0.6, DockBuilder::window("scan-results"))
                    .push_child(0.4, DockBuilder::window("property-viewer")),
            )
            .build()
    }
}

pub struct DockableWindowSettings {
    config: Arc<RwLock<DockSettingsConfig>>,
    config_file: PathBuf,
}

impl DockableWindowSettings {
    fn new() -> Self {
        let config_file = Self::default_config_path();
        let config = if config_file.exists() {
            match fs::read_to_string(&config_file) {
                Ok(json) => serde_json::from_str(&json).unwrap_or_default(),
                Err(_) => DockSettingsConfig::default(),
            }
        } else {
            DockSettingsConfig::default()
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
            INSTANCE.as_ref().unwrap_unchecked()
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

    pub fn get_full_config(&self) -> &Arc<RwLock<DockSettingsConfig>> {
        &self.config
    }

    pub fn get_dock_layout_settings(&self) -> DockNode {
        self.config.read().unwrap().dock_root.clone()
    }

    pub fn set_dock_layout_settings(
        &self,
        settings: &DockNode,
    ) {
        self.config.write().unwrap().dock_root = settings.clone();
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
