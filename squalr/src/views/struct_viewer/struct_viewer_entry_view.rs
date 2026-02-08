use crate::{
    app_context::AppContext,
    ui::{converters::data_type_to_icon_converter::DataTypeToIconConverter, draw::icon_draw::IconDraw, widgets::controls::state_layer::StateLayer},
    views::struct_viewer::view_data::struct_viewer_frame_action::StructViewerFrameAction,
};
use eframe::egui::{Align2, Response, Sense, Ui, Widget, vec2};
use epaint::{CornerRadius, Rect, Stroke, StrokeKind, pos2};
use squalr_engine_api::structures::structs::valued_struct_field::ValuedStructField;
use std::sync::Arc;

pub struct StructViewerEntryView<'lifetime> {
    app_context: Arc<AppContext>,
    valued_struct_field: &'lifetime ValuedStructField,
    is_selected: bool,
    struct_viewer_frame_action: &'lifetime mut StructViewerFrameAction,
    name_splitter_x: f32,
    value_splitter_x: f32,
}

impl<'lifetime> StructViewerEntryView<'lifetime> {
    pub fn new(
        app_context: Arc<AppContext>,
        valued_struct_field: &'lifetime ValuedStructField,
        is_selected: bool,
        struct_viewer_frame_action: &'lifetime mut StructViewerFrameAction,
        name_splitter_x: f32,
        value_splitter_x: f32,
    ) -> Self {
        Self {
            app_context: app_context,
            valued_struct_field,
            is_selected,
            struct_viewer_frame_action,
            name_splitter_x,
            value_splitter_x,
        }
    }
}

impl<'lifetime> Widget for StructViewerEntryView<'lifetime> {
    fn ui(
        self,
        user_interface: &mut Ui,
    ) -> Response {
        let theme = &self.app_context.theme;
        let icon_size = vec2(16.0, 16.0);
        let text_left_padding = 4.0;
        let row_height = 32.0;

        let desired_size = vec2(user_interface.available_width(), row_height);
        let (available_size_id, available_size_rect) = user_interface.allocate_space(desired_size);
        let response = user_interface.interact(available_size_rect, available_size_id, Sense::click());

        // Selected background.
        if self.is_selected {
            user_interface
                .painter()
                .rect_filled(available_size_rect, CornerRadius::ZERO, theme.selected_background);
            user_interface.painter().rect_stroke(
                available_size_rect,
                CornerRadius::ZERO,
                Stroke::new(1.0, theme.selected_border),
                StrokeKind::Inside,
            );
        }

        // State overlay.
        StateLayer {
            bounds_min: available_size_rect.min,
            bounds_max: available_size_rect.max,
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

        // Click handling
        if response.double_clicked() {
            *self.struct_viewer_frame_action = StructViewerFrameAction::None;
        } else if response.clicked() {
            *self.struct_viewer_frame_action = StructViewerFrameAction::SelectField(self.valued_struct_field.get_name().to_string());
        } else if response.secondary_clicked() {
            *self.struct_viewer_frame_action = StructViewerFrameAction::None;
        }

        let row_min_x = available_size_rect.min.x;
        let row_max_x = available_size_rect.max.x;
        let icon_position_x = row_min_x;
        let name_position_x = row_min_x + self.name_splitter_x;
        let value_position_x = self.value_splitter_x.min(row_max_x);

        // Draw icon.
        let icon_rect = Rect::from_min_max(
            pos2(icon_position_x, available_size_rect.min.y),
            pos2(name_position_x, available_size_rect.max.y),
        );
        let icon_center = icon_rect.center();
        let icon = DataTypeToIconConverter::convert_data_type_to_icon(self.valued_struct_field.get_icon_id(), &theme.icon_library);

        IconDraw::draw_sized(user_interface, icon_center, icon_size, &icon);

        // Draw text.
        let text_rectangle = Rect::from_min_max(
            pos2(name_position_x, available_size_rect.min.y),
            pos2(value_position_x, available_size_rect.max.y),
        );
        let text_pos = pos2(text_rectangle.min.x + text_left_padding, text_rectangle.center().y);

        user_interface.painter().text(
            text_pos,
            Align2::LEFT_CENTER,
            self.valued_struct_field.get_name(),
            theme.font_library.font_noto_sans.font_normal.clone(),
            theme.foreground,
        );

        response
    }
}
