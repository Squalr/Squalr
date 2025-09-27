use crate::ui::theme::Theme;
use eframe::egui::{Context, Ui};
use epaint::CornerRadius;

#[derive(Default)]
pub struct Footer {
    pub height: f32,
}

impl Footer {
    pub fn draw(
        &self,
        user_interface: &mut Ui,
        context: &Context,
        theme: &Theme,
    ) {
        let full_rect = user_interface.max_rect();

        // Background.
        user_interface
            .painter()
            .rect_filled(full_rect, CornerRadius { nw: 0, ne: 0, sw: 4, se: 4 }, theme.border_blue);
    }
}
