use crate::structures::data_types::data_type_ref::DataTypeRef;
use crate::structures::data_types::floating_point_tolerance::FloatingPointTolerance;
use crate::structures::data_values::data_value::DataValue;
use crate::structures::memory::memory_alignment::MemoryAlignment;
use crate::structures::scanning::comparisons::scan_compare_type::ScanCompareType;
use crate::structures::scanning::memory_read_mode::MemoryReadMode;
use crate::structures::scanning::parameters::element_scan::element_scan_value::ElementScanValue;
use crate::structures::snapshots::snapshot_region::SnapshotRegion;

/// Represents the scan arguments for an element-wise scan.
#[derive(Debug, Clone)]
pub struct ElementScanParameters {
    compare_type: ScanCompareType,
    element_scan_values: Vec<ElementScanValue>,
    floating_point_tolerance: FloatingPointTolerance,
    memory_read_mode: MemoryReadMode,
    is_single_thread_scan: bool,

    /// If this debug flag is provided, the scan will be performed twice. Once with a specialized scan, and once with the default scan.
    /// An assertion will be made that the default scan produced the exact same result as the specialized scan.
    debug_perform_validation_scan: bool,
}

impl ElementScanParameters {
    pub fn new(
        compare_type: ScanCompareType,
        element_scan_values: Vec<ElementScanValue>,
        floating_point_tolerance: FloatingPointTolerance,
        memory_read_mode: MemoryReadMode,
        is_single_thread_scan: bool,
        debug_perform_validation_scan: bool,
    ) -> Self {
        Self {
            compare_type,
            element_scan_values,
            floating_point_tolerance,
            memory_read_mode,
            is_single_thread_scan,
            debug_perform_validation_scan,
        }
    }

    pub fn is_valid_for_snapshot_region(
        &self,
        snapshot_region: &SnapshotRegion,
    ) -> bool {
        if snapshot_region.has_current_values() {
            match self.get_compare_type() {
                ScanCompareType::Immediate(_) => return true,
                ScanCompareType::Relative(_) | ScanCompareType::Delta(_) => snapshot_region.has_previous_values(),
            }
        } else {
            false
        }
    }

    pub fn is_valid_for_data_type(
        &self,
        data_type_ref: &DataTypeRef,
    ) -> bool {
        for data_value_and_alignment in &self.element_scan_values {
            if data_value_and_alignment.get_data_value().get_data_type() == data_type_ref {
                return true;
            }
        }

        false
    }

    pub fn get_compare_type(&self) -> ScanCompareType {
        self.compare_type.clone()
    }

    pub fn get_element_scan_values(&self) -> &Vec<ElementScanValue> {
        &self.element_scan_values
    }

    pub fn get_data_value_and_alignment_for_data_type(
        &self,
        data_type_ref: &DataTypeRef,
    ) -> ElementScanValue {
        match self.get_compare_type() {
            ScanCompareType::Immediate(_) | ScanCompareType::Delta(_) => {
                for data_value_and_alignment in &self.element_scan_values {
                    if data_value_and_alignment.get_data_value().get_data_type() == data_type_ref {
                        return data_value_and_alignment.clone();
                    }
                }
                ElementScanValue::new(DataValue::new(data_type_ref.clone(), vec![]), MemoryAlignment::Alignment1)
            }
            ScanCompareType::Relative(_) => ElementScanValue::new(DataValue::new(data_type_ref.clone(), vec![]), MemoryAlignment::Alignment1),
        }
    }

    pub fn get_floating_point_tolerance(&self) -> FloatingPointTolerance {
        self.floating_point_tolerance
    }

    pub fn get_memory_read_mode(&self) -> MemoryReadMode {
        self.memory_read_mode
    }

    pub fn is_single_thread_scan(&self) -> bool {
        self.is_single_thread_scan
    }

    pub fn get_debug_perform_validation_scan(&self) -> bool {
        self.debug_perform_validation_scan
    }
}
