use crate::{
    app_context::AppContext,
    ui::{
        draw::icon_draw::IconDraw,
        geometry::safe_clamp_f32,
        widgets::controls::{
            combo_box::combo_box_view::ComboBoxView, data_type_selector::data_type_selector_view::DataTypeSelectorView,
            data_value_box::data_value_box_convert_item_view::DataValueBoxConvertItemView,
        },
    },
    views::{
        element_scanner::{
            results::{
                element_scanner_result_entry_view::ElementScannerResultEntryView,
                view_data::{
                    element_scanner_result_frame_action::ElementScannerResultFrameAction, element_scanner_results_view_data::ElementScannerResultsViewData,
                },
            },
            scanner::{element_scanner_view_state::ElementScannerViewState, view_data::element_scanner_view_data::ElementScannerViewData},
        },
        project_explorer::project_hierarchy::view_data::project_hierarchy_view_data::ProjectHierarchyViewData,
        struct_viewer::view_data::struct_viewer_view_data::StructViewerViewData,
    },
};
use eframe::egui::{Align, Align2, CursorIcon, Direction, Layout, Response, ScrollArea, Sense, Spinner, Ui, Widget};
use epaint::{Margin, Rect, Vec2, pos2, vec2};
use squalr_engine_api::{
    dependency_injection::dependency::Dependency,
    structures::{
        data_types::data_type_ref::DataTypeRef,
        data_values::{anonymous_value_string::AnonymousValueString, anonymous_value_string_format::AnonymousValueStringFormat, container_type::ContainerType},
    },
};
use std::sync::Arc;

#[derive(Clone)]
pub struct ElementScannerResultsView {
    app_context: Arc<AppContext>,
    element_scanner_view_data: Dependency<ElementScannerViewData>,
    element_scanner_results_view_data: Dependency<ElementScannerResultsViewData>,
    project_hierarchy_view_data: Dependency<ProjectHierarchyViewData>,
    struct_viewer_view_data: Dependency<StructViewerViewData>,
}

impl ElementScannerResultsView {
    const DISPLAY_TYPE_SELECTOR_BUTTON_WIDTH: f32 = 32.0;
    const DISPLAY_TYPE_SELECTOR_POPUP_WIDTH: f32 = 176.0;
    pub const WINDOW_ID: &'static str = "window_element_scanner_results";

    pub fn new(app_context: Arc<AppContext>) -> Self {
        let element_scanner_view_data = app_context
            .dependency_container
            .register(ElementScannerViewData::new());
        let element_scanner_results_view_data = app_context
            .dependency_container
            .register(ElementScannerResultsViewData::new());
        let project_hierarchy_view_data = app_context
            .dependency_container
            .get_dependency::<ProjectHierarchyViewData>();
        let struct_viewer_view_data = app_context
            .dependency_container
            .get_dependency::<StructViewerViewData>();

        ElementScannerResultsViewData::sync_data_type_filters_from_scan_selection(element_scanner_results_view_data.clone(), element_scanner_view_data.clone());
        ElementScannerResultsViewData::poll_scan_results(
            element_scanner_results_view_data.clone(),
            element_scanner_view_data.clone(),
            app_context.engine_unprivileged_state.clone(),
        );

        Self {
            app_context,
            element_scanner_view_data,
            element_scanner_results_view_data,
            project_hierarchy_view_data,
            struct_viewer_view_data,
        }
    }

    fn display_format_sort_key(anonymous_value_string_format: AnonymousValueStringFormat) -> u8 {
        match anonymous_value_string_format {
            AnonymousValueStringFormat::Bool => 0,
            AnonymousValueStringFormat::String => 1,
            AnonymousValueStringFormat::Binary => 2,
            AnonymousValueStringFormat::Decimal => 3,
            AnonymousValueStringFormat::Hexadecimal => 4,
            AnonymousValueStringFormat::Address => 5,
            AnonymousValueStringFormat::DataTypeRef => 6,
            AnonymousValueStringFormat::Enumeration => 7,
        }
    }

