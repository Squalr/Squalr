use crate::structures::data_types::data_type_ref::DataTypeRef;
use crate::structures::data_types::floating_point_tolerance::FloatingPointTolerance;
use crate::structures::memory::memory_alignment::MemoryAlignment;
use crate::structures::scanning::constraints::scan_constraint::ScanConstraint;
use crate::structures::scanning::memory_read_mode::MemoryReadMode;

/// Represents the scan arguments for an element-wise scan.
#[derive(Debug, Clone)]
pub struct ElementScanParameters {
    scan_constraints: Vec<ScanConstraint>,
    data_types: Vec<DataTypeRef>,
    memory_alignment: MemoryAlignment,
    floating_point_tolerance: FloatingPointTolerance,
    memory_read_mode: MemoryReadMode,
    is_single_thread_scan: bool,

    /// If this debug flag is provided, the scan will be performed twice. Once with a specialized scan, and once with the default scan.
    /// An assertion will be made that the default scan produced the exact same result as the specialized scan.
    debug_perform_validation_scan: bool,
}

impl ElementScanParameters {
    pub fn new(
        scan_constraints: Vec<ScanConstraint>,
        data_types: Vec<DataTypeRef>,
        memory_alignment: MemoryAlignment,
        floating_point_tolerance: FloatingPointTolerance,
        memory_read_mode: MemoryReadMode,
        is_single_thread_scan: bool,
        debug_perform_validation_scan: bool,
    ) -> Self {
        Self {
            scan_constraints,
            data_types,
            memory_alignment,
            floating_point_tolerance,
            memory_read_mode,
            is_single_thread_scan,
            debug_perform_validation_scan,
        }
    }

    pub fn get_scan_constraints(&self) -> &Vec<ScanConstraint> {
        &self.scan_constraints
    }

    pub fn get_data_type_refs(&self) -> &Vec<DataTypeRef> {
        &self.data_types
    }

    pub fn get_memory_alignment(&self) -> MemoryAlignment {
        self.memory_alignment
    }

    pub fn get_floating_point_tolerance(&self) -> FloatingPointTolerance {
        self.floating_point_tolerance
    }

    pub fn get_memory_read_mode(&self) -> MemoryReadMode {
        self.memory_read_mode
    }

    pub fn get_is_single_thread_scan(&self) -> bool {
        self.is_single_thread_scan
    }

    pub fn get_debug_perform_validation_scan(&self) -> bool {
        self.debug_perform_validation_scan
    }
}
