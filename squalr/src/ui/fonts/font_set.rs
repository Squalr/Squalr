use eframe::egui::FontFamily;
use epaint::FontId;

pub struct FontSet {
    pub font_s: FontId,
    pub font_p: FontId,
    pub font_h1: FontId,
    pub font_h2: FontId,
    pub font_h3: FontId,
    pub font_h4: FontId,
    pub font_h5: FontId,
    pub font_window_title: FontId,
}

impl FontSet {
    pub fn new(
        font_family: FontFamily,
        font_size_s: f32,
        font_size_p: f32,
        font_size_h1: f32,
        font_size_h2: f32,
        font_size_h3: f32,
        font_size_h4: f32,
        font_size_h5: f32,
        font_size_window_title: f32,
    ) -> Self {
        Self {
            font_s: FontId {
                size: font_size_s,
                family: font_family.clone(),
            },
            font_p: FontId {
                size: font_size_p,
                family: font_family.clone(),
            },
            font_h1: FontId {
                size: font_size_h1,
                family: font_family.clone(),
            },
            font_h2: FontId {
                size: font_size_h2,
                family: font_family.clone(),
            },
            font_h3: FontId {
                size: font_size_h3,
                family: font_family.clone(),
            },
            font_h4: FontId {
                size: font_size_h4,
                family: font_family.clone(),
            },
            font_h5: FontId {
                size: font_size_h5,
                family: font_family.clone(),
            },
            font_window_title: FontId {
                size: font_size_window_title,
                family: font_family,
            },
        }
    }
}
