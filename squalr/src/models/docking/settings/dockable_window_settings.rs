use crate::models::docking::builder::dock_builder::DockBuilder;
use crate::models::docking::hierarchy::dock_node::DockNode;
use crate::models::docking::hierarchy::types::dock_split_direction::DockSplitDirection;
use crate::views::code_viewer::code_viewer_view::CodeViewerView;
use crate::views::element_scanner::scanner::element_scanner_view::ElementScannerView;
use crate::views::memory_viewer::memory_viewer_view::MemoryViewerView;
use crate::views::output::output_view::OutputView;
use crate::views::plugins::plugins_view::PluginsView;
#[cfg(any(target_os = "windows", target_os = "macos", target_os = "linux"))]
use crate::views::pointer_scanner::pointer_scanner_view::PointerScannerView;
use crate::views::process_selector::process_selector_view::ProcessSelectorView;
use crate::views::project_explorer::project_explorer_view::ProjectExplorerView;
use crate::views::settings::settings_view::SettingsView;
use crate::views::struct_viewer::struct_viewer_view::StructViewerView;
use crate::views::symbol_explorer::symbol_explorer_view::SymbolExplorerView;
use crate::views::symbol_struct_editor::symbol_struct_editor_view::SymbolStructEditorView;
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
                                    .push_tab(DockBuilder::window(ProjectExplorerView::WINDOW_ID)),
                            )
                            .push_child(
                                0.5,
                                DockBuilder::tab_node(StructViewerView::WINDOW_ID)
                                    .push_tab(DockBuilder::window(StructViewerView::WINDOW_ID))
                                    .push_tab(DockBuilder::window(SymbolStructEditorView::WINDOW_ID))
                                    .push_tab(DockBuilder::window(SettingsView::WINDOW_ID)),
                            ),
                    )
                    .push_child(
                        0.5,
                        DockBuilder::tab_node(OutputView::WINDOW_ID)
                            .push_tab(DockBuilder::window(OutputView::WINDOW_ID))
                            .push_tab(DockBuilder::window(MemoryViewerView::WINDOW_ID))
                            .push_tab(DockBuilder::window(CodeViewerView::WINDOW_ID))
                            .push_tab(DockBuilder::window(PluginsView::WINDOW_ID)),
                    ),
            )
            .push_child(
                0.4,
                DockBuilder::tab_node(ElementScannerView::WINDOW_ID)
                    .push_tab(DockBuilder::window(ElementScannerView::WINDOW_ID))
                    .push_tab(DockBuilder::window(PointerScannerView::WINDOW_ID))
                    .push_tab(DockBuilder::window(SymbolExplorerView::WINDOW_ID)),
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
                            .push_tab(DockBuilder::window(ProcessSelectorView::WINDOW_ID))
                            .push_tab(DockBuilder::window(ProjectExplorerView::WINDOW_ID)),
                    )
                    .push_child(
                        0.5,
                        DockBuilder::tab_node(ElementScannerView::WINDOW_ID)
                            .push_tab(DockBuilder::window(ElementScannerView::WINDOW_ID))
                            .push_tab(DockBuilder::window(MemoryViewerView::WINDOW_ID))
                            .push_tab(DockBuilder::window(SymbolExplorerView::WINDOW_ID)),
                    ),
            )
            .push_child(
                0.25,
                DockBuilder::tab_node(StructViewerView::WINDOW_ID)
                    .push_tab(DockBuilder::window(StructViewerView::WINDOW_ID))
                    .push_tab(DockBuilder::window(SymbolStructEditorView::WINDOW_ID))
                    .push_tab(DockBuilder::window(SettingsView::WINDOW_ID)),
            )
            .push_child(
                0.2,
                DockBuilder::tab_node(OutputView::WINDOW_ID)
                    .push_tab(DockBuilder::window(OutputView::WINDOW_ID))
                    .push_tab(DockBuilder::window(MemoryViewerView::WINDOW_ID))
                    .push_tab(DockBuilder::window(CodeViewerView::WINDOW_ID))
                    .push_tab(DockBuilder::window(PluginsView::WINDOW_ID)),
            )
            .build();

        default_layout
    }

    fn ensure_required_windows_present(&mut self) {
        Self::remove_obsolete_window(&mut self.dock_root, "window_symbol_table");
        Self::ensure_tab_window(&mut self.dock_root, OutputView::WINDOW_ID, PluginsView::WINDOW_ID);
        Self::ensure_tab_window(&mut self.dock_root, OutputView::WINDOW_ID, MemoryViewerView::WINDOW_ID);
        Self::ensure_tab_window(&mut self.dock_root, OutputView::WINDOW_ID, CodeViewerView::WINDOW_ID);
        Self::ensure_tab_window(&mut self.dock_root, ElementScannerView::WINDOW_ID, SymbolExplorerView::WINDOW_ID);
        Self::ensure_tab_window(&mut self.dock_root, StructViewerView::WINDOW_ID, SymbolStructEditorView::WINDOW_ID);
        Self::ensure_tab_window(&mut self.dock_root, StructViewerView::WINDOW_ID, SettingsView::WINDOW_ID);
    }

    fn remove_obsolete_window(
        dock_root: &mut DockNode,
        obsolete_window_id: &str,
    ) {
        while let Some(obsolete_window_path) = dock_root.find_path_to_window_id(obsolete_window_id) {
            dock_root.remove_window_by_path(&obsolete_window_path);
        }
    }

    fn ensure_tab_window(
        dock_root: &mut DockNode,
        anchor_window_id: &str,
        missing_window_id: &str,
    ) {
        if dock_root.find_path_to_window_id(missing_window_id).is_some() {
            return;
        }

        let Some(anchor_container_path) = dock_root.find_path_to_window_container(anchor_window_id) else {
            return;
        };
        let Some(anchor_container_node) = dock_root.get_node_from_path_mut(&anchor_container_path) else {
            return;
        };

        if let DockNode::Tab { tabs, .. } = anchor_container_node {
            tabs.push(DockNode::Window {
                window_identifier: missing_window_id.to_string(),
                is_visible: true,
            });
        }
    }
}

