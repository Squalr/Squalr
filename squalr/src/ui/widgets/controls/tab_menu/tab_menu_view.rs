use crate::{app_context::AppContext, models::tab_menu::tab_menu_data::TabMenuData, ui::widgets::controls::tab_menu::tab_item_view::TabItemView};
use eframe::egui::{Align, Layout, Response, Sense, Ui, UiBuilder, Widget};
use epaint::{CornerRadius, Rect, vec2};
use std::{rc::Rc, sync::atomic::Ordering};

pub struct TabMenuView<'lifetime> {
    app_context: Rc<AppContext>,
    height: f32,
    tab_menu_data: &'lifetime TabMenuData,
}

impl<'lifetime> TabMenuView<'lifetime> {
    pub fn new(
        app_context: Rc<AppContext>,
        tab_menu_data: &'lifetime TabMenuData,
    ) -> Self {
        Self {
            app_context,
            height: 28.0,
            tab_menu_data,
        }
    }
}

impl<'lifetime> Widget for TabMenuView<'lifetime> {
    fn ui(
        self,
        user_interface: &mut Ui,
    ) -> Response {
        let (available_size_rectangle, response) = user_interface.allocate_exact_size(vec2(user_interface.available_width(), self.height), Sense::empty());
        let theme = &self.app_context.theme;

        // Draw background.
        user_interface
            .painter()
            .rect_filled(available_size_rectangle, CornerRadius::ZERO, theme.background_primary);

        // Create child row area on which to place buttons.
        let available_size_rectangle = Rect::from_min_size(available_size_rectangle.min, vec2(available_size_rectangle.width(), self.height));
        let mut row_user_interface = user_interface.new_child(
            UiBuilder::new()
                .max_rect(available_size_rectangle)
                .layout(Layout::left_to_right(Align::Center)),
        );

        // Draw each tab header.
        for index in 0..self.tab_menu_data.headers.len() {
            let is_selected = index == self.tab_menu_data.active_tab_index.load(Ordering::Acquire) as usize;

            if TabItemView::new(
                self.app_context.clone(),
                &self.tab_menu_data.headers[index],
                96.0,
                self.height,
                8.0,
                is_selected,
            )
            .ui(&mut row_user_interface)
            .clicked()
            {
                self.tab_menu_data
                    .active_tab_index
                    .store(index as i32, Ordering::Release);
            }
        }

        response
    }
}
