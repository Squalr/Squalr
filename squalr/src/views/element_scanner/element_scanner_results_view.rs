use crate::{
    app_context::AppContext,
    views::element_scanner::{
        element_scanner_result_entry_view::ElementScannerResultEntryView, view_data::element_scanner_results_view_data::ElementScannerResultsViewData,
    },
};
use eframe::egui::{Align, CursorIcon, Layout, Response, ScrollArea, Sense, Ui, Widget};
use epaint::{Rect, pos2};
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
        const BAR_THICKNESS: f32 = 4.0;
        const MINIMUM_COLUMN_PIXEL_WIDTH: f32 = 80.0;
        const MINIMUM_SPLITTER_PIXEL_GAP: f32 = 40.0;
        const DEFAULT_VALUE_SPLITTER_RATIO: f32 = 0.45;
        const DEFAULT_PREVIOUS_VALUE_SPLITTER_RATIO: f32 = 0.75;

        let theme = &self.app_context.theme;
        let mut new_value_splitter_ratio: Option<f32> = None;
        let mut new_previous_value_splitter_ratio: Option<f32> = None;

        let response = user_interface
            .allocate_ui_with_layout(user_interface.available_size(), Layout::top_down(Align::Min), |mut user_interface| {
                ScrollArea::vertical()
                    .id_salt("element_scanner")
                    .auto_shrink([false, false])
                    .show(&mut user_interface, |user_interface| {
                        let allocate_resize_bar = |user_interface: &mut Ui, resize_rectangle: Rect, id_suffix: &str| -> Response {
                            let id = user_interface.id().with(id_suffix);
                            let response = user_interface.interact(resize_rectangle, id, Sense::drag());

                            user_interface
                                .painter()
                                .rect_filled(resize_rectangle, 0.0, theme.background_control);

                            response
                        };
                        let element_scanner_results_view_data = match self.element_scanner_results_view_data.read() {
                            Ok(element_scanner_results_view_data) => element_scanner_results_view_data,
                            Err(_error) => {
                                return;
                            }
                        };

                        let mut value_splitter_ratio = element_scanner_results_view_data.value_splitter_ratio;
                        let mut previous_value_splitter_ratio = element_scanner_results_view_data.previous_value_splitter_ratio;
                        let content_clip_rectangle = user_interface.available_rect_before_wrap();
                        let content_width = content_clip_rectangle.width();
                        let content_min_x = content_clip_rectangle.min.x;

                        // Clamp splitters to row height.
                        let mut rows_min_y: Option<f32> = None;
                        let mut rows_max_y: Option<f32> = None;

                        if content_width <= 0.0 {
                            return;
                        }

                        if value_splitter_ratio <= 0.0 || previous_value_splitter_ratio <= 0.0 || previous_value_splitter_ratio <= value_splitter_ratio {
                            value_splitter_ratio = DEFAULT_VALUE_SPLITTER_RATIO;
                            previous_value_splitter_ratio = DEFAULT_PREVIOUS_VALUE_SPLITTER_RATIO;

                            new_value_splitter_ratio = Some(value_splitter_ratio);
                            new_previous_value_splitter_ratio = Some(previous_value_splitter_ratio);
                        }

                        let value_splitter_position_x = content_min_x + content_width * value_splitter_ratio;
                        let previous_value_splitter_position_x = content_min_x + content_width * previous_value_splitter_ratio;

                        // Draw rows, capture min/max Y.
                        for scan_result in &element_scanner_results_view_data.current_scan_results {
                            let icon = None;

                            let row_response = user_interface.add(ElementScannerResultEntryView::new(
                                self.app_context.clone(),
                                scan_result,
                                icon,
                                value_splitter_position_x,
                                previous_value_splitter_position_x,
                            ));

                            if rows_min_y.is_none() {
                                rows_min_y = Some(row_response.rect.min.y);
                            }
                            rows_max_y = Some(row_response.rect.max.y);

                            if row_response.double_clicked() {
                                // JIRA: Double click logic.
                            }
                        }

                        // Use row bounds, fallback to content rectangle.
                        let content_min_y = rows_min_y.unwrap_or(content_clip_rectangle.min.y);
                        let content_max_y = rows_max_y.unwrap_or(content_clip_rectangle.max.y);

                        // Faux address splitter.
                        let faux_splitter_position_x = content_min_x + 32.0 + 4.0;

                        let faux_splitter_rectangle = Rect::from_min_max(
                            pos2(faux_splitter_position_x - BAR_THICKNESS * 0.5, content_min_y),
                            pos2(faux_splitter_position_x + BAR_THICKNESS * 0.5, content_max_y),
                        );

                        user_interface
                            .painter()
                            .rect_filled(faux_splitter_rectangle, 0.0, theme.background_control);

                        // Value splitter.
                        let value_splitter_rectangle = Rect::from_min_max(
                            pos2(value_splitter_position_x - BAR_THICKNESS * 0.5, content_min_y),
                            pos2(value_splitter_position_x + BAR_THICKNESS * 0.5, content_max_y),
                        );

                        let value_splitter_response =
                            allocate_resize_bar(user_interface, value_splitter_rectangle, "value_splitter").on_hover_cursor(CursorIcon::ResizeHorizontal);

                        if value_splitter_response.dragged() {
                            let drag_delta = value_splitter_response.drag_delta();
                            let mut new_value_splitter_position_x = value_splitter_position_x + drag_delta.x;

                            let minimum_value_splitter_position_x = content_min_x + MINIMUM_COLUMN_PIXEL_WIDTH;
                            let maximum_value_splitter_position_x = previous_value_splitter_position_x - MINIMUM_SPLITTER_PIXEL_GAP;

                            new_value_splitter_position_x =
                                new_value_splitter_position_x.clamp(minimum_value_splitter_position_x, maximum_value_splitter_position_x);

                            let bounded_value_splitter_ratio = (new_value_splitter_position_x - content_min_x) / content_width;

                            new_value_splitter_ratio = Some(bounded_value_splitter_ratio);
                        }

                        // Previous value splitter.
                        let previous_value_splitter_rectangle = Rect::from_min_max(
                            pos2(previous_value_splitter_position_x - BAR_THICKNESS * 0.5, content_min_y),
                            pos2(previous_value_splitter_position_x + BAR_THICKNESS * 0.5, content_max_y),
                        );

                        let previous_value_splitter_response =
                            allocate_resize_bar(user_interface, previous_value_splitter_rectangle, "previous_value_splitter")
                                .on_hover_cursor(CursorIcon::ResizeHorizontal);

                        if previous_value_splitter_response.dragged() {
                            let drag_delta = previous_value_splitter_response.drag_delta();
                            let mut new_previous_value_splitter_position_x = previous_value_splitter_position_x + drag_delta.x;

                            let minimum_previous_value_splitter_position_x = value_splitter_position_x + MINIMUM_SPLITTER_PIXEL_GAP;
                            let maximum_previous_value_splitter_position_x = content_min_x + content_width - MINIMUM_COLUMN_PIXEL_WIDTH;

                            new_previous_value_splitter_position_x = new_previous_value_splitter_position_x
                                .clamp(minimum_previous_value_splitter_position_x, maximum_previous_value_splitter_position_x);

                            let bounded_previous_value_splitter_ratio = (new_previous_value_splitter_position_x - content_min_x) / content_width;

                            new_previous_value_splitter_ratio = Some(bounded_previous_value_splitter_ratio);
                        }
                    });
            })
            .response;

        if new_value_splitter_ratio.is_some() || new_previous_value_splitter_ratio.is_some() {
            let mut element_scanner_results_view_data = match self.element_scanner_results_view_data.write() {
                Ok(element_scanner_results_view_data) => element_scanner_results_view_data,
                Err(_error) => {
                    return response;
                }
            };

            if let Some(new_value_splitter_ratio) = new_value_splitter_ratio {
                element_scanner_results_view_data.value_splitter_ratio = new_value_splitter_ratio;
            }

            if let Some(new_previous_value_splitter_ratio) = new_previous_value_splitter_ratio {
                element_scanner_results_view_data.previous_value_splitter_ratio = new_previous_value_splitter_ratio;
            }
        }

        response
    }
}
