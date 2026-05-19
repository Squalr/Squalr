use crate::ui::theme::Theme;
use eframe::egui::{Area, Frame, Id, Order, Response, RichText, Ui, pos2, vec2};
use epaint::{Color32, CornerRadius, FontId, Margin, Stroke};

#[derive(Clone)]
pub struct ThemedTooltipStyle {
    background: Color32,
    border: Color32,
    foreground: Color32,
    font_id: FontId,
}

impl ThemedTooltipStyle {
    pub fn from_theme(theme: &Theme) -> Self {
        Self {
            background: theme.background_primary,
            border: theme.submenu_border,
            foreground: theme.foreground,
            font_id: theme.font_library.font_noto_sans.font_normal.clone(),
        }
    }
}

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
        Self::show_text_with_style(user_interface, response, tooltip_id, &ThemedTooltipStyle::from_theme(theme), tooltip_text);
    }

    pub fn show_text_with_style(
        user_interface: &mut Ui,
        response: &Response,
        tooltip_id: Id,
        tooltip_style: &ThemedTooltipStyle,
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
                    .fill(tooltip_style.background)
                    .stroke(Stroke::new(1.0, tooltip_style.border))
                    .inner_margin(Margin::same(8))
                    .corner_radius(CornerRadius::ZERO)
                    .show(tooltip_user_interface, |tooltip_user_interface| {
                        tooltip_user_interface.set_max_width(Self::MAX_WIDTH);
                        tooltip_user_interface.label(
                            RichText::new(tooltip_text)
                                .font(tooltip_style.font_id.clone())
                                .color(tooltip_style.foreground),
                        );
                    });
            });
    }
}
