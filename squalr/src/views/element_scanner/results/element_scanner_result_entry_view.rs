use crate::{
    app_context::AppContext,
    ui::widgets::controls::{checkbox::Checkbox, state_layer::StateLayer},
    views::element_scanner::results::view_data::element_scanner_result_frame_action::ElementScannerResultFrameAction,
};
use eframe::egui::{Align2, Rect, Response, Sense, Ui, Widget, pos2, vec2};
use epaint::{Color32, CornerRadius, Stroke, StrokeKind};
use squalr_engine_api::{
    registries::symbols::symbol_registry::SymbolRegistry,
    structures::{data_values::anonymous_value_string_format::AnonymousValueStringFormat, scan_results::scan_result::ScanResult},
};
use std::sync::Arc;

pub struct ElementScannerResultEntryView<'lifetime> {
    app_context: Arc<AppContext>,
    scan_result: &'lifetime ScanResult,
    active_display_format: AnonymousValueStringFormat,
    index: usize,
    is_selected: bool,
    element_sanner_result_frame_action: &'lifetime mut ElementScannerResultFrameAction,
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
            address_splitter_position_x,
            value_splitter_position_x,
            previous_value_splitter_position_x,
        }
    }

    pub fn get_height(&self) -> f32 {
        32.0
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

        let (allocated_size_rectangle, response) = user_interface.allocate_exact_size(vec2(user_interface.available_size().x, row_height), Sense::click());

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

        // Address.
        let row_center_y = allocated_size_rectangle.center().y;
        let icon_size = vec2(16.0, 16.0);
        let data_type_ref = self.scan_result.get_data_type_ref();
        let icon_handle = crate::ui::converters::data_type_to_icon_converter::DataTypeToIconConverter::convert_data_type_to_icon(
            data_type_ref.get_data_type_id(),
            &theme.icon_library,
        );
        let icon_pos = pos2(self.address_splitter_position_x + text_left_padding, row_center_y - icon_size.y * 0.5);
        let address_text_position = pos2(icon_pos.x + icon_size.x + 6.0, row_center_y);
        let address = self.scan_result.get_address();
        let address_string = if self.scan_result.is_module() {
            format!("{}+{:X}", self.scan_result.get_module(), self.scan_result.get_module_offset())
        } else if address <= u32::MAX as u64 {
            format!("{:08X}", address)
        } else {
            format!("{:016X}", address)
        };

        user_interface.painter().image(
            icon_handle.id(),
            Rect::from_min_size(icon_pos, icon_size),
            Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
            Color32::WHITE,
        );

        user_interface.painter().text(
            address_text_position,
            Align2::LEFT_CENTER,
            address_string,
            theme.font_library.font_ubuntu_mono_bold.font_normal.clone(),
            theme.hexadecimal_green,
        );

        // Value.
        let current_value_text_position = pos2(self.value_splitter_position_x + text_left_padding, row_center_y);
        let current_value_string = self
            .scan_result
            .get_recently_read_display_value(self.active_display_format)
            .map(|recently_read_display_value| {
                recently_read_display_value
                    .get_anonymous_value_string()
                    .to_string()
            })
            .or_else(|| {
                let symbol_registry = SymbolRegistry::get_instance();

                self.scan_result
                    .get_recently_read_value()
                    .as_ref()
                    .and_then(|recently_read_value| {
                        symbol_registry
                            .anonymize_value(recently_read_value, self.active_display_format)
                            .ok()
                    })
                    .map(|recently_read_display_value| {
                        recently_read_display_value
                            .get_anonymous_value_string()
                            .to_string()
                    })
            })
            .or_else(|| {
                self.scan_result
                    .get_current_display_value(self.active_display_format)
                    .map(|current_display_value| current_display_value.get_anonymous_value_string().to_string())
            })
            .unwrap_or_else(|| "??".to_string());

        user_interface.painter().text(
            current_value_text_position,
            Align2::LEFT_CENTER,
            current_value_string,
            theme.font_library.font_ubuntu_mono_bold.font_normal.clone(),
            theme.foreground,
        );

        // Previous value.
        let previous_value_text_position = pos2(self.previous_value_splitter_position_x + text_left_padding, row_center_y);
        let previous_value_string = match self
            .scan_result
            .get_previous_display_value(self.active_display_format)
        {
            Some(previous_value) => previous_value.get_anonymous_value_string(),
            None => "??",
        };

        user_interface.painter().text(
            previous_value_text_position,
            Align2::LEFT_CENTER,
            previous_value_string,
            theme.font_library.font_ubuntu_mono_bold.font_normal.clone(),
            theme.foreground,
        );

        response
    }
}
