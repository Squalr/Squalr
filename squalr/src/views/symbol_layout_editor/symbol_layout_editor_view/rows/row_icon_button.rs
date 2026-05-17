use crate::{app_context::AppContext, ui::draw::icon_draw::IconDraw, ui::widgets::controls::button::Button as ThemeButton};
use eframe::egui::{Color32, Rect, Response, TextureHandle, Ui, vec2};

pub(super) fn render_row_icon_button_at(
    app_context: &AppContext,
    user_interface: &mut Ui,
    button_rect: Rect,
    icon_handle: &TextureHandle,
    tooltip_text: &str,
    is_disabled: bool,
) -> Response {
    let theme = &app_context.theme;
    let button_response = user_interface.put(
        button_rect,
        ThemeButton::new_from_theme(theme)
            .with_tooltip_text(tooltip_text)
            .background_color(Color32::TRANSPARENT)
            .disabled(is_disabled),
    );

    IconDraw::draw_tinted(
        user_interface,
        button_response.rect,
        icon_handle,
        if is_disabled { theme.foreground_preview } else { theme.foreground },
    );

    button_response
}

pub(super) fn render_row_icon_button(
    app_context: &AppContext,
    user_interface: &mut Ui,
    icon_handle: &TextureHandle,
    tooltip_text: &str,
    is_disabled: bool,
    button_width: f32,
    button_height: f32,
) -> Response {
    let theme = &app_context.theme;
    let button_response = user_interface.add_sized(
        vec2(button_width, button_height),
        ThemeButton::new_from_theme(theme)
            .with_tooltip_text(tooltip_text)
            .background_color(Color32::TRANSPARENT)
            .disabled(is_disabled),
    );

    IconDraw::draw_tinted(
        user_interface,
        button_response.rect,
        icon_handle,
        if is_disabled { theme.foreground_preview } else { theme.foreground },
    );

    button_response
}
