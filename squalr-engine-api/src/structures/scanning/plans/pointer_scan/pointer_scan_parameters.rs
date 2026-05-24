use crate::structures::pointer_scans::pointer_scan_pointer_size::PointerScanPointerSize;

/// Represents the scan arguments for a pointer scan.
#[derive(Debug, Clone)]
pub struct PointerScanParameters {
    pointer_size: PointerScanPointerSize,
    offset_radius: u64,
    max_depth: u64,
    is_single_thread_scan: bool,

    /// If this debug flag is provided, the scan will be performed twice. Once with a specialized scan, and once with the default scan.
    /// An assertion will be made that the default scan produced the exact same result as the specialized scan.
    debug_perform_validation_scan: bool,
}

impl PointerScanParameters {
    pub fn new(
        pointer_size: PointerScanPointerSize,
        offset_radius: u64,
        max_depth: u64,
        is_single_thread_scan: bool,
        debug_perform_validation_scan: bool,
    ) -> Self {
        Self {
            pointer_size,
            offset_radius,
            max_depth,
            is_single_thread_scan,
            debug_perform_validation_scan,
        }
    }

    pub fn get_pointer_size(&self) -> PointerScanPointerSize {
        self.pointer_size
    }

    pub fn get_offset_radius(&self) -> u64 {
        self.offset_radius
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
