use crate::scanners::comparers::scalar::scanner_scalar_iterative_chunked::ScannerScalarIterativeChunked;
use crate::scanners::comparers::scalar::scanner_scalar_single_element::ScannerScalarSingleElement;
use crate::scanners::comparers::snapshot_scanner::Scanner;
use crate::scanners::constraints::scan_constraint::ScanConstraint;
use crate::snapshots::snapshot_sub_region::SnapshotSubRegion;
use crate::snapshots::snapshot_region::SnapshotRegion;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use squalr_engine_architecture::vectors::vectors;
use squalr_engine_common::dynamic_struct::field_value::FieldValue;
use std::sync::{Arc, Once, RwLock};

pub struct ScanDispatcher {
}

impl ScanDispatcher {
    // Stateless by design
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

    pub fn dispatch_scan(&self, snapshot_region: &mut SnapshotRegion, constraint: &ScanConstraint) -> Vec<SnapshotSubRegion> {
        let has_snapshot_region = snapshot_region.get_snapshot_sub_regions().is_empty();
        let has_valid_size = snapshot_region.get_region_size() > 0;

        if has_snapshot_region && has_valid_size {
            let mut sub_regions = Vec::new();
            let sub_region = SnapshotSubRegion::new(&snapshot_region);
                
            sub_regions.push(sub_region);
            
            snapshot_region.set_snapshot_sub_regions(sub_regions);
        }

        let snapshot_sub_regions = snapshot_region.get_snapshot_sub_regions();
        let mut results = Vec::new();
    
        for snapshot_sub_region in snapshot_sub_regions {
            let snapshot_sub_region = snapshot_sub_region.clone();
            let scanner_instance = self.acquire_scanner_instance(&snapshot_sub_region, &constraint);
    
            let result_sub_regions = scanner_instance.scan_region(&snapshot_region, &snapshot_sub_region, &constraint);
            
            for result_sub_region in result_sub_regions {
                results.push(result_sub_region);
            }
        }
    
        return results;
    }

    pub fn dispatch_scan_parallel(&self, snapshot_region: &mut SnapshotRegion, constraint: &ScanConstraint) -> Vec<SnapshotSubRegion> {
        let has_snapshot_region = snapshot_region.get_snapshot_sub_regions().is_empty();
        let has_valid_size = snapshot_region.get_region_size() > 0;
    
        if has_snapshot_region && has_valid_size {
            let mut sub_regions = Vec::new();
            let sub_region = SnapshotSubRegion::new(&snapshot_region);
            
            sub_regions.push(sub_region);
            
            snapshot_region.set_snapshot_sub_regions(sub_regions);
        }
    
        let snapshot_sub_regions = snapshot_region.get_snapshot_sub_regions();
    
        snapshot_sub_regions
            // Convert the iterator to a parallel iterator
            .par_iter()
            .flat_map(|snapshot_sub_region| {
                let snapshot_sub_region = snapshot_sub_region.clone();
                let scanner_instance = self.acquire_scanner_instance(&snapshot_sub_region, &constraint);
    
                scanner_instance.scan_region(&snapshot_region, &snapshot_sub_region, &constraint)
            })
            .collect()
    }

    fn acquire_scanner_instance(&self, snapshot_sub_region: &SnapshotSubRegion, constraint: &ScanConstraint) -> Arc<dyn Scanner> {
        let alignment = constraint.get_alignment();
        let data_type_size = constraint.get_element_type().size_in_bytes();

        if snapshot_sub_region.get_element_count(alignment, data_type_size) == 1 {
            // Single element scanner
            return ScannerScalarSingleElement::get_instance();
        } else if vectors::has_vector_support() && snapshot_sub_region.is_vector_friendly_size(alignment) {
            match constraint.get_element_type() {
                FieldValue::Bytes(_) => {
                    // Vector array of bytes scanner
                    // return ScannerVectorArrayOfBytes::get_instance();
                }
                _ => {
                    /*
                    if alignment_size == element_size as i32 {
                        // Fast vector scanner
                        // return ScannerVectorFast::get_instance();
                    } else if alignment_size > element_size as i32 {
                        // Sparse vector scanner
                        // return ScannerVectorSparse::get_instance();
                    } else {
                        // Staggered vector scanner
                        // return ScannerVectorStaggered::get_instance();
                    } */
                }
            }
        } else {
            // Iterative scanner
            return ScannerScalarIterativeChunked::get_instance();
        }

        return ScannerScalarIterativeChunked::get_instance();
    }
}
