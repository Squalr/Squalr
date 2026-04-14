use crate::app_context::AppContext;
use crate::ui::widgets::controls::button::Button as ThemeButton;
use eframe::egui::{Align, Button, Frame, Hyperlink, Layout, Margin, RichText, Stroke, Ui, vec2};
use epaint::{Color32, CornerRadius, Rect, pos2};
use std::sync::Arc;

pub struct AboutTakeOverView {
    app_context: Arc<AppContext>,
}

pub struct AboutTakeOverViewResponse {
    pub should_close: bool,
}

impl AboutTakeOverView {
    const PRODUCT_NAME: &'static str = "Squalr";
    const WEBSITE_URL: &'static str = "https://www.squalr.com";
    const CARD_MAX_WIDTH: f32 = 560.0;
    const CARD_MIN_WIDTH: f32 = 360.0;
    const CARD_INNER_MARGIN: f32 = 24.0;
    const LOGO_SIZE: f32 = 84.0;

    pub fn new(app_context: Arc<AppContext>) -> Self {
        Self { app_context }
    }

    fn build_rows() -> [(&'static str, String); 4] {
        [
            ("Version", env!("CARGO_PKG_VERSION").to_string()),
            ("Build Profile", option_env!("PROFILE").unwrap_or("unknown").to_string()),
            ("Target", format!("{} / {}", std::env::consts::OS, std::env::consts::ARCH)),
            ("Package", env!("CARGO_PKG_NAME").to_string()),
        ]
    }

    pub fn show(
        self,
        user_interface: &mut Ui,
    ) -> AboutTakeOverViewResponse {
        let theme = &self.app_context.theme;
        let available_rectangle = user_interface.available_rect_before_wrap();
        let mut should_close = false;

        user_interface
            .painter()
            .rect_filled(available_rectangle, CornerRadius::ZERO, theme.background_panel);

        if user_interface.input(|input_state| input_state.key_pressed(eframe::egui::Key::Escape)) {
            should_close = true;
        }

        let card_width = (available_rectangle.width() - 48.0)
            .clamp(Self::CARD_MIN_WIDTH, Self::CARD_MAX_WIDTH)
            .min(available_rectangle.width().max(0.0));
        let horizontal_padding = ((available_rectangle.width() - card_width) * 0.5).max(0.0);

        user_interface.add_space(((available_rectangle.height() - 420.0) * 0.5).max(24.0));
        user_interface.horizontal(|user_interface| {
            user_interface.add_space(horizontal_padding);
            user_interface.allocate_ui(vec2(card_width, 0.0), |user_interface| {
                Frame::new()
                    .fill(theme.background_primary)
                    .stroke(Stroke::new(1.0, theme.background_control_secondary))
                    .corner_radius(CornerRadius::same(10))
                    .inner_margin(Margin::same(Self::CARD_INNER_MARGIN as i8))
                    .show(user_interface, |user_interface| {
                        user_interface.with_layout(Layout::top_down(Align::Center), |user_interface| {
                            let logo_rectangle = user_interface
                                .allocate_exact_size(vec2(Self::LOGO_SIZE, Self::LOGO_SIZE), eframe::egui::Sense::hover())
                                .0;
                            user_interface.painter().image(
                                theme.icon_library.icon_handle_logo.id(),
                                logo_rectangle,
                                Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                                Color32::WHITE,
                            );

                            user_interface.add_space(14.0);
                            user_interface.label(
                                RichText::new(Self::PRODUCT_NAME)
                                    .font(theme.font_library.font_noto_sans.font_window_title.clone())
                                    .color(theme.foreground),
                            );
                            user_interface.add_space(4.0);
                            user_interface.label(
                                RichText::new("Dynamic analysis and reverse-engineering toolkit.")
                                    .font(theme.font_library.font_noto_sans.font_normal.clone())
                                    .color(theme.foreground_preview),
                            );
                            user_interface.add_space(18.0);

                            for (label, value) in Self::build_rows() {
                                user_interface.allocate_ui(vec2(user_interface.available_width(), 24.0), |user_interface| {
                                    user_interface.columns(2, |columns| {
                                        columns[0].with_layout(Layout::left_to_right(Align::Center), |user_interface| {
                                            user_interface.label(
                                                RichText::new(label)
                                                    .font(theme.font_library.font_noto_sans.font_normal.clone())
                                                    .color(theme.foreground_preview),
                                            );
                                        });
                                        columns[1].with_layout(Layout::right_to_left(Align::Center), |user_interface| {
                                            user_interface.label(
                                                RichText::new(value)
                                                    .font(theme.font_library.font_ubuntu_mono_bold.font_normal.clone())
                                                    .color(theme.foreground),
                                            );
                                        });
                                    });
                                });
                            }

                            user_interface.add_space(18.0);
                            user_interface.add(Hyperlink::from_label_and_url(
                                RichText::new(Self::WEBSITE_URL)
                                    .font(theme.font_library.font_noto_sans.font_normal.clone())
                                    .color(theme.selected_border),
                                Self::WEBSITE_URL,
                            ));

                            user_interface.add_space(20.0);
                            user_interface.allocate_ui(vec2(user_interface.available_width(), 32.0), |user_interface| {
                                let button_width = 128.0;
                                let button_height = 28.0;
                                let button_spacing = 12.0;
                                let row_width = button_width * 2.0 + button_spacing;
                                let leading_space = ((user_interface.available_width() - row_width) * 0.5).max(0.0);

                                user_interface.horizontal(|user_interface| {
                                    user_interface.add_space(leading_space);
                                    user_interface.spacing_mut().item_spacing.x = button_spacing;

                                    let open_website_response = user_interface.add_sized(
                                        [button_width, button_height],
                                        Button::new(
                                            RichText::new("Open Website")
                                                .font(theme.font_library.font_noto_sans.font_normal.clone())
                                                .color(theme.foreground),
                                        )
                                        .fill(theme.background_control_primary)
                                        .stroke(Stroke::new(1.0, theme.background_control_primary_dark)),
                                    );

                                    if open_website_response.clicked() {
                                        user_interface
                                            .ctx()
                                            .open_url(eframe::egui::OpenUrl::same_tab(Self::WEBSITE_URL));
                                    }

                                    let close_response = user_interface.add_sized(
                                        [button_width, button_height],
                                        ThemeButton::new_from_theme(theme)
                                            .background_color(theme.background_control_secondary)
                                            .border_color(theme.background_control_secondary_dark),
                                    );

                                    if close_response.clicked() {
                                        should_close = true;
                                    }

                                    user_interface.painter().text(
                                        close_response.rect.center(),
                                        eframe::egui::Align2::CENTER_CENTER,
                                        "Close",
                                        theme.font_library.font_noto_sans.font_normal.clone(),
                                        theme.foreground,
                                    );
                                });
                            });
                        });
                    });
            });
        });

        AboutTakeOverViewResponse { should_close }
    }
}
