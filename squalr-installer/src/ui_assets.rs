use eframe::egui;
use eframe::egui::{Color32, ColorImage, IconData, TextureHandle};

pub(crate) const APP_NAME: &str = "Squalr Installer";

pub(crate) static ICON_APP: &[u8] = include_bytes!("../../squalr/images/app/app_icon.png");
static ICON_CLOSE: &[u8] = include_bytes!("../../squalr/images/app/close.png");
static ICON_MAXIMIZE: &[u8] = include_bytes!("../../squalr/images/app/maximize.png");
static ICON_MINIMIZE: &[u8] = include_bytes!("../../squalr/images/app/minimize.png");

#[derive(Clone)]
pub(crate) struct InstallerIconLibrary {
    pub(crate) app: TextureHandle,
    pub(crate) close: TextureHandle,
    pub(crate) maximize: TextureHandle,
    pub(crate) minimize: TextureHandle,
}

pub(crate) fn load_app_icon_data() -> Option<IconData> {
    let icon_rgba = image::load_from_memory(ICON_APP).ok()?.into_rgba8();
    let width = icon_rgba.width();
    let height = icon_rgba.height();

    Some(IconData {
        rgba: icon_rgba.into_raw(),
        width,
        height,
    })
}

fn load_installer_icon(
    context: &egui::Context,
    identifier: &str,
    icon_bytes: &[u8],
) -> Option<TextureHandle> {
    let icon = image::load_from_memory(icon_bytes).ok()?.into_rgba8();
    let icon_size = [icon.width() as usize, icon.height() as usize];
    let icon_color_image = ColorImage::from_rgba_unmultiplied(icon_size, icon.as_raw());
    Some(context.load_texture(identifier, icon_color_image, egui::TextureOptions::LINEAR))
}

pub(crate) fn load_installer_icon_library(context: &egui::Context) -> Option<InstallerIconLibrary> {
    Some(InstallerIconLibrary {
        app: load_installer_icon(context, "installer_app_icon", ICON_APP)?,
        close: load_installer_icon(context, "installer_close_icon", ICON_CLOSE)?,
        maximize: load_installer_icon(context, "installer_maximize_icon", ICON_MAXIMIZE)?,
        minimize: load_installer_icon(context, "installer_minimize_icon", ICON_MINIMIZE)?,
    })
}

pub(crate) fn draw_icon(
    ui: &egui::Ui,
    bounds_rectangle: egui::Rect,
    icon_texture: &TextureHandle,
) {
    let [texture_width, texture_height] = icon_texture.size();
    let icon_rectangle = egui::Rect::from_center_size(bounds_rectangle.center(), egui::vec2(texture_width as f32, texture_height as f32));
    ui.painter().image(
        icon_texture.id(),
        icon_rectangle,
        egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
        Color32::WHITE,
    );
}
