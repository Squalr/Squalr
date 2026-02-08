use crate::theme::InstallerTheme;
use crate::ui_assets::{APP_NAME, InstallerIconLibrary, draw_icon};
use crate::views::main_window::title_bar_button::TitleBarButton;
use eframe::egui::viewport::ViewportCommand;
use eframe::egui::{Align, CornerRadius, Id, Layout, Rect, Response, RichText, Sense, Ui, Widget, pos2};

#[derive(Clone)]
pub(crate) struct InstallerTitleBarView {
    installer_theme: InstallerTheme,
    installer_icon_library: Option<InstallerIconLibrary>,
}

impl InstallerTitleBarView {
    pub(crate) fn new(
        installer_theme: InstallerTheme,
        installer_icon_library: Option<InstallerIconLibrary>,
    ) -> Self {
        Self {
            installer_theme,
            installer_icon_library,
        }
    }
}

impl Widget for InstallerTitleBarView {
    fn ui(
        self,
        user_interface: &mut Ui,
    ) -> Response {
        let (title_bar_rectangle, response) = user_interface.allocate_exact_size(
            eframe::egui::vec2(user_interface.available_width(), self.installer_theme.title_bar_height),
            Sense::empty(),
        );

        user_interface.painter().rect_filled(
            title_bar_rectangle,
            CornerRadius {
                nw: self.installer_theme.corner_radius_panel,
                ne: self.installer_theme.corner_radius_panel,
                sw: 0,
                se: 0,
            },
            self.installer_theme.color_background_primary,
        );

        let mut title_bar_user_interface = user_interface.new_child(
            eframe::egui::UiBuilder::new()
                .max_rect(title_bar_rectangle)
                .layout(Layout::left_to_right(Align::Center)),
        );
        title_bar_user_interface.set_clip_rect(title_bar_rectangle);

        let context = title_bar_user_interface.ctx().clone();
        let mut window_controls_rectangle: Option<Rect> = None;

        title_bar_user_interface.add_space(8.0);

        if let Some(installer_icon_library) = &self.installer_icon_library {
            let [icon_width, icon_height] = installer_icon_library.app.size();
            let (_identifier, app_icon_rectangle) = title_bar_user_interface.allocate_space(eframe::egui::vec2(icon_width as f32, icon_height as f32));
            draw_icon(&title_bar_user_interface, app_icon_rectangle, &installer_icon_library.app);
            title_bar_user_interface.add_space(4.0);
        }

        title_bar_user_interface.label(
            RichText::new(APP_NAME)
                .font(self.installer_theme.fonts.font_window_title.clone())
                .color(self.installer_theme.color_foreground),
        );

        title_bar_user_interface.add_space(title_bar_user_interface.available_width());

        title_bar_user_interface.with_layout(Layout::right_to_left(Align::Center), |window_controls_user_interface| {
            let button_size = eframe::egui::vec2(36.0, self.installer_theme.title_bar_height);

            let close_button_response = window_controls_user_interface.add_sized(button_size, TitleBarButton::new(self.installer_theme.clone()));
            if let Some(installer_icon_library) = &self.installer_icon_library {
                draw_icon(window_controls_user_interface, close_button_response.rect, &installer_icon_library.close);
            }
            if close_button_response.clicked() {
                context.send_viewport_cmd(ViewportCommand::Close);
            }

            let maximize_button_response = window_controls_user_interface.add_sized(button_size, TitleBarButton::new(self.installer_theme.clone()));
            if let Some(installer_icon_library) = &self.installer_icon_library {
                draw_icon(window_controls_user_interface, maximize_button_response.rect, &installer_icon_library.maximize);
            }
            if maximize_button_response.clicked() {
                let viewport_maximized = context.input(|input_state| input_state.viewport().maximized.unwrap_or(false));
                context.send_viewport_cmd(ViewportCommand::Maximized(!viewport_maximized));
            }

            let minimize_button_response = window_controls_user_interface.add_sized(button_size, TitleBarButton::new(self.installer_theme.clone()));
            if let Some(installer_icon_library) = &self.installer_icon_library {
                draw_icon(window_controls_user_interface, minimize_button_response.rect, &installer_icon_library.minimize);
            }
            if minimize_button_response.clicked() {
                context.send_viewport_cmd(ViewportCommand::Minimized(true));
            }

            window_controls_rectangle = Some(
                close_button_response
                    .rect
                    .union(maximize_button_response.rect)
                    .union(minimize_button_response.rect),
            );
        });

        let drag_region_right_edge = window_controls_rectangle
            .map(|window_controls_region| window_controls_region.min.x)
            .unwrap_or(title_bar_rectangle.max.x);
        let title_bar_drag_region = Rect::from_min_max(title_bar_rectangle.min, pos2(drag_region_right_edge, title_bar_rectangle.max.y));
        let drag_region_response = user_interface.interact(title_bar_drag_region, Id::new("installer_title_bar_drag"), Sense::click_and_drag());

        if drag_region_response.drag_started() {
            context.send_viewport_cmd(ViewportCommand::StartDrag);
        }

        if drag_region_response.double_clicked() {
            let viewport_maximized = context.input(|input_state| input_state.viewport().maximized.unwrap_or(false));
            context.send_viewport_cmd(ViewportCommand::Maximized(!viewport_maximized));
        }

        response
    }
}
