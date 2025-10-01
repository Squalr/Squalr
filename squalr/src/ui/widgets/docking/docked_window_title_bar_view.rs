use crate::app_context::AppContext;
use crate::ui::draw::icon_draw::IconDraw;
use crate::ui::widgets::controls::button::Button;
use eframe::egui::{Align, Id, Layout, Rect, Response, RichText, Sense, Ui, UiBuilder, pos2};
use epaint::{Color32, CornerRadius, vec2};
use std::rc::Rc;

#[derive(Clone)]
pub struct DockedWindowTitleBarView {
    app_context: Rc<AppContext>,
    height: f32,
    title: String,
    identifier: String,
}

impl DockedWindowTitleBarView {
    pub fn new(
        app_context: Rc<AppContext>,
        title: String,
        identifier: String,
    ) -> Self {
        Self {
            app_context,
            height: 28.0,
            title,
            identifier,
        }
    }
}

impl eframe::egui::Widget for DockedWindowTitleBarView {
    fn ui(
        self,
        user_interface: &mut Ui,
    ) -> Response {
        let (available_size_rectangle, response) =
            user_interface.allocate_exact_size(vec2(user_interface.available_width(), self.height), Sense::click_and_drag());
        let theme = &self.app_context.theme;
        let docking_manager = &self.app_context.docking_manager;

        // Background highlight if this is the actively dragged window.
        let background = if let Ok(docking_manager) = docking_manager.try_read() {
            /*
            if docking_manager.active_dragged_window_id() == Some(&self.identifier) {
                theme.selected_border
            } else {
                theme.background_primary
            }*/
            theme.background_primary
        } else {
            theme.background_primary
        };
        user_interface
            .painter()
            .rect_filled(available_size_rectangle, CornerRadius::ZERO, background);

        // Child UI for layouting contents.
        let builder = UiBuilder::new()
            .max_rect(available_size_rectangle)
            .layout(Layout::left_to_right(Align::Center));
        let mut child_user_interface = user_interface.new_child(builder);

        child_user_interface.set_clip_rect(available_size_rectangle);

        // Title text.
        child_user_interface.add_space(8.0);

        child_user_interface.label(
            RichText::new(self.title.clone())
                .color(theme.foreground)
                .font(theme.font_library.font_noto_sans.font_window_title.clone()),
        );

        child_user_interface.add_space(child_user_interface.available_width());

        // Buttons aligned right-to-left.
        child_user_interface.with_layout(Layout::right_to_left(Align::Center), |ui| {
            let button_size = vec2(36.0, self.height);

            // Close button
            let close = ui.add_sized(button_size, Button::new_from_theme(&theme).background_color(Color32::TRANSPARENT));
            IconDraw::draw(ui, close.rect, &theme.icon_library.icon_handle_close);

            if close.clicked() {
                if let Ok(mut docking_manager) = docking_manager.try_write() {
                    if let Some(docked_node) = docking_manager.get_node_by_id_mut(&self.identifier) {
                        docked_node.set_visible(false);
                    }
                }
            }
        });

        // Drag area = everything except the close button
        let drag_rect = Rect::from_min_max(
            available_size_rectangle.min,
            pos2(available_size_rectangle.max.x - 36.0, available_size_rectangle.max.y),
        );
        let drag = user_interface.interact(drag_rect, Id::new(format!("dock_titlebar_{}", self.identifier)), Sense::click_and_drag());

        /*
        if drag.drag_started() {
            if let Ok(mut docking_manager) = docking_manager.write() {
                docking_manager.begin_drag(&self.identifier);
            }
        }

        if drag.drag_released() {
            if let Ok(mut docking_manager) = docking_manager.write() {
                docking_manager.end_drag(&self.identifier);
            }
        }*/

        response
    }
}
