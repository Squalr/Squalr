use crate::{
    app_context::AppContext,
    ui::{
        converters::data_type_to_icon_converter::DataTypeToIconConverter,
        draw::icon_draw::IconDraw,
        widgets::controls::{
            button::Button,
            combo_box::{combo_box_item_view::ComboBoxItemView, combo_box_view::ComboBoxView},
            data_type_selector::{data_type_selection::DataTypeSelection, data_type_selector_view::DataTypeSelectorView},
            data_value_box::data_value_box_view::DataValueBoxView,
            state_layer::StateLayer,
        },
    },
    views::struct_viewer::view_data::{
        struct_viewer_container_mode::StructViewerContainerMode,
        struct_viewer_field_presentation::{StructViewerFieldEditorKind, StructViewerFieldPresentation},
        struct_viewer_frame_action::StructViewerFrameAction,
        struct_viewer_view_data::StructViewerViewData,
    },
};
use eframe::egui::{Align2, Response, Sense, Ui, Widget, vec2};
use epaint::{CornerRadius, Rect, Stroke, StrokeKind, pos2};
use squalr_engine_api::structures::{
    data_types::built_in_types::string::utf8::data_type_string_utf8::DataTypeStringUtf8,
    data_types::data_type_ref::DataTypeRef,
    data_values::anonymous_value_string::AnonymousValueString,
    pointer_scans::pointer_scan_pointer_size::PointerScanPointerSize,
    structs::symbolic_field_definition::SymbolicFieldDefinition,
    structs::valued_struct_field::{ValuedStructField, ValuedStructFieldData},
};
use std::sync::Arc;

pub struct StructViewerEntryView<'lifetime> {
    app_context: Arc<AppContext>,
    valued_struct_field: &'lifetime ValuedStructField,
    field_presentation: &'lifetime StructViewerFieldPresentation,
    row_index: usize,
    is_selected: bool,
    struct_viewer_frame_action: &'lifetime mut StructViewerFrameAction,
    field_edit_value: Option<&'lifetime mut AnonymousValueString>,
    field_display_values: Option<&'lifetime [AnonymousValueString]>,
    field_data_type_selection: Option<&'lifetime mut DataTypeSelection>,
    validation_data_type_ref: Option<&'lifetime DataTypeRef>,
    name_splitter_x: f32,
    value_splitter_x: f32,
}

impl<'lifetime> StructViewerEntryView<'lifetime> {
    const NATIVE_POINTER_SIZES: [PointerScanPointerSize; 4] = [
        PointerScanPointerSize::Pointer32,
        PointerScanPointerSize::Pointer32be,
        PointerScanPointerSize::Pointer64,
        PointerScanPointerSize::Pointer64be,
    ];

    fn trailing_commit_slot_width(
        commit_button_width: f32,
        value_column_padding: f32,
    ) -> f32 {
        commit_button_width + value_column_padding
    }

    pub fn new(
        app_context: Arc<AppContext>,
        valued_struct_field: &'lifetime ValuedStructField,
        field_presentation: &'lifetime StructViewerFieldPresentation,
        row_index: usize,
        is_selected: bool,
        struct_viewer_frame_action: &'lifetime mut StructViewerFrameAction,
        field_edit_value: Option<&'lifetime mut AnonymousValueString>,
        field_display_values: Option<&'lifetime [AnonymousValueString]>,
        field_data_type_selection: Option<&'lifetime mut DataTypeSelection>,
        validation_data_type_ref: Option<&'lifetime DataTypeRef>,
        name_splitter_x: f32,
        value_splitter_x: f32,
    ) -> Self {
        Self {
            app_context,
            valued_struct_field,
            field_presentation,
            row_index,
            is_selected,
            struct_viewer_frame_action,
            field_edit_value,
            field_display_values,
            field_data_type_selection,
            validation_data_type_ref,
            name_splitter_x,
            value_splitter_x,
        }
    }

