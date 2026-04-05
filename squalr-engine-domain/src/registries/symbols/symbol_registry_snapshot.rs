use crate::registries::symbols::{data_type_descriptor::DataTypeDescriptor, symbolic_struct_descriptor::SymbolicStructDescriptor};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct SymbolRegistrySnapshot {
    generation: u64,
    data_type_descriptors: Vec<DataTypeDescriptor>,
    symbolic_struct_descriptors: Vec<SymbolicStructDescriptor>,
}

impl SymbolRegistrySnapshot {
    pub fn new(
        generation: u64,
        data_type_descriptors: Vec<DataTypeDescriptor>,
        symbolic_struct_descriptors: Vec<SymbolicStructDescriptor>,
    ) -> Self {
        Self {
            generation,
            data_type_descriptors,
            symbolic_struct_descriptors,
        }
    }

    pub fn get_generation(&self) -> u64 {
        self.generation
    }

    pub fn get_data_type_descriptors(&self) -> &[DataTypeDescriptor] {
        &self.data_type_descriptors
    }

    pub fn get_symbolic_struct_descriptors(&self) -> &[SymbolicStructDescriptor] {
        &self.symbolic_struct_descriptors
    }
}
