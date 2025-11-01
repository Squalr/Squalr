use eframe::egui::{Context, TextureOptions};
use epaint::{ColorImage, TextureHandle};

// App.
static ICON_LOGO: &[u8] = include_bytes!("../../images/app/app_icon_small.png");
static ICON_CLOSE: &[u8] = include_bytes!("../../images/app/close.png");
static ICON_MINIMIZE: &[u8] = include_bytes!("../../images/app/minimize.png");
static ICON_MAXIMIZE: &[u8] = include_bytes!("../../images/app/maximize.png");

// Common.
static ICON_CHECK_MARK: &[u8] = include_bytes!("../../images/app/common/check_mark.png");

// Navigation.
static ICON_NAVIGATION_DOWN_ARROW_SMALL: &[u8] = include_bytes!("../../images/navigation/down_arrow_small.png");
static ICON_NAVIGATION_REFRESH: &[u8] = include_bytes!("../../images/navigation/refresh.png");

pub struct IconLibrary {
    // App.
    pub icon_handle_logo: TextureHandle,
    pub icon_handle_close: TextureHandle,
    pub icon_handle_minimize: TextureHandle,
    pub icon_handle_maximize: TextureHandle,

    // Common.
    pub icon_handle_check_mark: TextureHandle,

    // Navigation.
    pub icon_navigation_down_arrow_small: TextureHandle,
    pub icon_navigation_refresh: TextureHandle,
}

impl IconLibrary {
    pub fn new(context: &Context) -> Self {
        let icon_handle_logo = Self::load_icon(context, ICON_LOGO);
        let icon_handle_close = Self::load_icon(context, ICON_CLOSE);
        let icon_handle_minimize = Self::load_icon(context, ICON_MINIMIZE);
        let icon_handle_maximize = Self::load_icon(context, ICON_MAXIMIZE);
        let icon_handle_check_mark = Self::load_icon(context, ICON_CHECK_MARK);
        let icon_navigation_down_arrow_small = Self::load_icon(context, ICON_NAVIGATION_DOWN_ARROW_SMALL);
        let icon_navigation_refresh = Self::load_icon(context, ICON_NAVIGATION_REFRESH);

        Self {
            icon_handle_logo,
            icon_handle_close,
            icon_handle_minimize,
            icon_handle_maximize,
            icon_handle_check_mark,
            icon_navigation_down_arrow_small,
            icon_navigation_refresh,
        }
    }

    fn load_icon(
        context: &Context,
        buffer: &[u8],
    ) -> TextureHandle {
        let image = image::load_from_memory(buffer).unwrap_or_default().to_rgba8();
        let size = [image.width() as usize, image.height() as usize];
        let pixels = image.into_raw();
        let texture_handle = context.load_texture("", ColorImage::from_rgba_unmultiplied(size, &pixels), TextureOptions::default());

        texture_handle
    }
}
