use crate::models::toolbar::toolbar_data::ToolbarData;
use crate::models::toolbar::toolbar_header_item_data::ToolbarHeaderItemData;
use crate::models::toolbar::toolbar_menu_item_data::ToolbarMenuItemData;
use crate::ui::widgets::controls::toolbar_menu::toolbar_view::ToolbarView;
use crate::views::element_scanner::scanner::element_scanner_view::ElementScannerView;
use crate::views::output::output_view::OutputView;
use crate::views::pointer_scanner::pointer_scanner_view::PointerScannerView;
use crate::views::process_selector::process_selector_view::ProcessSelectorView;
use crate::views::project_explorer::project_explorer_view::ProjectExplorerView;
use crate::views::settings::settings_view::SettingsView;
use crate::views::struct_viewer::struct_viewer_view::StructViewerView;
use crate::{app_context::AppContext, models::docking::settings::dockable_window_settings::DockSettingsConfig};
use eframe::egui::viewport::ViewportCommand;
use eframe::egui::{Response, Ui, Widget};
use std::sync::{Arc, RwLock};

#[derive(Clone)]
pub struct MainToolbarView {
    app_context: Arc<AppContext>,
    menu_toolbar_data: Arc<RwLock<ToolbarData>>,
}

impl MainToolbarView {
    pub const ACTION_ID_EXIT: &'static str = "exit";
    pub const ACTION_ID_SELECT_PROJECT: &'static str = "select_project";
    pub const ACTION_ID_EXPORT_PROJECT: &'static str = "export_project";
    pub const ACTION_ID_RESET_LAYOUT: &'static str = "layout_reset";

    pub fn new(app_context: Arc<AppContext>) -> Self {
        let docking_manager_for_process_selector = app_context.docking_manager.clone();
        let docking_manager_for_project_explorer = app_context.docking_manager.clone();
        let docking_manager_for_struct_viewer = app_context.docking_manager.clone();
        let docking_manager_for_output = app_context.docking_manager.clone();
        let docking_manager_for_pointer_scanner = app_context.docking_manager.clone();
        let docking_manager_for_element_scanner = app_context.docking_manager.clone();
        let docking_manager_for_settings = app_context.docking_manager.clone();

        let menus = vec![
            ToolbarHeaderItemData {
                header: "File".into(),
                items: vec![
                    ToolbarMenuItemData::new(MainToolbarView::ACTION_ID_SELECT_PROJECT, "Select Project", None),
                    ToolbarMenuItemData::new(MainToolbarView::ACTION_ID_EXPORT_PROJECT, "Export Project as Table...", None),
                    ToolbarMenuItemData::new(MainToolbarView::ACTION_ID_EXIT, "Exit Squalr", None).with_separator(),
                ]
                .into(),
            },
            ToolbarHeaderItemData {
                header: "Layout".into(),
                items: vec![ToolbarMenuItemData::new(
                    MainToolbarView::ACTION_ID_RESET_LAYOUT,
                    "Reset Layout",
                    None,
                )]
                .into(),
            },
            ToolbarHeaderItemData {
                header: "Windows".into(),
                items: vec![
                    ToolbarMenuItemData::new(
                        ProcessSelectorView::WINDOW_ID,
                        "Process Selector",
                        Some(Box::new(move || {
                            if let Ok(docking_manager) = docking_manager_for_process_selector.read() {
                                if let Some(docked_node) = docking_manager.get_node_by_id(ProcessSelectorView::WINDOW_ID) {
                                    return Some(docked_node.is_visible());
                                }
                            }

                            None
                        })),
                    ),
                    ToolbarMenuItemData::new(
                        ProjectExplorerView::WINDOW_ID,
                        "Project Explorer",
                        Some(Box::new(move || {
                            if let Ok(docking_manager) = docking_manager_for_project_explorer.read() {
                                if let Some(docked_node) = docking_manager.get_node_by_id(ProjectExplorerView::WINDOW_ID) {
                                    return Some(docked_node.is_visible());
                                }
                            }

                            None
                        })),
                    ),
                    ToolbarMenuItemData::new(
                        StructViewerView::WINDOW_ID,
                        "Struct Viewer",
                        Some(Box::new(move || {
                            if let Ok(docking_manager) = docking_manager_for_struct_viewer.read() {
                                if let Some(docked_node) = docking_manager.get_node_by_id(StructViewerView::WINDOW_ID) {
                                    return Some(docked_node.is_visible());
                                }
                            }

                            None
                        })),
                    ),
                    // ToolbarMenuItemData::new(MemoryViewerView::WINDOW_ID, "Memory Viewer", Some(move || false)),
                    ToolbarMenuItemData::new(
                        OutputView::WINDOW_ID,
                        "Output",
                        Some(Box::new(move || {
                            if let Ok(docking_manager) = docking_manager_for_output.read() {
                                if let Some(docked_node) = docking_manager.get_node_by_id(OutputView::WINDOW_ID) {
                                    return Some(docked_node.is_visible());
                                }
                            }

                            None
                        })),
                    ),
                    ToolbarMenuItemData::new(
                        PointerScannerView::WINDOW_ID,
                        "Pointer Scanner",
                        Some(Box::new(move || {
                            if let Ok(docking_manager) = docking_manager_for_pointer_scanner.read() {
                                if let Some(docked_node) = docking_manager.get_node_by_id(PointerScannerView::WINDOW_ID) {
                                    return Some(docked_node.is_visible());
                                }
                            }

                            None
                        })),
                    ),
                    ToolbarMenuItemData::new(
                        ElementScannerView::WINDOW_ID,
                        "Element Scanner",
                        Some(Box::new(move || {
                            if let Ok(docking_manager) = docking_manager_for_element_scanner.read() {
                                if let Some(docked_node) = docking_manager.get_node_by_id(ElementScannerView::WINDOW_ID) {
                                    return Some(docked_node.is_visible());
                                }
                            }

                            None
                        })),
                    ),
                    ToolbarMenuItemData::new(
                        SettingsView::WINDOW_ID,
                        "Settings",
                        Some(Box::new(move || {
                            if let Ok(docking_manager) = docking_manager_for_settings.read() {
                                if let Some(docked_node) = docking_manager.get_node_by_id(SettingsView::WINDOW_ID) {
                                    return Some(docked_node.is_visible());
                                }
                            }

                            None
                        })),
                    ),
                ]
                .into(),
            },
            ToolbarHeaderItemData {
                header: "Scans".into(),
                items: vec![ToolbarMenuItemData::new(
                    PointerScannerView::WINDOW_ID,
                    "Pointer Scan",
                    None,
                )]
                .into(),
            },
            /*
            ToolbarHeaderItemData {
                header: "Debugger".into(),
                items: vec![
                    ToolbarMenuItemData::new("window_disassembly", "Disassembly", None),
                    ToolbarMenuItemData::new("window_code_tracer", "Code Tracer", None),
                ]
                .into(),
            },*/
        ]
        .into();

        let menu_toolbar_data = Arc::new(RwLock::new(ToolbarData {
            active_menu: String::new(),
            menus,
        }));

        Self {
            app_context,
            menu_toolbar_data,
        }
    }
}

