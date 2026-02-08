use crate::theme::InstallerTheme;
use eframe::egui::{Color32, CornerRadius, Response, Sense, Ui, Widget};

pub(crate) struct TitleBarButton {
    installer_theme: InstallerTheme,
}

impl TitleBarButton {
    pub(crate) fn new(installer_theme: InstallerTheme) -> Self {
        Self { installer_theme }
    }
}

impl Widget for TitleBarButton {
    fn ui(
        self,
        user_interface: &mut Ui,
    ) -> Response {
        let (button_rectangle, response) = user_interface.allocate_exact_size(user_interface.available_size(), Sense::click());

        let button_fill = if response.is_pointer_button_down_on() {
            self.installer_theme.color_pressed_tint
        } else if response.hovered() {
            self.installer_theme.color_hover_tint
        } else {
            Color32::TRANSPARENT
        };

        user_interface
            .painter()
            .rect_filled(button_rectangle, CornerRadius::same(0), button_fill);

        response
    }
}
