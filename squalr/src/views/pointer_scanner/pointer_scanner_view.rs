use crate::app_context::AppContext;
use crate::views::pointer_scanner::pointer_scanner_footer_view::PointerScannerFooterView;
use crate::views::pointer_scanner::pointer_scanner_results_view::PointerScannerResultsView;
use crate::views::pointer_scanner::pointer_scanner_toolbar_view::PointerScannerToolbarView;
use crate::views::pointer_scanner::view_data::pointer_scanner_view_data::PointerScannerViewData;
use eframe::egui::{Align, Layout, Response, Sense, Ui, UiBuilder, Widget};
use epaint::{Rect, vec2};
use squalr_engine_api::dependency_injection::dependency::Dependency;
use std::sync::Arc;

#[derive(Clone)]
pub struct PointerScannerView {
    app_context: Arc<AppContext>,
    pointer_scanner_view_data: Dependency<PointerScannerViewData>,
    pointer_scanner_toolbar_view: PointerScannerToolbarView,
    pointer_scanner_results_view: PointerScannerResultsView,
    pointer_scanner_footer_view: PointerScannerFooterView,
}

impl PointerScannerView {
    pub const WINDOW_ID: &'static str = "window_pointer_scanner";

    pub fn new(app_context: Arc<AppContext>) -> Self {
        let pointer_scanner_view_data = app_context
            .dependency_container
            .register(PointerScannerViewData::new());
        if let Some(mut pointer_scanner_view_data_guard) = pointer_scanner_view_data.write("Pointer scanner register repaint callback") {
            let pointer_scanner_context = app_context.context.clone();
            pointer_scanner_view_data_guard.set_repaint_request_callback(Arc::new(move || {
                pointer_scanner_context.request_repaint();
            }));
        }
        let pointer_scanner_toolbar_view = PointerScannerToolbarView::new(app_context.clone());
        let pointer_scanner_results_view = PointerScannerResultsView::new(app_context.clone());
        let pointer_scanner_footer_view = PointerScannerFooterView::new(app_context.clone());

        PointerScannerViewData::initialize(pointer_scanner_view_data.clone(), app_context.engine_unprivileged_state.clone());

        Self {
            app_context,
            pointer_scanner_view_data,
            pointer_scanner_toolbar_view,
            pointer_scanner_results_view,
            pointer_scanner_footer_view,
        }
    }
}

impl Widget for PointerScannerView {
    fn ui(
        self,
        user_interface: &mut Ui,
    ) -> Response {
        user_interface
            .scope(|user_interface| {
                user_interface
                    .allocate_ui_with_layout(user_interface.available_size(), Layout::top_down(Align::Min), |user_interface| {
                        user_interface.add(self.pointer_scanner_toolbar_view.clone());
                        PointerScannerViewData::dispatch_queued_expand_requests(
                            self.pointer_scanner_view_data.clone(),
                            self.app_context.engine_unprivileged_state.clone(),
                        );

                        let footer_height = self.pointer_scanner_footer_view.get_height();
                        let full_rectangle = user_interface
                            .available_rect_before_wrap()
                            .intersect(user_interface.clip_rect());
                        let content_rectangle = Rect::from_min_max(full_rectangle.min, full_rectangle.max - vec2(0.0, footer_height));
                        let content_response = user_interface.allocate_rect(content_rectangle, Sense::empty());
                        let mut content_user_interface = user_interface.new_child(
                            UiBuilder::new()
                                .max_rect(content_response.rect)
                                .layout(Layout::left_to_right(Align::Min)),
                        );
                        content_user_interface.set_clip_rect(content_response.rect);

                        content_user_interface.add(self.pointer_scanner_results_view.clone());
                        user_interface.add(self.pointer_scanner_footer_view.clone());
                    })
                    .response
            })
            .inner
    }
}
