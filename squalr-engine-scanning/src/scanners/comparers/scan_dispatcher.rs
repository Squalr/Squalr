use crate::scanners::comparers::scalar::snapshot_element_scanner_scalar_iterative::SnapshotElementRangeScannerScalarIterative;
use crate::scanners::comparers::scalar::snapshot_element_scanner_scalar_single_element::SnapshotElementRangeScannerScalarSingleElement;
use crate::scanners::comparers::snapshot_element_range_scanner::SnapshotElementRangeScanner;
use crate::scanners::constraints::scan_constraints::ScanConstraints;
use crate::snapshots::snapshot_element_range::SnapshotElementRange;
use crate::snapshots::snapshot_region::SnapshotRegion;
use squalr_engine_architecture::vectors::vectors;
use squalr_engine_common::dynamic_struct::field_value::FieldValue;
use std::sync::{Arc, Once, RwLock};
use tokio::task::JoinHandle;

pub struct ScanDispatcher {
}

impl ScanDispatcher {
    // Stateless
    fn new() -> Self { Self { } }
    
    pub fn get_instance() -> Arc<RwLock<ScanDispatcher>> {
        static mut INSTANCE: Option<Arc<RwLock<ScanDispatcher>>> = None;
        static INIT: Once = Once::new();

        unsafe {
            INIT.call_once(|| {
                let instance = Arc::new(RwLock::new(ScanDispatcher::new()));
                INSTANCE = Some(instance);
            });

            return INSTANCE.as_ref().unwrap().clone();
        }
    }

    pub fn dispatch_scan(&self, snapshot_region: Arc<RwLock<SnapshotRegion>>, constraints: &ScanConstraints) -> Vec<Arc<RwLock<SnapshotElementRange>>> {
        let element_ranges = snapshot_region.read().unwrap().get_snapshot_element_ranges();
        let mut results = Vec::new();
    
        for element_range in element_ranges {
            let constraints = constraints.clone();
            let element_range = element_range.clone();
            let scanner_instance = self.acquire_scanner_instance(&element_range, &constraints);
    
            let mut scanner = scanner_instance.write().unwrap();
            scanner.scan_region(&element_range, Arc::new(constraints));
            results.push(element_range);
        }
    
        return results;
    }

    pub async fn dispatch_scan_parallel(&self, snapshot_region: Arc<RwLock<SnapshotRegion>>, constraints: &ScanConstraints) -> Vec<Arc<RwLock<SnapshotElementRange>>> {
        let element_ranges = snapshot_region.read().unwrap().get_snapshot_element_ranges();
        let mut handles = Vec::new();

        for element_range in element_ranges {
            let constraints = constraints.clone();
            let element_range = element_range.clone();
            let scanner_instance = self.acquire_scanner_instance(&element_range, &constraints);

            let handle: JoinHandle<Arc<RwLock<SnapshotElementRange>>> = tokio::spawn(async move {
                let mut scanner = scanner_instance.write().unwrap();
                scanner.scan_region(&element_range, Arc::new(constraints));
                return element_range;
            });

            handles.push(handle);
        }

        let mut results = Vec::new();
        for handle in handles {
            if let Ok(result) = handle.await {
                results.push(result);
            }
        }

        return results;
    }

    fn acquire_scanner_instance(&self, element_range: &Arc<RwLock<SnapshotElementRange>>, constraints: &ScanConstraints) -> Arc<RwLock<dyn SnapshotElementRangeScanner>> {
        if element_range.read().unwrap().get_range() == constraints.get_byte_alignment() as usize {
            // Single element scanner
            return SnapshotElementRangeScannerScalarSingleElement::get_instance();
        } else if vectors::has_vector_support() && element_range.read().unwrap().parent_region.read().unwrap().get_region_size() >= vectors::get_hardware_vector_size() as u64 {
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
}
