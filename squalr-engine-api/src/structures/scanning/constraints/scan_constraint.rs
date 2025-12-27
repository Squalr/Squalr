use crate::structures::data_values::data_value::DataValue;
use crate::structures::scanning::comparisons::scan_compare_type::ScanCompareType;

#[derive(Debug, Clone)]
pub struct ScanConstraint {
    compare_type: ScanCompareType,
    data_value: DataValue,
}

impl ScanConstraint {
    pub fn new(
        compare_type: ScanCompareType,
        data_value: DataValue,
    ) -> Self {
        Self { compare_type, data_value }
    }

    pub fn get_compare_type(&self) -> ScanCompareType {
        self.compare_type
    }

    pub fn get_data_value(&self) -> &DataValue {
        &self.data_value
    }

    pub fn get_data_value_mut(&mut self) -> &mut DataValue {
        &mut self.data_value
    }
}
