use crate::{
    app_context::AppContext,
    ui::widgets::{controls::button::Button, docking::dock_root_view_data::DockRootViewData},
};
use eframe::egui::{Align, Align2, CursorIcon, Layout, Response, Sense, Ui, UiBuilder, Widget};
use epaint::{CornerRadius, vec2};
use std::{rc::Rc, sync::Arc};

#[derive(Clone)]
pub struct DockedWindowFooterView {
    app_context: Arc<AppContext>,
    dock_view_data: Arc<DockRootViewData>,
    identifier: Rc<String>,
    height: f32,
}

impl DockedWindowFooterView {
    pub fn new(
        app_context: Arc<AppContext>,
        dock_view_data: Arc<DockRootViewData>,
        identifier: Rc<String>,
    ) -> Self {
        Self {
            app_context,
            dock_view_data,
            identifier,
            height: 24.0,
        }
    }

    pub fn get_height(&self) -> f32 {
        self.height
    }
}

impl Widget for DockedWindowFooterView {
    fn ui(
        self,
        user_interface: &mut Ui,
    ) -> Response {
        let (available_size_rect, response) = user_interface.allocate_exact_size(vec2(user_interface.available_size().x, self.height), Sense::empty());
        let theme = &self.app_context.theme;
        let (sibling_ids, active_tab_id, active_dragged_window_identifier) = match self.app_context.docking_manager.read() {
            Ok(docking_manager_guard) => (
                docking_manager_guard.get_sibling_tab_ids(&self.identifier, true),
                docking_manager_guard.get_active_tab(&self.identifier),
                docking_manager_guard
                    .active_dragged_window_id()
                    .map(str::to_string),
            ),
            Err(error) => {
                log::error!("Failed to acquire docking manager lock: {}", error);
                return response;
            }
        };
        let windows = match self.dock_view_data.windows.read() {
            Ok(windows) => windows,
            Err(error) => {
                log::error!("Failed to acquire windows lock: {}", error);
                return response;
            }
        };

        // Background.
        user_interface
            .painter()
            .rect_filled(available_size_rect, CornerRadius::ZERO, theme.background_primary);

        let builder = UiBuilder::new()
            .max_rect(available_size_rect)
            .layout(Layout::left_to_right(Align::Center));
        let mut child_user_interface = user_interface.new_child(builder);
        let mut selected_tab_id = None;
        let mut drag_start_request = None;

        for sibling_id in sibling_ids {
            let mut button = Button::new_from_theme(theme)
                .background_color(theme.background_control_secondary)
                .border_color(theme.submenu_border)
                .border_width(1.0);

            if sibling_id == active_tab_id {
                button.backgorund_color = theme.background_control_primary;
                button.border_color = theme.background_control_primary_light;
            }

            if active_dragged_window_identifier.as_deref() == Some(sibling_id.as_str()) {
                button.backgorund_color = theme.selected_background;
                button.border_color = theme.selected_border;
            }

            let response = child_user_interface
                .add_sized(
                    vec2(128.0, available_size_rect.height()),
                    button
                        .corner_radius(CornerRadius::ZERO)
                        .sense(Sense::click_and_drag()),
                )
                .on_hover_cursor(CursorIcon::Grab);

            if response.rect.is_positive() {
                for window in windows.iter() {
                    if window.get_identifier() == sibling_id {
                        child_user_interface.painter().text(
                            response.rect.center(),
                            Align2::CENTER_CENTER,
                            window.get_title(),
                            theme.font_library.font_noto_sans.font_header.clone(),
                            theme.foreground,
                        );

                        break;
                    }
                }
            }

            if response.clicked() {
                selected_tab_id = Some(sibling_id.clone());
            }

            if response.drag_started() {
                let pointer_press_origin = child_user_interface
                    .input(|input_state| input_state.pointer.press_origin())
                    .or_else(|| response.interact_pointer_pos());

                if let Some(pointer_press_origin) = pointer_press_origin {
                    drag_start_request = Some((sibling_id.clone(), pointer_press_origin));
                }

                child_user_interface.ctx().request_repaint();
            }

            if response.dragged() {
                child_user_interface.ctx().set_cursor_icon(CursorIcon::Grabbing);
                child_user_interface.ctx().request_repaint();
            }
        }

        if let Some((dragged_tab_identifier, pointer_press_origin)) = drag_start_request {
            if let Ok(mut docking_manager) = self.app_context.docking_manager.write() {
                docking_manager.begin_drag(&dragged_tab_identifier, pointer_press_origin);
            }
        }

        if let Some(selected_tab_id) = selected_tab_id {
            if let Ok(mut docking_manager) = self.app_context.docking_manager.write() {
                docking_manager.select_tab_by_window_id(&selected_tab_id);
            }
        }

        response
    }
}
