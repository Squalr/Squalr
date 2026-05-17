use crate::app_context::AppContext;
use crate::ui::widgets::controls::data_value_box::data_value_box_view::DataValueBoxView;
use crate::views::symbol_layout_editor::view_data::symbol_layout_editor_view_data::SymbolLayoutEditDraft;
use eframe::egui::Ui;
use squalr_engine_api::structures::{
    data_types::{
        built_in_types::{string::utf8::data_type_string_utf8::DataTypeStringUtf8, u64::data_type_u64::DataTypeU64},
        data_type_ref::DataTypeRef,
    },
    data_values::{anonymous_value_string::AnonymousValueString, anonymous_value_string_format::AnonymousValueStringFormat, container_type::ContainerType},
};
use std::sync::Arc;

fn string_data_type_ref() -> DataTypeRef {
    DataTypeRef::new(DataTypeStringUtf8::DATA_TYPE_ID)
}

pub(in crate::views::symbol_layout_editor::symbol_layout_editor_view) fn render_symbol_layout_string_value_box(
    app_context: Arc<AppContext>,
    user_interface: &mut Ui,
    value: &mut String,
    preview_text: &str,
    id: &str,
    width: f32,
    height: f32,
) {
    let validation_data_type_ref = string_data_type_ref();
    let mut value_string = AnonymousValueString::new(value.clone(), AnonymousValueStringFormat::String, ContainerType::None);

    user_interface.add(
        DataValueBoxView::new(app_context, &mut value_string, &validation_data_type_ref, false, true, preview_text, id)
            .allowed_anonymous_value_string_formats(vec![AnonymousValueStringFormat::String])
            .show_format_button(false)
            .normalize_value_format(false)
            .use_format_text_colors(false)
            .width(width)
            .height(height),
    );

    *value = value_string.get_anonymous_value_string().to_string();
}

pub(in crate::views::symbol_layout_editor::symbol_layout_editor_view) fn render_symbol_layout_u64_value_box(
    app_context: Arc<AppContext>,
    user_interface: &mut Ui,
    value: &mut String,
    value_format: &mut AnonymousValueStringFormat,
    preview_text: &str,
    id: &str,
    width: f32,
    height: f32,
) {
    let validation_data_type_ref = DataTypeRef::new(DataTypeU64::DATA_TYPE_ID);
    let mut value_string = AnonymousValueString::new(value.clone(), *value_format, ContainerType::None);

    user_interface.add(
        DataValueBoxView::new(app_context, &mut value_string, &validation_data_type_ref, false, true, preview_text, id)
            .allowed_anonymous_value_string_formats(vec![
                AnonymousValueStringFormat::Binary,
                AnonymousValueStringFormat::Decimal,
                AnonymousValueStringFormat::Hexadecimal,
            ])
            .show_format_button(true)
            .normalize_value_format(false)
            .use_format_text_colors(true)
            .width(width)
            .height(height),
    );

    *value = value_string.get_anonymous_value_string().to_string();
    *value_format = value_string.get_anonymous_value_string_format();
}

pub(in crate::views::symbol_layout_editor::symbol_layout_editor_view) fn render_symbol_layout_size_editor(
    app_context: Arc<AppContext>,
    user_interface: &mut Ui,
    draft: &mut SymbolLayoutEditDraft,
    row_height: f32,
) {
    render_symbol_layout_u64_value_box(
        app_context,
        user_interface,
        &mut draft.size_text,
        &mut draft.size_format,
        "size",
        "symbol_layout_editor_layout_size",
        user_interface.available_width().max(1.0),
        row_height,
    );
}
