use crate::{
    models::docking::docking_manager::DockingManager,
    ui::{theme::Theme, widgets::docking::docked_window_view::DockedWindowView},
};
use eframe::egui::{Context, Layout, Response, Sense, Ui, UiBuilder, Widget};
use epaint::{CornerRadius, Rect, pos2, vec2};
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
        let (available_size_rect, response) = user_interface.allocate_exact_size(user_interface.available_size(), Sense::empty());

        // Background.
        user_interface
            .painter()
            .rect_filled(available_size_rect, CornerRadius::ZERO, self.theme.background_panel);

        if let Ok(mut docking_manager) = self.docking_manager.try_write() {
            docking_manager.prepare_for_presentation();
            docking_manager
                .get_main_window_layout_mut()
                .set_available_size(available_size_rect.width(), available_size_rect.height());
        }

        for window in &self.windows {
            let window_identifier = window.get_identifier();
            let active_tab_id = match self.docking_manager.try_read() {
                Ok(docking_manager) => docking_manager.get_active_tab(&window_identifier),
                Err(_) => String::new(),
            };

            // We only need to render the active tab in windows that share the same space.
            if active_tab_id != window_identifier {
                continue;
            }

            let window_rect = {
                if let Ok(docking_manager) = self.docking_manager.try_read() {
                    docking_manager
                        .find_window_rect(window_identifier)
                        .map(|(x, y, w, h)| {
                            let offset = available_size_rect.min;
                            Rect::from_min_size(pos2(offset.x + x as f32, offset.y + y as f32), vec2(w as f32, h as f32))
                        })
                } else {
                    None
                }
            };

            if let Some(window_rect) = window_rect {
                let builder = UiBuilder::new().max_rect(window_rect);
                let mut child_user_interface = user_interface.new_child(builder);

                window.clone().ui(&mut child_user_interface);
            }
        }

        response
    }
}
