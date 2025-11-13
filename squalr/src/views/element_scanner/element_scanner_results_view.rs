use crate::{
    app_context::AppContext,
    views::element_scanner::{
        element_scanner_result_entry_view::ElementScannerResultEntryView, view_data::element_scanner_results_view_data::ElementScannerResultsViewData,
    },
};
use eframe::egui::{Align, Layout, Response, ScrollArea, Ui, Widget};
use squalr_engine_api::dependency_injection::dependency::Dependency;
use std::sync::Arc;

#[derive(Clone)]
pub struct ElementScannerResultsView {
    app_context: Arc<AppContext>,
    element_scanner_results_view_data: Dependency<ElementScannerResultsViewData>,
}

impl ElementScannerResultsView {
    pub fn new(app_context: Arc<AppContext>) -> Self {
        let element_scanner_results_view_data = app_context
            .dependency_container
            .register(ElementScannerResultsViewData::new());

        ElementScannerResultsViewData::poll_scan_results(element_scanner_results_view_data.clone(), app_context.engine_execution_context.clone());

        Self {
            app_context,
            element_scanner_results_view_data,
        }
    }
}

impl Widget for ElementScannerResultsView {
    fn ui(
        self,
        user_interface: &mut Ui,
    ) -> Response {
        let response = user_interface
            .allocate_ui_with_layout(user_interface.available_size(), Layout::top_down(Align::Min), |mut user_interface| {
                ScrollArea::vertical()
                    .id_salt("element_scanner")
                    .auto_shrink([false, false])
                    .show(&mut user_interface, |inner_user_interface| {
                        let element_scanner_results_view_data = match self.element_scanner_results_view_data.read() {
                            Ok(element_scanner_results_view_data) => element_scanner_results_view_data,
                            Err(_error) => {
                                return;
                            }
                        };

                        for scan_result in &element_scanner_results_view_data.current_scan_results {
                            let icon = None;

                            if inner_user_interface
                                .add(ElementScannerResultEntryView::new(self.app_context.clone(), scan_result, icon))
                                .double_clicked()
                            {
                                //
                            }
                        }
                    });
            })
            .response;

        response
    }
}
