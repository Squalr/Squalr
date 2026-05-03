use crate::{
    app_context::AppContext,
    ui::converters::data_type_to_string_converter::DataTypeToStringConverter,
    ui::widgets::controls::{checkbox::Checkbox, state_layer::StateLayer},
    views::element_scanner::results::view_data::element_scanner_result_frame_action::ElementScannerResultFrameAction,
};
use eframe::egui::{Align2, Rect, Response, Sense, Ui, Widget, pos2, vec2};
use epaint::{Color32, CornerRadius, Stroke, StrokeKind};
use squalr_engine_api::structures::{data_values::anonymous_value_string_format::AnonymousValueStringFormat, scan_results::scan_result::ScanResult};
use std::sync::Arc;

pub struct ElementScannerResultEntryView<'lifetime> {
    app_context: Arc<AppContext>,
    scan_result: &'lifetime ScanResult,
    active_display_format: AnonymousValueStringFormat,
    index: usize,
    is_selected: bool,
    element_sanner_result_frame_action: &'lifetime mut ElementScannerResultFrameAction,
    data_type_splitter_position_x: f32,
    address_splitter_position_x: f32,
    value_splitter_position_x: f32,
    previous_value_splitter_position_x: f32,
}

impl<'lifetime> ElementScannerResultEntryView<'lifetime> {
    pub fn new(
        app_context: Arc<AppContext>,
        scan_result: &'lifetime ScanResult,
        active_display_format: AnonymousValueStringFormat,
        index: usize,
        is_selected: bool,
        element_sanner_result_frame_action: &'lifetime mut ElementScannerResultFrameAction,
        data_type_splitter_position_x: f32,
        address_splitter_position_x: f32,
        value_splitter_position_x: f32,
        previous_value_splitter_position_x: f32,
    ) -> Self {
        Self {
            app_context,
            scan_result,
            active_display_format,
            index,
            is_selected,
            element_sanner_result_frame_action,
            data_type_splitter_position_x,
            address_splitter_position_x,
            value_splitter_position_x,
            previous_value_splitter_position_x,
        }
    }

    pub fn get_height(&self) -> f32 {
        32.0
    }

    fn add_cell_tooltip(
        user_interface: &mut Ui,
        cell_rectangle: Rect,
        tooltip_id_suffix: &str,
        tooltip_text: &str,
    ) {
        if tooltip_text.is_empty() {
            return;
        }

        let tooltip_rectangle = cell_rectangle.intersect(user_interface.clip_rect());

        if tooltip_rectangle.is_negative() {
            return;
        }

        user_interface
            .interact(tooltip_rectangle, user_interface.id().with(tooltip_id_suffix), Sense::hover())
            .on_hover_text(tooltip_text);
    }

    fn text_clip_rectangle(
        user_interface: &Ui,
        text_rectangle: Rect,
    ) -> Rect {
        text_rectangle.intersect(user_interface.clip_rect())
    }
}

