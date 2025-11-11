use crate::models::toolbar::toolbar_data::ToolbarData;
use crate::models::toolbar::toolbar_header_item_data::ToolbarHeaderItemData;
use crate::models::toolbar::toolbar_menu_item_data::ToolbarMenuItemData;
use crate::ui::widgets::controls::toolbar_menu::toolbar_view::ToolbarView;
use crate::{app_context::AppContext, models::docking::settings::dockable_window_settings::DockSettingsConfig};
use eframe::egui::{Response, Ui, Widget};
use std::sync::Arc;

#[derive(Clone)]
pub struct MainToolbarView {
    app_context: Arc<AppContext>,
    menu: ToolbarData,
}

impl MainToolbarView {
    pub fn new(app_context: Arc<AppContext>) -> Self {
        let menus = vec![
            ToolbarHeaderItemData {
                header: "File".into(),
                items: vec![
                    ToolbarMenuItemData::new("select_project", "Select Project", None),
                    ToolbarMenuItemData::new("export_project", "Export Project as Table...", None),
                    ToolbarMenuItemData::new("exit", "Exit Squalr", None).with_separator(),
                ]
                .into(),
            },
            ToolbarHeaderItemData {
                header: "Layout".into(),
                items: vec![ToolbarMenuItemData::new("layout_reset", "Reset Layout", None)].into(),
            },
            ToolbarHeaderItemData {
                header: "Windows".into(),
                items: vec![
                    ToolbarMenuItemData::new("window_process_selector", "Process Selector", Some(false)),
                    ToolbarMenuItemData::new("window_project_explorer", "Project Explorer", Some(true)),
                    ToolbarMenuItemData::new("window_struct_viewer", "Struct Viewer", Some(true)),
                    ToolbarMenuItemData::new("window_memory_viewer", "Memory Viewer", Some(false)),
                    ToolbarMenuItemData::new("window_output", "Output", Some(true)),
                    ToolbarMenuItemData::new("window_pointer_scanner", "Pointer Scanner", Some(false)),
                    ToolbarMenuItemData::new("window_element_scanner", "Element Scanner", Some(true)),
                    ToolbarMenuItemData::new("window_settings", "Settings", Some(true)),
                    ToolbarMenuItemData::new("window_snapshot_manager", "Snapshot Manager", Some(false)),
                ]
                .into(),
            },
            ToolbarHeaderItemData {
                header: "Scans".into(),
                items: vec![ToolbarMenuItemData::new("scans_pointer", "Pointer Scan", None)].into(),
            },
            ToolbarHeaderItemData {
                header: "Debugger".into(),
                items: vec![
                    ToolbarMenuItemData::new("disassembly", "Disassembly", None),
                    ToolbarMenuItemData::new("code_tracer", "Code Tracer", None),
                ]
                .into(),
            },
        ]
        .into();

        let menu = ToolbarData {
            active_menu: String::new(),
            menus,
        };

        Self { app_context, menu }
    }
}

impl Widget for MainToolbarView {
    fn ui(
        self,
        user_interface: &mut Ui,
    ) -> Response {
        let app_context = self.app_context.clone();
        let callback = &move |selected_id| match selected_id {
            "layout_reset" => match app_context.docking_manager.write() {
                Ok(mut docking_manager) => {
                    docking_manager.set_root(DockSettingsConfig::get_default_layout());
                }
                Err(error) => {
                    log::error!("Failed to acquire docking manager to reset layout: {}", error);
                }
            },
            _ => {}
        };
        let bar = ToolbarView::new(self.app_context.clone(), &self.menu, callback);

        user_interface.add(bar)
    }
}
