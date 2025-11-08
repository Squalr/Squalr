use crate::ui::icon_library::IconLibrary;
use epaint::TextureHandle;
use squalr_engine_api::structures::data_types::built_in_types::{
    bool8::data_type_bool8::DataTypeBool8, bool32::data_type_bool32::DataTypeBool32, f32::data_type_f32::DataTypeF32, f32be::data_type_f32be::DataTypeF32be,
    f64::data_type_f64::DataTypeF64, f64be::data_type_f64be::DataTypeF64be, i8::data_type_i8::DataTypeI8, i16::data_type_i16::DataTypeI16,
    i16be::data_type_i16be::DataTypeI16be, i32::data_type_i32::DataTypeI32, i32be::data_type_i32be::DataTypeI32be, i64::data_type_i64::DataTypeI64,
    i64be::data_type_i64be::DataTypeI64be, string::utf8::data_type_string_utf8::DataTypeStringUtf8, u8::data_type_u8::DataTypeU8,
    u16::data_type_u16::DataTypeU16, u16be::data_type_u16be::DataTypeU16be, u32::data_type_u32::DataTypeU32, u32be::data_type_u32be::DataTypeU32be,
    u64::data_type_u64::DataTypeU64, u64be::data_type_u64be::DataTypeU64be,
};

pub struct DataTypeToIconConverter {}

impl DataTypeToIconConverter {
    pub fn convert_data_type_to_icon(
        data_type_id: &str,
        icon_library: &IconLibrary,
    ) -> TextureHandle {
        match data_type_id {
            DataTypeBool8::DATA_TYPE_ID => icon_library.icon_handle_data_type_bool.clone(),
            DataTypeBool32::DATA_TYPE_ID => icon_library.icon_handle_data_type_bool.clone(),
            DataTypeU8::DATA_TYPE_ID => icon_library.icon_handle_data_type_purple_blocks_1.clone(),
            DataTypeU16::DATA_TYPE_ID => icon_library.icon_handle_data_type_purple_blocks_2.clone(),
            DataTypeU16be::DATA_TYPE_ID => icon_library
                .icon_handle_data_type_purple_blocks_reverse_2
                .clone(),
            DataTypeU32::DATA_TYPE_ID => icon_library.icon_handle_data_type_purple_blocks_4.clone(),
            DataTypeU32be::DATA_TYPE_ID => icon_library
                .icon_handle_data_type_purple_blocks_reverse_4
                .clone(),
            DataTypeU64::DATA_TYPE_ID => icon_library.icon_handle_data_type_purple_blocks_8.clone(),
            DataTypeU64be::DATA_TYPE_ID => icon_library
                .icon_handle_data_type_purple_blocks_reverse_8
                .clone(),
            DataTypeI8::DATA_TYPE_ID => icon_library.icon_handle_data_type_blue_blocks_1.clone(),
            DataTypeI16::DATA_TYPE_ID => icon_library.icon_handle_data_type_blue_blocks_2.clone(),
            DataTypeI16be::DATA_TYPE_ID => icon_library.icon_handle_data_type_blue_blocks_reverse_2.clone(),
            DataTypeI32::DATA_TYPE_ID => icon_library.icon_handle_data_type_blue_blocks_4.clone(),
            DataTypeI32be::DATA_TYPE_ID => icon_library.icon_handle_data_type_blue_blocks_reverse_4.clone(),
            DataTypeI64::DATA_TYPE_ID => icon_library.icon_handle_data_type_blue_blocks_8.clone(),
            DataTypeI64be::DATA_TYPE_ID => icon_library.icon_handle_data_type_blue_blocks_reverse_8.clone(),
            DataTypeF32::DATA_TYPE_ID => icon_library.icon_handle_data_type_orange_blocks_4.clone(),
            DataTypeF32be::DATA_TYPE_ID => icon_library
                .icon_handle_data_type_orange_blocks_reverse_4
                .clone(),
            DataTypeF64::DATA_TYPE_ID => icon_library.icon_handle_data_type_orange_blocks_8.clone(),
            DataTypeF64be::DATA_TYPE_ID => icon_library
                .icon_handle_data_type_orange_blocks_reverse_8
                .clone(),
            DataTypeStringUtf8::DATA_TYPE_ID => icon_library.icon_handle_data_type_string.clone(),
            _ => icon_library.icon_handle_data_type_unknown.clone(),
        }
    }
}