impl<'a> Widget for ElementScannerResultEntryView<'a> {
    fn ui(
        self,
        user_interface: &mut Ui,
    ) -> Response {
        let theme = &self.app_context.theme;
        let text_left_padding = 8.0;
        let row_height = self.get_height();

        let (allocated_size_rectangle, response) =
            user_interface.allocate_exact_size(vec2(user_interface.available_size().x.max(1.0), row_height), Sense::click());

        if self.is_selected {
            // Draw the background.
            user_interface
                .painter()
                .rect_filled(allocated_size_rectangle, CornerRadius::ZERO, theme.selected_background);

            // Draw the border.
            user_interface.painter().rect_stroke(
                allocated_size_rectangle,
                CornerRadius::ZERO,
                Stroke::new(1.0, theme.selected_border),
                StrokeKind::Inside,
            );
        }

        // Background and state overlay.
        StateLayer {
            bounds_min: allocated_size_rectangle.min,
            bounds_max: allocated_size_rectangle.max,
            enabled: true,
            pressed: response.is_pointer_button_down_on(),
            has_hover: response.hovered(),
            has_focus: response.has_focus(),
            corner_radius: CornerRadius::ZERO,
            border_width: 0.0,
            hover_color: theme.hover_tint,
            pressed_color: theme.pressed_tint,
            border_color: theme.background_control_secondary_dark,
            border_color_focused: theme.background_control_secondary_dark,
        }
        .ui(user_interface);

        // Checkbox.
        let checkbox_size = vec2(Checkbox::WIDTH, Checkbox::HEIGHT);
        let checkbox_position = pos2(
            allocated_size_rectangle.min.x + 8.0,
            allocated_size_rectangle.center().y - checkbox_size.y * 0.5,
        );
        let checkbox_rectangle = Rect::from_min_size(checkbox_position, checkbox_size);
        let is_frozen = self.scan_result.get_is_frozen();

        if response.clicked() {
            if user_interface.input(|input| input.modifiers.shift) {
                *self.element_sanner_result_frame_action = ElementScannerResultFrameAction::SetSelectionEnd(Some(self.index as i32));
            } else {
                *self.element_sanner_result_frame_action = ElementScannerResultFrameAction::SetSelectionStart(Some(self.index as i32));
            }
        }

        if user_interface
            .place(checkbox_rectangle, Checkbox::new_from_theme(theme).with_check_state_bool(is_frozen))
            .clicked()
        {
            *self.element_sanner_result_frame_action = ElementScannerResultFrameAction::FreezeIndex(self.index as i32, !is_frozen);
        }

        if response.is_pointer_button_down_on() {
            user_interface
                .painter()
                .rect_filled(checkbox_rectangle, CornerRadius::ZERO, theme.pressed_tint);
        }

        if self.scan_result.get_is_frozen() {
            let icon = &theme.icon_library.icon_handle_common_check_mark;
            let texture_size = icon.size_vec2();
            let icon_pos = checkbox_rectangle.center() - texture_size * 0.5;

            user_interface.painter().image(
                icon.id(),
                Rect::from_min_size(icon_pos, texture_size),
                Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                Color32::WHITE,
            );
        }

        // Data type.
        let row_center_y = allocated_size_rectangle.center().y;
        let icon_size = vec2(16.0, 16.0);
        let data_type_ref = self.scan_result.get_data_type_ref();
        let data_type_label = DataTypeToStringConverter::convert_data_type_to_string(data_type_ref.get_data_type_id());
        let icon_handle = crate::ui::converters::data_type_to_icon_converter::DataTypeToIconConverter::convert_data_type_to_icon(
            data_type_ref.get_data_type_id(),
            &theme.icon_library,
        );
        let data_type_icon_rectangle = Rect::from_min_size(
            pos2(self.data_type_splitter_position_x + text_left_padding, row_center_y - icon_size.y * 0.5),
            icon_size,
        );
        let data_type_text_position = pos2(data_type_icon_rectangle.max.x + text_left_padding, row_center_y);
        let data_type_text_clip_rectangle = Rect::from_min_max(
            pos2(data_type_text_position.x, allocated_size_rectangle.min.y),
            pos2(
                (self.address_splitter_position_x - text_left_padding).max(data_type_text_position.x),
                allocated_size_rectangle.max.y,
            ),
        );
        let address_text_position = pos2(self.address_splitter_position_x + text_left_padding, row_center_y);
        let address_cell_rectangle = Rect::from_min_max(
            pos2(self.address_splitter_position_x, allocated_size_rectangle.min.y),
            pos2(self.value_splitter_position_x, allocated_size_rectangle.max.y),
        );
        let address_text_clip_rectangle = Rect::from_min_max(
            pos2(address_text_position.x, allocated_size_rectangle.min.y),
            pos2(
                (self.value_splitter_position_x - text_left_padding).max(address_text_position.x),
                allocated_size_rectangle.max.y,
            ),
        );
        let address_string = self.scan_result.get_address_display_text();

        user_interface.painter().image(
            icon_handle.id(),
            data_type_icon_rectangle,
            Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
            Color32::WHITE,
        );

        user_interface
            .painter()
            .with_clip_rect(Self::text_clip_rectangle(user_interface, data_type_text_clip_rectangle))
            .text(
                data_type_text_position,
                Align2::LEFT_CENTER,
                data_type_label,
                theme.font_library.font_ubuntu_mono_bold.font_normal.clone(),
                theme.foreground,
            );

        user_interface
            .painter()
            .with_clip_rect(Self::text_clip_rectangle(user_interface, address_text_clip_rectangle))
            .text(
                address_text_position,
                Align2::LEFT_CENTER,
                address_string.as_str(),
                theme.font_library.font_ubuntu_mono_bold.font_normal.clone(),
                theme.hexadecimal_green,
            );

        Self::add_cell_tooltip(
            user_interface,
            address_cell_rectangle,
            &format!("scan_result_address_tooltip_{}", self.index),
            address_string.as_str(),
        );

        // Value.
        let current_value_text_position = pos2(self.value_splitter_position_x + text_left_padding, row_center_y);
        let current_value_cell_rectangle = Rect::from_min_max(
            pos2(self.value_splitter_position_x, allocated_size_rectangle.min.y),
            pos2(self.previous_value_splitter_position_x, allocated_size_rectangle.max.y),
        );
        let current_value_text_clip_rectangle = Rect::from_min_max(
            pos2(current_value_text_position.x, allocated_size_rectangle.min.y),
            pos2(
                (self.previous_value_splitter_position_x - text_left_padding).max(current_value_text_position.x),
                allocated_size_rectangle.max.y,
            ),
        );
        let current_value_string = self
            .scan_result
            .get_preferred_current_display_value(self.active_display_format)
            .map(|display_value| display_value.get_anonymous_value_string().to_string())
            .unwrap_or_else(|| "??".to_string());

        user_interface
            .painter()
            .with_clip_rect(Self::text_clip_rectangle(user_interface, current_value_text_clip_rectangle))
            .text(
                current_value_text_position,
                Align2::LEFT_CENTER,
                current_value_string.as_str(),
                theme.font_library.font_ubuntu_mono_bold.font_normal.clone(),
                theme.foreground,
            );

        Self::add_cell_tooltip(
            user_interface,
            current_value_cell_rectangle,
            &format!("scan_result_current_value_tooltip_{}", self.index),
            current_value_string.as_str(),
        );

        // Previous value.
        let previous_value_text_position = pos2(self.previous_value_splitter_position_x + text_left_padding, row_center_y);
        let previous_value_cell_rectangle = Rect::from_min_max(
            pos2(self.previous_value_splitter_position_x, allocated_size_rectangle.min.y),
            pos2(allocated_size_rectangle.max.x, allocated_size_rectangle.max.y),
        );
        let previous_value_text_clip_rectangle = Rect::from_min_max(
            pos2(previous_value_text_position.x, allocated_size_rectangle.min.y),
            pos2(
                (allocated_size_rectangle.max.x - text_left_padding).max(previous_value_text_position.x),
                allocated_size_rectangle.max.y,
            ),
        );
        let previous_value_string = match self
            .scan_result
            .get_preferred_previous_display_value(self.active_display_format)
        {
            Some(previous_value) => previous_value.get_anonymous_value_string(),
            None => "??",
        };

        user_interface
            .painter()
            .with_clip_rect(Self::text_clip_rectangle(user_interface, previous_value_text_clip_rectangle))
            .text(
                previous_value_text_position,
                Align2::LEFT_CENTER,
                previous_value_string,
                theme.font_library.font_ubuntu_mono_bold.font_normal.clone(),
                theme.foreground,
            );

        Self::add_cell_tooltip(
            user_interface,
            previous_value_cell_rectangle,
            &format!("scan_result_previous_value_tooltip_{}", self.index),
            previous_value_string,
        );

        response
    }
}
