// ui/widgets/main_window/main_toolbar_view.rs
use crate::ui::widgets::controls::toolbar_menu::data_model::toolbar_menu_data::ToolbarMenuData;
use crate::ui::widgets::controls::toolbar_menu::data_model::toolbar_menu_item_data::ToolbarMenuItemData;
use crate::ui::widgets::controls::toolbar_menu::toolbar_view::ToolbarView;
use crate::ui::{theme::Theme, widgets::controls::toolbar_menu::data_model::toolbar_data::ToolbarData};
use eframe::egui::{Context, Response, Sense, Ui, Widget};
use epaint::{CornerRadius, vec2};
use std::rc::Rc;

#[derive(Clone)]
pub struct MainToolbarView {
    _context: Context,
    theme: Rc<Theme>,
    height: f32,
    menu: ToolbarData,
}

impl MainToolbarView {
    pub fn new(
        context: Context,
        theme: Rc<Theme>,
        height: f32,
    ) -> Self {
        // Build your initial menu structure here (port of your Slint tree).
        let menus = vec![
            ToolbarMenuData {
                header: "File".into(),
                items: vec![
                    ToolbarMenuItemData::action("select_project", "Select Project"),
                    ToolbarMenuItemData::action("export_project", "Export Project as Table..."),
                    ToolbarMenuItemData::action("exit", "Exit Squalr").with_separator(),
                ],
            },
            ToolbarMenuData {
                header: "Layout".into(),
                items: vec![ToolbarMenuItemData::action("layout_reset", "Reset Layout")],
            },
            ToolbarMenuData {
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
            ToolbarMenuData {
                header: "Scans".into(),
                items: vec![ToolbarMenuItemData::action("scans_pointer", "Pointer Scan")],
            },
            ToolbarMenuData {
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

        Self {
            _context: context,
            theme,
            height,
            menu,
        }
    }
}

impl Widget for MainToolbarView {
    fn ui(
        self,
        user_interface: &mut Ui,
    ) -> Response {
        let (rect, response) = user_interface.allocate_exact_size(vec2(user_interface.available_size().x, self.height), Sense::hover());

        // Background strip (matches your Theme usage).
        user_interface
            .painter()
            .rect_filled(rect, CornerRadius::ZERO, self.theme.background_primary);

        // Compose the menu bar within this space.
        let bar = ToolbarView::new(self.theme.clone(), self.height, 4.0, &self.menu);

        user_interface.put(rect, bar);

        response
    }
}
