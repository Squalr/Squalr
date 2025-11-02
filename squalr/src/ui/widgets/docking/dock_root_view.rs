use crate::{app_context::AppContext, ui::widgets::docking::dock_root_view_data::DockRootViewData};
use eframe::egui::{Response, Sense, Ui, UiBuilder, Widget};
use epaint::{CornerRadius, Rect, pos2, vec2};
use std::sync::Arc;

#[derive(Clone)]
pub struct DockRootView {
    app_context: Arc<AppContext>,
    dock_view_data: Arc<DockRootViewData>,
}

impl DockRootView {
    pub fn new(
        app_context: Arc<AppContext>,
        dock_view_data: Arc<DockRootViewData>,
    ) -> Self {
        Self { app_context, dock_view_data }
    }
}

impl Widget for DockRootView {
    fn ui(
        self,
        user_interface: &mut Ui,
    ) -> Response {
        let (available_size_rect, response) = user_interface.allocate_exact_size(user_interface.available_size(), Sense::empty());
        let theme = &self.app_context.theme;
        let docking_manager = &self.app_context.docking_manager;
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
            .rect_filled(available_size_rect, CornerRadius::ZERO, theme.background_panel);

        if let Ok(mut docking_manager) = docking_manager.try_write() {
            docking_manager.prepare_for_presentation();
            docking_manager
                .get_main_window_layout_mut()
                .set_available_size(available_size_rect.width(), available_size_rect.height());
        }

        for window in windows.iter() {
            let window_identifier = window.get_identifier();
            let active_tab_id = match docking_manager.read() {
                Ok(docking_manager) => docking_manager.get_active_tab(&window_identifier),
                Err(_) => String::new(),
            };

            // We only need to render the active tab in windows that share the same space.
            if active_tab_id != window_identifier {
                continue;
            }

            let window_rect = {
                if let Ok(docking_manager) = docking_manager.read() {
                    docking_manager
                        .find_window_rect(window_identifier)
                        .map(|(x, y, w, h)| {
                            let offset = available_size_rect.min;
                            Rect::from_min_size(pos2(offset.x + x as f32, offset.y + y as f32), vec2(w as f32, h as f32))
                        })
                } else {
                    None
                }
            };

            if let Some(window_rect) = window_rect {
                let builder = UiBuilder::new().max_rect(window_rect);
                let mut child_user_interface = user_interface.new_child(builder);

                window.ui(&mut child_user_interface);
            }
        }

        response
    }
}
