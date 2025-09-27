use crate::ui::{main_window::main_window_view::MainWindowView, theme::Theme};
use eframe::egui::{CentralPanel, Context, Frame, Visuals};
use epaint::Rgba;
use squalr_engine_api::dependency_injection::dependency_container::DependencyContainer;
use std::rc::Rc;

#[derive(Clone)]
pub struct App {
    main_window_view: MainWindowView,
}

impl App {
    pub fn new(
        context: &Context,
        dependency_container: &DependencyContainer,
    ) -> Self {
        let theme = Rc::new(Theme::new(context));
        let main_window_view = MainWindowView::new(context.clone(), theme);

        Self { main_window_view }
    }
}

impl eframe::App for App {
    fn clear_color(
        &self,
        _visuals: &Visuals,
    ) -> [f32; 4] {
        Rgba::TRANSPARENT.to_array()
    }

    fn update(
        &mut self,
        context: &Context,
        _frame: &mut eframe::Frame,
    ) {
        let app_frame = Frame::new()
            .fill(context.style().visuals.window_fill())
            .corner_radius(10)
            .stroke(context.style().visuals.widgets.noninteractive.fg_stroke)
            .outer_margin(1.0);

        CentralPanel::default()
            .frame(app_frame)
            .show(context, move |user_interface| {
                user_interface.add(self.main_window_view.clone());
            });
    }
}
