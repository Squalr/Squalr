use crate::structures::data_types::data_type_ref::DataTypeRef;
use crate::structures::data_types::floating_point_tolerance::FloatingPointTolerance;
use crate::structures::memory::memory_alignment::MemoryAlignment;
use crate::structures::scanning::memory_read_mode::MemoryReadMode;
use crate::structures::scanning::plans::element_scan::element_scan_parameters::ElementScanParameters;
use std::collections::HashMap;

/// Represents the scan arguments for a collection of element-wise scans across varied data types and constraints.
#[derive(Debug, Clone)]
pub struct ElementScanParametersCollection {
    element_scan_parameters_by_data_type: HashMap<DataTypeRef, ElementScanParameters>,
    memory_alignment: MemoryAlignment,
    floating_point_tolerance: FloatingPointTolerance,
    memory_read_mode: MemoryReadMode,
    is_single_thread_scan: bool,

    /// If this debug flag is provided, the scan will be performed twice. Once with a specialized scan, and once with the default scan.
    /// An assertion will be made that the default scan produced the exact same result as the specialized scan.
    debug_perform_validation_scan: bool,
}

impl ElementScanParametersCollection {
    pub fn new(
        element_scan_parameters_by_data_type: HashMap<DataTypeRef, ElementScanParameters>,
        memory_alignment: MemoryAlignment,
        floating_point_tolerance: FloatingPointTolerance,
        memory_read_mode: MemoryReadMode,
        is_single_thread_scan: bool,
        debug_perform_validation_scan: bool,
    ) -> Self {
        Self {
            element_scan_parameters_by_data_type,
            memory_alignment,
            floating_point_tolerance,
            memory_read_mode,
            is_single_thread_scan,
            debug_perform_validation_scan,
        }
    }

    pub fn get_element_scan_parameters_by_data_type(&self) -> &HashMap<DataTypeRef, ElementScanParameters> {
        &self.element_scan_parameters_by_data_type
    }

    pub fn get_data_type_refs_iterator(&self) -> impl Iterator<Item = &DataTypeRef> + '_ {
        self.element_scan_parameters_by_data_type.keys()
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
