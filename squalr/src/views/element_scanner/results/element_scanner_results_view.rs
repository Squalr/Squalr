use crate::{
    app_context::AppContext,
    ui::draw::icon_draw::IconDraw,
    views::element_scanner::results::{
        element_scanner_result_entry_view::ElementScannerResultEntryView, view_data::element_scanner_results_view_data::ElementScannerResultsViewData,
    },
};
use eframe::egui::{Align, Align2, CursorIcon, Layout, Response, ScrollArea, Sense, Ui, Widget};
use epaint::{Rect, pos2, vec2};
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
        const FAUX_BAR_THICKNESS: f32 = 3.0;
        const BAR_THICKNESS: f32 = 4.0;
        const MINIMUM_COLUMN_PIXEL_WIDTH: f32 = 80.0;
        const MINIMUM_SPLITTER_PIXEL_GAP: f32 = 40.0;

        let theme = &self.app_context.theme;
        let mut new_value_splitter_ratio: Option<f32> = None;
        let mut new_previous_value_splitter_ratio: Option<f32> = None;

        let response = user_interface
            .allocate_ui_with_layout(user_interface.available_size(), Layout::top_down(Align::Min), |mut user_interface| {
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

                // Draw the header.
                let header_height = 32.0;
                let (header_rectangle, _header_response) =
                    user_interface.allocate_exact_size(vec2(user_interface.available_size().x, header_height), Sense::empty());
                let (separator_rect, _) = user_interface.allocate_exact_size(vec2(user_interface.available_size().x, FAUX_BAR_THICKNESS), Sense::empty());

                user_interface
                    .painter()
                    .rect_filled(separator_rect, 0.0, theme.background_control);

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
                    value_splitter_ratio = ElementScannerResultsViewData::DEFAULT_VALUE_SPLITTER_RATIO;
                    previous_value_splitter_ratio = ElementScannerResultsViewData::DEFAULT_PREVIOUS_VALUE_SPLITTER_RATIO;

                    new_value_splitter_ratio = Some(value_splitter_ratio);
                    new_previous_value_splitter_ratio = Some(previous_value_splitter_ratio);
                }

                let value_splitter_position_x = content_min_x + content_width * value_splitter_ratio;
                let previous_value_splitter_position_x = content_min_x + content_width * previous_value_splitter_ratio;

                // Faux address splitter.
                let faux_address_splitter_position_x = content_min_x + 36.0;

                ScrollArea::vertical()
                    .id_salt("element_scanner")
                    .auto_shrink([false, false])
                    .show(&mut user_interface, |user_interface| {
                        // Draw rows, capture min/max Y.
                        for index in 0..element_scanner_results_view_data.current_scan_results.len() {
                            let scan_result = &element_scanner_results_view_data.current_scan_results[index];
                            let row_response = user_interface.add(ElementScannerResultEntryView::new(
                                self.app_context.clone(),
                                scan_result,
                                faux_address_splitter_position_x,
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
                    });

                let splitter_min_y = header_rectangle.min.y;
                let splitter_max_y = content_clip_rectangle.max.y;

                let faux_address_splitter_rectangle = Rect::from_min_max(
                    pos2(faux_address_splitter_position_x - FAUX_BAR_THICKNESS * 0.5, splitter_min_y),
                    pos2(faux_address_splitter_position_x + FAUX_BAR_THICKNESS * 0.5, splitter_max_y),
                );

                let value_splitter_rectangle = Rect::from_min_max(
                    pos2(value_splitter_position_x - BAR_THICKNESS * 0.5, splitter_min_y),
                    pos2(value_splitter_position_x + BAR_THICKNESS * 0.5, splitter_max_y),
                );

                let previous_value_splitter_rectangle = Rect::from_min_max(
                    pos2(previous_value_splitter_position_x - BAR_THICKNESS * 0.5, splitter_min_y),
                    pos2(previous_value_splitter_position_x + BAR_THICKNESS * 0.5, splitter_max_y),
                );

                // Freeze column header.
                let freeze_icon_size = vec2(16.0, 16.0);
                let freeze_icon_padding = 8.0;
                let freeze_icon_pos_y = header_rectangle.center().y - freeze_icon_size.y * 0.5;
                let freeze_icon_rectangle = Rect::from_min_size(pos2(header_rectangle.min.x + freeze_icon_padding, freeze_icon_pos_y), freeze_icon_size);

                IconDraw::draw_sized(
                    user_interface,
                    freeze_icon_rectangle.center(),
                    freeze_icon_size,
                    &self.app_context.theme.icon_library.icon_handle_results_freeze,
                );

                // Address column header.
                let text_left_padding = 8.0;
                let address_header_x = faux_address_splitter_position_x + text_left_padding;
                let address_header_position = pos2(address_header_x, header_rectangle.center().y);

                user_interface.painter().text(
                    address_header_position,
                    Align2::LEFT_CENTER,
                    "Address",
                    theme.font_library.font_noto_sans.font_header.clone(),
                    theme.foreground,
                );

                // Value column header.
                let value_label_position = pos2(value_splitter_position_x + text_left_padding, header_rectangle.center().y);

                user_interface.painter().text(
                    value_label_position,
                    Align2::LEFT_CENTER,
                    "Value",
                    theme.font_library.font_noto_sans.font_header.clone(),
                    theme.foreground,
                );

                // Previous value column header.
                let previous_value_label_position = pos2(previous_value_splitter_position_x + text_left_padding, header_rectangle.center().y);

                user_interface.painter().text(
                    previous_value_label_position,
                    Align2::LEFT_CENTER,
                    "Previous Value",
                    theme.font_library.font_noto_sans.font_header.clone(),
                    theme.foreground,
                );

                // Faux address splitter.
                user_interface
                    .painter()
                    .rect_filled(faux_address_splitter_rectangle, 0.0, theme.background_control);

                // Value splitter.
                let value_splitter_response =
                    allocate_resize_bar(&mut user_interface, value_splitter_rectangle, "value_splitter").on_hover_cursor(CursorIcon::ResizeHorizontal);

                if value_splitter_response.dragged() {
                    let drag_delta = value_splitter_response.drag_delta();
                    let mut new_value_splitter_position_x = value_splitter_position_x + drag_delta.x;

                    let minimum_value_splitter_position_x = content_min_x + MINIMUM_COLUMN_PIXEL_WIDTH;
                    let maximum_value_splitter_position_x = previous_value_splitter_position_x - MINIMUM_SPLITTER_PIXEL_GAP;

                    new_value_splitter_position_x = new_value_splitter_position_x.clamp(minimum_value_splitter_position_x, maximum_value_splitter_position_x);

                    let bounded_value_splitter_ratio = (new_value_splitter_position_x - content_min_x) / content_width;

                    new_value_splitter_ratio = Some(bounded_value_splitter_ratio);
                }

                // Previous value splitter.
                let previous_value_splitter_response = allocate_resize_bar(&mut user_interface, previous_value_splitter_rectangle, "previous_value_splitter")
                    .on_hover_cursor(CursorIcon::ResizeHorizontal);

                if previous_value_splitter_response.dragged() {
                    let drag_delta = previous_value_splitter_response.drag_delta();
                    let mut new_previous_value_splitter_position_x = previous_value_splitter_position_x + drag_delta.x;

                    let minimum_previous_value_splitter_position_x = value_splitter_position_x + MINIMUM_SPLITTER_PIXEL_GAP;
                    let maximum_previous_value_splitter_position_x = content_min_x + content_width - MINIMUM_COLUMN_PIXEL_WIDTH;

                    new_previous_value_splitter_position_x =
                        new_previous_value_splitter_position_x.clamp(minimum_previous_value_splitter_position_x, maximum_previous_value_splitter_position_x);

                    let bounded_previous_value_splitter_ratio = (new_previous_value_splitter_position_x - content_min_x) / content_width;

                    new_previous_value_splitter_ratio = Some(bounded_previous_value_splitter_ratio);
                }
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
