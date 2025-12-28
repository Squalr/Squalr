use crate::structures::data_types::data_type_ref::DataTypeRef;
use crate::structures::data_types::floating_point_tolerance::FloatingPointTolerance;
use crate::structures::memory::memory_alignment::MemoryAlignment;
use crate::structures::scanning::constraints::scan_constraint::ScanConstraint;
use crate::structures::scanning::memory_read_mode::MemoryReadMode;
use std::collections::HashMap;

/// Represents parameters that can be optimized by rules to efficiently execute an element scan.
#[derive(Debug, Clone)]
pub struct ElementScanPlan {
    scan_constraints_by_data_type: HashMap<DataTypeRef, Vec<ScanConstraint>>,
    memory_alignment: MemoryAlignment,
    floating_point_tolerance: FloatingPointTolerance,
    memory_read_mode: MemoryReadMode,
    is_single_thread_scan: bool,

    /// If this debug flag is provided, the scan will be performed twice. Once with a specialized scan, and once with the default scan.
    /// An assertion will be made that the default scan produced the exact same result as the specialized scan.
    debug_perform_validation_scan: bool,
}

impl ElementScanPlan {
    pub fn new(
        scan_constraints_by_data_type: HashMap<DataTypeRef, Vec<ScanConstraint>>,
        memory_alignment: MemoryAlignment,
        floating_point_tolerance: FloatingPointTolerance,
        memory_read_mode: MemoryReadMode,
        is_single_thread_scan: bool,
        debug_perform_validation_scan: bool,
    ) -> Self {
        Self {
            scan_constraints_by_data_type,
            memory_alignment,
            floating_point_tolerance,
            memory_read_mode,
            is_single_thread_scan,
            debug_perform_validation_scan,
        }
    }

    pub fn get_scan_constraints_by_data_type(&self) -> &HashMap<DataTypeRef, Vec<ScanConstraint>> {
        &self.scan_constraints_by_data_type
    }

    pub fn get_data_type_refs_iterator(&self) -> impl Iterator<Item = &DataTypeRef> + '_ {
        self.scan_constraints_by_data_type.keys()
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
