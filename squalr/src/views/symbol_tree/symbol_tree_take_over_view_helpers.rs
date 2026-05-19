use crate::app_context::AppContext;
use crate::ui::widgets::controls::{button::Button as ThemeButton, data_value_box::data_value_box_view::DataValueBoxView};
use eframe::egui::{Color32, Response, Ui, Vec2};
use epaint::pos2;
use squalr_engine_api::structures::data_types::built_in_types::u64::data_type_u64::DataTypeU64;
use squalr_engine_api::structures::data_types::data_type_ref::DataTypeRef;
use squalr_engine_api::structures::data_values::{
    anonymous_value_string::AnonymousValueString, anonymous_value_string_format::AnonymousValueStringFormat, container_type::ContainerType,
};
use std::sync::Arc;

pub const SYMBOL_TREE_TAKE_OVER_ROW_HEIGHT: f32 = 28.0;
pub const SYMBOL_TREE_TAKE_OVER_ACTION_BUTTON_WIDTH: f32 = 120.0;
pub const SYMBOL_TREE_TAKE_OVER_ACTION_BUTTON_SPACING: f32 = 12.0;
pub const SYMBOL_TREE_TAKE_OVER_BOTTOM_PADDING: f32 = 8.0;
pub const SYMBOL_TREE_TAKE_OVER_GROUPBOX_SIDE_PADDING: f32 = 8.0;

const STRING_DATA_TYPE_ID: &str = "string_utf8";

pub fn draw_sized_action_button(
    app_context: &AppContext,
    user_interface: &mut Ui,
    label: &str,
    button_size: Vec2,
    fill_color: Color32,
    border_color: Color32,
    click_enabled: bool,
) -> Response {
    let theme = &app_context.theme;
    let text_color = theme.foreground;
    let label_galley = user_interface
        .painter()
        .layout_no_wrap(label.to_string(), theme.font_library.font_noto_sans.font_normal.clone(), text_color);
    let button_response = user_interface.add_sized(
        button_size,
        ThemeButton::new_from_theme(theme)
            .disabled(!click_enabled)
            .background_color(fill_color)
            .border_width(1.0)
            .border_color(border_color),
    );
    let text_position = pos2(
        button_response.rect.center().x - label_galley.size().x * 0.5,
        button_response.rect.center().y - label_galley.size().y * 0.5,
    );

    user_interface
        .painter()
        .galley(text_position, label_galley, text_color);

    button_response
}

pub fn render_string_data_value_box(
    app_context: Arc<AppContext>,
    user_interface: &mut Ui,
    value_text: &mut String,
    preview_text: &str,
    id: &str,
    width: f32,
) {
    let mut anonymous_value_string = AnonymousValueString::new(value_text.clone(), AnonymousValueStringFormat::String, ContainerType::None);
    let string_data_type = DataTypeRef::new(STRING_DATA_TYPE_ID);

    user_interface.add(
        DataValueBoxView::new(app_context, &mut anonymous_value_string, &string_data_type, false, true, preview_text, id)
            .width(width.max(1.0))
            .height(SYMBOL_TREE_TAKE_OVER_ROW_HEIGHT)
            .show_format_button(false)
            .use_format_text_colors(false),
    );

    let next_value_text = anonymous_value_string.get_anonymous_value_string().to_string();
    if *value_text != next_value_text {
        *value_text = next_value_text;
    }
}

pub fn render_offset_data_value_box(
    app_context: Arc<AppContext>,
    user_interface: &mut Ui,
    value_text: &mut String,
    value_format: &mut AnonymousValueStringFormat,
    preview_text: &str,
    id: &str,
    width: f32,
) {
    let mut anonymous_value_string = AnonymousValueString::new(value_text.clone(), *value_format, ContainerType::None);
    let unsigned_integer_data_type = DataTypeRef::new(DataTypeU64::DATA_TYPE_ID);

    user_interface.add(
        DataValueBoxView::new(
            app_context,
            &mut anonymous_value_string,
            &unsigned_integer_data_type,
            false,
            true,
            preview_text,
            id,
        )
        .width(width.max(1.0))
        .height(SYMBOL_TREE_TAKE_OVER_ROW_HEIGHT)
        .allowed_anonymous_value_string_formats(vec![
            AnonymousValueStringFormat::Binary,
            AnonymousValueStringFormat::Decimal,
            AnonymousValueStringFormat::Hexadecimal,
        ])
        .use_format_text_colors(true),
    );

    let next_value_text = anonymous_value_string.get_anonymous_value_string().to_string();
    if *value_text != next_value_text {
        *value_text = next_value_text;
    }

    let next_value_format = anonymous_value_string.get_anonymous_value_string_format();
    if *value_format != next_value_format {
        *value_format = next_value_format;
    }
}
