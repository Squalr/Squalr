// ui/widgets/main_window/main_toolbar_view.rs
use crate::ui::theme::Theme;
use crate::ui::widgets::controls::toolbar_menu::toolbar_menu_bar::ToolbarMenuBar;
use crate::ui::widgets::controls::toolbar_menu::toolbar_menu_check_state::ToolbarMenuCheckState;
use crate::ui::widgets::controls::toolbar_menu::toolbar_menu_data::ToolbarMenuData;
use crate::ui::widgets::controls::toolbar_menu::toolbar_menu_item_data::ToolbarMenuItemData;
use eframe::egui::{Context, Response, Sense, Ui, Widget};
use epaint::{CornerRadius, vec2};
use std::rc::Rc;

#[derive(Clone)]
pub struct MainToolbarView {
    _context: Context,
    theme: Rc<Theme>,
    height: f32,
    menus: Vec<ToolbarMenuData>,
    last_clicked: String,
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

        Self {
            _context: context,
            theme,
            height,
            menus,
            last_clicked: String::new(),
        }
    }

    /// Update a window item’s check state from your view model.
    pub fn set_window_checked(
        &mut self,
        id: &str,
        checked: bool,
    ) {
        for menu in self.menus.iter_mut() {
            for item in menu.items.iter_mut() {
                if item.id == id {
                    item.check_state = if checked {
                        ToolbarMenuCheckState::Checked
                    } else {
                        ToolbarMenuCheckState::Unchecked
                    };
                }
            }
        }
    }
}

impl Widget for MainToolbarView {
    fn ui(
        self,
        ui: &mut Ui,
    ) -> Response {
        let (rect, response) = ui.allocate_exact_size(vec2(ui.available_size().x, self.height), Sense::hover());

        // Background strip (matches your Theme usage)
        ui.painter()
            .rect_filled(rect, CornerRadius::ZERO, self.theme.background_primary);

        // Compose the menu bar within this space
        let bar = ToolbarMenuBar {
            theme: self.theme.clone(),
            height: self.height,
            bottom_padding: 4.0,
            menus: &self.menus,
            clicked_out: &self.last_clicked,
        };
        ui.put(rect, bar);

        response
    }
}
