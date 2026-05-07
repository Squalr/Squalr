#![feature(portable_simd)]

mod constants;
mod data_types;
mod plugin;

pub use data_types::{
    i24::data_type_i24::DataTypeI24, i24be::data_type_i24be::DataTypeI24be, u24::data_type_u24::DataTypeU24, u24be::data_type_u24be::DataTypeU24be,
};
pub use plugin::TwentyFourBitDataTypesPlugin;

#[cfg(test)]
mod tests {
    use super::{DataTypeI24, DataTypeI24be, DataTypeU24, DataTypeU24be, TwentyFourBitDataTypesPlugin};
    use squalr_engine_api::plugins::{Plugin, data_type::DataTypePlugin};
    use squalr_engine_api::structures::data_types::data_type::DataType;

    #[test]
    fn plugin_exposes_expected_metadata_and_data_types() {
        let plugin = TwentyFourBitDataTypesPlugin::new();

        assert_eq!(plugin.metadata().get_plugin_id(), "builtin.data-type.24bit-integers");
        assert!(!plugin.metadata().get_is_enabled_by_default());
        assert_eq!(
            plugin.contributed_data_type_ids(),
            &[
                DataTypeU24::DATA_TYPE_ID,
                DataTypeU24be::DATA_TYPE_ID,
                DataTypeI24::DATA_TYPE_ID,
                DataTypeI24be::DATA_TYPE_ID,
            ]
        );
    }

    #[test]
    fn twenty_four_bit_types_opt_into_scalar_integer_layout_values() {
        let unsigned_value = DataTypeU24
            .read_scalar_integer_value(DataTypeU24::get_value_from_primitive(0x12_3456).get_value_bytes())
            .expect("Expected u24 scalar integer value to decode.");
        let signed_big_endian_value = DataTypeI24be
            .read_scalar_integer_value(DataTypeI24be::get_value_from_primitive(-32768).get_value_bytes())
            .expect("Expected i24be scalar integer value to decode.");

        assert_eq!(unsigned_value, Some(0x12_3456));
        assert_eq!(signed_big_endian_value, Some(-32768));
    }
}
