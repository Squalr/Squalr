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
use eframe::egui::{Align, Layout, Response, Sense, Ui, UiBuilder, Widget, vec2};
use epaint::{Color32, CornerRadius, Stroke, StrokeKind};
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
    const HORIZONTAL_PADDING: f32 = 8.0;
    const CONTROL_SPACING: f32 = 6.0;
    const MINIMUM_EDIT_WIDTH: f32 = 120.0;

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
}

impl Widget for ElementScannerResultsActionBarView {
    fn ui(
        self,
        user_interface: &mut Ui,
    ) -> Response {
        let theme = &self.app_context.theme;
        let button_size = vec2(Self::BUTTON_SIZE[0], Self::BUTTON_SIZE[1]);
        let (allocated_size_rectangle, response) =
            user_interface.allocate_exact_size(vec2(user_interface.available_width().max(1.0), self.get_height()), Sense::hover());
        let mut toolbar_user_interface = user_interface.new_child(
            UiBuilder::new()
                .max_rect(allocated_size_rectangle)
                .layout(Layout::left_to_right(Align::Center)),
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

        toolbar_user_interface.with_layout(Layout::left_to_right(Align::Center), |user_interface| {
            user_interface.spacing_mut().item_spacing.x = Self::CONTROL_SPACING;
            user_interface.add_space(Self::HORIZONTAL_PADDING);

            if user_interface
                .add(Checkbox::new_from_theme(theme).with_check_state(self.selection_freeze_checkstate))
                .clicked()
            {
                should_toggle_selection_frozen = true;
            }

            let add_selection_response = user_interface.add_sized(
                button_size,
                Button::new_from_theme(theme)
                    .background_color(Color32::TRANSPARENT)
                    .with_tooltip_text("Add selection to project."),
            );
            IconDraw::draw(user_interface, add_selection_response.rect, &theme.icon_library.icon_handle_common_add);
            if add_selection_response.clicked() {
                should_add_selection = true;
            }

            let delete_selection_response = user_interface.add_sized(
                button_size,
                Button::new_from_theme(theme)
                    .background_color(Color32::TRANSPARENT)
                    .with_tooltip_text("Delete selection from results."),
            );
            IconDraw::draw(user_interface, delete_selection_response.rect, &theme.icon_library.icon_handle_common_delete);
            if delete_selection_response.clicked() {
                should_delete_selection = true;
            }

            user_interface.add_space(4.0);

            if let Some(mut element_scanner_results_view_data) = self
                .element_scanner_results_view_data
                .write("Element scanner results action bar edit value")
            {
                self.app_context
                    .engine_unprivileged_state
                    .normalize_anonymous_value_string_format(&validation_data_type_ref, &mut element_scanner_results_view_data.current_display_string);

                let reserved_width = Self::HORIZONTAL_PADDING + button_size.x * 4.0 + Self::CONTROL_SPACING * 5.0 + 4.0;
                let edit_value_box_width = (user_interface.available_width() - reserved_width).max(Self::MINIMUM_EDIT_WIDTH);

                user_interface.add(
                    DataValueBoxView::new(
                        self.app_context.clone(),
                        &mut element_scanner_results_view_data.current_display_string,
                        &validation_data_type_ref,
                        false,
                        true,
                        "Edit selected values...",
                        "data_value_box_edit_value",
                    )
                    .width(edit_value_box_width)
                    .height(allocated_size_rectangle.height()),
                );
            }

            let commit_on_enter_pressed = DataValueBoxView::consume_commit_on_enter(user_interface, "data_value_box_edit_value");
            let commit_value_response = user_interface.add_sized(
                button_size,
                Button::new_from_theme(theme)
                    .background_color(Color32::TRANSPARENT)
                    .with_tooltip_text("Commit value to selected scan results."),
            );
            IconDraw::draw(user_interface, commit_value_response.rect, &theme.icon_library.icon_handle_common_check_mark);
            if commit_value_response.clicked() || commit_on_enter_pressed {
                should_commit_selection_value = true;
            }

            user_interface.add_space(Self::HORIZONTAL_PADDING);
        });

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
