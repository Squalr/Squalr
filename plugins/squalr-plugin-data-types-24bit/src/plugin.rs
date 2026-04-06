use crate::constants::{TWENTY_FOUR_BIT_DATA_TYPE_IDS, TWENTY_FOUR_BIT_PLUGIN_DESCRIPTION, TWENTY_FOUR_BIT_PLUGIN_DISPLAY_NAME, TWENTY_FOUR_BIT_PLUGIN_ID};
use squalr_engine_api::plugins::{data_type::DataTypePlugin, Plugin, PluginKind, PluginMetadata};

pub struct TwentyFourBitDataTypesPlugin {
    metadata: PluginMetadata,
}

impl TwentyFourBitDataTypesPlugin {
    pub fn new() -> Self {
        Self {
            metadata: PluginMetadata::new(
                TWENTY_FOUR_BIT_PLUGIN_ID,
                TWENTY_FOUR_BIT_PLUGIN_DISPLAY_NAME,
                TWENTY_FOUR_BIT_PLUGIN_DESCRIPTION,
                PluginKind::DataType,
                true,
                false,
            ),
        }
    }
}

impl Default for TwentyFourBitDataTypesPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl Plugin for TwentyFourBitDataTypesPlugin {
    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }
}

impl DataTypePlugin for TwentyFourBitDataTypesPlugin {
    fn contributed_data_type_ids(&self) -> &'static [&'static str] {
        &TWENTY_FOUR_BIT_DATA_TYPE_IDS
    }
}
