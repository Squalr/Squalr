use crate::{app_context::AppContext, ui::widgets::controls::icon_button::IconButtonView};
use eframe::egui::{Ui, vec2};
use epaint::CornerRadius;
use std::sync::Arc;

pub(in crate::views::symbol_layout_editor::symbol_layout_editor_view) fn render_symbol_layout_add_entry_button(
    app_context: Arc<AppContext>,
    user_interface: &mut Ui,
    tooltip_text: &str,
    button_width: f32,
    button_height: f32,
) -> bool {
    user_interface
        .horizontal(|user_interface| {
            let theme = &app_context.theme;

            user_interface
                .add_sized(
                    vec2(button_width, button_height),
                    IconButtonView::new(theme, &theme.icon_library.icon_handle_common_add, tooltip_text),
                )
                .clicked()
        })
        .inner
}

pub(in crate::views::symbol_layout_editor::symbol_layout_editor_view) fn render_symbol_layout_centered_add_entry_button(
    app_context: Arc<AppContext>,
    user_interface: &mut Ui,
    tooltip_text: &str,
    is_enabled: bool,
    button_width: f32,
    button_height: f32,
    corner_radius: u8,
) -> bool {
    let button_size = vec2(button_width, button_height);

    user_interface
        .horizontal(|user_interface| {
            let theme = &app_context.theme;
            let leading_button_margin = (user_interface.available_width() - button_size.x).max(0.0) * 0.5;
            user_interface.add_space(leading_button_margin);

            let button_response = user_interface.add_sized(
                button_size,
                IconButtonView::new(theme, &theme.icon_library.icon_handle_common_add, tooltip_text)
                    .corner_radius(CornerRadius::same(corner_radius))
                    .background_color(theme.background_control_secondary)
                    .border_color(theme.background_control_secondary_dark)
                    .border_width(1.0)
                    .disabled(!is_enabled),
            );

            is_enabled && button_response.clicked()
        })
        .inner
}
