use crate::{
    app_context::AppContext,
    ui::{draw::icon_draw::IconDraw, widgets::controls::check_state::CheckState},
    views::{
        element_scanner::{
            results::{
                element_scanner_result_entry_view::ElementScannerResultEntryView,
                element_scanner_results_action_bar_view::ElementScannerResultsActionBarView,
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
use squalr_engine_api::{dependency_injection::dependency::Dependency, structures::scan_results::scan_result::ScanResult};
use std::sync::Arc;
use std::time::Duration;

#[derive(Clone)]
pub struct ElementScannerResultsView {
    app_context: Arc<AppContext>,
    element_scanner_view_data: Dependency<ElementScannerViewData>,
    element_scanner_results_view_data: Dependency<ElementScannerResultsViewData>,
    project_hierarchy_view_data: Dependency<ProjectHierarchyViewData>,
    struct_viewer_view_data: Dependency<StructViewerViewData>,
}

impl ElementScannerResultsView {
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

        ElementScannerResultsViewData::poll_scan_results(element_scanner_results_view_data.clone(), app_context.engine_unprivileged_state.clone());

        Self {
            app_context,
            element_scanner_view_data,
            element_scanner_results_view_data,
            project_hierarchy_view_data,
            struct_viewer_view_data,
        }
    }
}
impl Widget for ElementScannerResultsView {
    fn ui(
        self,
        user_interface: &mut Ui,
    ) -> Response {
        user_interface
            .ctx()
            .request_repaint_after(Duration::from_millis(100));

        const FAUX_BAR_THICKNESS: f32 = 3.0;
        const BAR_THICKNESS: f32 = 4.0;
        const MINIMUM_COLUMN_PIXEL_WIDTH: f32 = 80.0;
        const MINIMUM_SPLITTER_PIXEL_GAP: f32 = 40.0;

        let theme = &self.app_context.theme;
        let mut new_value_splitter_ratio: Option<f32> = None;
        let mut new_previous_value_splitter_ratio: Option<f32> = None;
        let mut element_sanner_result_frame_action: ElementScannerResultFrameAction = ElementScannerResultFrameAction::None;

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
                    user_interface.allocate_exact_size(vec2(user_interface.available_size().x, header_height), Sense::empty());
                let (separator_rect, _) = user_interface.allocate_exact_size(vec2(user_interface.available_size().x, FAUX_BAR_THICKNESS), Sense::empty());

                user_interface
                    .painter()
                    .rect_filled(separator_rect, 0.0, theme.background_control);

                let footer_height = ElementScannerResultsActionBarView::FOOTER_HEIGHT;
                let content_clip_rectangle = user_interface
                    .available_rect_before_wrap()
                    .with_max_y(user_interface.available_rect_before_wrap().max.y - footer_height);

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
                let faux_address_splitter_position_x = content_min_x + 36.0;

                let splitter_min_y = header_rectangle.min.y;
                let splitter_max_y = content_clip_rectangle.max.y + footer_height;

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

                // Assume all false.
                let mut selection_freeze_checkstate = CheckState::False;

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

                                // Update the cumulative check state based on whether this scan result is frozen.
                                if is_selected {
                                    match selection_freeze_checkstate {
                                        CheckState::False => {
                                            if scan_result.get_is_frozen() {
                                                selection_freeze_checkstate = CheckState::True;
                                            }
                                        }
                                        CheckState::True => {
                                            if !scan_result.get_is_frozen() {
                                                selection_freeze_checkstate = CheckState::Mixed;
                                            }
                                        }
                                        CheckState::Mixed => {}
                                    }
                                }

                                let entry_widget = ElementScannerResultEntryView::new(
                                    self.app_context.clone(),
                                    &scan_result,
                                    element_scanner_view_data.active_display_format,
                                    index,
                                    is_selected,
                                    &mut element_sanner_result_frame_action,
                                    faux_address_splitter_position_x,
                                    value_splitter_position_x,
                                    previous_value_splitter_position_x,
                                );
                                let row_response = user_interface.add(entry_widget);

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

                // Draw the footer.
                user_interface.add(ElementScannerResultsActionBarView::new(
                    self.app_context.clone(),
                    selection_freeze_checkstate,
                    &mut element_sanner_result_frame_action,
                    faux_address_splitter_position_x,
                    value_splitter_position_x,
                    previous_value_splitter_position_x,
                ));

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
                ElementScannerResultFrameAction::ToggleFreezeSelection(is_frozen) => {
                    ElementScannerResultsViewData::toggle_selected_scan_results_frozen(
                        self.element_scanner_results_view_data.clone(),
                        self.app_context.engine_unprivileged_state.clone(),
                        is_frozen,
                    );
                }
                ElementScannerResultFrameAction::AddSelection => {
                    let target_directory_path = ProjectHierarchyViewData::get_selected_directory_path(self.project_hierarchy_view_data.clone());
                    ElementScannerResultsViewData::add_scan_results_to_project(
                        self.element_scanner_results_view_data.clone(),
                        self.app_context.engine_unprivileged_state.clone(),
                        target_directory_path,
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
                ElementScannerResultFrameAction::DeleteSelection => {
                    ElementScannerResultsViewData::delete_selected_scan_results(
                        self.element_scanner_results_view_data.clone(),
                        self.app_context.engine_unprivileged_state.clone(),
                    );
                }
                ElementScannerResultFrameAction::CommitValueToSelection(edit_value) => {
                    ElementScannerResultsViewData::set_selected_scan_results_value(
                        self.element_scanner_results_view_data.clone(),
                        self.app_context.engine_unprivileged_state.clone(),
                        ScanResult::PROPERTY_NAME_VALUE,
                        edit_value,
                    );
                }
            }
        }

        response
    }
}
