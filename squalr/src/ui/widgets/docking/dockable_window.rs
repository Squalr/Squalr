use eframe::egui::{Response, Ui};

pub trait DockableWindow {
    fn get_identifier(&self) -> &str;
    fn ui(
        &self,
        ui: &mut Ui,
    ) -> Response;
}
