use crate::ui::fonts::font_set::FontSet;
use eframe::egui::{Context, FontData, FontDefinitions, FontFamily};

static FONT_NOTO_SANS: &[u8] = include_bytes!("../../../fonts/NotoSans.ttf");
static FONT_UBUNTU_MONO_BOLD: &[u8] = include_bytes!("../../../fonts/UbuntuMonoBold.ttf");

pub struct FontLibrary {
    pub font_noto_sans: FontSet,
    pub font_ubuntu_mono_bold: FontSet,
}

impl FontLibrary {
    pub fn new(context: &Context) -> Self {
        let mut fonts = FontDefinitions::default();

        Self::register_font(&mut fonts, "noto_sans", FONT_NOTO_SANS);
        Self::register_font(&mut fonts, "ubuntu_mono_bold", FONT_UBUNTU_MONO_BOLD);

        context.set_fonts(fonts);

        Self {
            font_noto_sans: FontSet::new(FontFamily::Name("noto_sans".into()), 9.0, 13.0, 14.0, 14.0),
            font_ubuntu_mono_bold: FontSet::new(FontFamily::Name("ubuntu_mono_bold".into()), 11.0, 15.0, 16.0, 16.0),
        }
    }

    fn register_font(
        fonts: &mut FontDefinitions,
        name: &str,
        data: &'static [u8],
    ) {
        fonts
            .font_data
            .insert(name.to_owned(), FontData::from_static(data).into());
        fonts
            .families
            .insert(FontFamily::Name(name.into()), vec![name.to_owned()]);
    }
}
