use crate::app_context::AppContext;
use crate::models::toolbar::toolbar_data::ToolbarData;
use crate::models::toolbar::toolbar_header_item_data::ToolbarHeaderItemData;
use crate::models::toolbar::toolbar_menu_item_data::ToolbarMenuItemData;
use crate::ui::widgets::controls::toolbar_menu::toolbar_view::ToolbarView;
use eframe::egui::{Response, Ui, Widget};
use std::rc::Rc;

#[derive(Clone)]
pub struct MainToolbarView {
    app_context: Rc<AppContext>,
    menu: ToolbarData,
}

impl MainToolbarView {
    pub fn new(app_context: Rc<AppContext>) -> Self {
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
                    ToolbarMenuItemData::new("win_process_selector", "Process Selector", Some(false)),
                    ToolbarMenuItemData::new("win_project_explorer", "Project Explorer", Some(true)),
                    ToolbarMenuItemData::new("win_struct_viewer", "Struct Viewer", Some(true)),
                    ToolbarMenuItemData::new("win_memory_viewer", "Memory Viewer", Some(false)),
                    ToolbarMenuItemData::new("win_output", "Output", Some(true)),
                    ToolbarMenuItemData::new("win_pointer_scanner", "Pointer Scanner", Some(false)),
                    ToolbarMenuItemData::new("win_element_scanner", "Element Scanner", Some(true)),
                    ToolbarMenuItemData::new("win_settings", "Settings", Some(true)),
                    ToolbarMenuItemData::new("win_snapshot_manager", "Snapshot Manager", Some(false)),
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
        let bar = ToolbarView::new(self.app_context.clone(), &self.menu);

        user_interface.add(bar)
    }
}
