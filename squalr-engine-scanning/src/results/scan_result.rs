use squalr_engine_common::dynamic_struct::field_value::FieldValue;

pub struct ScanResult {
    address: u64,
    field_value: FieldValue,
}
impl ScanResult {
    pub fn new(
        address: u64,
        field_value: &FieldValue,
    ) -> Self {
        Self {
            address: address,
            field_value: field_value.clone(),
        }
    }

    pub fn get_address(&self) -> u64 {
        return self.address;
    }

    pub fn get_value(&self) -> &FieldValue {
        return &self.field_value;
    }
}
