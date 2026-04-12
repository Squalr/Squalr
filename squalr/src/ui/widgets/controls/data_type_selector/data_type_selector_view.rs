use crate::ui::converters::data_type_to_string_converter::DataTypeToStringConverter;
use crate::ui::widgets::controls::check_state::CheckState;
use crate::ui::widgets::controls::combo_box::combo_box_view::ComboBoxView;
use crate::ui::widgets::controls::data_type_selector::data_type_item_view::DataTypeItemView;
use crate::ui::widgets::controls::data_type_selector::data_type_selection::DataTypeSelection;
use crate::{app_context::AppContext, ui::converters::data_type_to_icon_converter::DataTypeToIconConverter};
use eframe::egui::{Grid, Id, Response, Ui, Widget, vec2};
use squalr_engine_api::structures::data_types::{
    built_in_types::{
        f32::data_type_f32::DataTypeF32, f32be::data_type_f32be::DataTypeF32be, f64::data_type_f64::DataTypeF64, f64be::data_type_f64be::DataTypeF64be,
        i8::data_type_i8::DataTypeI8, i16::data_type_i16::DataTypeI16, i16be::data_type_i16be::DataTypeI16be, i32::data_type_i32::DataTypeI32,
        i32be::data_type_i32be::DataTypeI32be, i64::data_type_i64::DataTypeI64, i64be::data_type_i64be::DataTypeI64be, u8::data_type_u8::DataTypeU8,
        u16::data_type_u16::DataTypeU16, u16be::data_type_u16be::DataTypeU16be, u32::data_type_u32::DataTypeU32, u32be::data_type_u32be::DataTypeU32be,
        u64::data_type_u64::DataTypeU64, u64be::data_type_u64be::DataTypeU64be,
    },
    data_type_ref::DataTypeRef,
};
use squalr_engine_api::structures::data_values::anonymous_value_string_format::AnonymousValueStringFormat;
use std::sync::Arc;

#[derive(Clone, Copy)]
enum DataTypeSelectorLabelMode {
    Text,
    IconOnly,
}

/// A widget that allows selecting from a set of data types.
pub struct DataTypeSelectorView<'lifetime> {
    app_context: Arc<AppContext>,
    data_type_selection: &'lifetime mut DataTypeSelection,
    menu_id: &'lifetime str,
    disabled: bool,
    width: f32,
    height: f32,
    label_mode: DataTypeSelectorLabelMode,
    available_data_types: Option<Vec<DataTypeRef>>,
    selectable_data_type_column_count: usize,
    show_preview_text: bool,
    enforce_format_compatibility: bool,
}

