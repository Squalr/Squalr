use crate::{
    models::docking::docking_manager::DockingManager,
    ui::{theme::Theme, widgets::controls::button::Button},
};
use eframe::egui::{Align, Align2, Context, Layout, Response, Sense, Ui, UiBuilder, Widget};
use epaint::{CornerRadius, vec2};
use std::{
    rc::Rc,
    sync::{Arc, RwLock},
};

#[derive(Clone)]
pub struct DockedWindowFooterView {
    _context: Context,
    theme: Rc<Theme>,
    docking_manager: Arc<RwLock<DockingManager>>,
    identifier: String,
    height: f32,
}

impl DockedWindowFooterView {
    pub fn new(
        context: Context,
        theme: Rc<Theme>,
        docking_manager: Arc<RwLock<DockingManager>>,
        identifier: String,
    ) -> Self {
        let height = 28.0;

        Self {
            _context: context,
            theme,
            docking_manager,
            identifier,
            height,
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

        // Background.
        user_interface
            .painter()
            .rect_filled(available_size_rect, CornerRadius::ZERO, self.theme.background_primary);

        let (sibling_ids, active_tab_id) = {
            if let Ok(docking_manager) = self.docking_manager.try_read() {
                (
                    docking_manager.get_sibling_tab_ids(&self.identifier, true),
                    docking_manager.get_active_tab(&self.identifier),
                )
            } else {
                (vec![], String::new())
            }
        };

        let builder = UiBuilder::new()
            .max_rect(available_size_rect)
            .layout(Layout::left_to_right(Align::Center));
        let mut child_user_interface = user_interface.new_child(builder);

        for sibling_id in sibling_ids {
            let mut button = Button::new_from_theme(&self.theme)
                .background_color(self.theme.background_control_secondary)
                .border_color(self.theme.submenu_border)
                .border_width(1.0);

            if sibling_id == active_tab_id {
                button.backgorund_color = self.theme.background_control_primary;
            }

            let response = child_user_interface.add_sized(vec2(128.0, available_size_rect.height()), button.corner_radius(CornerRadius::ZERO));

            if response.rect.is_positive() {
                child_user_interface.painter().text(
                    response.rect.center(),
                    Align2::CENTER_CENTER,
                    sibling_id.clone(),
                    self.theme.font_library.font_noto_sans.font_header.clone(),
                    self.theme.foreground,
                );
            }

            if response.clicked() {
                if let Ok(mut docking_manager) = self.docking_manager.try_write() {
                    docking_manager.select_tab_by_window_id(&sibling_id);
                }
            }
        }

        response
    }
}
