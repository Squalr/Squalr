use crate::{app_context::AppContext, ui::widgets::controls::icon_button::IconButtonView};
use eframe::egui::{Rect, Response, TextureHandle, Ui, vec2};

pub(super) fn render_row_icon_button_at(
    app_context: &AppContext,
    user_interface: &mut Ui,
    button_rect: Rect,
    icon_handle: &TextureHandle,
    tooltip_text: &str,
    is_disabled: bool,
) -> Response {
    let theme = &app_context.theme;
    user_interface.put(button_rect, IconButtonView::new(theme, icon_handle, tooltip_text).disabled(is_disabled))
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
    user_interface.add_sized(
        vec2(button_width, button_height),
        IconButtonView::new(theme, icon_handle, tooltip_text).disabled(is_disabled),
    )
}
