use crate::{
    models::docking::docking_manager::DockingManager,
    ui::{theme::Theme, widgets::docking::docked_window_view::DockedWindowView},
};
use eframe::egui::{Align, Context, Layout, Response, Sense, Ui, UiBuilder, Widget};
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
        docking_manager: Arc<RwLock<DockingManager>>,
        built_in_windows: Vec<DockedWindowView>,
    ) -> Self {
        Self {
            _engine_execution_context: engine_execution_context,
            _context: context,
            theme,
            docking_manager,
            windows: built_in_windows,
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

        if let Ok(mut docking_manager) = self.docking_manager.try_write() {
            docking_manager.prepare_for_presentation();

            let builder = UiBuilder::new()
                .max_rect(rect)
                .layout(Layout::left_to_right(Align::Min));
            let mut child_user_interface = user_interface.new_child(builder);

            for window in self.windows {
                let window_identifier = window.get_identifier();

                // Find bounding rectangle.
                if let Some((x, y, w, h)) = docking_manager.find_window_rect(window_identifier) {
                    let found_active_tab_id = docking_manager.get_active_tab(window_identifier);
                    let siblings = {
                        let visible_siblings = docking_manager.get_sibling_tab_ids(window_identifier, true);
                        if visible_siblings.len() == 1 { vec![] } else { visible_siblings }
                    };
                }
            }
        }

        response
    }
}
