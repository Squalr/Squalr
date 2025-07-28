use crate::structures::data_values::data_value::DataValue;

/// Represents the scan arguments for an element-wise scan.
#[derive(Debug, Clone)]
pub struct PointerScanParameters {
    target_address: DataValue,
    offset_size: u64,
    max_depth: u64,
    is_single_thread_scan: bool,

    /// If this debug flag is provided, the scan will be performed twice. Once with a specialized scan, and once with the default scan.
    /// An assertion will be made that the default scan produced the exact same result as the specialized scan.
    debug_perform_validation_scan: bool,
}

impl PointerScanParameters {
    pub fn new(
        target_address: DataValue,
        offset_size: u64,
        max_depth: u64,
        is_single_thread_scan: bool,
        debug_perform_validation_scan: bool,
    ) -> Self {
        Self {
            target_address,
            offset_size,
            max_depth,
            is_single_thread_scan,
            debug_perform_validation_scan,
        }
    }

    pub fn get_target_address(&self) -> &DataValue {
        &self.target_address
    }

    pub fn get_offset_size(&self) -> u64 {
        self.offset_size
    }

    pub fn get_max_depth(&self) -> u64 {
        self.max_depth
    }

    pub fn get_is_single_thread_scan(&self) -> bool {
        self.is_single_thread_scan
    }

    pub fn get_debug_perform_validation_scan(&self) -> bool {
        self.debug_perform_validation_scan
    }
}