    fn commit_field_edit(
        app_context: &Arc<AppContext>,
        valued_struct_field: &ValuedStructField,
        validation_data_type_ref: &DataTypeRef,
        field_edit_value: &AnonymousValueString,
        struct_viewer_frame_action: &mut StructViewerFrameAction,
    ) {
        match app_context
            .engine_unprivileged_state
            .deanonymize_value_string(validation_data_type_ref, field_edit_value)
        {
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

    fn commit_data_type_selection(
        valued_struct_field: &ValuedStructField,
        data_type_selection: &DataTypeSelection,
        struct_viewer_frame_action: &mut StructViewerFrameAction,
    ) {
        let mut edited_field = valued_struct_field.clone();
        let updated_symbolic_field_definition = StructViewerViewData::read_symbolic_field_definition_reference_from_field_set(valued_struct_field)
            .map(|symbolic_field_definition| {
                SymbolicFieldDefinition::new(data_type_selection.visible_data_type().clone(), symbolic_field_definition.get_container_type())
            })
            .map(|symbolic_field_definition| symbolic_field_definition.to_string())
            .unwrap_or_else(|| {
                data_type_selection
                    .visible_data_type()
                    .get_data_type_id()
                    .to_string()
            });
        let data_type_string_value = DataTypeStringUtf8::get_value_from_primitive_string(&updated_symbolic_field_definition);

        edited_field.set_field_data(ValuedStructFieldData::Value(data_type_string_value));
        *struct_viewer_frame_action = StructViewerFrameAction::EditValue(edited_field);
    }

    fn commit_container_type_selection(
        valued_struct_field: &ValuedStructField,
        container_mode: StructViewerContainerMode,
        struct_viewer_frame_action: &mut StructViewerFrameAction,
    ) {
        let edited_field = DataTypeStringUtf8::get_value_from_primitive_string(container_mode.label())
            .to_named_valued_struct_field(valued_struct_field.get_name().to_string(), false);

        *struct_viewer_frame_action = StructViewerFrameAction::EditValue(edited_field);
    }

    fn commit_project_item_pointer_size_selection(
        valued_struct_field: &ValuedStructField,
        pointer_size_label: &str,
        struct_viewer_frame_action: &mut StructViewerFrameAction,
    ) {
        let edited_field = DataTypeStringUtf8::get_value_from_primitive_string(pointer_size_label)
            .to_named_valued_struct_field(valued_struct_field.get_name().to_string(), false);

        *struct_viewer_frame_action = StructViewerFrameAction::EditValue(edited_field);
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

        // Click handling.
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
        let commit_button_space = Self::trailing_commit_slot_width(commit_button_width, value_column_padding);
        let value_box_width = (row_max_x - value_box_position_x - commit_button_space).max(0.0);
        let available_data_type_refs = self
            .app_context
            .engine_unprivileged_state
            .get_registered_data_type_refs();

        // Draw icon.
        let icon_rect = Rect::from_min_max(
            pos2(icon_position_x, available_size_rect.min.y),
            pos2(name_position_x, available_size_rect.max.y),
        );
        let icon_center = icon_rect.center();
        let icon_data_type_id = match (self.field_presentation.editor_kind(), self.field_data_type_selection.as_ref()) {
            (StructViewerFieldEditorKind::DataTypeSelector, Some(field_data_type_selection)) => {
                field_data_type_selection.visible_data_type().get_data_type_id()
            }
            _ => self.valued_struct_field.get_icon_id(),
        };
        let icon = DataTypeToIconConverter::convert_data_type_to_icon(icon_data_type_id, &theme.icon_library);

        IconDraw::draw_sized(user_interface, icon_center, icon_size, &icon);

        // Draw text.
        let text_rectangle = Rect::from_min_max(
            pos2(name_position_x, available_size_rect.min.y),
            pos2(value_position_x, available_size_rect.max.y),
        );
        let text_pos = pos2(text_rectangle.min.x + text_left_padding, text_rectangle.center().y);

        let text_painter = user_interface
            .painter()
            .with_clip_rect(text_rectangle.shrink2(vec2(text_left_padding, 0.0)));

        text_painter.text(
            text_pos,
            Align2::LEFT_CENTER,
            self.field_presentation.display_name(),
            theme.font_library.font_noto_sans.font_normal.clone(),
            theme.foreground,
        );

        match self.field_presentation.editor_kind() {
            StructViewerFieldEditorKind::ValueBox => {
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
                            &self.app_context,
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
                                &self.app_context,
                                self.valued_struct_field,
                                validation_data_type_ref,
                                field_edit_value,
                                self.struct_viewer_frame_action,
                            );
                        }
                    }
                }
            }
            StructViewerFieldEditorKind::ProjectItemPointerOffsetsEditor => {
                let edit_button_width = 28.0;
                let edit_button_position_x = row_max_x - edit_button_width;
                let offsets_preview_width = (edit_button_position_x - value_position_x).max(0.0);

                if let (Some(field_edit_value), Some(validation_data_type_ref)) = (self.field_edit_value, self.validation_data_type_ref) {
                    let data_value_box_id = format!("struct_viewer_pointer_offsets_{}_{}", self.row_index, self.valued_struct_field.get_name());
                    user_interface.put(
                        Rect::from_min_size(
                            pos2(value_position_x, available_size_rect.min.y),
                            vec2(offsets_preview_width, available_size_rect.height()),
                        ),
                        DataValueBoxView::new(
                            self.app_context.clone(),
                            field_edit_value,
                            validation_data_type_ref,
                            true,
                            false,
                            "",
                            &data_value_box_id,
                        )
                        .allow_read_only_interpretation(true)
                        .use_preview_foreground(true)
                        .width(offsets_preview_width),
                    );
                }

                let edit_response = user_interface.put(
                    Rect::from_min_size(
                        pos2(edit_button_position_x, available_size_rect.min.y),
                        vec2(edit_button_width, available_size_rect.height()),
                    ),
                    Button::new_from_theme(theme)
                        .background_color(epaint::Color32::TRANSPARENT)
                        .with_tooltip_text("Edit offsets."),
                );

                IconDraw::draw(user_interface, edit_response.rect, &theme.icon_library.icon_handle_common_edit);

                if edit_response.clicked() {
                    *self.struct_viewer_frame_action = StructViewerFrameAction::RequestFieldEditor(self.valued_struct_field.clone());
                }
            }
            StructViewerFieldEditorKind::MemoryViewerButton => {
                let button_rect = Rect::from_min_size(
                    pos2(value_box_position_x, available_size_rect.min.y + value_column_padding),
                    vec2(value_box_width, available_size_rect.height() - value_column_padding * 2.0),
                );
                let button_response = user_interface.put(
                    button_rect,
                    Button::new_from_theme(theme).with_tooltip_text("Open this value in the Memory Viewer."),
                );

                user_interface.painter().text(
                    button_response.rect.center(),
                    Align2::CENTER_CENTER,
                    "Edit in Memory Viewer",
                    theme.font_library.font_noto_sans.font_normal.clone(),
                    theme.foreground,
                );

                if button_response.clicked() {
                    *self.struct_viewer_frame_action = StructViewerFrameAction::OpenInMemoryViewer(self.valued_struct_field.get_name().to_string());
                }
            }
            StructViewerFieldEditorKind::CodeViewerButton => {
                let button_rect = Rect::from_min_size(
                    pos2(value_box_position_x, available_size_rect.min.y + value_column_padding),
                    vec2(value_box_width, available_size_rect.height() - value_column_padding * 2.0),
                );
                let button_response = user_interface.put(
                    button_rect,
                    Button::new_from_theme(theme).with_tooltip_text("Open this value in the Code Viewer."),
                );

                user_interface.painter().text(
                    button_response.rect.center(),
                    Align2::CENTER_CENTER,
                    "Edit in Code Viewer",
                    theme.font_library.font_noto_sans.font_normal.clone(),
                    theme.foreground,
                );

                if button_response.clicked() {
                    *self.struct_viewer_frame_action = StructViewerFrameAction::OpenInCodeViewer(self.valued_struct_field.get_name().to_string());
                }
            }
            StructViewerFieldEditorKind::DataTypeSelector => {
                if let Some(field_data_type_selection) = self.field_data_type_selection {
                    let previous_data_type_ref = field_data_type_selection.visible_data_type().clone();
                    let data_type_selector_id = format!("struct_viewer_data_type_{}_{}", self.row_index, self.valued_struct_field.get_name());

                    // Reserve space for checkbox (fixed 28px), data type selector takes natural width.
                    let trailing_checkbox_space = Self::trailing_commit_slot_width(commit_button_width, value_column_padding);
                    let available_for_selectors = (row_max_x - value_box_position_x - trailing_checkbox_space).max(0.0);

                    user_interface.put(
                        Rect::from_min_size(
                            pos2(value_box_position_x, available_size_rect.min.y),
                            vec2(available_for_selectors, available_size_rect.height()),
                        ),
                        DataTypeSelectorView::new(self.app_context.clone(), field_data_type_selection, &data_type_selector_id)
                            .available_data_types(available_data_type_refs.clone())
                            .width(available_for_selectors)
                            .height(available_size_rect.height()),
                    );

                    let selected_data_type_ref = field_data_type_selection.visible_data_type().clone();
                    field_data_type_selection.replace_selected_data_types(vec![selected_data_type_ref.clone()]);

                    if previous_data_type_ref != selected_data_type_ref {
                        Self::commit_data_type_selection(self.valued_struct_field, field_data_type_selection, self.struct_viewer_frame_action);
                    }
                }
            }
            StructViewerFieldEditorKind::ContainerTypeSelector => {
                let container_selector_id = format!("struct_viewer_container_type_{}_{}", self.row_index, self.valued_struct_field.get_name());
                let current_container_mode = StructViewerViewData::read_utf8_field_text(self.valued_struct_field)
                    .parse::<StructViewerContainerMode>()
                    .unwrap_or(StructViewerContainerMode::Element);
                let mut selected_container_mode = None;

                // Reserve space for checkbox, container selector takes natural width.
                let trailing_checkbox_space = Self::trailing_commit_slot_width(commit_button_width, value_column_padding);
                let container_width = (row_max_x - value_box_position_x - trailing_checkbox_space).max(0.0);

                user_interface.put(
                    Rect::from_min_size(
                        pos2(value_box_position_x, available_size_rect.min.y),
                        vec2(container_width, available_size_rect.height()),
                    ),
                    ComboBoxView::new(
                        self.app_context.clone(),
                        current_container_mode.label(),
                        &container_selector_id,
                        None,
                        |popup_user_interface: &mut Ui, should_close: &mut bool| {
                            for container_mode in StructViewerContainerMode::ALL {
                                let container_mode_response =
                                    popup_user_interface.add(ComboBoxItemView::new(self.app_context.clone(), container_mode.label(), None, container_width));

                                if container_mode_response.clicked() {
                                    selected_container_mode = Some(container_mode);
                                    *should_close = true;
                                }
                            }
                        },
                    )
                    .width(container_width)
                    .height(available_size_rect.height()),
                );

                if let Some(selected_container_mode) = selected_container_mode {
                    Self::commit_container_type_selection(self.valued_struct_field, selected_container_mode, self.struct_viewer_frame_action);
                }
            }
            StructViewerFieldEditorKind::ProjectItemPointerSizeSelector => {
                let pointer_size_selector_id = format!(
                    "struct_viewer_project_item_pointer_size_{}_{}",
                    self.row_index,
                    self.valued_struct_field.get_name()
                );
                let current_pointer_size = StructViewerViewData::read_utf8_field_text(self.valued_struct_field);
                let pointer_size_label = if current_pointer_size.trim().is_empty() {
                    "None"
                } else {
                    current_pointer_size.as_str()
                };
                let mut selected_pointer_size_label = None;
                let trailing_checkbox_space = Self::trailing_commit_slot_width(commit_button_width, value_column_padding);
                let pointer_size_width = (row_max_x - value_box_position_x - trailing_checkbox_space).max(0.0);

                user_interface.put(
                    Rect::from_min_size(
                        pos2(value_box_position_x, available_size_rect.min.y),
                        vec2(pointer_size_width, available_size_rect.height()),
                    ),
                    ComboBoxView::new(
                        self.app_context.clone(),
                        pointer_size_label,
                        &pointer_size_selector_id,
                        None,
                        |popup_user_interface: &mut Ui, should_close: &mut bool| {
                            let none_response = popup_user_interface.add(ComboBoxItemView::new(self.app_context.clone(), "None", None, pointer_size_width));

                            if none_response.clicked() {
                                selected_pointer_size_label = Some("None".to_string());
                                *should_close = true;
                            }

                            for pointer_size in Self::NATIVE_POINTER_SIZES {
                                let pointer_size_label = pointer_size.to_string();
                                let pointer_size_response =
                                    popup_user_interface.add(ComboBoxItemView::new(self.app_context.clone(), &pointer_size_label, None, pointer_size_width));

                                if pointer_size_response.clicked() {
                                    selected_pointer_size_label = Some(pointer_size_label);
                                    *should_close = true;
                                }
                            }
                        },
                    )
                    .width(pointer_size_width)
                    .height(available_size_rect.height()),
                );

                if let Some(selected_pointer_size_label) = selected_pointer_size_label {
                    Self::commit_project_item_pointer_size_selection(self.valued_struct_field, &selected_pointer_size_label, self.struct_viewer_frame_action);
                }
            }
        }

        response
    }
}
