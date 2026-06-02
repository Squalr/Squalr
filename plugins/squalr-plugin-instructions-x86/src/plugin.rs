use crate::{
    DataTypeInstructionX64, DataTypeInstructionX86, X64InstructionSet, X86InstructionSet,
    constants::{
        X86_FAMILY_DATA_TYPE_IDS, X86_FAMILY_INSTRUCTION_SET_IDS, X86_FAMILY_PLUGIN_DESCRIPTION, X86_FAMILY_PLUGIN_DISPLAY_NAME, X86_FAMILY_PLUGIN_ID,
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

pub struct X86FamilyInstructionsPlugin {
    metadata: PluginMetadata,
    contributed_data_types: Vec<Arc<dyn DataType>>,
    contributed_instruction_sets: Vec<Arc<dyn InstructionSet>>,
}

impl X86FamilyInstructionsPlugin {
    pub fn new() -> Self {
        Self {
            metadata: PluginMetadata::new(
                X86_FAMILY_PLUGIN_ID,
                X86_FAMILY_PLUGIN_DISPLAY_NAME,
                X86_FAMILY_PLUGIN_DESCRIPTION,
                vec![PluginCapability::DataType, PluginCapability::InstructionSet],
                true,
                true,
            ),
            contributed_data_types: vec![
                Arc::new(DataTypeInstructionX86::new()),
                Arc::new(DataTypeInstructionX64::new()),
            ],
            contributed_instruction_sets: vec![
                Arc::new(X86InstructionSet::new()),
                Arc::new(X64InstructionSet::new()),
            ],
        }
    }
}

impl Default for X86FamilyInstructionsPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl Plugin for X86FamilyInstructionsPlugin {
    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }
}

impl PluginPackage for X86FamilyInstructionsPlugin {
    fn as_data_type_plugin(&self) -> Option<&dyn DataTypePlugin> {
        Some(self)
    }

    fn as_instruction_set_plugin(&self) -> Option<&dyn InstructionSetPlugin> {
        Some(self)
    }
}

impl DataTypePlugin for X86FamilyInstructionsPlugin {
    fn contributed_data_types(&self) -> &[Arc<dyn DataType>] {
        &self.contributed_data_types
    }

    fn contributed_data_type_ids(&self) -> &'static [&'static str] {
        &X86_FAMILY_DATA_TYPE_IDS
    }
}

impl InstructionSetPlugin for X86FamilyInstructionsPlugin {
    fn contributed_instruction_sets(&self) -> &[Arc<dyn InstructionSet>] {
        &self.contributed_instruction_sets
    }

    fn contributed_instruction_set_ids(&self) -> &'static [&'static str] {
        &X86_FAMILY_INSTRUCTION_SET_IDS
    }
}
