use crate::models::docking::docking_manager::DockingManager;
use crate::ui::draw::icon_draw::IconDraw;
use crate::ui::theme::Theme;
use crate::ui::widgets::controls::button::Button;
use eframe::egui::{Align, Context, Id, Layout, Rect, Response, RichText, Sense, Ui, UiBuilder, pos2};
use epaint::{CornerRadius, vec2};
use std::rc::Rc;
use std::sync::{Arc, RwLock};

#[derive(Clone)]
pub struct DockedWindowTitleBarView {
    context: Context,
    theme: Rc<Theme>,
    docking_manager: Arc<RwLock<DockingManager>>,
    height: f32,
    title: String,
    identifier: String,
}

impl DockedWindowTitleBarView {
    pub fn new(
        context: Context,
        theme: Rc<Theme>,
        docking_manager: Arc<RwLock<DockingManager>>,
        title: String,
        identifier: String,
    ) -> Self {
        Self {
            context,
            theme,
            docking_manager,
            height: 24.0,
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
        let (available_size_rect, response) = user_interface.allocate_exact_size(vec2(user_interface.available_width(), self.height), Sense::click_and_drag());

        // Background highlight if this is the actively dragged window.
        let background = if let Ok(docking_manager) = self.docking_manager.try_read() {
            /*
            if docking_manager.active_dragged_window_id() == Some(&self.identifier) {
                self.theme.selected_border
            } else {
                self.theme.background_primary
            }*/
            self.theme.background_primary
        } else {
            self.theme.background_primary
        };
        user_interface
            .painter()
            .rect_filled(available_size_rect, CornerRadius::ZERO, background);

        // Child UI for layouting contents.
        let builder = UiBuilder::new()
            .max_rect(available_size_rect)
            .layout(Layout::left_to_right(Align::Center));
        let mut child_user_interface = user_interface.new_child(builder);

        child_user_interface.set_clip_rect(available_size_rect);

        // Title text.
        child_user_interface.add_space(8.0);

        child_user_interface.label(
            RichText::new(self.title.clone())
                .color(self.theme.foreground)
                .font(self.theme.font_library.font_noto_sans.font_window_title.clone()),
        );

        child_user_interface.add_space(child_user_interface.available_width());

        // Buttons aligned right-to-left.
        child_user_interface.with_layout(Layout::right_to_left(Align::Center), |ui| {
            let button_size = vec2(36.0, self.height);

            // Close button
            let close = ui.add_sized(button_size, Button::new_from_theme(&self.theme));
            IconDraw::draw(ui, close.rect, &self.theme.icon_library.icon_handle_close);

            if close.clicked() {
                if let Ok(mut docking_manager) = self.docking_manager.write() {
                    // docking_manager.hide(&self.identifier);
                }
            }
        });

        // Drag area = everything except the close button
        let drag_rect = Rect::from_min_max(available_size_rect.min, pos2(available_size_rect.max.x - 36.0, available_size_rect.max.y));
        let drag = user_interface.interact(drag_rect, Id::new(format!("dock_titlebar_{}", self.identifier)), Sense::click_and_drag());

        /*
        if drag.drag_started() {
            if let Ok(mut docking_manager) = self.docking_manager.write() {
                docking_manager.begin_drag(&self.identifier);
            }
        }

        if drag.drag_released() {
            if let Ok(mut docking_manager) = self.docking_manager.write() {
                docking_manager.end_drag(&self.identifier);
            }
        }*/

        response
    }
}
