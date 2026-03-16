use crate::app_context::AppContext;
use crate::views::pointer_scanner::pointer_scanner_results_view::PointerScannerResultsView;
use crate::views::pointer_scanner::pointer_scanner_toolbar_view::PointerScannerToolbarView;
use crate::views::pointer_scanner::view_data::pointer_scanner_view_data::PointerScannerViewData;
use eframe::egui::{Align, Layout, Response, Ui, Widget};
use squalr_engine_api::dependency_injection::dependency::Dependency;
use std::sync::Arc;

#[derive(Clone)]
pub struct PointerScannerView {
    app_context: Arc<AppContext>,
    pointer_scanner_view_data: Dependency<PointerScannerViewData>,
    pointer_scanner_toolbar_view: PointerScannerToolbarView,
    pointer_scanner_results_view: PointerScannerResultsView,
}

impl PointerScannerView {
    pub const WINDOW_ID: &'static str = "window_pointer_scanner";

    pub fn new(app_context: Arc<AppContext>) -> Self {
        let pointer_scanner_view_data = app_context
            .dependency_container
            .register(PointerScannerViewData::new());
        let pointer_scanner_toolbar_view = PointerScannerToolbarView::new(app_context.clone());
        let pointer_scanner_results_view = PointerScannerResultsView::new(app_context.clone());

        PointerScannerViewData::initialize(pointer_scanner_view_data.clone(), app_context.engine_unprivileged_state.clone());

        Self {
            app_context,
            pointer_scanner_view_data,
            pointer_scanner_toolbar_view,
            pointer_scanner_results_view,
        }
    }
}

impl Widget for PointerScannerView {
    fn ui(
        self,
        user_interface: &mut Ui,
    ) -> Response {
        user_interface
            .allocate_ui_with_layout(user_interface.available_size(), Layout::top_down(Align::Min), |user_interface| {
                user_interface.add(self.pointer_scanner_toolbar_view.clone());
                PointerScannerViewData::dispatch_queued_expand_requests(
                    self.pointer_scanner_view_data.clone(),
                    self.app_context.engine_unprivileged_state.clone(),
                );
                user_interface.add(self.pointer_scanner_results_view.clone());
            })
            .response
    }
}
