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
    height: f32,
    menu: ToolbarData,
}

impl MainToolbarView {
    pub fn new(
        app_context: Rc<AppContext>,
        height: f32,
    ) -> Self {
        let menus = vec![
            ToolbarHeaderItemData {
                header: "File".into(),
                items: vec![
                    ToolbarMenuItemData::action("select_project", "Select Project"),
                    ToolbarMenuItemData::action("export_project", "Export Project as Table..."),
                    ToolbarMenuItemData::action("exit", "Exit Squalr").with_separator(),
                ],
            },
            ToolbarHeaderItemData {
                header: "Layout".into(),
                items: vec![ToolbarMenuItemData::action("layout_reset", "Reset Layout")],
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
                ],
            },
            ToolbarHeaderItemData {
                header: "Scans".into(),
                items: vec![ToolbarMenuItemData::action("scans_pointer", "Pointer Scan")],
            },
            ToolbarHeaderItemData {
                header: "Debugger".into(),
                items: vec![
                    ToolbarMenuItemData::action("disassembly", "Disassembly"),
                    ToolbarMenuItemData::action("code_tracer", "Code Tracer"),
                ],
            },
        ];

        let menu = ToolbarData {
            active_menu: String::new(),
            menus,
        };

        Self { app_context, height, menu }
    }
}

impl Widget for MainToolbarView {
    fn ui(
        self,
        user_interface: &mut Ui,
    ) -> Response {
        let (available_size_rect, response) = user_interface.allocate_exact_size(vec2(user_interface.available_size().x, self.height), Sense::hover());
        let theme = &self.app_context.theme;

        // Background strip (matches your Theme usage).
        user_interface
            .painter()
            .rect_filled(available_size_rect, CornerRadius::ZERO, theme.background_primary);

        // Compose the menu bar within this space.
        let bar = ToolbarView::new(theme.clone(), self.height, &self.menu);

        user_interface.put(available_size_rect, bar);

        response
    }
}
