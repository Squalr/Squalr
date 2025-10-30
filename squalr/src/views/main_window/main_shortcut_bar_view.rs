use crate::{
    app_context::AppContext,
    ui::widgets::controls::combo_box::{combo_box_item_view::ComboBoxItemView, combo_box_view::ComboBoxView},
};
use eframe::egui::{Align, Layout, Response, Sense, Ui, UiBuilder, Widget};
use epaint::{CornerRadius, Rect, vec2};
use std::rc::Rc;

#[derive(Clone)]
pub struct MainShortcutBarView {
    app_context: Rc<AppContext>,
}

impl MainShortcutBarView {
    pub fn new(app_context: Rc<AppContext>) -> Self {
        Self { app_context }
    }
}

impl Widget for MainShortcutBarView {
    fn ui(
        self,
        user_interface: &mut Ui,
    ) -> Response {
        let (available_size_rectangle, response) = user_interface.allocate_exact_size(vec2(user_interface.available_width(), 32.0), Sense::empty());
        let theme = &self.app_context.theme;
        let process_dropdown_list_width = 192.0;

        // Draw background.
        user_interface
            .painter()
            .rect_filled(available_size_rectangle, CornerRadius::ZERO, theme.background_primary);

        let available_size_rectangle = Rect::from_min_size(available_size_rectangle.min, vec2(available_size_rectangle.width(), 32.0));
        let mut row_user_interface = user_interface.new_child(
            UiBuilder::new()
                .max_rect(available_size_rectangle)
                .layout(Layout::left_to_right(Align::Center)),
        );

        row_user_interface.add_space(8.0);

        row_user_interface.add(
            ComboBoxView::new_from_theme(theme, self.app_context.clone(), "Select a process...", None, |user_interface: &mut Ui| {
                user_interface.add(ComboBoxItemView::new(self.app_context.clone(), "test", None, process_dropdown_list_width));
            })
            .width(process_dropdown_list_width),
        );

        response
    }
}
