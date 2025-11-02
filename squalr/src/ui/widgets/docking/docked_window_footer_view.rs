use crate::{
    app_context::AppContext,
    ui::widgets::{controls::button::Button, docking::dock_root_view_data::DockRootViewData},
};
use eframe::egui::{Align, Align2, Layout, Response, Sense, Ui, UiBuilder, Widget};
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
        let docking_manager = match self.app_context.docking_manager.read() {
            Ok(docking_manager) => docking_manager,
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

        let sibling_ids = docking_manager.get_sibling_tab_ids(&self.identifier, true);
        let active_tab_id = docking_manager.get_active_tab(&self.identifier);
        let builder = UiBuilder::new()
            .max_rect(available_size_rect)
            .layout(Layout::left_to_right(Align::Center));
        let mut child_user_interface = user_interface.new_child(builder);
        let mut selected_tab_id = None;

        for sibling_id in sibling_ids {
            let mut button = Button::new_from_theme(theme)
                .background_color(theme.background_control_secondary)
                .border_color(theme.submenu_border)
                .border_width(1.0);

            if sibling_id == active_tab_id {
                button.backgorund_color = theme.background_control_primary;
                button.border_color = theme.background_control_primary_light;
            }

            let response = child_user_interface.add_sized(vec2(128.0, available_size_rect.height()), button.corner_radius(CornerRadius::ZERO));

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
                selected_tab_id = Some(sibling_id);
            }
        }

        if let Some(selected_tab_id) = selected_tab_id {
            // Free read lock so that we can acquire write lock.
            drop(docking_manager);

            if let Ok(mut docking_manager) = self.app_context.docking_manager.write() {
                docking_manager.select_tab_by_window_id(&selected_tab_id);
            }
        }

        response
    }
}
