use crate::app_context::AppContext;
use crate::models::toolbar::toolbar_data::ToolbarData;
use crate::models::toolbar::toolbar_header_item_data::ToolbarHeaderItemData;
use crate::models::toolbar::toolbar_menu_item_data::ToolbarMenuItemData;
use crate::ui::widgets::controls::toolbar_menu::toolbar_view::ToolbarView;
use eframe::egui::{Response, Sense, Ui, Widget};
use epaint::{CornerRadius, vec2};
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
                    ToolbarMenuItemData::action("select_project", "Select Project"),
                    ToolbarMenuItemData::action("export_project", "Export Project as Table..."),
                    ToolbarMenuItemData::action("exit", "Exit Squalr").with_separator(),
                ]
                .into(),
            },
            ToolbarHeaderItemData {
                header: "Layout".into(),
                items: vec![ToolbarMenuItemData::action("layout_reset", "Reset Layout")].into(),
            },
            ToolbarHeaderItemData {
                header: "Windows".into(),
                items: vec![
                    ToolbarMenuItemData::checkable("win_process_selector", "Process Selector", false),
                    ToolbarMenuItemData::checkable("win_project_explorer", "Project Explorer", true),
                    ToolbarMenuItemData::checkable("win_struct_viewer", "Struct Viewer", true),
                    ToolbarMenuItemData::checkable("win_memory_viewer", "Memory Viewer", false),
                    ToolbarMenuItemData::checkable("win_output", "Output", true),
                    ToolbarMenuItemData::checkable("win_pointer_scanner", "Pointer Scanner", false),
                    ToolbarMenuItemData::checkable("win_element_scanner", "Element Scanner", true),
                    ToolbarMenuItemData::checkable("win_settings", "Settings", true),
                    ToolbarMenuItemData::checkable("win_snapshot_manager", "Snapshot Manager", false),
                ]
                .into(),
            },
            ToolbarHeaderItemData {
                header: "Scans".into(),
                items: vec![ToolbarMenuItemData::action("scans_pointer", "Pointer Scan")].into(),
            },
            ToolbarHeaderItemData {
                header: "Debugger".into(),
                items: vec![
                    ToolbarMenuItemData::action("disassembly", "Disassembly"),
                    ToolbarMenuItemData::action("code_tracer", "Code Tracer"),
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
