use crate::models::toolbar::toolbar_data::ToolbarData;
use crate::models::toolbar::toolbar_header_item_data::ToolbarHeaderItemData;
use crate::models::toolbar::toolbar_menu_item_data::ToolbarMenuItemData;
use crate::ui::widgets::controls::toolbar_menu::toolbar_view::ToolbarView;
use crate::views::code_viewer::code_viewer_view::CodeViewerView;
use crate::views::element_scanner::scanner::element_scanner_view::ElementScannerView;
use crate::views::main_window::main_window_take_over_state::MainWindowTakeOverState;
use crate::views::memory_viewer::memory_viewer_view::MemoryViewerView;
use crate::views::output::output_view::OutputView;
use crate::views::plugins::plugins_view::PluginsView;
use crate::views::pointer_scanner::pointer_scanner_view::PointerScannerView;
use crate::views::process_selector::process_selector_view::ProcessSelectorView;
use crate::views::project_explorer::project_explorer_view::ProjectExplorerView;
use crate::views::settings::settings_view::SettingsView;
use crate::views::struct_editor::struct_editor_view::StructEditorView;
use crate::views::struct_viewer::struct_viewer_view::StructViewerView;
use crate::views::symbol_explorer::symbol_explorer_view::SymbolExplorerView;
use crate::views::symbol_table::symbol_table_view::SymbolTableView;
use crate::{app_context::AppContext, models::docking::settings::dockable_window_settings::DockSettingsConfig};
use eframe::egui::viewport::ViewportCommand;
use eframe::egui::{Response, Ui, Widget};
use squalr_engine_api::commands::project::export::project_export_request::ProjectExportRequest;
use squalr_engine_api::commands::unprivileged_command_request::UnprivilegedCommandRequest;
use squalr_engine_api::engine::engine_execution_context::EngineExecutionContext;
use squalr_engine_api::structures::projects::project_manager::ProjectManager;
use std::sync::{Arc, RwLock};

#[derive(Clone)]
pub struct MainToolbarView {
    app_context: Arc<AppContext>,
    menu_toolbar_data: Arc<RwLock<ToolbarData>>,
    main_window_take_over_state: Arc<RwLock<MainWindowTakeOverState>>,
}

impl MainToolbarView {
    pub const ACTION_ID_EXIT: &'static str = "exit";
    pub const ACTION_ID_SELECT_PROJECT: &'static str = "select_project";
    pub const ACTION_ID_EXPORT_PROJECT: &'static str = "export_project";
    pub const ACTION_ID_RESET_LAYOUT: &'static str = "layout_reset";
    pub const ACTION_ID_SHOW_ABOUT: &'static str = "show_about";

