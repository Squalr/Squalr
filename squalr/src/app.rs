use crate::ui::{main_window::main_window_view::MainWindowView, theme::Theme};
use eframe::egui::{CentralPanel, Context, Frame, Visuals};
use epaint::{CornerRadius, Rgba, vec2};
use squalr_engine_api::dependency_injection::dependency_container::DependencyContainer;
use std::rc::Rc;

#[derive(Clone)]
pub struct App {
    main_window_view: MainWindowView,
    corner_radius: CornerRadius,
}

impl App {
    pub fn new(
        context: &Context,
        dependency_container: &DependencyContainer,
    ) -> Self {
        let theme = Rc::new(Theme::new(context));
        let corner_radius = CornerRadius::same(8);
        let main_window_view = MainWindowView::new(context.clone(), theme, corner_radius);

        Self {
            main_window_view,
            corner_radius,
        }
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
            .corner_radius(self.corner_radius)
            .stroke(context.style().visuals.widgets.noninteractive.fg_stroke)
            .outer_margin(1.0);

        CentralPanel::default()
            .frame(app_frame)
            .show(context, move |user_interface| {
                user_interface.style_mut().spacing.item_spacing = vec2(0.0, 0.0);
                user_interface.add(self.main_window_view.clone());
            });
    }
}