pub struct DockableWindowSettings {
    config: Arc<RwLock<DockSettingsConfig>>,
    config_file: PathBuf,
}

impl DockableWindowSettings {
    fn new() -> Self {
        let config_file = Self::default_config_path();
        let mut config = if config_file.exists() {
            match fs::read_to_string(&config_file) {
                Ok(json) => serde_json::from_str(&json).unwrap_or_default(),
                Err(_) => DockSettingsConfig::default(),
            }
        } else {
            DockSettingsConfig::default()
        };
        config.ensure_required_windows_present();

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

#[cfg(test)]
mod tests {
    use super::DockSettingsConfig;
    use crate::models::docking::builder::dock_builder::DockBuilder;
    use crate::views::{
        code_viewer::code_viewer_view::CodeViewerView, element_scanner::scanner::element_scanner_view::ElementScannerView,
        memory_viewer::memory_viewer_view::MemoryViewerView, output::output_view::OutputView, plugins::plugins_view::PluginsView,
        pointer_scanner::pointer_scanner_view::PointerScannerView, settings::settings_view::SettingsView, struct_viewer::struct_viewer_view::StructViewerView,
        symbol_explorer::symbol_explorer_view::SymbolExplorerView, symbol_struct_editor::symbol_struct_editor_view::SymbolStructEditorView,
    };

    #[test]
    fn default_layout_places_output_related_windows_in_same_tab_group() {
        let dock_root = DockSettingsConfig::get_default_layout();

        assert!(dock_root.are_windows_in_same_tab_group(OutputView::WINDOW_ID, PluginsView::WINDOW_ID));
        assert!(dock_root.are_windows_in_same_tab_group(OutputView::WINDOW_ID, MemoryViewerView::WINDOW_ID));
        assert!(dock_root.are_windows_in_same_tab_group(OutputView::WINDOW_ID, CodeViewerView::WINDOW_ID));
    }

    #[test]
    fn default_layout_places_symbol_explorer_with_scan_windows() {
        let dock_root = DockSettingsConfig::get_default_layout();

        assert!(dock_root.are_windows_in_same_tab_group(ElementScannerView::WINDOW_ID, SymbolExplorerView::WINDOW_ID));
        assert!(dock_root.are_windows_in_same_tab_group(PointerScannerView::WINDOW_ID, SymbolExplorerView::WINDOW_ID));
    }

    #[test]
    fn ensure_required_windows_present_removes_obsolete_symbol_table_window() {
        let mut dock_settings_config = DockSettingsConfig {
            dock_root: DockBuilder::tab_node(ElementScannerView::WINDOW_ID)
                .push_tab(DockBuilder::window(ElementScannerView::WINDOW_ID))
                .push_tab(DockBuilder::window("window_symbol_table"))
                .build(),
        };

        dock_settings_config.ensure_required_windows_present();

        assert!(
            dock_settings_config
                .dock_root
                .find_path_to_window_id("window_symbol_table")
                .is_none()
        );
        assert!(
            dock_settings_config
                .dock_root
                .find_path_to_window_id(SymbolExplorerView::WINDOW_ID)
                .is_some()
        );
    }

    #[test]
    fn default_layout_places_settings_with_struct_viewer() {
        let dock_root = DockSettingsConfig::get_default_layout();

        assert!(dock_root.are_windows_in_same_tab_group(StructViewerView::WINDOW_ID, SettingsView::WINDOW_ID));
        assert!(dock_root.are_windows_in_same_tab_group(StructViewerView::WINDOW_ID, SymbolStructEditorView::WINDOW_ID));
    }
}
