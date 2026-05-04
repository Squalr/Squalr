use crate::app_context::AppContext;
use crate::ui::draw::icon_draw::IconDraw;
use crate::ui::widgets::controls::button::Button;
use crate::ui::widgets::docking::dock_root_view_data::DockRootViewData;
use eframe::egui::{Align, CursorIcon, Id, Layout, Rect, Response, RichText, Sense, Ui, UiBuilder, Widget, pos2};
use epaint::{Color32, CornerRadius, vec2};
use std::{rc::Rc, sync::Arc};

#[derive(Clone)]
pub struct DockedWindowTitleBarView {
    app_context: Arc<AppContext>,
    dock_view_data: Arc<DockRootViewData>,
    height: f32,
    title: Rc<String>,
    identifier: Rc<String>,
}

impl DockedWindowTitleBarView {
    pub fn new(
        app_context: Arc<AppContext>,
        dock_view_data: Arc<DockRootViewData>,
        title: Rc<String>,
        identifier: Rc<String>,
    ) -> Self {
        Self {
            app_context,
            dock_view_data,
            height: 28.0,
            title,
            identifier,
        }
    }
}

impl Widget for DockedWindowTitleBarView {
    fn ui(
        self,
        user_interface: &mut Ui,
    ) -> Response {
        let (allocated_size_rectangle, response) =
            user_interface.allocate_exact_size(vec2(user_interface.available_width(), self.height), Sense::click_and_drag());
        let theme = &self.app_context.theme;
        let docking_manager = &self.app_context.docking_manager;
        let is_window_maximized = self
            .dock_view_data
            .is_window_maximized(self.identifier.as_ref());
        let is_window_focused = self
            .app_context
            .window_focus_manager
            .is_window_focused(&self.identifier);

        // Background highlight if this is the actively dragged window.
        let background = if let Ok(docking_manager) = docking_manager.read() {
            if docking_manager.active_dragged_window_id() == Some(self.identifier.as_ref()) {
                theme.selected_background
            } else if is_window_maximized || is_window_focused {
                theme.background_control_primary
            } else {
                theme.background_primary
            }
        } else if is_window_maximized || is_window_focused {
            theme.background_control_primary
        } else {
            theme.background_primary
        };
        user_interface
            .painter()
            .rect_filled(allocated_size_rectangle, CornerRadius::ZERO, background);

        // Child UI for layouting contents.
        let builder = UiBuilder::new()
            .max_rect(allocated_size_rectangle)
            .layout(Layout::left_to_right(Align::Center));
        let mut child_user_interface = user_interface.new_child(builder);

        child_user_interface.set_clip_rect(allocated_size_rectangle);

        // Title text.
        child_user_interface.add_space(8.0);

        child_user_interface.label(
            RichText::new(self.title.as_ref())
                .color(theme.foreground)
                .font(theme.font_library.font_noto_sans.font_window_title.clone()),
        );

        if is_window_maximized {
            child_user_interface.add_space(8.0);
            child_user_interface.label(
                RichText::new("Fullscreen")
                    .color(theme.selected_border)
                    .font(theme.font_library.font_noto_sans.font_normal.clone()),
            );
        }

        child_user_interface.add_space(child_user_interface.available_width());

        // Buttons aligned right-to-left.
        child_user_interface.with_layout(Layout::right_to_left(Align::Center), |ui| {
            let button_size = vec2(36.0, self.height);

            // Close button.
            let close = ui.add_sized(button_size, Button::new_from_theme(&theme).background_color(Color32::TRANSPARENT));
            IconDraw::draw(ui, close.rect, &theme.icon_library.icon_handle_close);

            if close.clicked() {
                if self.dock_view_data.get_maximized_window_identifier().as_deref() == Some(self.identifier.as_ref()) {
                    self.dock_view_data.set_maximized_window_identifier(None);
                }

                if let Ok(mut docking_manager) = docking_manager.try_write() {
                    docking_manager.set_window_visibility(&self.identifier, false);
                }
            }

            let maximize = ui.add_sized(button_size, Button::new_from_theme(&theme).background_color(Color32::TRANSPARENT));
            IconDraw::draw(ui, maximize.rect, &theme.icon_library.icon_handle_maximize);

            if maximize.clicked() {
                self.dock_view_data
                    .toggle_maximized_window_identifier(&self.identifier);
            }
        });

        // Drag area = everything except the title bar buttons.
        let drag_rect = Rect::from_min_max(
            allocated_size_rectangle.min,
            pos2(allocated_size_rectangle.max.x - 72.0, allocated_size_rectangle.max.y),
        );
        let drag = user_interface
            .interact(drag_rect, Id::new(format!("dock_titlebar_{}", self.identifier)), Sense::click_and_drag())
            .on_hover_cursor(CursorIcon::Grab);

        if drag.drag_started() {
            let pointer_press_origin = user_interface
                .input(|input_state| input_state.pointer.press_origin())
                .or_else(|| drag.interact_pointer_pos());

            if let Some(pointer_press_origin) = pointer_press_origin {
                if let Ok(mut docking_manager) = docking_manager.write() {
                    docking_manager.begin_drag(&self.identifier, pointer_press_origin);
                }
            }

            user_interface.ctx().request_repaint();
        }

        if drag.double_clicked() {
            self.dock_view_data
                .toggle_maximized_window_identifier(&self.identifier);
            user_interface.ctx().request_repaint();
        }

        if drag.dragged() {
            if let Ok(mut docking_manager) = docking_manager.try_write() {
                docking_manager.update_drag_pointer_position(drag.interact_pointer_pos());
            }

            user_interface.ctx().set_cursor_icon(CursorIcon::Grabbing);
            user_interface.ctx().request_repaint();
        }

        response
    }
}
