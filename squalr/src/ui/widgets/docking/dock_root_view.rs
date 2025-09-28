use crate::{
    models::docking::{docking_manager::DockingManager, settings::dockable_window_settings::DockableWindowSettings},
    ui::{theme::Theme, widgets::docking::docked_window_view::DockedWindowView},
};
use eframe::egui::{Context, Response, Sense, Ui, Widget};
use epaint::CornerRadius;
use squalr_engine_api::engine::engine_execution_context::EngineExecutionContext;
use std::{
    rc::Rc,
    sync::{Arc, RwLock},
};

#[derive(Clone)]
pub struct DockRootView {
    _engine_execution_context: Arc<EngineExecutionContext>,
    _context: Context,
    theme: Rc<Theme>,
    docking_manager: Arc<RwLock<DockingManager>>,
    windows: Vec<DockedWindowView>,
}

impl DockRootView {
    pub fn new(
        engine_execution_context: Arc<EngineExecutionContext>,
        context: Context,
        theme: Rc<Theme>,
    ) -> Self {
        let main_dock_root = DockableWindowSettings::get_dock_layout_settings();
        let docking_manager = Arc::new(RwLock::new(DockingManager::new(main_dock_root)));

        Self {
            _engine_execution_context: engine_execution_context,
            _context: context,
            theme,
            docking_manager,
            windows: vec![],
        }
    }
}

impl Widget for DockRootView {
    fn ui(
        self,
        user_interface: &mut Ui,
    ) -> Response {
        let (rect, response) = user_interface.allocate_exact_size(user_interface.available_size(), Sense::empty());

        // Background.
        user_interface
            .painter()
            .rect_filled(rect, CornerRadius::ZERO, self.theme.hex_green);

        response
    }
}
