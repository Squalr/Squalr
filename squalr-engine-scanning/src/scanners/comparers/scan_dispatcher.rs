use crate::scanners::comparers::scalar::snapshot_element_scanner_scalar_iterative::SnapshotElementRangeScannerScalarIterative;
use crate::scanners::comparers::scalar::snapshot_element_scanner_scalar_single_element::SnapshotElementRangeScannerScalarSingleElement;
use crate::scanners::comparers::snapshot_element_range_scanner::SnapshotElementRangeScanner;
use crate::scanners::constraints::scan_constraints::ScanConstraints;
use crate::snapshots::snapshot_element_range::SnapshotElementRange;
use crate::snapshots::snapshot_region::SnapshotRegion;
use squalr_engine_architecture::vectors::vectors;
use squalr_engine_common::dynamic_struct::field_value::FieldValue;
use std::sync::Arc;

pub fn dispatch_scan(snapshot_region: &SnapshotRegion, constraints: &ScanConstraints) {

    // snapshot_region.get
    // let scanner = acquire_scan_instance(element_range, constraints).as_ref();
}

pub fn acquire_scan_instance(element_range: &SnapshotElementRange, constraints: &ScanConstraints) -> Arc<dyn SnapshotElementRangeScanner> {
    if element_range.get_range() == constraints.get_byte_alignment() as usize {
        // Single element scanner
        return SnapshotElementRangeScannerScalarSingleElement::get_instance();
    } else if vectors::has_vector_support() && element_range.parent_region.read().unwrap().get_region_size() >= vectors::get_hardware_vector_size() as u64 {
        match constraints.get_element_type() {
            FieldValue::Bytes(_) => {
                // Vector array of bytes scanner
                // return SnapshotElementRangeScannerVectorArrayOfBytes::get_instance();
            }
            _ => {
                let alignment_size = constraints.get_byte_alignment() as i32;
                let element_size = constraints.get_element_type().size_in_bytes() as i32;

                if alignment_size == element_size as i32 {
                    // Fast vector scanner
                    // return SnapshotElementRangeScannerVectorFast::get_instance();
                } else if alignment_size > element_size as i32 {
                    // Sparse vector scanner
                    // return SnapshotElementRangeScannerVectorSparse::get_instance();
                } else {
                    // Staggered vector scanner
                    // return SnapshotElementRangeScannerVectorStaggered::get_instance();
                }
            }
        }
    } else {
        // Iterative scanner
        return SnapshotElementRangeScannerScalarIterative::get_instance();
    }

    return SnapshotElementRangeScannerScalarIterative::get_instance();
}
