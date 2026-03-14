use crate::ui::converters::data_type_to_string_converter::DataTypeToStringConverter;
use crate::ui::widgets::controls::check_state::CheckState;
use crate::ui::widgets::controls::combo_box::combo_box_view::ComboBoxView;
use crate::ui::widgets::controls::data_type_selector::data_type_item_view::DataTypeItemView;
use crate::ui::widgets::controls::data_type_selector::data_type_selection::DataTypeSelection;
use crate::{app_context::AppContext, ui::converters::data_type_to_icon_converter::DataTypeToIconConverter};
use eframe::egui::{Id, Response, Ui, Widget};
use epaint::TextureHandle;
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
use std::sync::Arc;

#[derive(Clone, Copy)]
enum PlaceholderDataTypeEntry {
    String,
    Custom,
}

/// A widget that allows selecting from a set of data types.
pub struct DataTypeSelectorView<'lifetime> {
    app_context: Arc<AppContext>,
    data_type_selection: &'lifetime mut DataTypeSelection,
    menu_id: &'lifetime str,
    width: f32,
    height: f32,
}

impl<'lifetime> DataTypeSelectorView<'lifetime> {
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
    const PLACEHOLDER_DATA_TYPE_ROW: [PlaceholderDataTypeEntry; 2] = [
        PlaceholderDataTypeEntry::String,
        PlaceholderDataTypeEntry::Custom,
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
            width: 160.0,
            height: 28.0,
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

    pub fn close(
        &self,
        user_interface: &mut Ui,
    ) {
        let popup_id = Id::new(("combo_popup", self.menu_id, user_interface.id().value()));

        user_interface.memory_mut(|memory| {
            memory.data.insert_temp(popup_id, false);
        });
    }

    fn combo_label(data_type_selection: &DataTypeSelection) -> String {
        let visible_data_type_label = DataTypeToStringConverter::convert_data_type_to_string(data_type_selection.visible_data_type().get_data_type_id());

        match data_type_selection.selected_data_type_count() {
            0 => "Select types".to_string(),
            1 => visible_data_type_label.to_string(),
            selected_data_type_count => format!("{} +{}", visible_data_type_label, selected_data_type_count - 1),
        }
    }

    fn drag_active_id(menu_id: &str) -> Id {
        Id::new(("data_type_selector_drag_active", menu_id))
    }

    fn drag_selection_state_id(menu_id: &str) -> Id {
        Id::new(("data_type_selector_drag_selection_state", menu_id))
    }

    fn reset_drag_state_if_needed(
        user_interface: &mut Ui,
        menu_id: &str,
    ) {
        if user_interface.input(|input_state| !input_state.pointer.primary_down()) {
            user_interface.memory_mut(|memory| {
                memory.data.insert_temp(Self::drag_active_id(menu_id), false);
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
        let is_primary_pressed = user_interface.input(|input_state| input_state.pointer.primary_pressed());

        if item_response.is_pointer_button_down_on() && is_primary_pressed {
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

        if is_drag_active && is_primary_down && item_response.hovered() {
            data_type_selection.set_data_type_selected(data_type_ref, drag_selection_state);
        }
    }

    fn placeholder_entry(
        app_context: &Arc<AppContext>,
        placeholder_data_type_entry: PlaceholderDataTypeEntry,
    ) -> (&'static str, TextureHandle) {
        match placeholder_data_type_entry {
            PlaceholderDataTypeEntry::String => (
                "String...",
                app_context
                    .theme
                    .icon_library
                    .icon_handle_data_type_string
                    .clone(),
            ),
            PlaceholderDataTypeEntry::Custom => (
                "Custom...",
                app_context
                    .theme
                    .icon_library
                    .icon_handle_data_type_purple_blocks_array
                    .clone(),
            ),
        }
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
        let width = self.width;
        let height = self.height;
        let element_width = 120.0;
        let combo_data_type_id = data_type_selection.visible_data_type().get_data_type_id();
        let combo_icon = DataTypeToIconConverter::convert_data_type_to_icon(combo_data_type_id, &app_context.theme.icon_library);
        let combo_label = Self::combo_label(data_type_selection);

        let combo_box = ComboBoxView::new(
            app_context.clone(),
            combo_label,
            menu_id,
            Some(combo_icon),
            move |popup_user_interface: &mut Ui, _should_close: &mut bool| {
                Self::reset_drag_state_if_needed(popup_user_interface, menu_id);

                popup_user_interface.vertical(|user_interface| {
                    for data_type_row in Self::SELECTABLE_DATA_TYPE_ROWS {
                        user_interface.horizontal(|user_interface| {
                            for data_type_id in data_type_row {
                                let data_type_ref = DataTypeRef::new(data_type_id);
                                let data_type_item_response = user_interface.add(
                                    DataTypeItemView::new(
                                        app_context.clone(),
                                        DataTypeToStringConverter::convert_data_type_to_string(data_type_id),
                                        Some(DataTypeToIconConverter::convert_data_type_to_icon(
                                            data_type_id,
                                            &app_context.theme.icon_library,
                                        )),
                                        element_width,
                                    )
                                    .with_check_state(CheckState::from_bool(data_type_selection.is_data_type_selected(&data_type_ref))),
                                );

                                Self::handle_selectable_data_type_interaction(
                                    user_interface,
                                    menu_id,
                                    data_type_selection,
                                    data_type_ref,
                                    &data_type_item_response,
                                );
                            }
                        });
                    }

                    user_interface.separator();
                    user_interface.horizontal(|user_interface| {
                        for placeholder_data_type_entry in Self::PLACEHOLDER_DATA_TYPE_ROW {
                            let (label, icon) = Self::placeholder_entry(&app_context, placeholder_data_type_entry);
                            user_interface.add(DataTypeItemView::new(app_context.clone(), label, Some(icon), element_width).disabled(true));
                        }
                    });
                });
            },
        )
        .width(width)
        .height(height);

        // Add the combo box to the layout.
        user_interface.add(combo_box)
    }
}
