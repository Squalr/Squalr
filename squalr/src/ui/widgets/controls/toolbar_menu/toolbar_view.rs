use crate::{
    models::toolbar::toolbar_data::ToolbarData,
    ui::{theme::Theme, widgets::controls::toolbar_menu::toolbar_header_item_view::ToolbarHeaderItemView},
};
use eframe::egui::{Align, Layout, Response, Sense, Ui, UiBuilder, Widget};
use epaint::{CornerRadius, Rect, vec2};
use std::rc::Rc;

pub struct ToolbarView<'a> {
    theme: Rc<Theme>,
    height: f32,
    menu: &'a ToolbarData,
}

impl<'a> ToolbarView<'a> {
    pub fn new(
        theme: Rc<Theme>,
        height: f32,
        menu: &'a ToolbarData,
    ) -> Self {
        Self { theme, height, menu }
    }
}

impl<'a> Widget for ToolbarView<'a> {
    fn ui(
        self,
        user_interface: &mut Ui,
    ) -> Response {
        let (available_size, response) = user_interface.allocate_exact_size(vec2(user_interface.available_width(), self.height), Sense::hover());

        // Draw background.
        user_interface
            .painter()
            .rect_filled(available_size, CornerRadius::ZERO, self.theme.background_primary);

        // Create child row area on which to place buttons.
        let available_size_rectangle = Rect::from_min_size(available_size.min, vec2(available_size.width(), self.height));
        let mut row_user_interface = user_interface.new_child(
            UiBuilder::new()
                .max_rect(available_size_rectangle)
                .layout(Layout::left_to_right(Align::Center)),
        );

        // Draw each menu header.
        for menu in &self.menu.menus {
            ToolbarHeaderItemView::new(self.theme.clone(), &menu.header, &menu.items, self.height, 8.0).ui(&mut row_user_interface);
        }

        response
    }
}
