use crate::ui::theme::Theme;
use eframe::egui::{Area, Frame, Id, Order, Response, RichText, Ui, pos2, vec2};
use epaint::{CornerRadius, Margin, Stroke};

pub struct ThemedTooltip;

impl ThemedTooltip {
    const POINTER_OFFSET_X: f32 = 12.0;
    const POINTER_OFFSET_Y: f32 = 12.0;
    const MAX_WIDTH: f32 = 420.0;

    /// Shows a tooltip using the application theme instead of egui's default tooltip visuals.
    pub fn show_text(
        user_interface: &mut Ui,
        response: &Response,
        tooltip_id: Id,
        theme: &Theme,
        tooltip_text: &str,
    ) {
        if tooltip_text.is_empty() || !response.hovered() {
            return;
        }

        let tooltip_position = response
            .hover_pos()
            .map(|hover_position| hover_position + vec2(Self::POINTER_OFFSET_X, Self::POINTER_OFFSET_Y))
            .unwrap_or_else(|| pos2(response.rect.min.x, response.rect.max.y + 2.0));

        Area::new(tooltip_id)
            .order(Order::Tooltip)
            .fixed_pos(tooltip_position)
            .constrain_to(user_interface.ctx().content_rect())
            .show(user_interface.ctx(), |tooltip_user_interface| {
                Frame::new()
                    .fill(theme.background_primary)
                    .stroke(Stroke::new(1.0, theme.submenu_border))
                    .inner_margin(Margin::same(8))
                    .corner_radius(CornerRadius::ZERO)
                    .show(tooltip_user_interface, |tooltip_user_interface| {
                        tooltip_user_interface.set_max_width(Self::MAX_WIDTH);
                        tooltip_user_interface.label(
                            RichText::new(tooltip_text)
                                .font(theme.font_library.font_noto_sans.font_normal.clone())
                                .color(theme.foreground),
                        );
                    });
            });
    }
}
