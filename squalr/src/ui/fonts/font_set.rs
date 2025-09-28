use eframe::egui::FontFamily;
use epaint::FontId;

pub struct FontSet {
    pub font_small: FontId,
    pub font_normal: FontId,
    pub font_header: FontId,
    pub font_window_title: FontId,
}

impl FontSet {
    pub fn new(
        font_family: FontFamily,
        font_size_small: f32,
        font_size_normal: f32,
        font_size_header: f32,
        font_size_window_title: f32,
    ) -> Self {
        Self {
            font_small: FontId {
                size: font_size_small,
                family: font_family.clone(),
            },
            font_normal: FontId {
                size: font_size_normal,
                family: font_family.clone(),
            },
            font_header: FontId {
                size: font_size_header,
                family: font_family.clone(),
            },
            font_window_title: FontId {
                size: font_size_window_title,
                family: font_family,
            },
        }
    }
}
