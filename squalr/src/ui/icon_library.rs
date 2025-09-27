use eframe::egui::{Context, TextureOptions};
use epaint::{ColorImage, TextureHandle};

static ICON_CLOSE: &[u8] = include_bytes!("../../images/window/close.png");

pub struct IconLibrary {
    pub close_handle: TextureHandle,
}

impl IconLibrary {
    pub fn new(context: &Context) -> Self {
        let close_handle = Self::load_icon(context, ICON_CLOSE);

        Self { close_handle }
    }

    fn load_icon(
        context: &Context,
        buffer: &[u8],
    ) -> TextureHandle {
        let image = image::load_from_memory(buffer).unwrap_or_default().to_rgba8();
        let size = [image.width() as usize, image.height() as usize];
        let pixels = image.into_raw();
        let texture_handle = context.load_texture("my_image", ColorImage::from_rgba_unmultiplied(size, &pixels), TextureOptions::default());

        texture_handle
    }
}
