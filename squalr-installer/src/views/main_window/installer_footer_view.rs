use crate::installer_runtime::launch_app;
use crate::theme::InstallerTheme;
use eframe::egui::{Align, CornerRadius, Layout, Response, RichText, Sense, Ui, Widget};

#[derive(Clone)]
pub(crate) struct InstallerFooterView {
    installer_theme: InstallerTheme,
    install_complete: bool,
}

impl InstallerFooterView {
    pub(crate) fn new(
        installer_theme: InstallerTheme,
        install_complete: bool,
    ) -> Self {
        Self {
            installer_theme,
            install_complete,
        }
    }

    pub(crate) fn get_height(&self) -> f32 {
        self.installer_theme.footer_height
    }
}

impl Widget for InstallerFooterView {
    fn ui(
        self,
        user_interface: &mut Ui,
    ) -> Response {
        let (footer_rectangle, response) = user_interface.allocate_exact_size(
            eframe::egui::vec2(user_interface.available_width(), self.installer_theme.footer_height),
            Sense::empty(),
        );

        user_interface.painter().rect_filled(
            footer_rectangle,
            CornerRadius {
                nw: 0,
                ne: 0,
                sw: self.installer_theme.corner_radius_panel,
                se: self.installer_theme.corner_radius_panel,
            },
            self.installer_theme.color_border_blue,
        );

        let mut footer_user_interface = user_interface.new_child(
            eframe::egui::UiBuilder::new()
                .max_rect(footer_rectangle)
                .layout(Layout::left_to_right(Align::Center)),
        );

        footer_user_interface.add_space(8.0);
        footer_user_interface.label(
            RichText::new("Install location: default user application directory.")
                .font(self.installer_theme.fonts.font_normal.clone())
                .color(self.installer_theme.color_foreground),
        );

        footer_user_interface.add_space(footer_user_interface.available_width());

        if self.install_complete && footer_user_interface.button("Launch Squalr").clicked() {
            launch_app();
        }

        response
    }
}
