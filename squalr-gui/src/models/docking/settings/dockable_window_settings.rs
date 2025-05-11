use crate::models::docking::builder::dock_builder::DockBuilder;
use crate::models::docking::hierarchy::dock_node::DockNode;
use crate::models::docking::hierarchy::types::dock_split_direction::DockSplitDirection;
use serde::{Deserialize, Serialize};
use serde_json::to_string_pretty;
use std::fs;
use std::path::{Path, PathBuf};
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
        #[cfg(any(target_os = "windows", target_os = "macos", target_os = "linux"))]
        let default_layout = DockBuilder::split_node(DockSplitDirection::VerticalDivider)
            .push_child(
                0.6,
                DockBuilder::split_node(DockSplitDirection::HorizontalDivider)
                    .push_child(
                        0.5,
                        DockBuilder::split_node(DockSplitDirection::VerticalDivider)
                            .push_child(
                                0.5,
                                DockBuilder::tab_node("project_explorer")
                                    .push_tab(DockBuilder::window("process_selector"))
                                    .visible(false)
                                    .push_tab(DockBuilder::window("project_explorer")),
                            )
                            .push_child(0.5, DockBuilder::window("property_viewer")),
                    )
                    .push_child(0.5, DockBuilder::window("output")),
            )
            .push_child(
                0.4,
                DockBuilder::tab_node("scan_results")
                    .push_tab(DockBuilder::window("scan_results"))
                    .push_tab(DockBuilder::window("settings")),
            )
            .build();

        #[cfg(target_os = "android")]
        let default_layout = DockBuilder::split_node(DockSplitDirection::HorizontalDivider)
            .push_child(
                0.55,
                DockBuilder::split_node(DockSplitDirection::VerticalDivider)
                    .push_child(
                        0.5,
                        DockBuilder::tab_node("project_explorer")
                            .push_tab(DockBuilder::window("process_selector").visible(false))
                            .push_tab(DockBuilder::window("project_explorer")),
                    )
                    .push_child(
                        0.5,
                        DockBuilder::tab_node("scan_results")
                            .push_tab(DockBuilder::window("scan_results"))
                            .push_tab(DockBuilder::window("settings")),
                    ),
            )
            .push_child(0.25, DockBuilder::window("property_viewer"))
            .push_child(0.2, DockBuilder::window("output"))
            .build();

        default_layout
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

    fn get_instance() -> &'static DockableWindowSettings {
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
            .unwrap_or_default()
            .parent()
            .unwrap_or(&Path::new(""))
            .join("docking_settings.json")
    }

    fn save_config() {
        if let Ok(config) = Self::get_instance().config.read() {
            if let Ok(json) = to_string_pretty(&*config) {
                let _ = fs::write(&Self::get_instance().config_file, json);
            }
        }
    }

    pub fn get_full_config() -> &'static Arc<RwLock<DockSettingsConfig>> {
        &Self::get_instance().config
    }

    pub fn get_dock_layout_settings() -> DockNode {
        if let Ok(config) = Self::get_instance().config.read() {
            config.dock_root.clone()
        } else {
            DockNode::default()
        }
    }

    pub fn set_dock_layout_settings(settings: &DockNode) {
        if let Ok(mut config) = Self::get_instance().config.write() {
            config.dock_root = settings.clone();
        }

        Self::save_config();
    }
}
