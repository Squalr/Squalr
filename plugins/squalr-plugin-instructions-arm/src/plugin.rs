use crate::{
    Arm32InstructionSet, Arm64InstructionSet, DataTypeIArm, DataTypeIArm64,
    constants::{
        ARM_FAMILY_DATA_TYPE_IDS, ARM_FAMILY_INSTRUCTION_SET_IDS, ARM_FAMILY_PLUGIN_DESCRIPTION, ARM_FAMILY_PLUGIN_DISPLAY_NAME, ARM_FAMILY_PLUGIN_ID,
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

pub struct ArmFamilyInstructionsPlugin {
    metadata: PluginMetadata,
    contributed_data_types: Vec<Arc<dyn DataType>>,
    contributed_instruction_sets: Vec<Arc<dyn InstructionSet>>,
}

impl ArmFamilyInstructionsPlugin {
    pub fn new() -> Self {
        Self {
            metadata: PluginMetadata::new(
                ARM_FAMILY_PLUGIN_ID,
                ARM_FAMILY_PLUGIN_DISPLAY_NAME,
                ARM_FAMILY_PLUGIN_DESCRIPTION,
                vec![PluginCapability::DataType, PluginCapability::InstructionSet],
                true,
                true,
            ),
            contributed_data_types: vec![Arc::new(DataTypeIArm::new()), Arc::new(DataTypeIArm64::new())],
            contributed_instruction_sets: vec![
                Arc::new(Arm32InstructionSet::new()),
                Arc::new(Arm64InstructionSet::new()),
            ],
        }
    }
}

impl Default for ArmFamilyInstructionsPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl Plugin for ArmFamilyInstructionsPlugin {
    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }
}

impl PluginPackage for ArmFamilyInstructionsPlugin {
    fn as_data_type_plugin(&self) -> Option<&dyn DataTypePlugin> {
        Some(self)
    }

    fn as_instruction_set_plugin(&self) -> Option<&dyn InstructionSetPlugin> {
        Some(self)
    }
}

impl DataTypePlugin for ArmFamilyInstructionsPlugin {
    fn contributed_data_types(&self) -> &[Arc<dyn DataType>] {
        &self.contributed_data_types
    }

    fn contributed_data_type_ids(&self) -> &'static [&'static str] {
        &ARM_FAMILY_DATA_TYPE_IDS
    }
}

impl InstructionSetPlugin for ArmFamilyInstructionsPlugin {
    fn contributed_instruction_sets(&self) -> &[Arc<dyn InstructionSet>] {
        &self.contributed_instruction_sets
    }

    fn contributed_instruction_set_ids(&self) -> &'static [&'static str] {
        &ARM_FAMILY_INSTRUCTION_SET_IDS
    }
}
