use crate::ui::theme::Theme;
use crate::ui::widgets::main_window::main_window_view::MainWindowView;
use eframe::egui::{CentralPanel, Context, Frame, Visuals};
use epaint::{CornerRadius, Rgba, vec2};
use squalr_engine_api::{dependency_injection::dependency_container::DependencyContainer, engine::engine_execution_context::EngineExecutionContext};
use std::{rc::Rc, sync::Arc};

#[derive(Clone)]
pub struct App {
    main_window_view: MainWindowView,
    corner_radius: CornerRadius,
}

impl App {
    pub fn new(
        context: &Context,
        engine_execution_context: Arc<EngineExecutionContext>,
        dependency_container: &DependencyContainer,
        app_title: String,
    ) -> Self {
        let theme = Rc::new(Theme::new(context));
        let corner_radius = CornerRadius::same(8);
        let main_window_view = MainWindowView::new(engine_execution_context, context.clone(), theme, app_title, corner_radius);

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
            .corner_radius(self.corner_radius)
            .stroke(context.style().visuals.widgets.noninteractive.fg_stroke)
            .outer_margin(2.0);

        CentralPanel::default()
            .frame(app_frame)
            .show(context, move |user_interface| {
                user_interface.style_mut().spacing.item_spacing = vec2(0.0, 0.0);
                user_interface.add(self.main_window_view.clone());
            });
    }
}
