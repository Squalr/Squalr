use crate::{
    app_context::AppContext,
    ui::{
        converters::{data_type_to_icon_converter::DataTypeToIconConverter, scan_compare_type_to_icon_converter::ScanCompareTypeToIconConverter},
        draw::icon_draw::IconDraw,
        widgets::controls::{
            button::Button,
            combo_box::{combo_box_item_view::ComboBoxItemView, combo_box_view::ComboBoxView},
            data_type_selector::{data_type_selection::DataTypeSelection, data_type_selector_view::DataTypeSelectorView},
            data_value_box::data_value_box_view::DataValueBoxView,
            search_box::SearchBoxView,
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
use eframe::egui::{Align2, Id, Response, ScrollArea, Sense, TextureHandle, Ui, Widget, vec2};
use epaint::{CornerRadius, Rect, Stroke, StrokeKind, pos2};
use squalr_engine_api::{
    engine::engine_execution_context::EngineExecutionContext,
    structures::{
        data_types::built_in_types::string::utf8::data_type_string_utf8::DataTypeStringUtf8,
        data_types::data_type_ref::DataTypeRef,
        data_values::anonymous_value_string::AnonymousValueString,
        pointer_scans::pointer_scan_pointer_size::PointerScanPointerSize,
        projects::project_symbol_catalog::ProjectSymbolCatalog,
        scanning::comparisons::{scan_compare_type_delta::ScanCompareTypeDelta, scan_compare_type_immediate::ScanCompareTypeImmediate},
        structs::valued_struct_field::{ValuedStructField, ValuedStructFieldData},
        structs::{
            symbolic_field_definition::SymbolicFieldDefinition, symbolic_resolver_definition::SymbolicResolverBinaryOperator,
            symbolic_struct_definition::SymbolicLayoutKind,
        },
    },
};
use std::{str::FromStr, sync::Arc};

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
    const SYMBOL_RESOLVER_NODE_KIND_LABELS: [&'static str; 9] = [
        "Literal",
        "Local Field",
        "Relative Symbol Field",
        "Global Symbol Field",
        "Relative Pointer Chain",
        "Global Pointer Chain",
        "Type Size",
        "Operation",
        "Conditional",
    ];
    const SYMBOL_LAYOUT_FIELD_ELEMENT_TYPE_LABELS: [&'static str; 2] = ["Data Type", "Symbol Layout"];
    const SYMBOL_LAYOUT_FIELD_CONTAINER_KIND_LABELS: [&'static str; 7] = [
        "Element",
        "Array",
        "Fixed Array",
        "Dynamic Array",
        "Pointer",
        "Fixed Pointer Array",
        "Dynamic Pointer Array",
    ];
    const SYMBOL_LAYOUT_FIELD_OFFSET_MODE_LABELS: [&'static str; 3] = ["Sequential", "Static", "Resolver"];
    const SEARCHABLE_SELECTOR_POPUP_DEFAULT_WIDTH: f32 = 240.0;
    const SEARCHABLE_SELECTOR_POPUP_MAX_WIDTH: f32 = 640.0;
    const SEARCHABLE_SELECTOR_POPUP_HORIZONTAL_PADDING: f32 = 56.0;
    const SEARCHABLE_SELECTOR_VISIBLE_ROW_COUNT: usize = 12;
    const COMBO_BOX_ROW_HEIGHT: f32 = 28.0;

    fn value_box_position_x(
        value_position_x: f32,
        value_column_padding: f32,
    ) -> f32 {
        value_position_x + value_column_padding
    }

    fn trailing_action_slot_width(
        action_button_width: f32,
        value_column_padding: f32,
    ) -> f32 {
        action_button_width + value_column_padding
    }

    fn value_box_width(
        row_max_x: f32,
        value_box_position_x: f32,
        action_button_width: f32,
        value_column_padding: f32,
    ) -> f32 {
        let trailing_action_space = Self::trailing_action_slot_width(action_button_width, value_column_padding);

        (row_max_x - value_box_position_x - trailing_action_space).max(0.0)
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
                    .with_active_when_resolver(symbolic_field_definition.get_active_when_resolver().cloned())
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

    fn commit_symbol_resolver_data_type_selection(
        valued_struct_field: &ValuedStructField,
        data_type_selection: &DataTypeSelection,
        struct_viewer_frame_action: &mut StructViewerFrameAction,
    ) {
        let edited_field = DataTypeStringUtf8::get_value_from_primitive_string(data_type_selection.visible_data_type().get_data_type_id())
            .to_named_valued_struct_field(valued_struct_field.get_name().to_string(), false);

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

    fn commit_symbol_resolver_text_selection(
        valued_struct_field: &ValuedStructField,
        selected_label: &str,
        struct_viewer_frame_action: &mut StructViewerFrameAction,
    ) {
        let edited_field =
            DataTypeStringUtf8::get_value_from_primitive_string(selected_label).to_named_valued_struct_field(valued_struct_field.get_name().to_string(), false);

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

    fn project_item_pointer_size_data_type_ref(pointer_size_label: &str) -> Option<DataTypeRef> {
        let pointer_size_label = pointer_size_label.trim();

        if pointer_size_label.is_empty() {
            return None;
        }

        PointerScanPointerSize::from_str(pointer_size_label)
            .ok()
            .map(|pointer_size| pointer_size.to_data_type_ref())
    }

    fn project_item_pointer_size_icon(
        app_context: &Arc<AppContext>,
        pointer_size_label: &str,
    ) -> Option<TextureHandle> {
        Self::project_item_pointer_size_data_type_ref(pointer_size_label)
            .map(|data_type_ref| DataTypeToIconConverter::convert_data_type_to_icon(data_type_ref.get_data_type_id(), &app_context.theme.icon_library))
    }

    fn symbol_resolver_operator_icon(
        app_context: &Arc<AppContext>,
        operator: SymbolicResolverBinaryOperator,
    ) -> Option<TextureHandle> {
        let icon_library = &app_context.theme.icon_library;

        match operator {
            SymbolicResolverBinaryOperator::Add => Some(ScanCompareTypeToIconConverter::convert_scan_compare_type_delta_to_icon(
                &ScanCompareTypeDelta::IncreasedByX,
                icon_library,
            )),
            SymbolicResolverBinaryOperator::Subtract => Some(ScanCompareTypeToIconConverter::convert_scan_compare_type_delta_to_icon(
                &ScanCompareTypeDelta::DecreasedByX,
                icon_library,
            )),
            SymbolicResolverBinaryOperator::Multiply => Some(ScanCompareTypeToIconConverter::convert_scan_compare_type_delta_to_icon(
                &ScanCompareTypeDelta::MultipliedByX,
                icon_library,
            )),
            SymbolicResolverBinaryOperator::Divide => Some(ScanCompareTypeToIconConverter::convert_scan_compare_type_delta_to_icon(
                &ScanCompareTypeDelta::DividedByX,
                icon_library,
            )),
            SymbolicResolverBinaryOperator::Modulo => Some(ScanCompareTypeToIconConverter::convert_scan_compare_type_delta_to_icon(
                &ScanCompareTypeDelta::ModuloByX,
                icon_library,
            )),
            SymbolicResolverBinaryOperator::BitwiseAnd => Some(ScanCompareTypeToIconConverter::convert_scan_compare_type_delta_to_icon(
                &ScanCompareTypeDelta::LogicalAndByX,
                icon_library,
            )),
            SymbolicResolverBinaryOperator::BitwiseOr => Some(ScanCompareTypeToIconConverter::convert_scan_compare_type_delta_to_icon(
                &ScanCompareTypeDelta::LogicalOrByX,
                icon_library,
            )),
            SymbolicResolverBinaryOperator::BitwiseXor => Some(ScanCompareTypeToIconConverter::convert_scan_compare_type_delta_to_icon(
                &ScanCompareTypeDelta::LogicalXorByX,
                icon_library,
            )),
            SymbolicResolverBinaryOperator::ShiftLeft => Some(ScanCompareTypeToIconConverter::convert_scan_compare_type_delta_to_icon(
                &ScanCompareTypeDelta::ShiftLeftByX,
                icon_library,
            )),
            SymbolicResolverBinaryOperator::ShiftRight => Some(ScanCompareTypeToIconConverter::convert_scan_compare_type_delta_to_icon(
                &ScanCompareTypeDelta::ShiftRightByX,
                icon_library,
            )),
            SymbolicResolverBinaryOperator::Equal => Some(ScanCompareTypeToIconConverter::convert_scan_compare_type_immediate_to_icon(
                &ScanCompareTypeImmediate::Equal,
                icon_library,
            )),
            SymbolicResolverBinaryOperator::NotEqual => Some(ScanCompareTypeToIconConverter::convert_scan_compare_type_immediate_to_icon(
                &ScanCompareTypeImmediate::NotEqual,
                icon_library,
            )),
            SymbolicResolverBinaryOperator::LessThan => Some(ScanCompareTypeToIconConverter::convert_scan_compare_type_immediate_to_icon(
                &ScanCompareTypeImmediate::LessThan,
                icon_library,
            )),
            SymbolicResolverBinaryOperator::LessThanOrEqual => Some(ScanCompareTypeToIconConverter::convert_scan_compare_type_immediate_to_icon(
                &ScanCompareTypeImmediate::LessThanOrEqual,
                icon_library,
            )),
            SymbolicResolverBinaryOperator::GreaterThan => Some(ScanCompareTypeToIconConverter::convert_scan_compare_type_immediate_to_icon(
                &ScanCompareTypeImmediate::GreaterThan,
                icon_library,
            )),
            SymbolicResolverBinaryOperator::GreaterThanOrEqual => Some(ScanCompareTypeToIconConverter::convert_scan_compare_type_immediate_to_icon(
                &ScanCompareTypeImmediate::GreaterThanOrEqual,
                icon_library,
            )),
            SymbolicResolverBinaryOperator::Minimum | SymbolicResolverBinaryOperator::Maximum => None,
        }
    }

    fn get_opened_project_symbol_catalog(app_context: &Arc<AppContext>) -> Option<ProjectSymbolCatalog> {
        let opened_project_lock = app_context
            .engine_unprivileged_state
            .get_project_manager()
            .get_opened_project();
        let opened_project_guard = opened_project_lock.read().ok()?;

        opened_project_guard.as_ref().map(|opened_project| {
            opened_project
                .get_project_info()
                .get_project_symbol_catalog()
                .clone()
        })
    }

    fn selector_label_matches_filter(
        label: &str,
        filter_text: &str,
    ) -> bool {
        let trimmed_filter_text = filter_text.trim();

        trimmed_filter_text.is_empty()
            || label
                .to_ascii_lowercase()
                .contains(&trimmed_filter_text.to_ascii_lowercase())
    }

    fn filter_symbol_layout_layouts_from_data_type_refs(
        app_context: &Arc<AppContext>,
        data_type_refs: &[DataTypeRef],
    ) -> Vec<DataTypeRef> {
        let Some(project_symbol_catalog) = Self::get_opened_project_symbol_catalog(app_context) else {
            return data_type_refs.to_vec();
        };

        data_type_refs
            .iter()
            .filter(|data_type_ref| {
                !project_symbol_catalog
                    .get_struct_layout_descriptors()
                    .iter()
                    .any(|struct_layout_descriptor| struct_layout_descriptor.get_struct_layout_id() == data_type_ref.get_data_type_id())
            })
            .cloned()
            .collect()
    }

    fn selector_text_width(
        user_interface: &Ui,
        app_context: &Arc<AppContext>,
        text: &str,
    ) -> f32 {
        let theme = &app_context.theme;

        user_interface.ctx().fonts_mut(|fonts| {
            fonts
                .layout_no_wrap(text.to_string(), theme.font_library.font_noto_sans.font_normal.clone(), theme.foreground)
                .size()
                .x
        })
    }

    fn searchable_selector_popup_width<'label>(
        user_interface: &Ui,
        app_context: &Arc<AppContext>,
        labels: impl Iterator<Item = &'label str>,
        search_placeholder: &str,
        empty_message: &str,
    ) -> f32 {
        let widest_label_width = labels
            .map(|label| Self::selector_text_width(user_interface, app_context, label))
            .fold(0.0, f32::max);
        let search_placeholder_width = Self::selector_text_width(user_interface, app_context, search_placeholder);
        let empty_message_width = Self::selector_text_width(user_interface, app_context, empty_message);
        let widest_content_width = widest_label_width
            .max(search_placeholder_width)
            .max(empty_message_width);
        let content_width = widest_content_width + Self::SEARCHABLE_SELECTOR_POPUP_HORIZONTAL_PADDING;

        Self::SEARCHABLE_SELECTOR_POPUP_DEFAULT_WIDTH
            .max(content_width)
            .min(Self::SEARCHABLE_SELECTOR_POPUP_MAX_WIDTH)
    }

    fn symbol_layout_selector_popup_width(
        user_interface: &Ui,
        app_context: &Arc<AppContext>,
        project_symbol_catalog: &ProjectSymbolCatalog,
    ) -> f32 {
        Self::searchable_selector_popup_width(
            user_interface,
            app_context,
            project_symbol_catalog
                .get_struct_layout_descriptors()
                .iter()
                .map(|struct_layout_descriptor| struct_layout_descriptor.get_struct_layout_id()),
            "Search structs",
            "No matching structs",
        )
    }

    fn resolver_selector_popup_width(
        user_interface: &Ui,
        app_context: &Arc<AppContext>,
        project_symbol_catalog: &ProjectSymbolCatalog,
    ) -> f32 {
        Self::searchable_selector_popup_width(
            user_interface,
            app_context,
            project_symbol_catalog
                .get_symbolic_resolver_descriptors()
                .iter()
                .map(|resolver_descriptor| resolver_descriptor.get_resolver_id()),
            "Search resolvers",
            "No matching resolvers",
        )
    }

    fn searchable_selector_scroll_height(visible_label_count: usize) -> f32 {
        let visible_row_count = visible_label_count
            .max(1)
            .min(Self::SEARCHABLE_SELECTOR_VISIBLE_ROW_COUNT);

        visible_row_count as f32 * Self::COMBO_BOX_ROW_HEIGHT
    }

    fn render_combo_message_row(
        &self,
        user_interface: &mut Ui,
        width: f32,
        message: &str,
    ) {
        let theme = &self.app_context.theme;
        let (message_rect, _) = user_interface.allocate_exact_size(vec2(width.max(1.0), Self::COMBO_BOX_ROW_HEIGHT), Sense::hover());

        user_interface.painter().text(
            pos2(message_rect.left() + 8.0, message_rect.center().y),
            Align2::LEFT_CENTER,
            message,
            theme.font_library.font_noto_sans.font_normal.clone(),
            theme.foreground_preview,
        );
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
        let value_box_position_x = Self::value_box_position_x(value_position_x, value_column_padding);
        let value_box_width = Self::value_box_width(row_max_x, value_box_position_x, commit_button_width, value_column_padding);
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
            (StructViewerFieldEditorKind::DataTypeSelector, Some(field_data_type_selection))
            | (StructViewerFieldEditorKind::SymbolResolverDataTypeSelector, Some(field_data_type_selection))
            | (StructViewerFieldEditorKind::SymbolLayoutFieldDataTypeSelector, Some(field_data_type_selection)) => field_data_type_selection
                .visible_data_type()
                .get_data_type_id()
                .to_string(),
            (StructViewerFieldEditorKind::SymbolLayoutFieldSymbolLayoutSelector, _) => StructViewerViewData::read_utf8_field_text(self.valued_struct_field),
            _ => self.valued_struct_field.get_icon_id().to_string(),
        };
        let icon = DataTypeToIconConverter::convert_data_type_to_icon(icon_data_type_id.as_ref(), &theme.icon_library);

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
                let edit_button_position_x = row_max_x - edit_button_width - value_column_padding;
                let offsets_preview_width = Self::value_box_width(row_max_x, value_box_position_x, edit_button_width, value_column_padding);

                if let (Some(field_edit_value), Some(validation_data_type_ref)) = (self.field_edit_value, self.validation_data_type_ref) {
                    let data_value_box_id = format!("struct_viewer_pointer_offsets_{}_{}", self.row_index, self.valued_struct_field.get_name());
                    user_interface.put(
                        Rect::from_min_size(
                            pos2(value_box_position_x, available_size_rect.min.y),
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
                        pos2(edit_button_position_x, available_size_rect.min.y + value_column_padding),
                        vec2(edit_button_width, available_size_rect.height() - value_column_padding * 2.0),
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
            StructViewerFieldEditorKind::DataTypeSelector
            | StructViewerFieldEditorKind::SymbolResolverDataTypeSelector
            | StructViewerFieldEditorKind::SymbolLayoutFieldDataTypeSelector => {
                if let Some(field_data_type_selection) = self.field_data_type_selection {
                    let previous_data_type_ref = field_data_type_selection.visible_data_type().clone();
                    let data_type_selector_id = format!("struct_viewer_data_type_{}_{}", self.row_index, self.valued_struct_field.get_name());

                    // Reserve space for checkbox (fixed 28px), data type selector takes natural width.
                    let trailing_checkbox_space = Self::trailing_action_slot_width(commit_button_width, value_column_padding);
                    let available_for_selectors = (row_max_x - value_box_position_x - trailing_checkbox_space).max(0.0);

                    user_interface.put(
                        Rect::from_min_size(
                            pos2(value_box_position_x, available_size_rect.min.y),
                            vec2(available_for_selectors, available_size_rect.height()),
                        ),
                        DataTypeSelectorView::new(self.app_context.clone(), field_data_type_selection, &data_type_selector_id)
                            .with_label_tooltip()
                            .available_data_types(
                                if matches!(
                                    self.field_presentation.editor_kind(),
                                    StructViewerFieldEditorKind::SymbolLayoutFieldDataTypeSelector
                                ) {
                                    Self::filter_symbol_layout_layouts_from_data_type_refs(&self.app_context, &available_data_type_refs)
                                } else {
                                    available_data_type_refs.clone()
                                },
                            )
                            .width(available_for_selectors)
                            .height(available_size_rect.height()),
                    );

                    let selected_data_type_ref = field_data_type_selection.visible_data_type().clone();
                    field_data_type_selection.replace_selected_data_types(vec![selected_data_type_ref.clone()]);

                    if previous_data_type_ref != selected_data_type_ref {
                        if matches!(
                            self.field_presentation.editor_kind(),
                            StructViewerFieldEditorKind::SymbolResolverDataTypeSelector | StructViewerFieldEditorKind::SymbolLayoutFieldDataTypeSelector
                        ) {
                            Self::commit_symbol_resolver_data_type_selection(
                                self.valued_struct_field,
                                field_data_type_selection,
                                self.struct_viewer_frame_action,
                            );
                        } else {
                            Self::commit_data_type_selection(self.valued_struct_field, field_data_type_selection, self.struct_viewer_frame_action);
                        }
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
                let trailing_checkbox_space = Self::trailing_action_slot_width(commit_button_width, value_column_padding);
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
                    .with_label_tooltip()
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
                    "u64"
                } else {
                    current_pointer_size.as_str()
                };
                let pointer_size_icon = Self::project_item_pointer_size_icon(&self.app_context, pointer_size_label);
                let mut selected_pointer_size_label = None;
                let trailing_checkbox_space = Self::trailing_action_slot_width(commit_button_width, value_column_padding);
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
                        pointer_size_icon,
                        |popup_user_interface: &mut Ui, should_close: &mut bool| {
                            for pointer_size in Self::NATIVE_POINTER_SIZES {
                                let pointer_size_label = pointer_size.to_string();
                                let pointer_size_icon = Self::project_item_pointer_size_icon(&self.app_context, &pointer_size_label);
                                let pointer_size_response = popup_user_interface.add(ComboBoxItemView::new(
                                    self.app_context.clone(),
                                    &pointer_size_label,
                                    pointer_size_icon,
                                    pointer_size_width,
                                ));

                                if pointer_size_response.clicked() {
                                    selected_pointer_size_label = Some(pointer_size_label);
                                    *should_close = true;
                                }
                            }
                        },
                    )
                    .with_label_tooltip()
                    .width(pointer_size_width)
                    .height(available_size_rect.height()),
                );

                if let Some(selected_pointer_size_label) = selected_pointer_size_label {
                    Self::commit_project_item_pointer_size_selection(self.valued_struct_field, &selected_pointer_size_label, self.struct_viewer_frame_action);
                }
            }
            StructViewerFieldEditorKind::SymbolResolverNodeKindSelector => {
                let node_kind_selector_id = format!(
                    "struct_viewer_symbol_resolver_node_kind_{}_{}",
                    self.row_index,
                    self.valued_struct_field.get_name()
                );
                let current_node_kind = StructViewerViewData::read_utf8_field_text(self.valued_struct_field);
                let node_kind_label = Self::SYMBOL_RESOLVER_NODE_KIND_LABELS
                    .iter()
                    .copied()
                    .find(|candidate_label| *candidate_label == current_node_kind)
                    .unwrap_or("Literal");
                let mut selected_node_kind_label = None;
                let trailing_checkbox_space = Self::trailing_action_slot_width(commit_button_width, value_column_padding);
                let node_kind_width = (row_max_x - value_box_position_x - trailing_checkbox_space).max(0.0);

                user_interface.put(
                    Rect::from_min_size(
                        pos2(value_box_position_x, available_size_rect.min.y),
                        vec2(node_kind_width, available_size_rect.height()),
                    ),
                    ComboBoxView::new(
                        self.app_context.clone(),
                        node_kind_label,
                        &node_kind_selector_id,
                        None,
                        |popup_user_interface: &mut Ui, should_close: &mut bool| {
                            for candidate_label in Self::SYMBOL_RESOLVER_NODE_KIND_LABELS {
                                let node_kind_response =
                                    popup_user_interface.add(ComboBoxItemView::new(self.app_context.clone(), candidate_label, None, node_kind_width));

                                if node_kind_response.clicked() {
                                    selected_node_kind_label = Some(candidate_label);
                                    *should_close = true;
                                }
                            }
                        },
                    )
                    .with_label_tooltip()
                    .width(node_kind_width)
                    .height(available_size_rect.height()),
                );

                if let Some(selected_node_kind_label) = selected_node_kind_label {
                    Self::commit_symbol_resolver_text_selection(self.valued_struct_field, selected_node_kind_label, self.struct_viewer_frame_action);
                }
            }
            StructViewerFieldEditorKind::SymbolResolverOperatorSelector => {
                let operator_selector_id = format!(
                    "struct_viewer_symbol_resolver_operator_{}_{}",
                    self.row_index,
                    self.valued_struct_field.get_name()
                );
                let current_operator = StructViewerViewData::read_utf8_field_text(self.valued_struct_field);
                let operator_label = SymbolicResolverBinaryOperator::ALL
                    .iter()
                    .copied()
                    .map(SymbolicResolverBinaryOperator::label)
                    .find(|candidate_label| *candidate_label == current_operator)
                    .unwrap_or(SymbolicResolverBinaryOperator::Add.label());
                let current_operator_icon = SymbolicResolverBinaryOperator::ALL
                    .iter()
                    .copied()
                    .find(|candidate_operator| candidate_operator.label() == operator_label)
                    .and_then(|candidate_operator| Self::symbol_resolver_operator_icon(&self.app_context, candidate_operator));
                let mut selected_operator_label = None;
                let trailing_checkbox_space = Self::trailing_action_slot_width(commit_button_width, value_column_padding);
                let operator_width = (row_max_x - value_box_position_x - trailing_checkbox_space).max(0.0);

                user_interface.put(
                    Rect::from_min_size(
                        pos2(value_box_position_x, available_size_rect.min.y),
                        vec2(operator_width, available_size_rect.height()),
                    ),
                    ComboBoxView::new(
                        self.app_context.clone(),
                        operator_label,
                        &operator_selector_id,
                        current_operator_icon,
                        |popup_user_interface: &mut Ui, should_close: &mut bool| {
                            for candidate_operator in SymbolicResolverBinaryOperator::ALL {
                                let candidate_label = candidate_operator.label();
                                let candidate_icon = Self::symbol_resolver_operator_icon(&self.app_context, candidate_operator);
                                let operator_response =
                                    popup_user_interface.add(ComboBoxItemView::new(self.app_context.clone(), candidate_label, candidate_icon, operator_width));

                                if operator_response.clicked() {
                                    selected_operator_label = Some(candidate_label);
                                    *should_close = true;
                                }
                            }
                        },
                    )
                    .with_label_tooltip()
                    .width(operator_width)
                    .height(available_size_rect.height()),
                );

                if let Some(selected_operator_label) = selected_operator_label {
                    Self::commit_symbol_resolver_text_selection(self.valued_struct_field, selected_operator_label, self.struct_viewer_frame_action);
                }
            }
            StructViewerFieldEditorKind::SymbolLayoutFieldElementTypeSelector => {
                let element_type_selector_id = format!(
                    "struct_viewer_symbol_layout_field_element_type_{}_{}",
                    self.row_index,
                    self.valued_struct_field.get_name()
                );
                let current_element_type = StructViewerViewData::read_utf8_field_text(self.valued_struct_field);
                let element_type_label = Self::SYMBOL_LAYOUT_FIELD_ELEMENT_TYPE_LABELS
                    .iter()
                    .copied()
                    .find(|candidate_label| *candidate_label == current_element_type)
                    .unwrap_or("Data Type");
                let mut selected_element_type_label = None;
                let trailing_checkbox_space = Self::trailing_action_slot_width(commit_button_width, value_column_padding);
                let element_type_width = (row_max_x - value_box_position_x - trailing_checkbox_space).max(0.0);

                user_interface.put(
                    Rect::from_min_size(
                        pos2(value_box_position_x, available_size_rect.min.y),
                        vec2(element_type_width, available_size_rect.height()),
                    ),
                    ComboBoxView::new(
                        self.app_context.clone(),
                        element_type_label,
                        &element_type_selector_id,
                        None,
                        |popup_user_interface: &mut Ui, should_close: &mut bool| {
                            for candidate_label in Self::SYMBOL_LAYOUT_FIELD_ELEMENT_TYPE_LABELS {
                                let element_type_response =
                                    popup_user_interface.add(ComboBoxItemView::new(self.app_context.clone(), candidate_label, None, element_type_width));

                                if element_type_response.clicked() {
                                    selected_element_type_label = Some(candidate_label);
                                    *should_close = true;
                                }
                            }
                        },
                    )
                    .with_label_tooltip()
                    .width(element_type_width)
                    .height(available_size_rect.height()),
                );

                if let Some(selected_element_type_label) = selected_element_type_label {
                    Self::commit_symbol_resolver_text_selection(self.valued_struct_field, selected_element_type_label, self.struct_viewer_frame_action);
                }
            }
            StructViewerFieldEditorKind::SymbolLayoutKindSelector => {
                let layout_kind_selector_id = format!("struct_viewer_symbol_layout_kind_{}_{}", self.row_index, self.valued_struct_field.get_name());
                let current_layout_kind = StructViewerViewData::read_utf8_field_text(self.valued_struct_field);
                let layout_kind_label = SymbolicLayoutKind::ALL
                    .iter()
                    .copied()
                    .map(|layout_kind| layout_kind.label())
                    .find(|candidate_label| *candidate_label == current_layout_kind)
                    .unwrap_or(SymbolicLayoutKind::Struct.label());
                let mut selected_layout_kind_label = None;
                let trailing_checkbox_space = Self::trailing_action_slot_width(commit_button_width, value_column_padding);
                let layout_kind_width = (row_max_x - value_box_position_x - trailing_checkbox_space).max(0.0);

                user_interface.put(
                    Rect::from_min_size(
                        pos2(value_box_position_x, available_size_rect.min.y),
                        vec2(layout_kind_width, available_size_rect.height()),
                    ),
                    ComboBoxView::new(
                        self.app_context.clone(),
                        layout_kind_label,
                        &layout_kind_selector_id,
                        None,
                        |popup_user_interface: &mut Ui, should_close: &mut bool| {
                            for candidate_layout_kind in SymbolicLayoutKind::ALL {
                                let candidate_label = candidate_layout_kind.label();
                                let layout_kind_response =
                                    popup_user_interface.add(ComboBoxItemView::new(self.app_context.clone(), candidate_label, None, layout_kind_width));

                                if layout_kind_response.clicked() {
                                    selected_layout_kind_label = Some(candidate_label);
                                    *should_close = true;
                                }
                            }
                        },
                    )
                    .with_label_tooltip()
                    .width(layout_kind_width)
                    .height(available_size_rect.height()),
                );

                if let Some(selected_layout_kind_label) = selected_layout_kind_label {
                    Self::commit_symbol_resolver_text_selection(self.valued_struct_field, selected_layout_kind_label, self.struct_viewer_frame_action);
                }
            }
            StructViewerFieldEditorKind::SymbolLayoutFieldSymbolLayoutSelector => {
                let symbol_layout_selector_id = format!(
                    "struct_viewer_symbol_layout_field_symbol_layout_{}_{}",
                    self.row_index,
                    self.valued_struct_field.get_name()
                );
                let current_struct_layout_id = StructViewerViewData::read_utf8_field_text(self.valued_struct_field);
                let symbol_layout_width =
                    (row_max_x - value_box_position_x - Self::trailing_action_slot_width(commit_button_width, value_column_padding)).max(0.0);
                let project_symbol_catalog = Self::get_opened_project_symbol_catalog(&self.app_context).unwrap_or_default();
                let symbol_layout_popup_width = Self::symbol_layout_selector_popup_width(user_interface, &self.app_context, &project_symbol_catalog);
                let symbol_layout_search_id = Id::new(("symbol_layout_field_search", symbol_layout_selector_id.as_str(), user_interface.id().value()));
                let mut selected_struct_layout_id = None;
                let symbol_layout_label = if current_struct_layout_id.trim().is_empty() {
                    "Select symbol layout"
                } else {
                    current_struct_layout_id.as_str()
                };

                user_interface.put(
                    Rect::from_min_size(
                        pos2(value_box_position_x, available_size_rect.min.y),
                        vec2(symbol_layout_width, available_size_rect.height()),
                    ),
                    ComboBoxView::new(
                        self.app_context.clone(),
                        symbol_layout_label,
                        &symbol_layout_selector_id,
                        None,
                        |popup_user_interface: &mut Ui, should_close: &mut bool| {
                            let mut search_text = popup_user_interface.memory(|memory| {
                                memory
                                    .data
                                    .get_temp::<String>(symbol_layout_search_id)
                                    .unwrap_or_default()
                            });
                            let search_box_id = format!("{}_search", symbol_layout_selector_id);
                            let search_response = popup_user_interface.add(
                                SearchBoxView::new(self.app_context.clone(), &mut search_text, "Search structs", &search_box_id)
                                    .width(symbol_layout_popup_width)
                                    .height(Self::COMBO_BOX_ROW_HEIGHT),
                            );
                            popup_user_interface.memory_mut(|memory| {
                                memory
                                    .data
                                    .insert_temp(symbol_layout_search_id, search_text.clone())
                            });

                            if search_response.changed() {
                                popup_user_interface.ctx().request_repaint();
                            }

                            let visible_struct_layout_descriptors = project_symbol_catalog
                                .get_struct_layout_descriptors()
                                .iter()
                                .filter(|struct_layout_descriptor| {
                                    Self::selector_label_matches_filter(struct_layout_descriptor.get_struct_layout_id(), &search_text)
                                })
                                .collect::<Vec<_>>();
                            let has_visible_layout = !visible_struct_layout_descriptors.is_empty();

                            ScrollArea::vertical()
                                .id_salt(("symbol_layout_field_options", symbol_layout_selector_id.as_str()))
                                .max_height(Self::searchable_selector_scroll_height(visible_struct_layout_descriptors.len()))
                                .auto_shrink([false, true])
                                .show(popup_user_interface, |scroll_user_interface| {
                                    for struct_layout_descriptor in visible_struct_layout_descriptors {
                                        let candidate_layout_id = struct_layout_descriptor.get_struct_layout_id();

                                        let candidate_icon =
                                            DataTypeToIconConverter::convert_data_type_to_icon(candidate_layout_id, &self.app_context.theme.icon_library);
                                        let layout_response = scroll_user_interface.add(ComboBoxItemView::new(
                                            self.app_context.clone(),
                                            candidate_layout_id,
                                            Some(candidate_icon),
                                            symbol_layout_popup_width,
                                        ));

                                        if layout_response.clicked() {
                                            selected_struct_layout_id = Some(candidate_layout_id.to_string());
                                            *should_close = true;
                                        }
                                    }

                                    if !has_visible_layout {
                                        self.render_combo_message_row(scroll_user_interface, symbol_layout_popup_width, "No matching structs");
                                    }
                                });
                        },
                    )
                    .with_label_tooltip()
                    .width(symbol_layout_width)
                    .popup_width(symbol_layout_popup_width)
                    .height(available_size_rect.height()),
                );

                if let Some(selected_struct_layout_id) = selected_struct_layout_id {
                    Self::commit_symbol_resolver_text_selection(self.valued_struct_field, &selected_struct_layout_id, self.struct_viewer_frame_action);
                }
            }
            StructViewerFieldEditorKind::SymbolLayoutFieldResolverSelector => {
                let resolver_selector_id = format!(
                    "struct_viewer_symbol_layout_field_resolver_{}_{}",
                    self.row_index,
                    self.valued_struct_field.get_name()
                );
                let current_resolver_id = StructViewerViewData::read_utf8_field_text(self.valued_struct_field);
                let resolver_width = (row_max_x - value_box_position_x - Self::trailing_action_slot_width(commit_button_width, value_column_padding)).max(0.0);
                let project_symbol_catalog = Self::get_opened_project_symbol_catalog(&self.app_context).unwrap_or_default();
                let resolver_popup_width = Self::resolver_selector_popup_width(user_interface, &self.app_context, &project_symbol_catalog);
                let resolver_search_id = Id::new((
                    "symbol_layout_field_resolver_search",
                    resolver_selector_id.as_str(),
                    user_interface.id().value(),
                ));
                let mut selected_resolver_id = None;
                let resolver_label = if current_resolver_id.trim().is_empty() {
                    "Select resolver"
                } else {
                    current_resolver_id.as_str()
                };

                user_interface.put(
                    Rect::from_min_size(
                        pos2(value_box_position_x, available_size_rect.min.y),
                        vec2(resolver_width, available_size_rect.height()),
                    ),
                    ComboBoxView::new(
                        self.app_context.clone(),
                        resolver_label,
                        &resolver_selector_id,
                        None,
                        |popup_user_interface: &mut Ui, should_close: &mut bool| {
                            let mut search_text = popup_user_interface.memory(|memory| {
                                memory
                                    .data
                                    .get_temp::<String>(resolver_search_id)
                                    .unwrap_or_default()
                            });
                            let search_box_id = format!("{}_search", resolver_selector_id);
                            let search_response = popup_user_interface.add(
                                SearchBoxView::new(self.app_context.clone(), &mut search_text, "Search resolvers", &search_box_id)
                                    .width(resolver_popup_width)
                                    .height(Self::COMBO_BOX_ROW_HEIGHT),
                            );
                            popup_user_interface.memory_mut(|memory| memory.data.insert_temp(resolver_search_id, search_text.clone()));

                            if search_response.changed() {
                                popup_user_interface.ctx().request_repaint();
                            }

                            let visible_resolver_descriptors = project_symbol_catalog
                                .get_symbolic_resolver_descriptors()
                                .iter()
                                .filter(|resolver_descriptor| Self::selector_label_matches_filter(resolver_descriptor.get_resolver_id(), &search_text))
                                .collect::<Vec<_>>();
                            let has_visible_resolver = !visible_resolver_descriptors.is_empty();

                            ScrollArea::vertical()
                                .id_salt(("symbol_layout_field_resolver_options", resolver_selector_id.as_str()))
                                .max_height(Self::searchable_selector_scroll_height(visible_resolver_descriptors.len()))
                                .auto_shrink([false, true])
                                .show(popup_user_interface, |scroll_user_interface| {
                                    for resolver_descriptor in visible_resolver_descriptors {
                                        let candidate_resolver_id = resolver_descriptor.get_resolver_id();
                                        let resolver_response = scroll_user_interface.add(ComboBoxItemView::new(
                                            self.app_context.clone(),
                                            candidate_resolver_id,
                                            None,
                                            resolver_popup_width,
                                        ));

                                        if resolver_response.clicked() {
                                            selected_resolver_id = Some(candidate_resolver_id.to_string());
                                            *should_close = true;
                                        }
                                    }

                                    if !has_visible_resolver {
                                        self.render_combo_message_row(scroll_user_interface, resolver_popup_width, "No matching resolvers");
                                    }
                                });
                        },
                    )
                    .with_label_tooltip()
                    .width(resolver_width)
                    .popup_width(resolver_popup_width)
                    .height(available_size_rect.height()),
                );

                if let Some(selected_resolver_id) = selected_resolver_id {
                    Self::commit_symbol_resolver_text_selection(self.valued_struct_field, &selected_resolver_id, self.struct_viewer_frame_action);
                }
            }
            StructViewerFieldEditorKind::SymbolLayoutFieldContainerKindSelector => {
                let container_kind_selector_id = format!(
                    "struct_viewer_symbol_layout_field_container_kind_{}_{}",
                    self.row_index,
                    self.valued_struct_field.get_name()
                );
                let current_container_kind = StructViewerViewData::read_utf8_field_text(self.valued_struct_field);
                let container_kind_label = Self::SYMBOL_LAYOUT_FIELD_CONTAINER_KIND_LABELS
                    .iter()
                    .copied()
                    .find(|candidate_label| *candidate_label == current_container_kind)
                    .unwrap_or("Element");
                let mut selected_container_kind_label = None;
                let trailing_checkbox_space = Self::trailing_action_slot_width(commit_button_width, value_column_padding);
                let container_kind_width = (row_max_x - value_box_position_x - trailing_checkbox_space).max(0.0);

                user_interface.put(
                    Rect::from_min_size(
                        pos2(value_box_position_x, available_size_rect.min.y),
                        vec2(container_kind_width, available_size_rect.height()),
                    ),
                    ComboBoxView::new(
                        self.app_context.clone(),
                        container_kind_label,
                        &container_kind_selector_id,
                        None,
                        |popup_user_interface: &mut Ui, should_close: &mut bool| {
                            for candidate_label in Self::SYMBOL_LAYOUT_FIELD_CONTAINER_KIND_LABELS {
                                let container_kind_response =
                                    popup_user_interface.add(ComboBoxItemView::new(self.app_context.clone(), candidate_label, None, container_kind_width));

                                if container_kind_response.clicked() {
                                    selected_container_kind_label = Some(candidate_label);
                                    *should_close = true;
                                }
                            }
                        },
                    )
                    .with_label_tooltip()
                    .width(container_kind_width)
                    .height(available_size_rect.height()),
                );

                if let Some(selected_container_kind_label) = selected_container_kind_label {
                    Self::commit_symbol_resolver_text_selection(self.valued_struct_field, selected_container_kind_label, self.struct_viewer_frame_action);
                }
            }
            StructViewerFieldEditorKind::SymbolLayoutFieldPointerSizeSelector => {
                let pointer_size_selector_id = format!(
                    "struct_viewer_symbol_layout_field_pointer_size_{}_{}",
                    self.row_index,
                    self.valued_struct_field.get_name()
                );
                let current_pointer_size = StructViewerViewData::read_utf8_field_text(self.valued_struct_field);
                let pointer_size_label = PointerScanPointerSize::ALL
                    .iter()
                    .copied()
                    .map(|pointer_size| pointer_size.to_string())
                    .find(|candidate_label| *candidate_label == current_pointer_size)
                    .unwrap_or_else(|| PointerScanPointerSize::Pointer64.to_string());
                let mut selected_pointer_size_label = None;
                let trailing_checkbox_space = Self::trailing_action_slot_width(commit_button_width, value_column_padding);
                let pointer_size_width = (row_max_x - value_box_position_x - trailing_checkbox_space).max(0.0);

                user_interface.put(
                    Rect::from_min_size(
                        pos2(value_box_position_x, available_size_rect.min.y),
                        vec2(pointer_size_width, available_size_rect.height()),
                    ),
                    ComboBoxView::new(
                        self.app_context.clone(),
                        &pointer_size_label,
                        &pointer_size_selector_id,
                        None,
                        |popup_user_interface: &mut Ui, should_close: &mut bool| {
                            for pointer_size in PointerScanPointerSize::ALL {
                                let candidate_label = pointer_size.to_string();
                                let pointer_size_response =
                                    popup_user_interface.add(ComboBoxItemView::new(self.app_context.clone(), &candidate_label, None, pointer_size_width));

                                if pointer_size_response.clicked() {
                                    selected_pointer_size_label = Some(candidate_label);
                                    *should_close = true;
                                }
                            }
                        },
                    )
                    .with_label_tooltip()
                    .width(pointer_size_width)
                    .height(available_size_rect.height()),
                );

                if let Some(selected_pointer_size_label) = selected_pointer_size_label {
                    Self::commit_symbol_resolver_text_selection(self.valued_struct_field, &selected_pointer_size_label, self.struct_viewer_frame_action);
                }
            }
            StructViewerFieldEditorKind::SymbolLayoutFieldOffsetModeSelector => {
                let offset_mode_selector_id = format!(
                    "struct_viewer_symbol_layout_field_offset_mode_{}_{}",
                    self.row_index,
                    self.valued_struct_field.get_name()
                );
                let current_offset_mode = StructViewerViewData::read_utf8_field_text(self.valued_struct_field);
                let offset_mode_label = Self::SYMBOL_LAYOUT_FIELD_OFFSET_MODE_LABELS
                    .iter()
                    .copied()
                    .find(|candidate_label| *candidate_label == current_offset_mode)
                    .unwrap_or("Sequential");
                let mut selected_offset_mode_label = None;
                let trailing_checkbox_space = Self::trailing_action_slot_width(commit_button_width, value_column_padding);
                let offset_mode_width = (row_max_x - value_box_position_x - trailing_checkbox_space).max(0.0);

                user_interface.put(
                    Rect::from_min_size(
                        pos2(value_box_position_x, available_size_rect.min.y),
                        vec2(offset_mode_width, available_size_rect.height()),
                    ),
                    ComboBoxView::new(
                        self.app_context.clone(),
                        offset_mode_label,
                        &offset_mode_selector_id,
                        None,
                        |popup_user_interface: &mut Ui, should_close: &mut bool| {
                            for candidate_label in Self::SYMBOL_LAYOUT_FIELD_OFFSET_MODE_LABELS {
                                let offset_mode_response =
                                    popup_user_interface.add(ComboBoxItemView::new(self.app_context.clone(), candidate_label, None, offset_mode_width));

                                if offset_mode_response.clicked() {
                                    selected_offset_mode_label = Some(candidate_label);
                                    *should_close = true;
                                }
                            }
                        },
                    )
                    .with_label_tooltip()
                    .width(offset_mode_width)
                    .height(available_size_rect.height()),
                );

                if let Some(selected_offset_mode_label) = selected_offset_mode_label {
                    Self::commit_symbol_resolver_text_selection(self.valued_struct_field, selected_offset_mode_label, self.struct_viewer_frame_action);
                }
            }
        }

        response
    }
}

#[cfg(test)]
mod tests {
    use super::StructViewerEntryView;
    use squalr_engine_api::structures::data_types::{
        built_in_types::{u32::data_type_u32::DataTypeU32, u64::data_type_u64::DataTypeU64},
        data_type_ref::DataTypeRef,
    };

    #[test]
    fn project_item_pointer_size_data_type_ref_maps_pointer_sizes() {
        assert_eq!(
            StructViewerEntryView::project_item_pointer_size_data_type_ref("u32"),
            Some(DataTypeRef::new(DataTypeU32::DATA_TYPE_ID))
        );
        assert_eq!(
            StructViewerEntryView::project_item_pointer_size_data_type_ref("u64"),
            Some(DataTypeRef::new(DataTypeU64::DATA_TYPE_ID))
        );
    }

    #[test]
    fn project_item_pointer_size_data_type_ref_omits_none_and_unknown() {
        assert_eq!(StructViewerEntryView::project_item_pointer_size_data_type_ref("None"), None);
        assert_eq!(StructViewerEntryView::project_item_pointer_size_data_type_ref(""), None);
        assert_eq!(StructViewerEntryView::project_item_pointer_size_data_type_ref("nope"), None);
    }

    #[test]
    fn value_box_width_reserves_matching_left_and_right_padding() {
        let value_position_x = 100.0;
        let row_max_x = 300.0;
        let action_button_width = 28.0;
        let value_column_padding = 2.0;
        let value_box_position_x = StructViewerEntryView::value_box_position_x(value_position_x, value_column_padding);
        let value_box_width = StructViewerEntryView::value_box_width(row_max_x, value_box_position_x, action_button_width, value_column_padding);

        assert_eq!(value_box_position_x, 102.0);
        assert_eq!(value_box_width, 168.0);
        assert_eq!(value_box_position_x + value_box_width, row_max_x - action_button_width - value_column_padding);
    }
}
