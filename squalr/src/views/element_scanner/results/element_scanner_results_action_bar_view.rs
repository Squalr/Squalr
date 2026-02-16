use crate::{
    app_context::AppContext,
    ui::{
        draw::icon_draw::IconDraw,
        widgets::controls::{button::Button, check_state::CheckState, checkbox::Checkbox, data_value_box::data_value_box_view::DataValueBoxView},
    },
    views::element_scanner::{
        results::view_data::{
            element_scanner_result_frame_action::ElementScannerResultFrameAction, element_scanner_results_view_data::ElementScannerResultsViewData,
        },
        scanner::view_data::element_scanner_view_data::ElementScannerViewData,
    },
};
use eframe::egui::{Align, Layout, Response, Sense, Ui, UiBuilder, Widget};
use epaint::{Color32, CornerRadius, Rect, Stroke, StrokeKind, pos2, vec2};
use squalr_engine_api::dependency_injection::dependency::Dependency;
use std::sync::Arc;

pub struct ElementScannerResultsActionBarView<'lifetime> {
    app_context: Arc<AppContext>,
    element_scanner_results_view_data: Dependency<ElementScannerResultsViewData>,
    element_scanner_view_data: Dependency<ElementScannerViewData>,
    selection_freeze_checkstate: CheckState,
    element_sanner_result_frame_action: &'lifetime mut ElementScannerResultFrameAction,
    address_splitter_position_x: f32,
    value_splitter_position_x: f32,
    previous_value_splitter_position_x: f32,
}

impl<'lifetime> ElementScannerResultsActionBarView<'lifetime> {
    pub const FOOTER_HEIGHT: f32 = 32.0;

    pub fn new(
        app_context: Arc<AppContext>,
        selection_freeze_checkstate: CheckState,
        element_sanner_result_frame_action: &'lifetime mut ElementScannerResultFrameAction,
        address_splitter_position_x: f32,
        value_splitter_position_x: f32,
        previous_value_splitter_position_x: f32,
    ) -> Self {
        let element_scanner_results_view_data = app_context
            .dependency_container
            .get_dependency::<ElementScannerResultsViewData>();
        let element_scanner_view_data = app_context
            .dependency_container
            .get_dependency::<ElementScannerViewData>();

        Self {
            app_context,
            element_scanner_results_view_data,
            element_scanner_view_data,
            selection_freeze_checkstate,
            element_sanner_result_frame_action,
            address_splitter_position_x,
            value_splitter_position_x,
            previous_value_splitter_position_x,
        }
    }

    pub fn get_height(&self) -> f32 {
        Self::FOOTER_HEIGHT
    }
}

impl<'lifetime> Widget for ElementScannerResultsActionBarView<'lifetime> {
    fn ui(
        self,
        user_interface: &mut Ui,
    ) -> Response {
        let theme = &self.app_context.theme;
        let button_size = vec2(36.0, 28.0);

        let (allocated_size_rectangle, response) =
            user_interface.allocate_exact_size(vec2(user_interface.available_width(), self.get_height()), Sense::hover());

        let builder = UiBuilder::new()
            .max_rect(allocated_size_rectangle)
            .layout(Layout::left_to_right(Align::Center));

        let mut toolbar_user_interface = user_interface.new_child(builder);

        let mut element_scanner_results_view_data = match self
            .element_scanner_results_view_data
            .write("Element scanner results action bar element scanner results view data")
        {
            Some(element_scanner_results_view_data) => element_scanner_results_view_data,
            None => return response,
        };
        let element_scanner_view_data = match self
            .element_scanner_view_data
            .read("Element scanner results action bar element scanner view data")
        {
            Some(element_scanner_view_data) => element_scanner_view_data,
            None => return response,
        };

        // Background.
        user_interface
            .painter()
            .rect_filled(allocated_size_rectangle, CornerRadius::ZERO, theme.background_panel);

        // Border.
        toolbar_user_interface.painter().rect_stroke(
            allocated_size_rectangle,
            CornerRadius::ZERO,
            Stroke::new(1.0, theme.submenu_border),
            StrokeKind::Inside,
        );

        // Toolbar buttons.
        toolbar_user_interface.with_layout(Layout::left_to_right(Align::Center), |user_interface| {
            user_interface.add_space(8.0);
            if user_interface
                .add(Checkbox::new_from_theme(theme).with_check_state(self.selection_freeze_checkstate))
                .clicked()
            {
                match self.selection_freeze_checkstate {
                    CheckState::False => {
                        *self.element_sanner_result_frame_action = ElementScannerResultFrameAction::ToggleFreezeSelection(true);
                    }
                    CheckState::Mixed => {
                        *self.element_sanner_result_frame_action = ElementScannerResultFrameAction::ToggleFreezeSelection(false);
                    }
                    CheckState::True => {
                        *self.element_sanner_result_frame_action = ElementScannerResultFrameAction::ToggleFreezeSelection(false);
                    }
                }
            }

            let y_center = allocated_size_rectangle.center().y - button_size.y * 0.5;
            let add_selection_response = user_interface.put(
                Rect::from_min_size(pos2(self.address_splitter_position_x, y_center), button_size),
                Button::new_from_theme(theme)
                    .background_color(Color32::TRANSPARENT)
                    .with_tooltip_text("Add selection to project."),
            );

            IconDraw::draw(user_interface, add_selection_response.rect, &theme.icon_library.icon_handle_common_add);

            if add_selection_response.clicked() {
                *self.element_sanner_result_frame_action = ElementScannerResultFrameAction::AddSelection;
            }

            let delete_selection_response = user_interface.add_sized(
                button_size,
                Button::new_from_theme(theme)
                    .background_color(Color32::TRANSPARENT)
                    .with_tooltip_text("Delete selection from results."),
            );

            IconDraw::draw(user_interface, delete_selection_response.rect, &theme.icon_library.icon_handle_common_delete);

            if delete_selection_response.clicked() {
                *self.element_sanner_result_frame_action = ElementScannerResultFrameAction::DeleteSelection;
            }

            let padding = 2.0;
            let data_value_box_width = self.previous_value_splitter_position_x - self.value_splitter_position_x - padding * 2.0;

            user_interface.put(
                Rect::from_min_size(
                    pos2(self.value_splitter_position_x + padding, allocated_size_rectangle.min.y),
                    vec2(data_value_box_width, allocated_size_rectangle.height()),
                ),
                DataValueBoxView::new(
                    self.app_context.clone(),
                    &mut element_scanner_results_view_data.current_display_string,
                    &element_scanner_view_data.selected_data_type,
                    false,
                    true,
                    "Edit selected values...",
                    "data_value_box_edit_value",
                )
                .width(data_value_box_width),
            );
            let commit_on_enter_pressed = DataValueBoxView::consume_commit_on_enter(user_interface, "data_value_box_edit_value");

            let commit_value_response = user_interface.add_sized(
                button_size,
                Button::new_from_theme(theme)
                    .background_color(Color32::TRANSPARENT)
                    .with_tooltip_text("Commit value to selected scan results."),
            );

            IconDraw::draw(user_interface, commit_value_response.rect, &theme.icon_library.icon_handle_common_check_mark);

            if commit_value_response.clicked() || commit_on_enter_pressed {
                *self.element_sanner_result_frame_action =
                    ElementScannerResultFrameAction::CommitValueToSelection(element_scanner_results_view_data.current_display_string.clone());
            }
        });

        response
    }
}
