use crate::ui::widgets::controls::toolbar_menu::toolbar_menu_data::ToolbarMenuData;
use crate::ui::{theme::Theme, widgets::controls::toolbar_menu::toolbar_menu_button::ToolbarMenuButton};
use eframe::egui::{Align, Layout, Response, Sense, Ui, UiBuilder, Widget};
use epaint::{CornerRadius, vec2};
use std::rc::Rc;

pub struct ToolbarMenuBar<'a> {
    pub theme: Rc<Theme>,
    pub height: f32,
    pub bottom_padding: f32,
    pub menus: &'a Vec<ToolbarMenuData>,
    pub clicked_out: &'a String,
}

impl<'a> ToolbarMenuBar<'a> {
    pub fn new(
        theme: Rc<Theme>,
        height: f32,
        menus: &'a Vec<ToolbarMenuData>,
        clicked_out: &'a String,
    ) -> Self {
        Self {
            theme,
            height,
            bottom_padding: 4.0,
            menus,
            clicked_out,
        }
    }
}

impl<'a> Widget for ToolbarMenuBar<'a> {
    fn ui(
        self,
        ui: &mut Ui,
    ) -> Response {
        // Reserve space (full width x height + bottom padding)
        let total_h = self.height + self.bottom_padding;
        let (rect, response) = ui.allocate_exact_size(vec2(ui.available_width(), total_h), Sense::hover());

        // Background
        ui.painter()
            .rect_filled(rect, CornerRadius::ZERO, self.theme.background_primary);

        // Row area (use new_child instead of deprecated child_ui)
        let row_rect = eframe::egui::Rect::from_min_size(rect.min, vec2(rect.width(), self.height));
        let mut row_ui = ui.new_child(
            UiBuilder::new()
                .max_rect(row_rect)
                .layout(Layout::left_to_right(Align::Center)),
        );

        // Draw each top-level menu
        for menu in self.menus {
            // Local slot for this button’s click result.
            let mut local_click: Option<String> = None;

            ToolbarMenuButton {
                theme: self.theme.clone(),
                label: &menu.header,
                items: &menu.items,
                height: self.height,
                horizontal_padding: 8.0,
                clicked_out: None,
            }
            .ui(&mut row_ui);

            // If something was clicked, write it out and (optionally) break.
            if local_click.is_some() {
                // *self.clicked_out = local_click;
                // If you want the *last* one when multiple open, don't break.
                break;
            }
        }

        response
    }
}

// tiny helper to allow `.with(...)` on any type without a builder macro
trait With: Sized {
    fn with<F: FnOnce(Self) -> Self>(
        self,
        f: F,
    ) -> Self {
        f(self)
    }
}
impl<T> With for T {}
