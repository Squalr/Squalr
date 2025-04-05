use crate::structures::scanning::comparisons::scan_compare_type::ScanCompareType;
use crate::structures::scanning::scan_memory_read_mode::ScanMemoryReadMode;
use crate::structures::{data_types::floating_point_tolerance::FloatingPointTolerance, data_values::anonymous_value::AnonymousValue};

/// Represents the global scan arguments that are used by all current scans, regardless of `DataType`.
#[derive(Debug, Clone)]
pub struct UserScanParametersGlobal {
    compare_type: ScanCompareType,
    compare_immediate: Option<AnonymousValue>,
    floating_point_tolerance: FloatingPointTolerance,
    memory_read_mode: ScanMemoryReadMode,
    is_single_thread_scan: bool,
}

impl UserScanParametersGlobal {
    pub fn new(
        compare_type: ScanCompareType,
        value: Option<AnonymousValue>,
        floating_point_tolerance: FloatingPointTolerance,
        memory_read_mode: ScanMemoryReadMode,
        is_single_thread_scan: bool,
    ) -> Self {
        Self {
            compare_type,
            compare_immediate: value,
            floating_point_tolerance,
            memory_read_mode,
            is_single_thread_scan,
        }
    }

    pub fn get_compare_type(&self) -> ScanCompareType {
        self.compare_type.clone()
    }

    pub fn set_compare_immediate(
        &mut self,
        compare_immediate: Option<AnonymousValue>,
    ) {
        self.compare_immediate = compare_immediate;
    }

    pub fn get_compare_immediate(&self) -> Option<&AnonymousValue> {
        match self.get_compare_type() {
            ScanCompareType::Immediate(_) => self.compare_immediate.as_ref(),
            ScanCompareType::Relative(_) => None,
            ScanCompareType::Delta(_) => None,
        }
    }

    pub fn get_floating_point_tolerance(&self) -> FloatingPointTolerance {
        self.floating_point_tolerance
    }

    pub fn get_memory_read_mode(&self) -> ScanMemoryReadMode {
        self.memory_read_mode
    }

    pub fn is_single_thread_scan(&self) -> bool {
        self.is_single_thread_scan
    }

    pub fn is_valid(&self) -> bool {
        match self.get_compare_type() {
            ScanCompareType::Immediate(_) | ScanCompareType::Delta(_) => self.compare_immediate.is_some(),
            ScanCompareType::Relative(_) => true,
        }
    }
}
