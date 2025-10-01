use crate::{app_context::AppContext, models::tab_menu::tab_menu_data::TabMenuData, ui::widgets::controls::tab_menu::tab_item_view::TabItemView};
use eframe::egui::{Align, Layout, Response, Sense, Ui, UiBuilder, Widget};
use epaint::{CornerRadius, Rect, vec2};
use std::rc::Rc;

pub struct TabMenuView<'a> {
    app_context: Rc<AppContext>,
    height: f32,
    tab_menu_data: &'a TabMenuData,
}

impl<'a> TabMenuView<'a> {
    pub fn new(
        app_context: Rc<AppContext>,
        tab_menu_data: &'a TabMenuData,
    ) -> Self {
        Self {
            app_context,
            height: 24.0,
            tab_menu_data,
        }
    }
}

impl<'a> Widget for TabMenuView<'a> {
    fn ui(
        self,
        user_interface: &mut Ui,
    ) -> Response {
        let (available_size, response) = user_interface.allocate_exact_size(vec2(user_interface.available_width(), self.height), Sense::hover());
        let theme = &self.app_context.theme;

        // Draw background.
        user_interface
            .painter()
            .rect_filled(available_size, CornerRadius::ZERO, theme.background_primary);

        // Create child row area on which to place buttons.
        let available_size_rectangle = Rect::from_min_size(available_size.min, vec2(available_size.width(), self.height));
        let mut row_user_interface = user_interface.new_child(
            UiBuilder::new()
                .max_rect(available_size_rectangle)
                .layout(Layout::left_to_right(Align::Center)),
        );

        // Draw each tab header.
        for index in 0..self.tab_menu_data.headers.len() - 1 {
            if TabItemView::new(self.app_context.clone(), &self.tab_menu_data.headers[index], self.height, 8.0)
                .ui(&mut row_user_interface)
                .clicked()
            {
                //
            }
        }

        response
    }
}
