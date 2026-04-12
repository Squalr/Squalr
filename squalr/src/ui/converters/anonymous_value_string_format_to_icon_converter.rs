use crate::ui::icon_library::IconLibrary;
use eframe::egui::TextureHandle;
use squalr_engine_api::structures::data_values::anonymous_value_string_format::AnonymousValueStringFormat;

pub struct AnonymousValueStringFormatToIconConverter;

impl AnonymousValueStringFormatToIconConverter {
    pub fn convert(
        anonymous_value_string_format: AnonymousValueStringFormat,
        icon_library: &IconLibrary,
    ) -> TextureHandle {
        match anonymous_value_string_format {
            AnonymousValueStringFormat::Binary => icon_library.icon_handle_display_type_binary.clone(),
            AnonymousValueStringFormat::Decimal => icon_library.icon_handle_display_type_decimal.clone(),
            AnonymousValueStringFormat::Hexadecimal | AnonymousValueStringFormat::HexPattern | AnonymousValueStringFormat::Address => {
                icon_library.icon_handle_display_type_hexadecimal.clone()
            }
            AnonymousValueStringFormat::String
            | AnonymousValueStringFormat::Bool
            | AnonymousValueStringFormat::DataTypeRef
            | AnonymousValueStringFormat::Enumeration => icon_library.icon_handle_display_type_string.clone(),
        }
    }
}
