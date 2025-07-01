use crate::structures::memory::memory_alignment::MemoryAlignment;
use crate::structures::{data_types::data_type_ref::DataTypeRef, data_values::data_value::DataValue};
use serde::{Deserialize, Serialize};

/// Defines a unique pair of a `DataValue` and `MemoryAlignment` used within a larger struct scan job.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructScanValue {
    data_value: DataValue,
    memory_alignment: MemoryAlignment,
}

impl StructScanValue {
    pub fn new(
        data_value: DataValue,
        memory_alignment: MemoryAlignment,
    ) -> Self {
        Self { data_value, memory_alignment }
    }

    pub fn get_memory_alignment(&self) -> MemoryAlignment {
        self.memory_alignment
    }

    pub fn get_data_value(&self) -> &DataValue {
        &self.data_value
    }

    pub fn get_data_type(&self) -> &DataTypeRef {
        &self.data_value.get_data_type()
    }
}
