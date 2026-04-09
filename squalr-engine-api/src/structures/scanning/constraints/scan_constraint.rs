use crate::structures::data_types::data_type_ref::DataTypeRef;
use crate::structures::data_types::floating_point_tolerance::FloatingPointTolerance;
use crate::structures::data_values::container_type::ContainerType;
use crate::structures::data_values::data_value::DataValue;
use crate::structures::scanning::comparisons::scan_compare_type::ScanCompareType;

pub struct ScanConstraint {
    scan_compare_type: ScanCompareType,
    data_value: DataValue,
    floating_point_tolerance: FloatingPointTolerance,
    result_container_type: ContainerType,
}

impl ScanConstraint {
    pub fn new(
        scan_compare_type: ScanCompareType,
        data_value: DataValue,
        floating_point_tolerance: FloatingPointTolerance,
    ) -> Self {
        Self {
            scan_compare_type,
            data_value,
            floating_point_tolerance,
            result_container_type: ContainerType::None,
        }
    }

    pub fn get_scan_compare_type(&self) -> ScanCompareType {
        self.scan_compare_type
    }

    pub fn set_scan_compare_type(
        &mut self,
        scan_compare_type: ScanCompareType,
    ) {
        self.scan_compare_type = scan_compare_type
    }

    pub fn get_data_value(&self) -> &DataValue {
        &self.data_value
    }

    pub fn get_data_value_mut(&mut self) -> &mut DataValue {
        &mut self.data_value
    }

    pub fn set_data_value(
        &mut self,
        data_value: DataValue,
    ) {
        self.data_value = data_value;
    }

    /// Updates the data type in place without updating the value bytes.
    pub fn set_data_type_in_place(
        &mut self,
        data_type_ref: DataTypeRef,
    ) {
        self.data_value.set_data_type_in_place(data_type_ref);
    }

    pub fn get_floating_point_tolerance(&self) -> FloatingPointTolerance {
        self.floating_point_tolerance
    }

    pub fn set_floating_point_tolerance(
        &mut self,
        floating_point_tolerance: FloatingPointTolerance,
    ) {
        self.floating_point_tolerance = floating_point_tolerance
    }

    pub fn get_result_container_type(&self) -> ContainerType {
        self.result_container_type
    }

    pub fn set_result_container_type(
        &mut self,
        result_container_type: ContainerType,
    ) {
        self.result_container_type = result_container_type;
    }
}
