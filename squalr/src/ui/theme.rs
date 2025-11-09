use crate::ui::{fonts::font_library::FontLibrary, icon_library::IconLibrary};
use eframe::egui::{Color32, Context};

pub struct Theme {
    pub icon_library: IconLibrary,
    pub font_library: FontLibrary,

    // Core colors
    pub border_blue: Color32,
    pub background_primary: Color32,
    pub background_panel: Color32,
    pub background_control: Color32,
    pub foreground: Color32,
    pub foreground_preview: Color32,
    pub submenu_border: Color32,
    pub selected_background: Color32,
    pub selected_border: Color32,
    pub transparent: Color32,

    // Control backgrounds
    pub background_control_primary_light: Color32,
    pub background_control_primary: Color32,
    pub background_control_primary_dark: Color32,
    pub background_control_secondary: Color32,
    pub background_control_secondary_dark: Color32,
    pub background_control_success: Color32,
    pub background_control_success_dark: Color32,
    pub background_control_danger: Color32,
    pub background_control_danger_dark: Color32,
    pub background_control_warning: Color32,
    pub background_control_warning_dark: Color32,
    pub background_control_info: Color32,
    pub background_control_info_dark: Color32,
    pub background_control_light: Color32,
    pub background_control_border: Color32,

    // Special theme
    pub dec_white: Color32,
    pub dec_white_preview: Color32,
    pub binary_blue: Color32,
    pub binary_blue_preview: Color32,
    pub hexadecimal_green: Color32,
    pub hexadecimal_green_preview: Color32,
    pub error_red: Color32,

    // Focus
    pub focused_background: Color32,
    pub focused_border: Color32,
    pub hover_tint: Color32,
    pub pressed_tint: Color32,

    // Animation settings (not built into egui, but useful to store)
    pub color_duration_ms: u64,
    pub move_duration_ms: u64,
}

impl Theme {
    pub fn new(context: &Context) -> Self {
        Self {
            icon_library: IconLibrary::new(context),
            font_library: FontLibrary::new(context),

            // Core colors.
            border_blue: Color32::from_rgb(0x00, 0x7A, 0xCC),
            background_primary: Color32::from_rgb(0x33, 0x33, 0x33),
            background_panel: Color32::from_rgb(0x27, 0x27, 0x27),
            background_control: Color32::from_rgb(0x44, 0x44, 0x44),
            foreground: Color32::WHITE,
            foreground_preview: Color32::from_rgb(0xAF, 0xAF, 0xAF),
            submenu_border: Color32::from_rgb(0x7F, 0x7F, 0x7F),
            selected_background: Color32::from_rgba_unmultiplied(0x26, 0xA0, 0xDA, 0x3D),
            selected_border: Color32::from_rgb(0x26, 0xA0, 0xDA),
            transparent: Color32::TRANSPARENT,

            // Control backgrounds.
            background_control_primary_light: Color32::from_rgb(0x3D, 0x69, 0x9C),
            background_control_primary: Color32::from_rgb(0x1E, 0x54, 0x92),
            background_control_primary_dark: Color32::from_rgb(0x06, 0x1E, 0x3E),
            background_control_secondary: Color32::from_rgb(0x43, 0x4E, 0x51),
            background_control_secondary_dark: Color32::from_rgb(0x1F, 0x25, 0x26),
            background_control_success: Color32::from_rgb(0x14, 0xA4, 0x4D),
            background_control_success_dark: Color32::from_rgb(0x0E, 0x72, 0x36),
            background_control_danger: Color32::from_rgb(0xDC, 0x4C, 0x64),
            background_control_danger_dark: Color32::from_rgb(0xAE, 0x3C, 0x4F),
            background_control_warning: Color32::from_rgb(0xE4, 0xA1, 0x1B),
            background_control_warning_dark: Color32::from_rgb(0xB0, 0x7D, 0x15),
            background_control_info: Color32::from_rgb(0x32, 0xC4, 0xE6),
            background_control_info_dark: Color32::from_rgb(0x0B, 0x2D, 0x5D),
            background_control_light: Color32::from_rgb(0xFB, 0xFB, 0xFB),
            background_control_border: Color32::from_rgb(0x20, 0x1C, 0x1C),

            // Special theme.
            dec_white: Color32::WHITE,
            dec_white_preview: Color32::from_rgb(0xAF, 0xAF, 0xAF),
            binary_blue: Color32::from_rgb(0x02, 0x91, 0xF0),
            binary_blue_preview: Color32::from_rgb(0x66, 0xA2, 0xC9),
            hexadecimal_green: Color32::from_rgb(0x14, 0xA4, 0x4D),
            hexadecimal_green_preview: Color32::from_rgb(0x75, 0xA0, 0x75),
            error_red: Color32::from_rgb(0xE7, 0x20, 0x20),

            // Focus / states.
            focused_background: Color32::from_rgba_unmultiplied(0x15, 0x50, 0x6C, 0xFF),
            focused_border: Color32::from_rgb(0x26, 0xA0, 0xDA),
            hover_tint: Color32::from_rgba_unmultiplied(0xFF, 0xFF, 0xFF, 0x10),
            pressed_tint: Color32::from_rgba_unmultiplied(0x00, 0x00, 0x00, 0x20),

            // Animations.
            color_duration_ms: 50,
            move_duration_ms: 50,
        }
    }
}
