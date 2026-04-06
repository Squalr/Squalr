mod constants;
mod plugin;

pub use plugin::TwentyFourBitDataTypesPlugin;

#[cfg(test)]
mod tests {
    use super::TwentyFourBitDataTypesPlugin;
    use squalr_engine_api::plugins::{data_type::DataTypePlugin, Plugin};

    #[test]
    fn plugin_exposes_expected_metadata_and_data_types() {
        let plugin = TwentyFourBitDataTypesPlugin::new();

        assert_eq!(plugin.metadata().get_plugin_id(), "builtin.data-type.24bit-integers");
        assert!(!plugin.metadata().get_is_enabled_by_default());
        assert_eq!(plugin.contributed_data_type_ids(), &["u24", "u24be", "i24", "i24be"]);
    }
}
