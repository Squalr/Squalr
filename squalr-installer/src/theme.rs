use eframe::egui;
use eframe::egui::{Color32, CornerRadius, FontData, FontDefinitions, FontFamily, FontId, Margin, Stroke};

pub(crate) static FONT_NOTO_SANS: &[u8] = include_bytes!("../../squalr/fonts/NotoSans.ttf");
pub(crate) static FONT_UBUNTU_MONO_BOLD: &[u8] = include_bytes!("../../squalr/fonts/UbuntuMonoBold.ttf");

#[derive(Clone)]
pub(crate) struct InstallerFontLibrary {
    pub(crate) font_normal: FontId,
    pub(crate) font_window_title: FontId,
    pub(crate) font_ubuntu_mono_normal: FontId,
}

impl InstallerFontLibrary {
    fn new() -> Self {
        Self {
            font_normal: FontId::new(13.0, FontFamily::Name("noto_sans".into())),
            font_window_title: FontId::new(14.0, FontFamily::Name("noto_sans".into())),
            font_ubuntu_mono_normal: FontId::new(15.0, FontFamily::Name("ubuntu_mono_bold".into())),
        }
    }
}

#[derive(Clone)]
pub(crate) struct InstallerTheme {
    pub(crate) fonts: InstallerFontLibrary,
    pub(crate) color_background_primary: Color32,
    pub(crate) color_background_panel: Color32,
    pub(crate) color_background_control: Color32,
    pub(crate) color_background_control_primary: Color32,
    pub(crate) color_background_control_primary_dark: Color32,
    pub(crate) color_background_control_success: Color32,
    pub(crate) color_background_control_success_dark: Color32,
    pub(crate) color_foreground: Color32,
    pub(crate) color_foreground_preview: Color32,
    pub(crate) color_foreground_warning: Color32,
    pub(crate) color_foreground_error: Color32,
    pub(crate) color_foreground_info: Color32,
    pub(crate) color_foreground_debug: Color32,
    pub(crate) color_foreground_trace: Color32,
    pub(crate) color_border_blue: Color32,
    pub(crate) color_border_panel: Color32,
    pub(crate) color_log_background: Color32,
    pub(crate) color_hover_tint: Color32,
    pub(crate) color_pressed_tint: Color32,
    pub(crate) corner_radius_panel: u8,
    pub(crate) title_bar_height: f32,
    pub(crate) footer_height: f32,
}

impl Default for InstallerTheme {
    fn default() -> Self {
        Self {
            fonts: InstallerFontLibrary::new(),
            color_background_primary: Color32::from_rgb(0x33, 0x33, 0x33),
            color_background_panel: Color32::from_rgb(0x27, 0x27, 0x27),
            color_background_control: Color32::from_rgb(0x44, 0x44, 0x44),
            color_background_control_primary: Color32::from_rgb(0x1E, 0x54, 0x92),
            color_background_control_primary_dark: Color32::from_rgb(0x06, 0x1E, 0x3E),
            color_background_control_success: Color32::from_rgb(0x14, 0xA4, 0x4D),
            color_background_control_success_dark: Color32::from_rgb(0x0E, 0x72, 0x36),
            color_foreground: Color32::WHITE,
            color_foreground_preview: Color32::from_rgb(0xAF, 0xAF, 0xAF),
            color_foreground_warning: Color32::from_rgb(0xE4, 0xA1, 0x1B),
            color_foreground_error: Color32::from_rgb(0xDC, 0x4C, 0x64),
            color_foreground_info: Color32::from_rgb(0x32, 0xC4, 0xE6),
            color_foreground_debug: Color32::from_rgb(0x32, 0xC4, 0xE6),
            color_foreground_trace: Color32::from_rgb(0x14, 0xA4, 0x4D),
            color_border_blue: Color32::from_rgb(0x00, 0x7A, 0xCC),
            color_border_panel: Color32::from_rgb(0x20, 0x1C, 0x1C),
            color_log_background: Color32::from_rgb(0x1E, 0x1E, 0x1E),
            color_hover_tint: Color32::from_rgba_unmultiplied(0xFF, 0xFF, 0xFF, 0x10),
            color_pressed_tint: Color32::from_rgba_unmultiplied(0x00, 0x00, 0x00, 0x20),
            corner_radius_panel: 8,
            title_bar_height: 32.0,
            footer_height: 24.0,
        }
    }
}

