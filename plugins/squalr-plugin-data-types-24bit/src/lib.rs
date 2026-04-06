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
}
