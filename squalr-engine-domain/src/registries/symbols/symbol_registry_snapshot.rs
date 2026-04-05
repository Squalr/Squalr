use crate::registries::symbols::{data_type_descriptor::DataTypeDescriptor, symbolic_struct_descriptor::StructLayoutDescriptor};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct RegistryMetadata {
    generation: u64,
    data_type_descriptors: Vec<DataTypeDescriptor>,
    struct_layout_descriptors: Vec<StructLayoutDescriptor>,
}

impl RegistryMetadata {
    pub fn new(
        generation: u64,
        data_type_descriptors: Vec<DataTypeDescriptor>,
        struct_layout_descriptors: Vec<StructLayoutDescriptor>,
    ) -> Self {
        Self {
            generation,
            data_type_descriptors,
            struct_layout_descriptors,
        }
    }

    pub fn get_generation(&self) -> u64 {
        self.generation
    }

    pub fn get_data_type_descriptors(&self) -> &[DataTypeDescriptor] {
        &self.data_type_descriptors
    }

    pub fn get_struct_layout_descriptors(&self) -> &[StructLayoutDescriptor] {
        &self.struct_layout_descriptors
    }
}
