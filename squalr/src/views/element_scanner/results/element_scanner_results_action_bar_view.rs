use crate::{
    app_context::AppContext,
    ui::{
        draw::icon_draw::IconDraw,
        widgets::controls::{button::Button, data_value_box::data_value_box_view::DataValueBoxView},
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
    element_sanner_result_frame_action: &'lifetime mut ElementScannerResultFrameAction,
    address_splitter_position_x: f32,
    value_splitter_position_x: f32,
    previous_value_splitter_position_x: f32,
}

impl<'lifetime> ElementScannerResultsActionBarView<'lifetime> {
    pub const FOOTER_HEIGHT: f32 = 32.0;

    pub fn new(
        app_context: Arc<AppContext>,
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

        let mut element_scanner_results_view_data = match self.element_scanner_results_view_data.write() {
            Ok(element_scanner_results_view_data) => element_scanner_results_view_data,
            Err(error) => {
                log::error!("Failed to acquire element scanner results view data: {}", error);
                return response;
            }
        };
        let element_scanner_view_data = match self.element_scanner_view_data.read() {
            Ok(element_scanner_view_data) => element_scanner_view_data,
            Err(error) => {
                log::error!("Failed to acquire element scanner view data: {}", error);
                return response;
            }
        };

        // Background.
        toolbar_user_interface.painter().rect_stroke(
            allocated_size_rectangle,
            CornerRadius::ZERO,
            Stroke::new(1.0, theme.submenu_border),
            StrokeKind::Inside,
        );

        // Toolbar buttons.
        toolbar_user_interface.with_layout(Layout::left_to_right(Align::Center), |user_interface| {
            let freeze_selection_response = user_interface.add_sized(
                button_size,
                Button::new_from_theme(theme)
                    .background_color(Color32::TRANSPARENT)
                    .tooltip_text("Freeze selection."),
            );

            IconDraw::draw(user_interface, freeze_selection_response.rect, &theme.icon_library.icon_handle_results_freeze);

            if freeze_selection_response.clicked() {
                //
            }

            let unfreeze_selection_response = user_interface.add_sized(
                button_size,
                Button::new_from_theme(theme)
                    .background_color(Color32::TRANSPARENT)
                    .tooltip_text("Unfreeze selection."),
            );

            IconDraw::draw(user_interface, unfreeze_selection_response.rect, &theme.icon_library.icon_handle_results_freeze);

            if unfreeze_selection_response.clicked() {
                //
            }

            let add_selection_response = user_interface.add_sized(
                button_size,
                Button::new_from_theme(theme)
                    .background_color(Color32::TRANSPARENT)
                    .tooltip_text("Add selection to project."),
            );

            IconDraw::draw(user_interface, add_selection_response.rect, &theme.icon_library.icon_handle_common_add);

            if add_selection_response.clicked() {
                //
            }

            let delete_selection_response = user_interface.add_sized(
                button_size,
                Button::new_from_theme(theme)
                    .background_color(Color32::TRANSPARENT)
                    .tooltip_text("Delete selection from results."),
            );

            IconDraw::draw(user_interface, delete_selection_response.rect, &theme.icon_library.icon_handle_common_delete);

            if delete_selection_response.clicked() {
                //
            }
        });

        // Treat value_splitter_position_x as an absolute X in the same space as allocated_size_rectangle.
        let value_box_height = button_size.y;
        let value_box_min_x = self
            .value_splitter_position_x
            .clamp(allocated_size_rectangle.left(), allocated_size_rectangle.right());
        let value_box_min_y = allocated_size_rectangle.center().y - value_box_height * 0.5;
        let value_box_width = (allocated_size_rectangle.right() - value_box_min_x).max(1.0);
        let value_box_rectangle = Rect::from_min_size(pos2(value_box_min_x, value_box_min_y), vec2(value_box_width, value_box_height));

        user_interface.place(
            value_box_rectangle,
            DataValueBoxView::new(
                self.app_context.clone(),
                &mut element_scanner_results_view_data.edit_value,
                &element_scanner_view_data.selected_data_type,
                false,
                true,
                "Edit selected values...",
                "data_value_box_edit_value",
            ),
        );

        response
    }
}