impl<'lifetime> DataTypeSelectorView<'lifetime> {
    const SELECTABLE_DATA_TYPE_COLUMN_COUNT: usize = 2;
    const SELECTABLE_DATA_TYPE_ITEM_WIDTH: f32 = 128.0;
    const SELECTABLE_DATA_TYPE_COLUMN_SPACING: f32 = 4.0;
    const SELECTABLE_DATA_TYPE_ROWS: [[&'static str; 2]; 9] = [
        [DataTypeU8::DATA_TYPE_ID, DataTypeI8::DATA_TYPE_ID],
        [DataTypeI16::DATA_TYPE_ID, DataTypeI16be::DATA_TYPE_ID],
        [DataTypeI32::DATA_TYPE_ID, DataTypeI32be::DATA_TYPE_ID],
        [DataTypeI64::DATA_TYPE_ID, DataTypeI64be::DATA_TYPE_ID],
        [DataTypeU16::DATA_TYPE_ID, DataTypeU16be::DATA_TYPE_ID],
        [DataTypeU32::DATA_TYPE_ID, DataTypeU32be::DATA_TYPE_ID],
        [DataTypeU64::DATA_TYPE_ID, DataTypeU64be::DATA_TYPE_ID],
        [DataTypeF32::DATA_TYPE_ID, DataTypeF32be::DATA_TYPE_ID],
        [DataTypeF64::DATA_TYPE_ID, DataTypeF64be::DATA_TYPE_ID],
    ];

    pub fn new(
        app_context: Arc<AppContext>,
        data_type_selection: &'lifetime mut DataTypeSelection,
        menu_id: &'lifetime str,
    ) -> Self {
        Self {
            app_context,
            data_type_selection,
            menu_id,
            disabled: false,
            width: 160.0,
            height: 28.0,
            label_mode: DataTypeSelectorLabelMode::Text,
            available_data_types: None,
            selectable_data_type_column_count: Self::SELECTABLE_DATA_TYPE_COLUMN_COUNT,
            show_preview_text: true,
            enforce_format_compatibility: false,
        }
    }

    pub fn width(
        mut self,
        width: f32,
    ) -> Self {
        self.width = width;
        self
    }

    pub fn height(
        mut self,
        height: f32,
    ) -> Self {
        self.height = height;
        self
    }

    pub fn disabled(
        mut self,
        disabled: bool,
    ) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn icon_only_label(mut self) -> Self {
        self.label_mode = DataTypeSelectorLabelMode::IconOnly;
        self
    }

    pub fn available_data_types(
        mut self,
        available_data_types: Vec<DataTypeRef>,
    ) -> Self {
        self.available_data_types = Some(available_data_types);
        self
    }

    pub fn stacked_list(mut self) -> Self {
        self.selectable_data_type_column_count = 1;
        self
    }

    pub fn hide_preview_text(mut self) -> Self {
        self.show_preview_text = false;
        self
    }

    pub fn enforce_format_compatibility(mut self) -> Self {
        self.enforce_format_compatibility = true;
        self
    }

    pub fn close(
        &self,
        user_interface: &mut Ui,
    ) {
        let popup_id = Id::new(("combo_popup", self.menu_id, user_interface.id().value()));

        user_interface.memory_mut(|memory| {
            memory.data.insert_temp(popup_id, false);
        });
    }

    fn combo_label(
        data_type_selection: &DataTypeSelection,
        label_mode: DataTypeSelectorLabelMode,
        show_preview_text: bool,
    ) -> String {
        if !show_preview_text {
            return String::new();
        }

        match label_mode {
            DataTypeSelectorLabelMode::Text => {
                let visible_data_type_label =
                    DataTypeToStringConverter::convert_data_type_to_string(data_type_selection.visible_data_type().get_data_type_id());

                match data_type_selection.selected_data_type_count() {
                    0 => "Select types".to_string(),
                    1 => visible_data_type_label.to_string(),
                    selected_data_type_count => format!("{} +{}", visible_data_type_label, selected_data_type_count - 1),
                }
            }
            DataTypeSelectorLabelMode::IconOnly => match data_type_selection.selected_data_type_count() {
                0 => "0".to_string(),
                1 => String::new(),
                selected_data_type_count => format!("+{}", selected_data_type_count),
            },
        }
    }

    fn should_render_combo_icon(
        data_type_selection: &DataTypeSelection,
        label_mode: DataTypeSelectorLabelMode,
    ) -> bool {
        match (label_mode, data_type_selection.selected_data_type_count()) {
            (_, 0) => false,
            (DataTypeSelectorLabelMode::IconOnly, selected_data_type_count) if selected_data_type_count > 1 => false,
            _ => true,
        }
    }

    fn drag_active_id(menu_id: &str) -> Id {
        Id::new(("data_type_selector_drag_active", menu_id))
    }

    fn drag_selection_state_id(menu_id: &str) -> Id {
        Id::new(("data_type_selector_drag_selection_state", menu_id))
    }

    fn selectable_popup_width(selectable_data_type_column_count: usize) -> f32 {
        Self::SELECTABLE_DATA_TYPE_ITEM_WIDTH * selectable_data_type_column_count as f32
            + Self::SELECTABLE_DATA_TYPE_COLUMN_SPACING * (selectable_data_type_column_count.saturating_sub(1) as f32)
    }

    fn selectable_data_type_grid_id(menu_id: &str) -> Id {
        Id::new(("selectable_data_type_grid", menu_id))
    }

    fn is_pointer_over_item(
        user_interface: &Ui,
        item_response: &Response,
    ) -> bool {
        user_interface.input(|input_state| {
            input_state
                .pointer
                .interact_pos()
                .is_some_and(|pointer_position| item_response.rect.contains(pointer_position))
        })
    }

    fn reset_drag_state_if_needed(
        user_interface: &mut Ui,
        menu_id: &str,
    ) {
        if user_interface.input(|input_state| !input_state.pointer.primary_down()) {
            user_interface.memory_mut(|memory| {
                memory.data.insert_temp(Self::drag_active_id(menu_id), false);
                memory
                    .data
                    .insert_temp(Self::drag_selection_state_id(menu_id), false);
            });
        }
    }

    fn handle_selectable_data_type_interaction(
        user_interface: &mut Ui,
        menu_id: &str,
        data_type_selection: &mut DataTypeSelection,
        data_type_ref: DataTypeRef,
        item_response: &Response,
    ) {
        let drag_active_id = Self::drag_active_id(menu_id);
        let drag_selection_state_id = Self::drag_selection_state_id(menu_id);
        let is_pointer_over_item = Self::is_pointer_over_item(user_interface, item_response);
        let is_primary_pressed = user_interface.input(|input_state| input_state.pointer.primary_pressed());

        if is_pointer_over_item && is_primary_pressed {
            let should_select = !data_type_selection.is_data_type_selected(&data_type_ref);
            data_type_selection.set_data_type_selected(data_type_ref, should_select);
            user_interface.memory_mut(|memory| {
                memory.data.insert_temp(drag_active_id, true);
                memory.data.insert_temp(drag_selection_state_id, should_select);
            });

            return;
        }

        let is_drag_active = user_interface.memory(|memory| memory.data.get_temp::<bool>(drag_active_id).unwrap_or(false));
        let drag_selection_state = user_interface.memory(|memory| {
            memory
                .data
                .get_temp::<bool>(drag_selection_state_id)
                .unwrap_or(false)
        });
        let is_primary_down = user_interface.input(|input_state| input_state.pointer.primary_down());

        if is_drag_active && is_primary_down && is_pointer_over_item {
            data_type_selection.set_data_type_selected(data_type_ref, drag_selection_state);
        }
    }

    fn default_selectable_data_types() -> Vec<DataTypeRef> {
        Self::SELECTABLE_DATA_TYPE_ROWS
            .iter()
            .flatten()
            .map(|data_type_id| DataTypeRef::new(data_type_id))
            .collect()
    }

    fn ordered_selectable_data_types(available_data_types: Option<&[DataTypeRef]>) -> Vec<DataTypeRef> {
        let default_selectable_data_types = Self::default_selectable_data_types();

        let Some(available_data_types) = available_data_types else {
            return default_selectable_data_types;
        };

        let mut ordered_selectable_data_types = default_selectable_data_types
            .iter()
            .filter(|default_selectable_data_type| available_data_types.contains(default_selectable_data_type))
            .cloned()
            .collect::<Vec<_>>();

        for available_data_type in available_data_types {
            if !ordered_selectable_data_types.contains(available_data_type) {
                ordered_selectable_data_types.push(available_data_type.clone());
            }
        }

        ordered_selectable_data_types
    }

    fn anonymous_value_string_format_sort_key(anonymous_value_string_format: AnonymousValueStringFormat) -> u8 {
        match anonymous_value_string_format {
            AnonymousValueStringFormat::Bool => 0,
            AnonymousValueStringFormat::String => 1,
            AnonymousValueStringFormat::Binary => 2,
            AnonymousValueStringFormat::Decimal => 3,
            AnonymousValueStringFormat::Hexadecimal => 4,
            AnonymousValueStringFormat::HexPattern => 5,
            AnonymousValueStringFormat::Address => 6,
            AnonymousValueStringFormat::DataTypeRef => 7,
            AnonymousValueStringFormat::Enumeration => 8,
        }
    }

    fn normalize_supported_formats(supported_formats: &[AnonymousValueStringFormat]) -> Vec<AnonymousValueStringFormat> {
        let mut normalized_supported_formats = supported_formats.to_vec();
        normalized_supported_formats.sort_by_key(|anonymous_value_string_format| Self::anonymous_value_string_format_sort_key(*anonymous_value_string_format));
        normalized_supported_formats.dedup();

        normalized_supported_formats
    }

    fn has_matching_supported_formats(
        app_context: &AppContext,
        reference_data_type_ref: &DataTypeRef,
        candidate_data_type_ref: &DataTypeRef,
    ) -> bool {
        let reference_supported_formats = Self::normalize_supported_formats(
            &app_context
                .engine_unprivileged_state
                .get_supported_anonymous_value_string_formats(reference_data_type_ref),
        );
        let candidate_supported_formats = Self::normalize_supported_formats(
            &app_context
                .engine_unprivileged_state
                .get_supported_anonymous_value_string_formats(candidate_data_type_ref),
        );

        reference_supported_formats == candidate_supported_formats
    }

    fn is_data_type_compatible_from_match_state(
        selected_data_type_count: usize,
        is_candidate_selected: bool,
        has_matching_supported_formats: bool,
        enforce_format_compatibility: bool,
    ) -> bool {
        !enforce_format_compatibility || selected_data_type_count == 0 || is_candidate_selected || has_matching_supported_formats
    }
}

impl<'lifetime> Widget for DataTypeSelectorView<'lifetime> {
    fn ui(
        self,
        user_interface: &mut Ui,
    ) -> Response {
        let app_context = self.app_context;
        let data_type_selection = self.data_type_selection;
        let menu_id = self.menu_id;
        let disabled = self.disabled;
        let width = self.width;
        let height = self.height;
        let label_mode = self.label_mode;
        let available_data_types = self.available_data_types;
        let selectable_data_type_column_count = self.selectable_data_type_column_count.max(1);
        let show_preview_text = self.show_preview_text;
        let enforce_format_compatibility = self.enforce_format_compatibility;
        let popup_width = Self::selectable_popup_width(selectable_data_type_column_count);
        let combo_data_type_id = data_type_selection.visible_data_type().get_data_type_id();
        let combo_icon = if Self::should_render_combo_icon(data_type_selection, label_mode) {
            Some(DataTypeToIconConverter::convert_data_type_to_icon(
                combo_data_type_id,
                &app_context.theme.icon_library,
            ))
        } else {
            None
        };
        let combo_label = Self::combo_label(data_type_selection, label_mode, show_preview_text);
        let selectable_data_types = Self::ordered_selectable_data_types(available_data_types.as_deref());

        let combo_box = ComboBoxView::new(
            app_context.clone(),
            combo_label,
            menu_id,
            combo_icon,
            move |popup_user_interface: &mut Ui, _should_close: &mut bool| {
                Self::reset_drag_state_if_needed(popup_user_interface, menu_id);
                popup_user_interface.set_min_width(popup_width);

                popup_user_interface.vertical(|user_interface| {
                    Grid::new(Self::selectable_data_type_grid_id(menu_id))
                        .spacing(vec2(Self::SELECTABLE_DATA_TYPE_COLUMN_SPACING, 0.0))
                        .min_col_width(Self::SELECTABLE_DATA_TYPE_ITEM_WIDTH)
                        .show(user_interface, |user_interface| {
                            for (data_type_index, data_type_ref) in selectable_data_types.iter().enumerate() {
                                let is_data_type_compatible = Self::is_data_type_compatible_from_match_state(
                                    data_type_selection.selected_data_type_count(),
                                    data_type_selection.is_data_type_selected(data_type_ref),
                                    Self::has_matching_supported_formats(&app_context, data_type_selection.visible_data_type(), data_type_ref),
                                    enforce_format_compatibility,
                                );
                                let data_type_item_response = user_interface.add(
                                    DataTypeItemView::new(
                                        app_context.clone(),
                                        DataTypeToStringConverter::convert_data_type_to_string(data_type_ref.get_data_type_id()),
                                        Some(DataTypeToIconConverter::convert_data_type_to_icon(
                                            data_type_ref.get_data_type_id(),
                                            &app_context.theme.icon_library,
                                        )),
                                        Self::SELECTABLE_DATA_TYPE_ITEM_WIDTH,
                                    )
                                    .with_check_state(CheckState::from_bool(data_type_selection.is_data_type_selected(data_type_ref)))
                                    .disabled(!is_data_type_compatible),
                                );

                                if is_data_type_compatible {
                                    Self::handle_selectable_data_type_interaction(
                                        user_interface,
                                        menu_id,
                                        data_type_selection,
                                        data_type_ref.clone(),
                                        &data_type_item_response,
                                    );
                                }

                                if (data_type_index + 1) % selectable_data_type_column_count == 0 {
                                    user_interface.end_row();
                                }
                            }

                            if !selectable_data_types.is_empty() && selectable_data_types.len() % selectable_data_type_column_count != 0 {
                                user_interface.end_row();
                            }
                        });
                });
            },
        )
        .disabled(disabled)
        .width(width)
        .height(height);

        // Add the combo box to the layout.
        user_interface.add(combo_box)
    }
}

#[cfg(test)]
mod tests {
    use super::{DataTypeSelectorLabelMode, DataTypeSelectorView};
    use crate::ui::widgets::controls::data_type_selector::data_type_selection::DataTypeSelection;
    use squalr_engine_api::structures::{data_types::data_type_ref::DataTypeRef, data_values::anonymous_value_string_format::AnonymousValueStringFormat};

