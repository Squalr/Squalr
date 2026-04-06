use crate::{
    DataTypeI24, DataTypeI24be, DataTypeU24, DataTypeU24be,
    constants::{TWENTY_FOUR_BIT_DATA_TYPE_IDS, TWENTY_FOUR_BIT_PLUGIN_DESCRIPTION, TWENTY_FOUR_BIT_PLUGIN_DISPLAY_NAME, TWENTY_FOUR_BIT_PLUGIN_ID},
};
use squalr_engine_api::{
    plugins::{Plugin, PluginKind, PluginMetadata, data_type::DataTypePlugin},
    structures::data_types::data_type::DataType,
};
use std::sync::Arc;

pub struct TwentyFourBitDataTypesPlugin {
    metadata: PluginMetadata,
    contributed_data_types: Vec<Arc<dyn DataType>>,
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
            contributed_data_types: vec![
                Arc::new(DataTypeU24 {}),
                Arc::new(DataTypeU24be {}),
                Arc::new(DataTypeI24 {}),
                Arc::new(DataTypeI24be {}),
            ],
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
    fn contributed_data_types(&self) -> &[Arc<dyn DataType>] {
        &self.contributed_data_types
    }

    fn contributed_data_type_ids(&self) -> &'static [&'static str] {
        &TWENTY_FOUR_BIT_DATA_TYPE_IDS
    }
}
