use crate::models::docking::builder::dock_builder::DockBuilder;
use crate::models::docking::hierarchy::dock_node::DockNode;
use crate::models::docking::hierarchy::types::dock_split_direction::DockSplitDirection;
use crate::views::element_scanner::scanner::element_scanner_view::ElementScannerView;
use crate::views::output::output_view::OutputView;
#[cfg(any(target_os = "windows", target_os = "macos", target_os = "linux"))]
use crate::views::pointer_scanner::pointer_scanner_view::PointerScannerView;
use crate::views::process_selector::process_selector_view::ProcessSelectorView;
use crate::views::project_explorer::project_explorer_view::ProjectExplorerView;
use crate::views::settings::settings_view::SettingsView;
use crate::views::struct_viewer::struct_viewer_view::StructViewerView;
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
                                DockBuilder::tab_node(ProjectExplorerView::WINDOW_ID)
                                    .push_tab(DockBuilder::window(ProcessSelectorView::WINDOW_ID))
                                    .visible(false)
                                    .push_tab(DockBuilder::window(ProjectExplorerView::WINDOW_ID)),
                            )
                            .push_child(0.5, DockBuilder::window(StructViewerView::WINDOW_ID)),
                    )
                    .push_child(0.5, DockBuilder::window(OutputView::WINDOW_ID)),
            )
            .push_child(
                0.4,
                DockBuilder::tab_node(ElementScannerView::WINDOW_ID)
                    .push_tab(DockBuilder::window(ElementScannerView::WINDOW_ID))
                    .push_tab(DockBuilder::window(PointerScannerView::WINDOW_ID))
                    .push_tab(DockBuilder::window(SettingsView::WINDOW_ID)),
            )
            .build();

        #[cfg(target_os = "android")]
        let default_layout = DockBuilder::split_node(DockSplitDirection::HorizontalDivider)
            .push_child(
                0.55,
                DockBuilder::split_node(DockSplitDirection::VerticalDivider)
                    .push_child(
                        0.5,
                        DockBuilder::tab_node(ProjectExplorerView::WINDOW_ID)
                            .push_tab(DockBuilder::window(ProcessSelectorView::WINDOW_ID).visible(false))
                            .push_tab(DockBuilder::window(ProjectExplorerView::WINDOW_ID)),
                    )
                    .push_child(
                        0.5,
                        DockBuilder::tab_node(ElementScannerView::WINDOW_ID)
                            .push_tab(DockBuilder::window(ElementScannerView::WINDOW_ID))
                            .push_tab(DockBuilder::window(SettingsView::WINDOW_ID)),
                    ),
            )
            .push_child(0.25, DockBuilder::window(StructViewerView::WINDOW_ID))
            .push_child(0.2, DockBuilder::window(OutputView::WINDOW_ID))
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
