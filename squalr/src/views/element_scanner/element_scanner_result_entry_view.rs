use crate::{
    app_context::AppContext,
    ui::{draw::icon_draw::IconDraw, widgets::controls::state_layer::StateLayer},
};
use eframe::egui::{Align2, Rect, Response, Sense, TextureHandle, Ui, Widget, pos2, vec2};
use epaint::CornerRadius;
use squalr_engine_api::structures::scan_results::scan_result::ScanResult;
use std::sync::Arc;

pub struct ElementScannerResultEntryView<'lifetime> {
    app_context: Arc<AppContext>,
    scan_result: &'lifetime ScanResult,
    icon: Option<TextureHandle>,
    value_splitter_position_x: f32,
    previous_value_splitter_position_x: f32,
}

impl<'lifetime> ElementScannerResultEntryView<'lifetime> {
    pub fn new(
        app_context: Arc<AppContext>,
        scan_result: &'lifetime ScanResult,
        icon: Option<TextureHandle>,
        value_splitter_position_x: f32,
        previous_value_splitter_position_x: f32,
    ) -> Self {
        Self {
            app_context,
            scan_result,
            icon,
            value_splitter_position_x,
            previous_value_splitter_position_x,
        }
    }
}

impl<'a> Widget for ElementScannerResultEntryView<'a> {
    fn ui(
        self,
        user_interface: &mut Ui,
    ) -> Response {
        let theme = &self.app_context.theme;
        let icon_size = vec2(16.0, 16.0);
        let text_left_padding = 4.0;
        let row_height = 28.0;

        let (allocated_size_rectangle, response) = user_interface.allocate_exact_size(vec2(user_interface.available_size().x, row_height), Sense::click());

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

        // Icon.
        let icon_position_x = allocated_size_rectangle.min.x;
        let icon_position_y = allocated_size_rectangle.center().y - icon_size.y * 0.5;
        let icon_rectangle = Rect::from_min_size(pos2(icon_position_x, icon_position_y), icon_size);

        if let Some(icon) = &self.icon {
            IconDraw::draw_sized(user_interface, icon_rectangle.center(), icon_size, icon);
        }

        // Text positions for each column.
        let address_text_position_x = icon_rectangle.max.x + text_left_padding;
        let row_center_y = allocated_size_rectangle.center().y;
        let address_text_position = pos2(address_text_position_x, row_center_y);
        let value_text_position = pos2(self.value_splitter_position_x + text_left_padding, row_center_y);
        let previous_value_text_position = pos2(self.previous_value_splitter_position_x + text_left_padding, row_center_y);

        // Address column.
        user_interface.painter().text(
            address_text_position,
            Align2::LEFT_CENTER,
            self.scan_result.get_address().to_string(),
            theme.font_library.font_noto_sans.font_normal.clone(),
            theme.foreground,
        );

        // Value column (placeholder: same as address for now).
        user_interface.painter().text(
            value_text_position,
            Align2::LEFT_CENTER,
            self.scan_result.get_address().to_string(),
            theme.font_library.font_noto_sans.font_normal.clone(),
            theme.foreground,
        );

        // Previous value column (placeholder: same as address for now).
        user_interface.painter().text(
            previous_value_text_position,
            Align2::LEFT_CENTER,
            self.scan_result.get_address().to_string(),
            theme.font_library.font_noto_sans.font_normal.clone(),
            theme.foreground,
        );

        response
    }
}