    #[test]
    fn combo_label_includes_extra_selection_count() {
        let mut data_type_selection = DataTypeSelection::new(DataTypeRef::new("i32"));
        data_type_selection.set_data_type_selected(DataTypeRef::new("u32"), true);

        assert_eq!(
            DataTypeSelectorView::combo_label(&data_type_selection, DataTypeSelectorLabelMode::Text, true),
            "u32 +1"
        );
    }

    #[test]
    fn icon_only_combo_label_uses_total_selection_count() {
        let mut data_type_selection = DataTypeSelection::new(DataTypeRef::new("i32"));
        data_type_selection.set_data_type_selected(DataTypeRef::new("u32"), true);

        assert_eq!(
            DataTypeSelectorView::combo_label(&data_type_selection, DataTypeSelectorLabelMode::IconOnly, true),
            "+2"
        );
    }

    #[test]
    fn icon_only_combo_hides_icon_for_multiple_selected_data_types() {
        let mut data_type_selection = DataTypeSelection::new(DataTypeRef::new("i32"));
        data_type_selection.set_data_type_selected(DataTypeRef::new("u32"), true);

        assert!(!DataTypeSelectorView::should_render_combo_icon(
            &data_type_selection,
            DataTypeSelectorLabelMode::IconOnly,
        ));
    }

