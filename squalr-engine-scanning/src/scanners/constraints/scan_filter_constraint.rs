use squalr_engine_common::dynamic_struct::data_type::DataType;
use squalr_engine_memory::memory_alignment::MemoryAlignment;

#[derive(Debug, Clone)]
pub struct ScanFilterConstraint {
    alignment: Option<MemoryAlignment>,
    data_type: DataType,
}

impl Default for ScanFilterConstraint {
    fn default(
    ) -> Self {
        ScanFilterConstraint::new()
    }
}

impl ScanFilterConstraint {
    pub fn new() -> Self {
        Self {
            alignment: None,
            data_type: DataType::default(),
        }
    }

    pub fn new_with_value(
        alignment: Option<MemoryAlignment>,
        data_type: DataType,
    ) -> Self {
        Self {
            alignment: alignment,
            data_type: data_type,
        }
    }

    pub fn get_memory_alignment(
        &self
    ) -> &Option<MemoryAlignment>{
        return &self.alignment;
    }

    pub fn get_memory_alignment_or_default(
        &self,
        data_type: &DataType,
    ) -> MemoryAlignment{
        if let Some(alignment) = &self.alignment {
            return alignment.to_owned();
        }

        return MemoryAlignment::from(data_type.size_in_bytes() as i32);
    }

    pub fn get_data_type(
        &self
    ) -> &DataType{
        return &self.data_type;
    }
}
