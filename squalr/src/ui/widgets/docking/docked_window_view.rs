use crate::models::docking::docking_manager::{self, DockingManager};
use crate::ui::widgets::docking::docked_window_footer_view::DockedWindowFooterView;
use crate::ui::{theme::Theme, widgets::docking::docked_window_title_bar_view::DockedWindowTitleBarView};
use eframe::egui::{Align, Context, Layout, Response, Ui, Widget};
use squalr_engine_api::engine::engine_execution_context::EngineExecutionContext;
use std::rc::Rc;
use std::sync::{Arc, RwLock};

#[derive(Clone)]
pub struct DockedWindowView {
    _engine_execution_context: Arc<EngineExecutionContext>,
    _context: Context,
    _theme: Rc<Theme>,
    docking_manager: Arc<RwLock<DockingManager>>,
    docked_window_title_bar_view: DockedWindowTitleBarView,
    content: Arc<dyn Fn(&mut Ui) -> Response>,
    docked_window_footer_view: DockedWindowFooterView,
    identifier: String,
}

impl DockedWindowView {
    pub fn new(
        engine_execution_context: Arc<EngineExecutionContext>,
        context: Context,
        theme: Rc<Theme>,
        docking_manager: Arc<RwLock<DockingManager>>,
        content: Arc<dyn Fn(&mut Ui) -> Response>,
        identifier: String,
    ) -> Self {
        let docked_window_title_bar_view = DockedWindowTitleBarView::new(context.clone(), theme.clone(), docking_manager.clone(), identifier.clone());
        let docked_window_footer_view = DockedWindowFooterView::new(context.clone(), theme.clone());

        Self {
            _engine_execution_context: engine_execution_context,
            _context: context,
            _theme: theme,
            docking_manager,
            docked_window_title_bar_view,
            content,
            docked_window_footer_view,
            identifier,
        }
    }

    pub fn get_identifier(&self) -> &str {
        &self.identifier
    }
}

impl Widget for DockedWindowView {
    fn ui(
        self,
        user_interface: &mut Ui,
    ) -> Response {
        let response = user_interface
            .allocate_ui_with_layout(user_interface.available_size(), Layout::top_down(Align::Min), |user_interface| {
                user_interface.add(self.docked_window_title_bar_view);
                (self.content)(user_interface);
                user_interface.add(self.docked_window_footer_view);
            })
            .response;

        response
    }
}
