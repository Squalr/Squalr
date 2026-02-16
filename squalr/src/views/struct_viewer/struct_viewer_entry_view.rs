use crate::{
    app_context::AppContext,
    ui::{
        converters::data_type_to_icon_converter::DataTypeToIconConverter,
        draw::icon_draw::IconDraw,
        widgets::controls::{button::Button, data_value_box::data_value_box_view::DataValueBoxView, state_layer::StateLayer},
    },
    views::struct_viewer::view_data::struct_viewer_frame_action::StructViewerFrameAction,
};
use eframe::egui::{Align2, Response, Sense, Ui, Widget, vec2};
use epaint::{CornerRadius, Rect, Stroke, StrokeKind, pos2};
use squalr_engine_api::{
    registries::symbols::symbol_registry::SymbolRegistry,
    structures::{
        data_types::data_type_ref::DataTypeRef,
        data_values::anonymous_value_string::AnonymousValueString,
        structs::valued_struct_field::{ValuedStructField, ValuedStructFieldData},
    },
};
use std::sync::Arc;

pub struct StructViewerEntryView<'lifetime> {
    app_context: Arc<AppContext>,
    valued_struct_field: &'lifetime ValuedStructField,
    row_index: usize,
    is_selected: bool,
    struct_viewer_frame_action: &'lifetime mut StructViewerFrameAction,
    field_edit_value: Option<&'lifetime mut AnonymousValueString>,
    field_display_values: Option<&'lifetime [AnonymousValueString]>,
    validation_data_type_ref: Option<&'lifetime DataTypeRef>,
    name_splitter_x: f32,
    value_splitter_x: f32,
}

impl<'lifetime> StructViewerEntryView<'lifetime> {
    pub fn new(
        app_context: Arc<AppContext>,
        valued_struct_field: &'lifetime ValuedStructField,
        row_index: usize,
        is_selected: bool,
        struct_viewer_frame_action: &'lifetime mut StructViewerFrameAction,
        field_edit_value: Option<&'lifetime mut AnonymousValueString>,
        field_display_values: Option<&'lifetime [AnonymousValueString]>,
        validation_data_type_ref: Option<&'lifetime DataTypeRef>,
        name_splitter_x: f32,
        value_splitter_x: f32,
    ) -> Self {
        Self {
            app_context,
            valued_struct_field,
            row_index,
            is_selected,
            struct_viewer_frame_action,
            field_edit_value,
            field_display_values,
            validation_data_type_ref,
            name_splitter_x,
            value_splitter_x,
        }
    }

    fn commit_field_edit(
        valued_struct_field: &ValuedStructField,
        validation_data_type_ref: &DataTypeRef,
        field_edit_value: &AnonymousValueString,
        struct_viewer_frame_action: &mut StructViewerFrameAction,
    ) {
        let symbol_registry = SymbolRegistry::get_instance();

        match symbol_registry.deanonymize_value_string(validation_data_type_ref, field_edit_value) {
            Ok(new_data_value) => {
                let mut edited_field = valued_struct_field.clone();

                edited_field.set_field_data(ValuedStructFieldData::Value(new_data_value));
                *struct_viewer_frame_action = StructViewerFrameAction::EditValue(edited_field);
            }
            Err(error) => {
                log::warn!("Failed to commit struct viewer value: {}", error);
            }
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
        let value_column_padding = 2.0;
        let commit_button_width = 28.0;
        let show_commit_button = !self.valued_struct_field.get_is_read_only();

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
        let value_box_position_x = value_position_x + value_column_padding;
        let commit_button_space = if show_commit_button {
            commit_button_width + value_column_padding
        } else {
            0.0
        };
        let value_box_width = (row_max_x - value_box_position_x - commit_button_space).max(0.0);

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

        if let (Some(field_edit_value), Some(validation_data_type_ref)) = (self.field_edit_value, self.validation_data_type_ref) {
            let data_value_box_id = format!("struct_viewer_value_{}_{}", self.row_index, self.valued_struct_field.get_name());
            user_interface.put(
                Rect::from_min_size(
                    pos2(value_box_position_x, available_size_rect.min.y),
                    vec2(value_box_width, available_size_rect.height()),
                ),
                DataValueBoxView::new(
                    self.app_context.clone(),
                    field_edit_value,
                    validation_data_type_ref,
                    self.valued_struct_field.get_is_read_only(),
                    !self.valued_struct_field.get_is_read_only(),
                    "",
                    &data_value_box_id,
                )
                .allow_read_only_interpretation(true)
                .display_values(self.field_display_values.unwrap_or(&[]))
                .use_preview_foreground(self.valued_struct_field.get_is_read_only())
                .width(value_box_width),
            );

            let commit_on_enter_pressed = DataValueBoxView::consume_commit_on_enter(user_interface, &data_value_box_id);

            if show_commit_button && commit_on_enter_pressed {
                Self::commit_field_edit(
                    self.valued_struct_field,
                    validation_data_type_ref,
                    field_edit_value,
                    self.struct_viewer_frame_action,
                );
            }

            if show_commit_button {
                let commit_response = user_interface.put(
                    Rect::from_min_size(
                        pos2(
                            row_max_x - commit_button_width - value_column_padding,
                            available_size_rect.min.y + value_column_padding,
                        ),
                        vec2(commit_button_width, available_size_rect.height() - value_column_padding * 2.0),
                    ),
                    Button::new_from_theme(theme)
                        .background_color(epaint::Color32::TRANSPARENT)
                        .with_tooltip_text("Commit value."),
                );

                IconDraw::draw(user_interface, commit_response.rect, &theme.icon_library.icon_handle_common_check_mark);

                if commit_response.clicked() {
                    Self::commit_field_edit(
                        self.valued_struct_field,
                        validation_data_type_ref,
                        field_edit_value,
                        self.struct_viewer_frame_action,
                    );
                }
            }
        }

        response
    }
}
