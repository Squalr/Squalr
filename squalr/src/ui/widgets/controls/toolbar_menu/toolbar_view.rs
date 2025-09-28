use crate::ui::widgets::controls::toolbar_menu::data_model::toolbar_data::ToolbarData;
use crate::ui::{theme::Theme, widgets::controls::toolbar_menu::toolbar_button_view::ToolbarButtonView};
use eframe::egui::{Align, Layout, Response, Sense, Ui, UiBuilder, Widget};
use epaint::{CornerRadius, vec2};
use std::rc::Rc;

pub struct ToolbarView<'a> {
    theme: Rc<Theme>,
    height: f32,
    bottom_padding: f32,
    menu: &'a ToolbarData,
}

impl<'a> ToolbarView<'a> {
    pub fn new(
        theme: Rc<Theme>,
        height: f32,
        bottom_padding: f32,
        menu: &'a ToolbarData,
    ) -> Self {
        Self {
            theme,
            height,
            bottom_padding,
            menu,
        }
    }
}

impl<'a> Widget for ToolbarView<'a> {
    fn ui(
        self,
        user_interface: &mut Ui,
    ) -> Response {
        // Reserve space (full width x height + bottom padding)
        let total_h = self.height + self.bottom_padding;
        let (rect, response) = user_interface.allocate_exact_size(vec2(user_interface.available_width(), total_h), Sense::hover());

        // Background
        user_interface
            .painter()
            .rect_filled(rect, CornerRadius::ZERO, self.theme.background_primary);

        // Row area (use new_child instead of deprecated child_ui)
        let row_rect = eframe::egui::Rect::from_min_size(rect.min, vec2(rect.width(), self.height));
        let mut row_ui = user_interface.new_child(
            UiBuilder::new()
                .max_rect(row_rect)
                .layout(Layout::left_to_right(Align::Center)),
        );

        // Draw each top-level menu.
        for menu in &self.menu.menus {
            ToolbarButtonView::new(self.theme.clone(), &menu.header, &menu.items, self.height, 8.0).ui(&mut row_ui);
        }

        response
    }
}