    fn normalize_display_formats(supported_display_formats: &mut Vec<AnonymousValueStringFormat>) {
        supported_display_formats.sort_by_key(|anonymous_value_string_format| Self::display_format_sort_key(*anonymous_value_string_format));
        supported_display_formats.dedup();
    }

    fn resolve_display_type_selector_rectangle(
        value_splitter_position_x: f32,
        previous_value_splitter_position_x: f32,
        header_center_y: f32,
        horizontal_padding: f32,
    ) -> Rect {
        let selector_max_x = previous_value_splitter_position_x - horizontal_padding;
        let selector_min_x = (selector_max_x - Self::DISPLAY_TYPE_SELECTOR_BUTTON_WIDTH).max(value_splitter_position_x + horizontal_padding);

        Rect::from_min_max(pos2(selector_min_x, header_center_y - 12.0), pos2(selector_max_x, header_center_y + 12.0))
    }

    fn resolve_value_header_clip_rectangle(
        value_splitter_position_x: f32,
        header_rectangle: Rect,
        display_type_selector_rectangle: Rect,
        text_left_padding: f32,
    ) -> Rect {
        Rect::from_min_max(
            pos2(value_splitter_position_x + text_left_padding, header_rectangle.min.y),
            pos2(
                (display_type_selector_rectangle.min.x - text_left_padding).max(value_splitter_position_x + text_left_padding),
                header_rectangle.max.y,
            ),
        )
    }

    fn display_format_icon(
        &self,
        anonymous_value_string_format: AnonymousValueStringFormat,
    ) -> eframe::egui::TextureHandle {
        let icon_library = &self.app_context.theme.icon_library;

        match anonymous_value_string_format {
            AnonymousValueStringFormat::Binary => icon_library.icon_handle_display_type_binary.clone(),
            AnonymousValueStringFormat::Decimal => icon_library.icon_handle_display_type_decimal.clone(),
            AnonymousValueStringFormat::Hexadecimal | AnonymousValueStringFormat::Address => icon_library.icon_handle_display_type_hexadecimal.clone(),
            AnonymousValueStringFormat::String
            | AnonymousValueStringFormat::Bool
            | AnonymousValueStringFormat::DataTypeRef
            | AnonymousValueStringFormat::Enumeration => icon_library.icon_handle_display_type_string.clone(),
        }
    }

