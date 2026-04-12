use crate::{
    app_context::AppContext,
    ui::{
        draw::icon_draw::IconDraw,
        widgets::controls::{button::Button, check_state::CheckState, checkbox::Checkbox, data_value_box::data_value_box_view::DataValueBoxView},
    },
    views::{
        element_scanner::results::view_data::element_scanner_results_view_data::ElementScannerResultsViewData,
        project_explorer::project_hierarchy::view_data::project_hierarchy_view_data::ProjectHierarchyViewData,
    },
};
use eframe::egui::{Response, Sense, Ui, UiBuilder, Widget, pos2, vec2};
use epaint::{Color32, CornerRadius, Rect, Stroke, StrokeKind};
use squalr_engine_api::{dependency_injection::dependency::Dependency, structures::scan_results::scan_result::ScanResult};
use std::sync::Arc;

#[derive(Clone)]
pub struct ElementScannerResultsActionBarView {
    app_context: Arc<AppContext>,
    element_scanner_results_view_data: Dependency<ElementScannerResultsViewData>,
    project_hierarchy_view_data: Dependency<ProjectHierarchyViewData>,
    selection_freeze_checkstate: CheckState,
}

impl ElementScannerResultsActionBarView {
    pub const FOOTER_HEIGHT: f32 = 32.0;
    const BUTTON_SIZE: [f32; 2] = [36.0, 28.0];
    const FAUX_BAR_THICKNESS: f32 = 3.0;
    const BAR_THICKNESS: f32 = 4.0;
    const FAUX_DATA_TYPE_SPLITTER_POSITION_X: f32 = 36.0;
    const DATA_TYPE_COLUMN_PIXEL_WIDTH: f32 = 80.0;
    const EDIT_VALUE_PADDING: f32 = 2.0;
    const DATA_TYPE_ACTION_BUTTON_PADDING: f32 = 4.0;

    pub fn new(
        app_context: Arc<AppContext>,
        selection_freeze_checkstate: CheckState,
    ) -> Self {
        let element_scanner_results_view_data = app_context
            .dependency_container
            .get_dependency::<ElementScannerResultsViewData>();
        let project_hierarchy_view_data = app_context
            .dependency_container
            .get_dependency::<ProjectHierarchyViewData>();

        Self {
            app_context,
            element_scanner_results_view_data,
            project_hierarchy_view_data,
            selection_freeze_checkstate,
        }
    }

    pub fn get_height(&self) -> f32 {
        Self::FOOTER_HEIGHT
    }

    fn resolve_splitter_positions(
        &self,
        allocated_size_rectangle: Rect,
    ) -> Option<(f32, f32, f32, f32)> {
        let element_scanner_results_view_data = self
            .element_scanner_results_view_data
            .read("Element scanner action bar splitter positions")?;
        let content_min_x = allocated_size_rectangle.min.x;
        let content_width = allocated_size_rectangle.width();

        if content_width <= 0.0 {
            return None;
        }

        let value_splitter_position_x = content_min_x + content_width * element_scanner_results_view_data.value_splitter_ratio;
        let previous_value_splitter_position_x = content_min_x + content_width * element_scanner_results_view_data.previous_value_splitter_ratio;
        let faux_data_type_splitter_position_x = content_min_x + Self::FAUX_DATA_TYPE_SPLITTER_POSITION_X;
        let faux_address_splitter_position_x = faux_data_type_splitter_position_x + Self::DATA_TYPE_COLUMN_PIXEL_WIDTH;

        Some((
            faux_data_type_splitter_position_x,
            faux_address_splitter_position_x,
            value_splitter_position_x,
            previous_value_splitter_position_x,
        ))
    }
}

