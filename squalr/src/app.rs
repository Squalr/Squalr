use crate::models::docking::docking_manager::DockingManager;
use crate::models::docking::settings::dockable_window_settings::DockableWindowSettings;
use crate::views::main_window::main_window_view::MainWindowView;
use crate::{app_context::AppContext, ui::theme::Theme};
use eframe::egui::{CentralPanel, Context, Frame, Visuals};
use epaint::{CornerRadius, Rgba, vec2};
use squalr_engine_api::dependency_injection::dependency_container::DependencyContainer;
use squalr_engine_session::engine_unprivileged_state::EngineUnprivilegedState;
use std::sync::RwLock;
use std::{rc::Rc, sync::Arc};

#[derive(Clone)]
pub struct App {
    app_context: Arc<AppContext>,
    main_window_view: MainWindowView,
    corner_radius: CornerRadius,
}

impl App {
    pub fn new(
        context: &Context,
        engine_unprivileged_state: Arc<EngineUnprivilegedState>,
        dependency_container: &DependencyContainer,
        app_title: String,
    ) -> Self {
        let theme = Arc::new(Theme::new(context));
        // Create built in docked windows.
        let main_dock_root = DockableWindowSettings::get_dock_layout_settings();
        let docking_manager = Arc::new(RwLock::new(DockingManager::new(main_dock_root)));
        let app_context = Arc::new(AppContext::new(context.clone(), theme, docking_manager, engine_unprivileged_state));
        let corner_radius = CornerRadius::same(8);
        let main_window_view = MainWindowView::new(app_context.clone(), Rc::new(app_title), corner_radius);

        Self {
            app_context,
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
