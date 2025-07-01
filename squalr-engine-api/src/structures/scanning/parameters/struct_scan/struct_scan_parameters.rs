use crate::structures::data_types::data_type_ref::DataTypeRef;
use crate::structures::data_types::floating_point_tolerance::FloatingPointTolerance;
use crate::structures::dynamic_struct::dynamic_struct::DynamicStruct;
use crate::structures::scanning::comparisons::scan_compare_type::ScanCompareType;
use crate::structures::scanning::memory_read_mode::MemoryReadMode;
use crate::structures::scanning::parameters::struct_scan::struct_scan_value::StructScanValue;

/// Represents the scan arguments for an element-wise scan.
#[derive(Debug, Clone)]
pub struct StructScanParameters {
    compare_type: ScanCompareType,
    struct_scan_values: Vec<StructScanValue>,
    floating_point_tolerance: FloatingPointTolerance,
    memory_read_mode: MemoryReadMode,
    is_single_thread_scan: bool,

    /// If this debug flag is provided, the scan will be performed twice. Once with a specialized scan, and once with the default scan.
    /// An assertion will be made that the default scan produced the exact same result as the specialized scan.
    debug_perform_validation_scan: bool,
}

impl StructScanParameters {
    pub fn new(
        compare_type: ScanCompareType,
        struct_scan_values: Vec<StructScanValue>,
        floating_point_tolerance: FloatingPointTolerance,
        memory_read_mode: MemoryReadMode,
        is_single_thread_scan: bool,
        debug_perform_validation_scan: bool,
    ) -> Self {
        Self {
            compare_type,
            struct_scan_values,
            floating_point_tolerance,
            memory_read_mode,
            is_single_thread_scan,
            debug_perform_validation_scan,
        }
    }

    pub fn get_compare_type(&self) -> ScanCompareType {
        self.compare_type.clone()
    }

    pub fn get_struct_scan_values(&self) -> &Vec<StructScanValue> {
        &self.struct_scan_values
    }

    pub fn get_dynamic_struct_and_alignment_for_data_type(
        &self,
        data_type_ref: &DataTypeRef,
    ) -> Option<DynamicStruct> {
        None
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
