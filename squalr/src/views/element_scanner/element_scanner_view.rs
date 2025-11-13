use crate::{
    app_context::AppContext,
    views::element_scanner::{
        element_scanner_results_view::ElementScannerResultsView,
        element_scanner_toolbar_view::ElementScannerToolbarView,
        view_data::{element_scanner_results_view_data::ElementScannerResultsViewData, element_scanner_view_data::ElementScannerViewData},
    },
};
use eframe::egui::{Align, Layout, Response, Ui, Widget};
use squalr_engine_api::dependency_injection::dependency::Dependency;
use std::sync::Arc;

#[derive(Clone)]
pub struct ElementScannerView {
    app_context: Arc<AppContext>,
    element_scanner_view_data: Dependency<ElementScannerViewData>,
    element_scanner_results_view_data: Dependency<ElementScannerResultsViewData>,
    element_scanner_toolbar_view: ElementScannerToolbarView,
    element_scanner_results_view: ElementScannerResultsView,
}

impl ElementScannerView {
    pub fn new(app_context: Arc<AppContext>) -> Self {
        let element_scanner_view_data = app_context
            .dependency_container
            .register(ElementScannerViewData::new());
        let element_scanner_results_view_data = app_context
            .dependency_container
            .register(ElementScannerResultsViewData::new());
        let element_scanner_toolbar_view = ElementScannerToolbarView::new(app_context.clone());
        let element_scanner_results_view = ElementScannerResultsView::new(app_context.clone());

        Self {
            app_context,
            element_scanner_view_data,
            element_scanner_results_view_data,
            element_scanner_toolbar_view,
            element_scanner_results_view,
        }
    }
}

impl Widget for ElementScannerView {
    fn ui(
        self,
        user_interface: &mut Ui,
    ) -> Response {
        let response = user_interface
            .allocate_ui_with_layout(user_interface.available_size(), Layout::top_down(Align::Min), |mut user_interface| {
                user_interface.add(self.element_scanner_toolbar_view.clone());
                user_interface.add(self.element_scanner_results_view.clone());
            })
            .response;

        response
    }
}