    fn resolve_supported_display_formats(
        &self,
        selected_data_types: &[DataTypeRef],
        fallback_available_data_types: &[DataTypeRef],
        fallback_active_display_format: AnonymousValueStringFormat,
    ) -> Vec<AnonymousValueStringFormat> {
        let candidate_data_types = if selected_data_types.is_empty() {
            fallback_available_data_types
        } else {
            selected_data_types
        };

        if candidate_data_types.is_empty() {
            return vec![fallback_active_display_format];
        }

        let mut shared_supported_display_formats: Option<Vec<AnonymousValueStringFormat>> = None;

        for data_type_ref in candidate_data_types {
            let supported_display_formats = self
                .app_context
                .engine_unprivileged_state
                .get_supported_anonymous_value_string_formats(data_type_ref)
                .into_iter()
                .collect::<Vec<_>>();

            if let Some(shared_supported_display_formats) = shared_supported_display_formats.as_mut() {
                shared_supported_display_formats.retain(|anonymous_value_string_format| supported_display_formats.contains(anonymous_value_string_format));
            } else {
                shared_supported_display_formats = Some(supported_display_formats);
            }
        }

        let mut shared_supported_display_formats = shared_supported_display_formats.unwrap_or_else(|| vec![fallback_active_display_format]);

        if shared_supported_display_formats.is_empty() {
            shared_supported_display_formats.push(fallback_active_display_format);
        }

        Self::normalize_display_formats(&mut shared_supported_display_formats);

        shared_supported_display_formats
    }
}
impl Widget for ElementScannerResultsView {
    fn ui(
        self,
        user_interface: &mut Ui,
    ) -> Response {
        let results_read_interval = ElementScannerResultsViewData::get_results_read_interval(self.element_scanner_results_view_data.clone());
        user_interface
            .ctx()
            .request_repaint_after(results_read_interval);

        const FAUX_BAR_THICKNESS: f32 = 3.0;
        const BAR_THICKNESS: f32 = 4.0;
        const MINIMUM_COLUMN_PIXEL_WIDTH: f32 = 80.0;
        const MINIMUM_SPLITTER_PIXEL_GAP: f32 = 40.0;
        const DATA_TYPE_COLUMN_PIXEL_WIDTH: f32 = 80.0;

        let theme = &self.app_context.theme;
        let mut new_value_splitter_ratio: Option<f32> = None;
        let mut new_previous_value_splitter_ratio: Option<f32> = None;
        let mut did_change_data_type_filters = false;
        let mut element_sanner_result_frame_action: ElementScannerResultFrameAction = ElementScannerResultFrameAction::None;
        let mut scan_results_has_keyboard_focus = false;
        let bounded_results_rectangle = user_interface
            .available_rect_before_wrap()
            .intersect(user_interface.clip_rect());
        let results_response = user_interface.allocate_rect(bounded_results_rectangle, Sense::click());
        let mut results_user_interface = user_interface.new_child(
            eframe::egui::UiBuilder::new()
                .max_rect(results_response.rect)
                .layout(Layout::top_down(Align::Min)),
        );
        results_user_interface.set_clip_rect(results_response.rect);

        results_user_interface.allocate_ui_with_layout(results_user_interface.available_size(), Layout::top_down(Align::Min), |mut user_interface| {
            let allocate_resize_bar = |user_interface: &mut Ui, resize_rectangle: Rect, id_suffix: &str| -> Response {
                let id = user_interface.id().with(id_suffix);
                let response = user_interface.interact(resize_rectangle, id, Sense::drag());

                user_interface
                    .painter()
                    .rect_filled(resize_rectangle, 0.0, theme.background_control);

                response
            };

            let (mut value_splitter_ratio, mut previous_value_splitter_ratio) = match self
                .element_scanner_results_view_data
                .read("Element scanner results view")
            {
                Some(element_scanner_results_view_data) => (
                    element_scanner_results_view_data.value_splitter_ratio,
                    element_scanner_results_view_data.previous_value_splitter_ratio,
                ),
                None => return,
            };

            // Draw the header.
            let header_height = 32.0;
            let (header_rectangle, _header_response) =
                user_interface.allocate_exact_size(vec2(user_interface.available_size().x.max(1.0), header_height), Sense::empty());
            let (separator_rect, _) = user_interface.allocate_exact_size(vec2(user_interface.available_size().x.max(1.0), FAUX_BAR_THICKNESS), Sense::empty());

            user_interface
                .painter()
                .rect_filled(separator_rect, 0.0, theme.background_control);

            let content_clip_rectangle = user_interface.available_rect_before_wrap();

            let content_width = content_clip_rectangle.width();
            let content_height = content_clip_rectangle.height();
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
            let faux_data_type_splitter_position_x = content_min_x + 36.0;
            let faux_address_splitter_position_x = faux_data_type_splitter_position_x + DATA_TYPE_COLUMN_PIXEL_WIDTH;

            let splitter_min_y = header_rectangle.min.y;
            let splitter_max_y = content_clip_rectangle.max.y;

            let faux_data_type_splitter_rectangle = Rect::from_min_max(
                pos2(faux_data_type_splitter_position_x - FAUX_BAR_THICKNESS * 0.5, splitter_min_y),
                pos2(faux_data_type_splitter_position_x + FAUX_BAR_THICKNESS * 0.5, splitter_max_y),
            );

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

            let data_type_filter_combo_padding = 4.0;
            let data_type_filter_combo_rectangle = Rect::from_min_max(
                pos2(
                    faux_data_type_splitter_position_x + data_type_filter_combo_padding,
                    header_rectangle.center().y - 12.0,
                ),
                pos2(
                    faux_address_splitter_position_x - data_type_filter_combo_padding,
                    header_rectangle.center().y + 12.0,
                ),
            );

            if let Some(mut element_scanner_results_view_data) = self
                .element_scanner_results_view_data
                .write("Element scanner results header data type filters")
            {
                let previous_data_type_filter_selection = element_scanner_results_view_data
                    .data_type_filter_selection
                    .clone();
                let available_data_types = element_scanner_results_view_data.available_data_types.clone();

                user_interface.put(
                    data_type_filter_combo_rectangle,
                    DataTypeSelectorView::new(
                        self.app_context.clone(),
                        &mut element_scanner_results_view_data.data_type_filter_selection,
                        "element_scanner_results_data_type_filters",
                    )
                    .width(data_type_filter_combo_rectangle.width())
                    .height(data_type_filter_combo_rectangle.height())
                    .available_data_types(available_data_types)
                    .stacked_list()
                    .icon_only_label(),
                );

                did_change_data_type_filters = element_scanner_results_view_data.data_type_filter_selection != previous_data_type_filter_selection;
            }

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

            let display_type_selector_rectangle = Self::resolve_display_type_selector_rectangle(
                value_splitter_position_x,
                previous_value_splitter_position_x,
                header_rectangle.center().y,
                data_type_filter_combo_padding,
            );
            let value_header_position = pos2(value_splitter_position_x + text_left_padding, header_rectangle.center().y);
            let value_header_clip_rectangle =
                Self::resolve_value_header_clip_rectangle(value_splitter_position_x, header_rectangle, display_type_selector_rectangle, text_left_padding);

            user_interface
                .painter()
                .with_clip_rect(value_header_clip_rectangle)
                .text(
                    value_header_position,
                    Align2::LEFT_CENTER,
                    "Value",
                    theme.font_library.font_noto_sans.font_header.clone(),
                    theme.foreground,
                );

            if let Some(mut element_scanner_results_view_data) = self
                .element_scanner_results_view_data
                .write("Element scanner results header display type")
            {
                let supported_display_formats = self.resolve_supported_display_formats(
                    element_scanner_results_view_data
                        .data_type_filter_selection
                        .selected_data_types(),
                    &element_scanner_results_view_data.available_data_types,
                    element_scanner_results_view_data.active_display_format,
                );
                let resolved_active_display_format = if supported_display_formats.contains(&element_scanner_results_view_data.active_display_format) {
                    element_scanner_results_view_data.active_display_format
                } else {
                    supported_display_formats
                        .first()
                        .copied()
                        .unwrap_or(element_scanner_results_view_data.active_display_format)
                };

                element_scanner_results_view_data.active_display_format = resolved_active_display_format;
                element_scanner_results_view_data
                    .current_display_string
                    .set_anonymous_value_string_format(resolved_active_display_format);

                let mut header_display_format_value =
                    AnonymousValueString::new(String::new(), element_scanner_results_view_data.active_display_format, ContainerType::None);

                user_interface.put(
                    display_type_selector_rectangle,
                    ComboBoxView::new(
                        self.app_context.clone(),
                        String::new(),
                        "element_scanner_results_display_type",
                        Some(self.display_format_icon(element_scanner_results_view_data.active_display_format)),
                        |popup_user_interface, should_close| {
                            for anonymous_value_string_format in &supported_display_formats {
                                if popup_user_interface
                                    .add(
                                        DataValueBoxConvertItemView::new(
                                            self.app_context.clone(),
                                            &mut header_display_format_value,
                                            anonymous_value_string_format,
                                            None,
                                            false,
                                            false,
                                            Self::DISPLAY_TYPE_SELECTOR_POPUP_WIDTH,
                                        )
                                        .width(Self::DISPLAY_TYPE_SELECTOR_POPUP_WIDTH),
                                    )
                                    .clicked()
                                {
                                    *should_close = true;
                                }
                            }
                        },
                    )
                    .width(display_type_selector_rectangle.width())
                    .height(display_type_selector_rectangle.height())
                    .show_dropdown_arrow(false),
                );

                let active_display_format = header_display_format_value.get_anonymous_value_string_format();
                element_scanner_results_view_data.active_display_format = active_display_format;
                element_scanner_results_view_data
                    .current_display_string
                    .set_anonymous_value_string_format(active_display_format);
            }

            // Previous value column header.
            let previous_value_label_position = pos2(previous_value_splitter_position_x + text_left_padding, header_rectangle.center().y);

            user_interface.painter().text(
                previous_value_label_position,
                Align2::LEFT_CENTER,
                "Previous Value",
                theme.font_library.font_noto_sans.font_header.clone(),
                theme.foreground,
            );

            // Result entries.
            ScrollArea::vertical()
                .id_salt("element_scanner_result_entries")
                .max_height(content_height)
                .auto_shrink([false, false])
                .show(&mut user_interface, |user_interface| {
                    let element_scanner_results_view_data = match self
                        .element_scanner_results_view_data
                        .read("Element scanner results view element scanner results view data")
                    {
                        Some(element_scanner_results_view_data) => element_scanner_results_view_data,
                        None => return,
                    };
                    let element_scanner_view_data = match self
                        .element_scanner_view_data
                        .read("Element scanner results view element scanner view data")
                    {
                        Some(element_scanner_view_data) => element_scanner_view_data,
                        None => return,
                    };

                    user_interface.spacing_mut().menu_margin = Margin::ZERO;
                    user_interface.spacing_mut().window_margin = Margin::ZERO;
                    user_interface.spacing_mut().menu_spacing = 0.0;
                    user_interface.spacing_mut().item_spacing = Vec2::ZERO;

                    if element_scanner_view_data.view_state == ElementScannerViewState::ScanInProgress
                        || element_scanner_results_view_data.is_querying_scan_results
                    {
                        user_interface.allocate_ui_with_layout(
                            vec2(user_interface.available_width(), 32.0),
                            Layout::centered_and_justified(Direction::LeftToRight),
                            |user_interface| {
                                user_interface.add(Spinner::new().color(theme.foreground));
                            },
                        );

                        return;
                    }

                    user_interface.with_layout(Layout::top_down(Align::Min), |user_interface| {
                        // Draw rows, capture min/max Y.
                        for index in 0..element_scanner_results_view_data.current_scan_results.len() {
                            let is_selected = {
                                match (
                                    element_scanner_results_view_data.selection_index_start,
                                    element_scanner_results_view_data.selection_index_end,
                                ) {
                                    (Some(start), Some(end)) => {
                                        let (min_index, max_index) = if start <= end { (start, end) } else { (end, start) };
                                        index as i32 >= min_index && index as i32 <= max_index
                                    }
                                    (Some(start), None) => index as i32 == start,
                                    (None, Some(end)) => index as i32 == end,
                                    (None, None) => false,
                                }
                            };

                            let scan_result = &element_scanner_results_view_data.current_scan_results[index];

                            let entry_widget = ElementScannerResultEntryView::new(
                                self.app_context.clone(),
                                &scan_result,
                                element_scanner_results_view_data.active_display_format,
                                index,
                                is_selected,
                                &mut element_sanner_result_frame_action,
                                faux_data_type_splitter_position_x,
                                faux_address_splitter_position_x,
                                value_splitter_position_x,
                                previous_value_splitter_position_x,
                            );
                            let row_response = user_interface.add(entry_widget);

                            if row_response.clicked() || row_response.double_clicked() {
                                row_response.request_focus();
                            }

                            scan_results_has_keyboard_focus |= row_response.has_focus();

                            if rows_min_y.is_none() {
                                rows_min_y = Some(row_response.rect.min.y);
                            }

                            rows_max_y = Some(row_response.rect.max.y);

                            if row_response.double_clicked() {
                                element_sanner_result_frame_action = ElementScannerResultFrameAction::AddScanResult(index as i32);
                            }
                        }
                    });
                });

            user_interface
                .painter()
                .rect_filled(faux_data_type_splitter_rectangle, 0.0, theme.background_control);

            // Faux address splitter.
            user_interface
                .painter()
                .rect_filled(faux_address_splitter_rectangle, 0.0, theme.background_control);

            // Value splitter.
            let value_splitter_response =
                allocate_resize_bar(&mut user_interface, value_splitter_rectangle, "value_splitter").on_hover_cursor(CursorIcon::ResizeHorizontal);

            if value_splitter_response.dragged() {
                let drag_delta = value_splitter_response.drag_delta();
                let minimum_value_splitter_position_x = faux_address_splitter_position_x + MINIMUM_COLUMN_PIXEL_WIDTH;
                let maximum_previous_value_splitter_position_x = content_min_x + content_width - MINIMUM_COLUMN_PIXEL_WIDTH;
                let minimum_drag_delta_x = minimum_value_splitter_position_x - value_splitter_position_x;
                let maximum_drag_delta_x = maximum_previous_value_splitter_position_x - previous_value_splitter_position_x;
                let bounded_drag_delta_x = safe_clamp_f32(drag_delta.x, minimum_drag_delta_x, maximum_drag_delta_x);
                let new_value_splitter_position_x = value_splitter_position_x + bounded_drag_delta_x;
                let new_previous_value_splitter_position_x = previous_value_splitter_position_x + bounded_drag_delta_x;

                new_value_splitter_ratio = Some((new_value_splitter_position_x - content_min_x) / content_width);
                new_previous_value_splitter_ratio = Some((new_previous_value_splitter_position_x - content_min_x) / content_width);
            }

            // Previous value splitter.
            let previous_value_splitter_response = allocate_resize_bar(&mut user_interface, previous_value_splitter_rectangle, "previous_value_splitter")
                .on_hover_cursor(CursorIcon::ResizeHorizontal);

            if previous_value_splitter_response.dragged() {
                let drag_delta = previous_value_splitter_response.drag_delta();
                let mut new_previous_value_splitter_position_x = previous_value_splitter_position_x + drag_delta.x;

                let minimum_previous_value_splitter_position_x = value_splitter_position_x + MINIMUM_SPLITTER_PIXEL_GAP;
                let maximum_previous_value_splitter_position_x = content_min_x + content_width - MINIMUM_COLUMN_PIXEL_WIDTH;

                new_previous_value_splitter_position_x = safe_clamp_f32(
                    new_previous_value_splitter_position_x,
                    minimum_previous_value_splitter_position_x,
                    maximum_previous_value_splitter_position_x,
                );

                let bounded_previous_value_splitter_ratio = (new_previous_value_splitter_position_x - content_min_x) / content_width;

                new_previous_value_splitter_ratio = Some(bounded_previous_value_splitter_ratio);
            }
        });

        if new_value_splitter_ratio.is_some() || new_previous_value_splitter_ratio.is_some() {
            if let Some(mut element_scanner_results_view_data) = self
                .element_scanner_results_view_data
                .write("Element scanner results view")
            {
                if let Some(new_value_splitter_ratio) = new_value_splitter_ratio {
                    element_scanner_results_view_data.value_splitter_ratio = new_value_splitter_ratio;
                }

                if let Some(new_previous_value_splitter_ratio) = new_previous_value_splitter_ratio {
                    element_scanner_results_view_data.previous_value_splitter_ratio = new_previous_value_splitter_ratio;
                }
            }
        }

        if did_change_data_type_filters {
            ElementScannerResultsViewData::query_scan_results_for_active_data_type_filters(
                self.element_scanner_results_view_data.clone(),
                self.app_context.engine_unprivileged_state.clone(),
            );
        }

        if scan_results_has_keyboard_focus && user_interface.input(|input_state| input_state.key_pressed(eframe::egui::Key::Space)) {
            element_sanner_result_frame_action = ElementScannerResultFrameAction::from_selection_freeze_checkstate(
                ElementScannerResultsViewData::get_selection_freeze_checkstate(self.element_scanner_results_view_data.clone()),
            );
        }

        if element_sanner_result_frame_action != ElementScannerResultFrameAction::None {
            match element_sanner_result_frame_action {
                ElementScannerResultFrameAction::None => {}
                ElementScannerResultFrameAction::SetSelectionStart(index) => {
                    ElementScannerResultsViewData::set_scan_result_selection_start(
                        self.element_scanner_results_view_data.clone(),
                        self.struct_viewer_view_data.clone(),
                        self.app_context.engine_unprivileged_state.clone(),
                        index,
                    );
                }
                ElementScannerResultFrameAction::SetSelectionEnd(index) => {
                    ElementScannerResultsViewData::set_scan_result_selection_end(
                        self.element_scanner_results_view_data.clone(),
                        self.struct_viewer_view_data.clone(),
                        self.app_context.engine_unprivileged_state.clone(),
                        index,
                    );
                }
                ElementScannerResultFrameAction::FreezeIndex(index, is_frozen) => {
                    ElementScannerResultsViewData::set_scan_result_frozen(
                        self.element_scanner_results_view_data.clone(),
                        self.app_context.engine_unprivileged_state.clone(),
                        index,
                        is_frozen,
                    );
                }
                ElementScannerResultFrameAction::AddScanResult(index) => {
                    let target_directory_path = ProjectHierarchyViewData::get_selected_directory_path(self.project_hierarchy_view_data.clone());
                    ElementScannerResultsViewData::add_scan_result_to_project_by_index(
                        self.element_scanner_results_view_data.clone(),
                        self.app_context.engine_unprivileged_state.clone(),
                        index,
                        target_directory_path,
                    );
                }
                ElementScannerResultFrameAction::ToggleFreezeSelection(_) => {}
                ElementScannerResultFrameAction::AddSelection => {}
                ElementScannerResultFrameAction::DeleteSelection => {}
                ElementScannerResultFrameAction::CommitValueToSelection(_) => {}
            }
        }

        results_response
    }
}