impl Widget for ElementScannerResultsActionBarView {
    fn ui(
        self,
        user_interface: &mut Ui,
    ) -> Response {
        let theme = &self.app_context.theme;
        let (allocated_size_rectangle, response) =
            user_interface.allocate_exact_size(vec2(user_interface.available_width().max(1.0), self.get_height()), Sense::hover());
        let mut toolbar_user_interface = user_interface.new_child(
            UiBuilder::new()
                .max_rect(allocated_size_rectangle)
                .sense(Sense::hover()),
        );
        toolbar_user_interface.set_clip_rect(allocated_size_rectangle);

        user_interface
            .painter()
            .rect_filled(allocated_size_rectangle, CornerRadius::ZERO, theme.background_panel);
        user_interface.painter().rect_stroke(
            allocated_size_rectangle,
            CornerRadius::ZERO,
            Stroke::new(1.0, theme.submenu_border),
            StrokeKind::Inside,
        );

        let mut should_toggle_selection_frozen = false;
        let mut should_add_selection = false;
        let mut should_delete_selection = false;
        let mut should_commit_selection_value = false;
        let validation_data_type_ref = self
            .element_scanner_results_view_data
            .read("Element scanner action bar validation data type")
            .map(|element_scanner_results_view_data| {
                element_scanner_results_view_data
                    .data_type_filter_selection
                    .visible_data_type()
                    .clone()
            });

        let Some(validation_data_type_ref) = validation_data_type_ref else {
            return response;
        };
        let Some((faux_data_type_splitter_position_x, faux_address_splitter_position_x, value_splitter_position_x, previous_value_splitter_position_x)) =
            self.resolve_splitter_positions(allocated_size_rectangle)
        else {
            return response;
        };
        let button_size = vec2(Self::BUTTON_SIZE[0], Self::BUTTON_SIZE[1]);
        let checkbox_size = vec2(Checkbox::WIDTH, Checkbox::HEIGHT);
        let button_center_y = allocated_size_rectangle.center().y - button_size.y * 0.5;
        let checkbox_rect = Rect::from_center_size(
            pos2(
                (allocated_size_rectangle.min.x + faux_data_type_splitter_position_x) * 0.5,
                allocated_size_rectangle.center().y,
            ),
            checkbox_size,
        );
        let add_selection_button_rect = Rect::from_min_size(
            pos2(faux_data_type_splitter_position_x + Self::DATA_TYPE_ACTION_BUTTON_PADDING, button_center_y),
            button_size,
        );
        let delete_selection_button_rect = Rect::from_min_size(pos2(add_selection_button_rect.max.x, button_center_y), button_size);
        let edit_value_box_rect = Rect::from_min_max(
            pos2(value_splitter_position_x + Self::EDIT_VALUE_PADDING, allocated_size_rectangle.min.y),
            pos2(
                (previous_value_splitter_position_x - Self::EDIT_VALUE_PADDING).max(value_splitter_position_x + Self::EDIT_VALUE_PADDING),
                allocated_size_rectangle.max.y,
            ),
        );
        let commit_value_button_rect = Rect::from_min_size(pos2(previous_value_splitter_position_x, button_center_y), button_size);

        for (splitter_position_x, splitter_thickness) in [
            (faux_data_type_splitter_position_x, Self::FAUX_BAR_THICKNESS),
            (faux_address_splitter_position_x, Self::FAUX_BAR_THICKNESS),
            (value_splitter_position_x, Self::BAR_THICKNESS),
            (previous_value_splitter_position_x, Self::BAR_THICKNESS),
        ] {
            let splitter_rect = Rect::from_min_max(
                pos2(splitter_position_x - splitter_thickness * 0.5, allocated_size_rectangle.min.y),
                pos2(splitter_position_x + splitter_thickness * 0.5, allocated_size_rectangle.max.y),
            );
            toolbar_user_interface
                .painter()
                .rect_filled(splitter_rect, CornerRadius::ZERO, theme.background_control);
        }

        if toolbar_user_interface
            .put(
                checkbox_rect,
                Checkbox::new_from_theme(theme).with_check_state(self.selection_freeze_checkstate),
            )
            .clicked()
        {
            should_toggle_selection_frozen = true;
        }

        let add_selection_response = toolbar_user_interface.put(
            add_selection_button_rect,
            Button::new_from_theme(theme)
                .background_color(Color32::TRANSPARENT)
                .with_tooltip_text("Add selection to project."),
        );
        IconDraw::draw(&toolbar_user_interface, add_selection_response.rect, &theme.icon_library.icon_handle_common_add);
        if add_selection_response.clicked() {
            should_add_selection = true;
        }

        let delete_selection_response = toolbar_user_interface.put(
            delete_selection_button_rect,
            Button::new_from_theme(theme)
                .background_color(Color32::TRANSPARENT)
                .with_tooltip_text("Delete selection from results."),
        );
        IconDraw::draw(
            &toolbar_user_interface,
            delete_selection_response.rect,
            &theme.icon_library.icon_handle_common_delete,
        );
        if delete_selection_response.clicked() {
            should_delete_selection = true;
        }

        if let Some(mut element_scanner_results_view_data) = self
            .element_scanner_results_view_data
            .write("Element scanner results action bar edit value")
        {
            self.app_context
                .engine_unprivileged_state
                .normalize_anonymous_value_string_format(&validation_data_type_ref, &mut element_scanner_results_view_data.current_display_string);

            toolbar_user_interface.put(
                edit_value_box_rect,
                DataValueBoxView::new(
                    self.app_context.clone(),
                    &mut element_scanner_results_view_data.current_display_string,
                    &validation_data_type_ref,
                    false,
                    true,
                    "Edit selected values...",
                    "data_value_box_edit_value",
                )
                .width(edit_value_box_rect.width())
                .height(edit_value_box_rect.height()),
            );
        }

        let commit_on_enter_pressed = DataValueBoxView::consume_commit_on_enter(&mut toolbar_user_interface, "data_value_box_edit_value");
        let commit_value_response = toolbar_user_interface.put(
            commit_value_button_rect,
            Button::new_from_theme(theme)
                .background_color(Color32::TRANSPARENT)
                .with_tooltip_text("Commit value to selected scan results."),
        );
        IconDraw::draw(
            &toolbar_user_interface,
            commit_value_response.rect,
            &theme.icon_library.icon_handle_common_check_mark,
        );
        if commit_value_response.clicked() || commit_on_enter_pressed {
            should_commit_selection_value = true;
        }

        if should_toggle_selection_frozen {
            ElementScannerResultsViewData::toggle_selected_scan_results_frozen(
                self.element_scanner_results_view_data.clone(),
                self.app_context.engine_unprivileged_state.clone(),
                matches!(self.selection_freeze_checkstate, CheckState::False),
            );
        } else if should_add_selection {
            let target_directory_path = ProjectHierarchyViewData::get_selected_directory_path(self.project_hierarchy_view_data.clone());
            ElementScannerResultsViewData::add_scan_results_to_project(
                self.element_scanner_results_view_data.clone(),
                self.app_context.engine_unprivileged_state.clone(),
                target_directory_path,
            );
        } else if should_delete_selection {
            ElementScannerResultsViewData::delete_selected_scan_results(
                self.element_scanner_results_view_data.clone(),
                self.app_context.engine_unprivileged_state.clone(),
            );
        } else if should_commit_selection_value {
            let edit_value = self
                .element_scanner_results_view_data
                .read("Element scanner action bar commit value")
                .map(|element_scanner_results_view_data| element_scanner_results_view_data.current_display_string.clone());

            if let Some(edit_value) = edit_value {
                ElementScannerResultsViewData::set_selected_scan_results_value(
                    self.element_scanner_results_view_data.clone(),
                    self.app_context.engine_unprivileged_state.clone(),
                    ScanResult::PROPERTY_NAME_VALUE,
                    edit_value,
                );
            }
        }

        response
    }
}
