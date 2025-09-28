use crate::models::docking::docking_manager::DockingManager;
use crate::models::docking::settings::dockable_window_settings::DockableWindowSettings;
use crate::ui::widgets::docking::docked_window_content_view::DockedWindowContentView;
use crate::ui::widgets::docking::docked_window_footer_view::DockedWindowFooterView;
use crate::ui::{theme::Theme, widgets::docking::docked_window_title_bar_view::DockedWindowTitleBarView};
use eframe::egui::{Align, Context, Layout, Response, Ui, Widget};
use epaint::CornerRadius;
use epaint::mutex::RwLock;
use squalr_engine_api::engine::engine_execution_context::EngineExecutionContext;
use std::rc::Rc;
use std::sync::Arc;

#[derive(Clone)]
pub struct DockedWindowView {
    _engine_execution_context: Arc<EngineExecutionContext>,
    _context: Context,
    _theme: Rc<Theme>,
    docking_manager: Arc<RwLock<DockingManager>>,
    docked_window_title_bar_view: DockedWindowTitleBarView,
    docked_window_content_view: DockedWindowContentView,
    docked_window_footer_view: DockedWindowFooterView,
}

impl DockedWindowView {
    pub fn new(
        engine_execution_context: Arc<EngineExecutionContext>,
        context: Context,
        theme: Rc<Theme>,
        docking_manager: Arc<RwLock<DockingManager>>,
        title: String,
        corner_radius: CornerRadius,
    ) -> Self {
        let docked_window_title_bar_view = DockedWindowTitleBarView::new(context.clone(), theme.clone(), corner_radius, 32.0, title);
        let docked_window_content_view = DockedWindowContentView::new(context.clone(), theme.clone());
        let docked_window_footer_view = DockedWindowFooterView::new(context.clone(), theme.clone(), corner_radius, 28.0);

        Self {
            _engine_execution_context: engine_execution_context,
            _context: context,
            _theme: theme,
            docking_manager,
            docked_window_title_bar_view,
            docked_window_content_view,
            docked_window_footer_view,
        }
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
                user_interface.add_sized(
                    [
                        user_interface.available_width(),
                        user_interface.available_height() - self.docked_window_footer_view.get_height(),
                    ],
                    self.docked_window_content_view,
                );
                user_interface.add(self.docked_window_footer_view);
            })
            .response;

        response
    }
}