#[cfg(test)]
mod tests {
    use super::ElementScannerResultsView;
    use epaint::{Rect, pos2};

    #[test]
    fn display_type_selector_stays_right_aligned_in_value_header() {
        let display_type_selector_rectangle = ElementScannerResultsView::resolve_display_type_selector_rectangle(120.0, 320.0, 40.0, 4.0);

        assert_eq!(display_type_selector_rectangle.min.x, 284.0);
        assert_eq!(display_type_selector_rectangle.max.x, 316.0);
        assert_eq!(display_type_selector_rectangle.height(), 24.0);
    }

    #[test]
    fn value_header_clip_stops_before_display_type_selector() {
        let header_rectangle = Rect::from_min_max(pos2(0.0, 24.0), pos2(400.0, 56.0));
        let display_type_selector_rectangle = ElementScannerResultsView::resolve_display_type_selector_rectangle(120.0, 320.0, 40.0, 4.0);
        let value_header_clip_rectangle =
            ElementScannerResultsView::resolve_value_header_clip_rectangle(120.0, header_rectangle, display_type_selector_rectangle, 8.0);

        assert_eq!(value_header_clip_rectangle.min.x, 128.0);
        assert_eq!(value_header_clip_rectangle.max.x, 276.0);
        assert_eq!(value_header_clip_rectangle.min.y, 24.0);
        assert_eq!(value_header_clip_rectangle.max.y, 56.0);
    }
}