    pub fn new(
        app_context: Arc<AppContext>,
        main_window_take_over_state: Arc<RwLock<MainWindowTakeOverState>>,
    ) -> Self {
        let app_context_for_project_export = app_context.clone();
        let docking_manager_for_process_selector = app_context.docking_manager.clone();
        let docking_manager_for_project_explorer = app_context.docking_manager.clone();
        let docking_manager_for_symbol_explorer = app_context.docking_manager.clone();
        let docking_manager_for_symbol_table = app_context.docking_manager.clone();
        let docking_manager_for_struct_viewer = app_context.docking_manager.clone();
        let docking_manager_for_struct_editor = app_context.docking_manager.clone();
        let docking_manager_for_memory_viewer = app_context.docking_manager.clone();
        let docking_manager_for_code_viewer = app_context.docking_manager.clone();
        let docking_manager_for_output = app_context.docking_manager.clone();
        let docking_manager_for_pointer_scanner = app_context.docking_manager.clone();
        let docking_manager_for_plugins = app_context.docking_manager.clone();
        let docking_manager_for_element_scanner = app_context.docking_manager.clone();
        let docking_manager_for_settings = app_context.docking_manager.clone();

        let menus = vec![
            ToolbarHeaderItemData {
                header: "File".into(),
                items: vec![
                    ToolbarMenuItemData::new(MainToolbarView::ACTION_ID_SELECT_PROJECT, "Select Project", None),
                    ToolbarMenuItemData::new(MainToolbarView::ACTION_ID_EXPORT_PROJECT, "Export Project as Table...", None)
                        .with_enabled_state(Box::new(move || MainToolbarView::has_opened_project(app_context_for_project_export.as_ref()))),
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
                        SymbolExplorerView::WINDOW_ID,
                        "Symbol Tree",
                        Some(Box::new(move || {
                            if let Ok(docking_manager) = docking_manager_for_symbol_explorer.read() {
                                if let Some(docked_node) = docking_manager.get_node_by_id(SymbolExplorerView::WINDOW_ID) {
                                    return Some(docked_node.is_visible());
                                }
                            }

                            None
                        })),
                    )
                    .with_separator(),
                    ToolbarMenuItemData::new(
                        SymbolTableView::WINDOW_ID,
                        "Symbol Table",
                        Some(Box::new(move || {
                            if let Ok(docking_manager) = docking_manager_for_symbol_table.read() {
                                if let Some(docked_node) = docking_manager.get_node_by_id(SymbolTableView::WINDOW_ID) {
                                    return Some(docked_node.is_visible());
                                }
                            }

                            None
                        })),
                    ),
                    ToolbarMenuItemData::new(
                        StructViewerView::WINDOW_ID,
                        "Details Viewer",
                        Some(Box::new(move || {
                            if let Ok(docking_manager) = docking_manager_for_struct_viewer.read() {
                                if let Some(docked_node) = docking_manager.get_node_by_id(StructViewerView::WINDOW_ID) {
                                    return Some(docked_node.is_visible());
                                }
                            }

                            None
                        })),
                    ),
                    ToolbarMenuItemData::new(
                        StructEditorView::WINDOW_ID,
                        "Struct Editor",
                        Some(Box::new(move || {
                            if let Ok(docking_manager) = docking_manager_for_struct_editor.read() {
                                if let Some(docked_node) = docking_manager.get_node_by_id(StructEditorView::WINDOW_ID) {
                                    return Some(docked_node.is_visible());
                                }
                            }

                            None
                        })),
                    ),
                    ToolbarMenuItemData::new(
                        MemoryViewerView::WINDOW_ID,
                        "Memory Viewer",
                        Some(Box::new(move || {
                            if let Ok(docking_manager) = docking_manager_for_memory_viewer.read() {
                                if let Some(docked_node) = docking_manager.get_node_by_id(MemoryViewerView::WINDOW_ID) {
                                    return Some(docked_node.is_visible());
                                }
                            }

                            None
                        })),
                    )
                    .with_separator(),
                    ToolbarMenuItemData::new(
                        CodeViewerView::WINDOW_ID,
                        "Code Viewer",
                        Some(Box::new(move || {
                            if let Ok(docking_manager) = docking_manager_for_code_viewer.read() {
                                if let Some(docked_node) = docking_manager.get_node_by_id(CodeViewerView::WINDOW_ID) {
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
                    )
                    .with_separator(),
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
                    )
                    .with_separator(),
                    ToolbarMenuItemData::new(
                        PluginsView::WINDOW_ID,
                        "Plugins",
                        Some(Box::new(move || {
                            if let Ok(docking_manager) = docking_manager_for_plugins.read() {
                                if let Some(docked_node) = docking_manager.get_node_by_id(PluginsView::WINDOW_ID) {
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
                header: "Help".into(),
                items: vec![ToolbarMenuItemData::new(
                    MainToolbarView::ACTION_ID_SHOW_ABOUT,
                    "About",
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
            main_window_take_over_state,
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
            MainToolbarView::ACTION_ID_EXPORT_PROJECT => {
                let project_export_request = ProjectExportRequest {
                    project_directory_path: None,
                    project_name: None,
                    open_export_folder: true,
                };

                project_export_request.send(&app_context.engine_unprivileged_state, |project_export_response| {
                    if project_export_response.success {
                        log::info!("Exported opened project as a JSON table.");
                    } else {
                        log::error!("Failed to export opened project as a JSON table.");
                    }
                });
            }
            MainToolbarView::ACTION_ID_SHOW_ABOUT => match self.main_window_take_over_state.write() {
                Ok(mut main_window_take_over_state) => {
                    *main_window_take_over_state = MainWindowTakeOverState::About;
                }
                Err(error) => {
                    log::error!("Failed to acquire main window take over state while opening About: {}", error);
                }
            },
            ProcessSelectorView::WINDOW_ID
            | ProjectExplorerView::WINDOW_ID
            | SymbolExplorerView::WINDOW_ID
            | SymbolTableView::WINDOW_ID
            | StructViewerView::WINDOW_ID
            | StructEditorView::WINDOW_ID
            | MemoryViewerView::WINDOW_ID
            | CodeViewerView::WINDOW_ID
            | OutputView::WINDOW_ID
            | PointerScannerView::WINDOW_ID
            | PluginsView::WINDOW_ID
            | ElementScannerView::WINDOW_ID
            | SettingsView::WINDOW_ID
            // | "window_disassembly"
            // | "window_code_tracer"
            => {
                let docking_manager = &app_context.docking_manager;

                if let Ok(mut docking_manager) = docking_manager.write() {
                    docking_manager.toggle_window_visibility(selected_id);
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

impl MainToolbarView {
    fn has_opened_project(app_context: &AppContext) -> bool {
        Self::project_manager_has_opened_project(
            app_context
                .engine_unprivileged_state
                .get_project_manager()
                .as_ref(),
        )
    }

    fn project_manager_has_opened_project(project_manager: &ProjectManager) -> bool {
        let opened_project = project_manager.get_opened_project();

        match opened_project.read() {
            Ok(opened_project) => opened_project.is_some(),
            Err(error) => {
                log::error!("Failed to acquire opened project lock while checking toolbar project availability: {}", error);
                false
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::MainToolbarView;
    use squalr_engine_api::structures::projects::project::Project;
    use squalr_engine_api::structures::projects::project_info::ProjectInfo;
    use squalr_engine_api::structures::projects::project_items::built_in_types::project_item_type_directory::ProjectItemTypeDirectory;
    use squalr_engine_api::structures::projects::project_items::project_item_ref::ProjectItemRef;
    use squalr_engine_api::structures::projects::project_manager::ProjectManager;
    use squalr_engine_api::structures::projects::project_manifest::ProjectManifest;
    use std::collections::HashMap;
    use std::path::PathBuf;

    fn create_opened_project() -> Project {
        let project_file_path = PathBuf::from("C:/Projects/TestProject/project.json");
        let project_info = ProjectInfo::new(project_file_path, None, ProjectManifest::default());
        let project_root_ref = ProjectItemRef::new(PathBuf::from("C:/Projects/TestProject/project_items"));
        let mut project_items = HashMap::new();

        project_items.insert(project_root_ref.clone(), ProjectItemTypeDirectory::new_project_item(&project_root_ref));

        Project::new(project_info, project_items, project_root_ref)
    }

    #[test]
    fn project_manager_has_opened_project_returns_false_without_open_project() {
        let project_manager = ProjectManager::new();

        assert!(!MainToolbarView::project_manager_has_opened_project(&project_manager));
    }

    #[test]
    fn project_manager_has_opened_project_returns_true_with_open_project() {
        let project_manager = ProjectManager::new();
        let opened_project = project_manager.get_opened_project();

        *opened_project
            .write()
            .expect("Expected to acquire opened project write lock for test.") = Some(create_opened_project());

        assert!(MainToolbarView::project_manager_has_opened_project(&project_manager));
    }
}
