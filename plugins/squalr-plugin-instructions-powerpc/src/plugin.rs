use crate::{
    DataTypeInstructionPowerPc32Be, PowerPc32BeInstructionSet,
    constants::{
        POWERPC_FAMILY_DATA_TYPE_IDS, POWERPC_FAMILY_INSTRUCTION_SET_IDS, POWERPC_FAMILY_PLUGIN_DESCRIPTION, POWERPC_FAMILY_PLUGIN_DISPLAY_NAME,
        POWERPC_FAMILY_PLUGIN_ID,
    },
};
use squalr_engine_api::{
    plugins::{
        Plugin, PluginCapability, PluginMetadata, PluginPackage,
        data_type::DataTypePlugin,
        instruction_set::{InstructionSet, InstructionSetPlugin},
    },
    structures::data_types::data_type::DataType,
};
use std::sync::Arc;

pub struct PowerPcFamilyInstructionsPlugin {
    metadata: PluginMetadata,
    contributed_data_types: Vec<Arc<dyn DataType>>,
    contributed_instruction_sets: Vec<Arc<dyn InstructionSet>>,
}

impl PowerPcFamilyInstructionsPlugin {
    pub fn new() -> Self {
        Self {
            metadata: PluginMetadata::new(
                POWERPC_FAMILY_PLUGIN_ID,
                POWERPC_FAMILY_PLUGIN_DISPLAY_NAME,
                POWERPC_FAMILY_PLUGIN_DESCRIPTION,
                vec![PluginCapability::DataType, PluginCapability::InstructionSet],
                true,
                is_enabled_by_default_for_current_target(),
            ),
            contributed_data_types: vec![Arc::new(DataTypeInstructionPowerPc32Be::new())],
            contributed_instruction_sets: vec![Arc::new(PowerPc32BeInstructionSet::new())],
        }
    }
}

fn is_enabled_by_default_for_current_target() -> bool {
    cfg!(any(target_arch = "powerpc", target_arch = "powerpc64"))
}

impl Default for PowerPcFamilyInstructionsPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl Plugin for PowerPcFamilyInstructionsPlugin {
    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }
}

impl PluginPackage for PowerPcFamilyInstructionsPlugin {
    fn as_data_type_plugin(&self) -> Option<&dyn DataTypePlugin> {
        Some(self)
    }

    fn as_instruction_set_plugin(&self) -> Option<&dyn InstructionSetPlugin> {
        Some(self)
    }
}

impl DataTypePlugin for PowerPcFamilyInstructionsPlugin {
    fn contributed_data_types(&self) -> &[Arc<dyn DataType>] {
        &self.contributed_data_types
    }

    fn contributed_data_type_ids(&self) -> &'static [&'static str] {
        &POWERPC_FAMILY_DATA_TYPE_IDS
    }
}

impl InstructionSetPlugin for PowerPcFamilyInstructionsPlugin {
    fn contributed_instruction_sets(&self) -> &[Arc<dyn InstructionSet>] {
        &self.contributed_instruction_sets
    }

    fn contributed_instruction_set_ids(&self) -> &'static [&'static str] {
        &POWERPC_FAMILY_INSTRUCTION_SET_IDS
    }
}
