use eframe::egui::{Color32, Ui};
use epaint::{Rect, TextureHandle, pos2, vec2};

pub struct IconDraw {}

impl IconDraw {
    pub fn draw(
        ui: &Ui,
        bounds_rect: Rect,
        handle: &TextureHandle,
    ) {
        let size = handle.size();
        let texture_rect = Rect::from_center_size(bounds_rect.center(), vec2(size[0] as f32, size[1] as f32));

        ui.painter()
            .image(handle.id(), texture_rect, Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)), Color32::WHITE);
    }
}