impl Widget for MainToolbarView {
    fn ui(
        self,
        user_interface: &mut Ui,
    ) -> Response {
        let app_context = self.app_context.clone();
        let callback = &move |selected_id| match selected_id {
            MainToolbarView::ACTION_ID_EXIT => {
                app_context.context.send_viewport_cmd(ViewportCommand::Close);
            }
            ProcessSelectorView::WINDOW_ID
            | ProjectExplorerView::WINDOW_ID
            | StructViewerView::WINDOW_ID
            // | "window_memory_viewer"
            | OutputView::WINDOW_ID
            | PointerScannerView::WINDOW_ID
            | ElementScannerView::WINDOW_ID
            | SettingsView::WINDOW_ID
            | PointerScannerView::WINDOW_ID
            // | "window_disassembly"
            // | "window_code_tracer"
            => {
                let docking_manager = &app_context.docking_manager;

                if let Ok(mut docking_manager) = docking_manager.write() {
                    if let Some(docked_node) = docking_manager.get_node_by_id_mut(selected_id) {
                        docked_node.set_visible(!docked_node.is_visible());
                    }
                }
            }
            MainToolbarView::ACTION_ID_RESET_LAYOUT => match app_context.docking_manager.write() {
                Ok(mut docking_manager) => {
                    docking_manager.set_root(DockSettingsConfig::get_default_layout());
                }
                Err(error) => {
                    log::error!("Failed to acquire docking manager to reset layout: {}", error);
                }
            },
            _ => {}
        };

        match self.menu_toolbar_data.read() {
            Ok(menu_toolbar_data) => {
                let bar = ToolbarView::new(self.app_context.clone(), &menu_toolbar_data, callback);

                user_interface.add(bar)
            }
            Err(error) => {
                log::error!("Failed to acquire main toolbar menu data lock: {}", error);

                user_interface.response()
            }
        }
    }
}
