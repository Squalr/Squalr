use eframe::egui;
// use squalr_gui::view_models::main_window::main_window_view_model::MainWindowViewModel;

#[derive(Default)]
pub struct MyApp {
    counter: i32,
}

impl eframe::App for MyApp {
    fn update(
        &mut self,
        ctx: &egui::Context,
        frame: &mut eframe::Frame,
    ) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.label("Hello, world!");

            if ui.button("Click me").clicked() {
                self.counter += 1;
            }

            ui.label(format!("Counter: {}", self.counter));
        });
    }
}