impl InstallerTheme {
    pub(crate) fn apply(
        &self,
        context: &egui::Context,
    ) {
        let mut style = (*context.style()).clone();
        let visuals = &mut style.visuals;

        visuals.override_text_color = Some(self.color_foreground);
        visuals.panel_fill = self.color_background_primary;
        visuals.window_fill = self.color_background_panel;
        visuals.faint_bg_color = self.color_background_panel;
        visuals.extreme_bg_color = self.color_background_control;
        visuals.code_bg_color = self.color_log_background;
        visuals.window_stroke = Stroke::new(1.0, self.color_border_panel);
        visuals.selection.bg_fill = self.color_border_blue;
        visuals.selection.stroke = Stroke::new(1.0, self.color_border_blue);

        visuals.widgets.noninteractive.bg_fill = self.color_background_panel;
        visuals.widgets.noninteractive.weak_bg_fill = self.color_background_panel;
        visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, self.color_border_panel);
        visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, self.color_foreground_preview);
        visuals.widgets.noninteractive.corner_radius = CornerRadius::same(self.corner_radius_panel);

        visuals.widgets.inactive.bg_fill = self.color_background_control_primary;
        visuals.widgets.inactive.weak_bg_fill = self.color_background_control_primary_dark;
        visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, self.color_border_blue);
        visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, self.color_foreground);
        visuals.widgets.inactive.corner_radius = CornerRadius::same(self.corner_radius_panel);

        visuals.widgets.hovered.bg_fill = self.color_border_blue;
        visuals.widgets.hovered.weak_bg_fill = self.color_background_control_primary;
        visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, self.color_border_blue);
        visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, self.color_foreground);
        visuals.widgets.hovered.corner_radius = CornerRadius::same(self.corner_radius_panel);

        visuals.widgets.active.bg_fill = self.color_background_control_primary_dark;
        visuals.widgets.active.weak_bg_fill = self.color_background_control_primary_dark;
        visuals.widgets.active.bg_stroke = Stroke::new(1.0, self.color_border_blue);
        visuals.widgets.active.fg_stroke = Stroke::new(1.0, self.color_foreground);
        visuals.widgets.active.corner_radius = CornerRadius::same(self.corner_radius_panel);

        visuals.widgets.open.bg_fill = self.color_background_control;
        visuals.widgets.open.weak_bg_fill = self.color_background_control;
        visuals.widgets.open.bg_stroke = Stroke::new(1.0, self.color_border_panel);
        visuals.widgets.open.fg_stroke = Stroke::new(1.0, self.color_foreground);
        visuals.widgets.open.corner_radius = CornerRadius::same(self.corner_radius_panel);

        style.spacing.item_spacing = egui::vec2(8.0, 6.0);
        style.spacing.button_padding = egui::vec2(12.0, 6.0);
        style.spacing.window_margin = Margin::same(8);

        context.set_style(style);
    }

    pub(crate) fn install_fonts(
        &self,
        context: &egui::Context,
    ) {
        let mut font_definitions = FontDefinitions::default();
        font_definitions
            .font_data
            .insert("noto_sans".to_owned(), FontData::from_static(FONT_NOTO_SANS).into());
        font_definitions
            .font_data
            .insert("ubuntu_mono_bold".to_owned(), FontData::from_static(FONT_UBUNTU_MONO_BOLD).into());

        font_definitions
            .families
            .insert(FontFamily::Name("noto_sans".into()), vec!["noto_sans".to_owned()]);
        font_definitions
            .families
            .insert(FontFamily::Name("ubuntu_mono_bold".into()), vec!["ubuntu_mono_bold".to_owned()]);

        context.set_fonts(font_definitions);
    }
}
