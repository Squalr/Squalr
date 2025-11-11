use crate::{
    app_context::AppContext, models::toolbar::toolbar_data::ToolbarData, ui::widgets::controls::toolbar_menu::toolbar_header_item_view::ToolbarHeaderItemView,
};
use eframe::egui::{Align, Layout, Response, Sense, Ui, UiBuilder, Widget};
use epaint::{CornerRadius, Rect, vec2};
use std::sync::Arc;

pub struct ToolbarView<'lifetime> {
    app_context: Arc<AppContext>,
    height: f32,
    menu: &'lifetime ToolbarData,
    on_select: &'lifetime dyn Fn(&'lifetime str),
}

impl<'lifetime> ToolbarView<'lifetime> {
    pub fn new(
        app_context: Arc<AppContext>,
        menu: &'lifetime ToolbarData,
        on_select: &'lifetime dyn Fn(&'lifetime str),
    ) -> Self {
        Self {
            app_context,
            height: 32.0,
            menu,
            on_select,
        }
    }
}

impl<'lifetime> Widget for ToolbarView<'lifetime> {
    fn ui(
        self,
        user_interface: &mut Ui,
    ) -> Response {
        let (allocated_size_rectangle, response) = user_interface.allocate_exact_size(vec2(user_interface.available_width(), self.height), Sense::empty());
        let theme = &self.app_context.theme;

        // Draw background.
        user_interface
            .painter()
            .rect_filled(allocated_size_rectangle, CornerRadius::ZERO, theme.background_primary);

        // Create child row area on which to place buttons.
        let allocated_size_rectangle = Rect::from_min_size(allocated_size_rectangle.min, vec2(allocated_size_rectangle.width(), self.height));
        let mut row_user_interface = user_interface.new_child(
            UiBuilder::new()
                .max_rect(allocated_size_rectangle)
                .layout(Layout::left_to_right(Align::Center)),
        );

        // Draw each menu header.
        for menu in &self.menu.menus {
            ToolbarHeaderItemView::new(self.app_context.clone(), &menu.header, &menu.items, 256.0, self.height, 8.0, self.on_select)
                .ui(&mut row_user_interface);
        }

        response
    }
}
