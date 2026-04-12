use eframe::egui::{Color32, Ui};
use epaint::{Pos2, Rect, TextureHandle, Vec2, pos2, vec2};

pub struct IconDraw {}

impl IconDraw {
    pub fn draw_sized_tinted(
        ui: &Ui,
        center_position: Pos2,
        size: Vec2,
        handle: &TextureHandle,
        tint: Color32,
    ) {
        let texture_rect = Rect::from_center_size(center_position, vec2(size[0] as f32, size[1] as f32));

        ui.painter()
            .image(handle.id(), texture_rect, Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)), tint);
    }

    pub fn draw_sized(
        ui: &Ui,
        center_position: Pos2,
        size: Vec2,
        handle: &TextureHandle,
    ) {
        Self::draw_sized_tinted(ui, center_position, size, handle, Color32::WHITE);
    }

    pub fn draw_tinted(
        ui: &Ui,
        bounds_rect: Rect,
        handle: &TextureHandle,
        tint: Color32,
    ) {
        let size = handle.size();
        let texture_rect = Rect::from_center_size(bounds_rect.center(), vec2(size[0] as f32, size[1] as f32));

        ui.painter()
            .image(handle.id(), texture_rect, Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)), tint);
    }

    pub fn draw(
        ui: &Ui,
        bounds_rect: Rect,
        handle: &TextureHandle,
    ) {
        Self::draw_tinted(ui, bounds_rect, handle, Color32::WHITE);
    }
}
