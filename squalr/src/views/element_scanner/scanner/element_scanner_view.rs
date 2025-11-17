use crate::app_context::AppContext;
use crate::views::element_scanner::results::element_scanner_results_view::ElementScannerResultsView;
use crate::views::element_scanner::results::view_data::element_scanner_results_view_data::ElementScannerResultsViewData;
use crate::views::element_scanner::scanner::element_scanner_footer_view::ElementScannerFooterView;
use crate::views::element_scanner::scanner::element_scanner_toolbar_view::ElementScannerToolbarView;
use crate::views::element_scanner::scanner::view_data::element_scanner_view_data::ElementScannerViewData;
use eframe::egui::{Align, Layout, Response, Sense, Ui, UiBuilder, Widget};
use epaint::{Rect, vec2};
use squalr_engine_api::dependency_injection::dependency::Dependency;
use std::sync::Arc;

#[derive(Clone)]
pub struct ElementScannerView {
    app_context: Arc<AppContext>,
    element_scanner_view_data: Dependency<ElementScannerViewData>,
    element_scanner_results_view_data: Dependency<ElementScannerResultsViewData>,
    element_scanner_toolbar_view: ElementScannerToolbarView,
    element_scanner_results_view: ElementScannerResultsView,
    element_scanner_footer_view: ElementScannerFooterView,
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
        let element_scanner_footer_view = ElementScannerFooterView::new(app_context.clone());

        Self {
            app_context,
            element_scanner_view_data,
            element_scanner_results_view_data,
            element_scanner_toolbar_view,
            element_scanner_results_view,
            element_scanner_footer_view,
        }
    }
}

impl Widget for ElementScannerView {
    fn ui(
        self,
        user_interface: &mut Ui,
    ) -> Response {
        let response = user_interface
            .allocate_ui_with_layout(user_interface.available_size(), Layout::top_down(Align::Min), |user_interface| {
                user_interface.add(self.element_scanner_toolbar_view.clone());

                let footer_height = self.element_scanner_footer_view.get_height();
                let full_rectangle = user_interface.available_rect_before_wrap();
                let content_rectangle = Rect::from_min_max(full_rectangle.min, full_rectangle.max - vec2(0.0, footer_height));
                let content_response = user_interface.allocate_rect(content_rectangle, Sense::empty());
                let mut content_user_interface = user_interface.new_child(
                    UiBuilder::new()
                        .max_rect(content_response.rect)
                        .layout(Layout::left_to_right(Align::Min)),
                );

                content_user_interface.add(self.element_scanner_results_view.clone());

                user_interface.add(self.element_scanner_footer_view.clone());
            })
            .response;

        response
    }
}