    #[test]
    fn icon_only_combo_keeps_icon_for_single_selected_data_type() {
        let data_type_selection = DataTypeSelection::new(DataTypeRef::new("i32"));

        assert!(DataTypeSelectorView::should_render_combo_icon(
            &data_type_selection,
            DataTypeSelectorLabelMode::IconOnly,
        ));
    }

    #[test]
    fn hidden_preview_text_returns_empty_label() {
        let mut data_type_selection = DataTypeSelection::new(DataTypeRef::new("i32"));
        data_type_selection.set_data_type_selected(DataTypeRef::new("u32"), true);

        assert_eq!(
            DataTypeSelectorView::combo_label(&data_type_selection, DataTypeSelectorLabelMode::Text, false),
            String::new()
        );
    }

    #[test]
    fn selectable_popup_width_accounts_for_two_columns_and_spacing() {
        assert_eq!(DataTypeSelectorView::selectable_popup_width(2), 248.0);
    }

    #[test]
    fn ordered_selectable_data_types_preserves_builtin_order_for_filtered_types() {
        assert_eq!(
            DataTypeSelectorView::ordered_selectable_data_types(Some(&[
                DataTypeRef::new("u32"),
                DataTypeRef::new("i8"),
                DataTypeRef::new("u16")
            ])),
            vec![
                DataTypeRef::new("i8"),
                DataTypeRef::new("u16"),
                DataTypeRef::new("u32"),
            ]
        );
    }

    #[test]
    fn normalize_supported_formats_sorts_and_deduplicates() {
        assert_eq!(
            DataTypeSelectorView::normalize_supported_formats(&[
                AnonymousValueStringFormat::Hexadecimal,
                AnonymousValueStringFormat::String,
                AnonymousValueStringFormat::Hexadecimal,
                AnonymousValueStringFormat::Decimal,
            ]),
            vec![
                AnonymousValueStringFormat::String,
                AnonymousValueStringFormat::Decimal,
                AnonymousValueStringFormat::Hexadecimal,
            ]
        );
    }

    #[test]
    fn empty_selection_keeps_all_data_types_compatible() {
        assert!(DataTypeSelectorView::is_data_type_compatible_from_match_state(0, false, false, true));
    }
}
