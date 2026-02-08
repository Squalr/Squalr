use crate::scanners::scalar::scanner_scalar_byte_array_booyer_moore::ScannerScalarByteArrayBooyerMoore;
use crate::scanners::scalar::scanner_scalar_iterative::ScannerScalarIterative;
use crate::scanners::scalar::scanner_scalar_single_element::ScannerScalarSingleElement;
use crate::scanners::scanner_null::ScannerNull;
use crate::scanners::snapshot_scanner::Scanner;
use crate::scanners::vector::scanner_vector_aligned::ScannerVectorAligned;
use crate::scanners::vector::scanner_vector_overlapping::ScannerVectorOverlapping;
use crate::scanners::vector::scanner_vector_sparse::ScannerVectorSparse;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use squalr_engine_api::registries::scan_rules::element_scan_rule_registry::ElementScanRuleRegistry;
use squalr_engine_api::structures::scanning::constraints::scan_constraint_finalized::ScanConstraintFinalized;
use squalr_engine_api::structures::scanning::filters::snapshot_region_filter::SnapshotRegionFilter;
use squalr_engine_api::structures::scanning::filters::snapshot_region_filter_collection::SnapshotRegionFilterCollection;
use squalr_engine_api::structures::scanning::plans::element_scan::element_scan_plan::ElementScanPlan;
use squalr_engine_api::structures::scanning::plans::element_scan::snapshot_filter_element_scan_plan::SnapshotFilterElementScanPlan;
use squalr_engine_api::structures::scanning::plans::plan_types::planned_scan_type::PlannedScanType;
use squalr_engine_api::structures::scanning::plans::plan_types::planned_scan_type_byte_array::PlannedScanTypeByteArray;
use squalr_engine_api::structures::scanning::plans::plan_types::planned_scan_type_scalar::PlannedScanTypeScalar;
use squalr_engine_api::structures::scanning::plans::plan_types::planned_scan_type_vector::PlannedScanTypeVector;
use squalr_engine_api::structures::scanning::plans::plan_types::planned_scan_vectorization_size::PlannedScanVectorizationSize;
use squalr_engine_api::structures::snapshots::snapshot_region::SnapshotRegion;
use std::cmp;

pub struct ElementScanDispatcher {}

/// Implements a scan dispatcher, which picks the best scanner based on the scan constraints and the region being scanned.
/// Choosing the best scanner is critical to maintaining high performance scans.
impl ElementScanDispatcher {
    /// Performs a scan over a provided filter collection, returning a new filter collection with the results.
    pub fn dispatch_scan(
        snapshot_region: &SnapshotRegion,
        snapshot_region_filter_collection: &SnapshotRegionFilterCollection,
        element_scan_plan: &ElementScanPlan,
    ) -> SnapshotRegionFilterCollection {
        // Run the scan either single-threaded or parallel based on settings. Single-thread is not advised unless debugging.
        let result_snapshot_region_filters = if element_scan_plan.get_is_single_thread_scan() {
            snapshot_region_filter_collection
                .iter()
                .filter_map(|snapshot_region_filter| {
                    Self::dispatch_scan_for_snapshot_filter_collection(
                        snapshot_region_filter_collection,
                        snapshot_region_filter,
                        snapshot_region,
                        element_scan_plan,
                    )
                })
                .collect()
        } else {
            snapshot_region_filter_collection
                .par_iter()
                .filter_map(|snapshot_region_filter| {
                    Self::dispatch_scan_for_snapshot_filter_collection(
                        snapshot_region_filter_collection,
                        snapshot_region_filter,
                        snapshot_region,
                        element_scan_plan,
                    )
                })
                .collect()
        };

        SnapshotRegionFilterCollection::new(
            result_snapshot_region_filters,
            snapshot_region_filter_collection.get_data_type_ref().clone(),
            snapshot_region_filter_collection.get_memory_alignment(),
        )
    }

    // This method orchestrates multiple scan parameters to combine when scanning a single snapshot region.
    // Currently, this assumes only AND operations are supported, ie value >= 2000 && value <= 5000.
    // This works by simply running constraints sequentially to produce filters.
    fn dispatch_scan_for_snapshot_filter_collection(
        snapshot_region_filter_collection: &SnapshotRegionFilterCollection,
        snapshot_region_filter: &SnapshotRegionFilter,
        snapshot_region: &SnapshotRegion,
        element_scan_plan: &ElementScanPlan,
    ) -> Option<Vec<SnapshotRegionFilter>> {
        let scan_constraints = match element_scan_plan
            .get_scan_constraints_by_data_type()
            .get(snapshot_region_filter_collection.get_data_type_ref())
        {
            Some(scan_constraints) => scan_constraints,
            None => return None,
        };
        let mut scan_result_filters = vec![snapshot_region_filter.clone()];

        // Helper function to map the given element scan parameters to optimized mapped parameters for the given filter.
        let process_constraint = |snapshot_region_filter: &SnapshotRegionFilter, scan_constraint_finalized: &ScanConstraintFinalized| {
            let mut snapshot_filter_element_scan_plan = SnapshotFilterElementScanPlan::new(
                scan_constraint_finalized,
                element_scan_plan.get_memory_alignment(),
                element_scan_plan.get_floating_point_tolerance(),
            );

            // Apply all scan rules to the mapped parameters.
            for (_id, scan_filter_rule) in ElementScanRuleRegistry::get_instance()
                .get_scan_filter_rule_registry()
                .iter()
            {
                scan_filter_rule.map_parameters(
                    snapshot_region,
                    snapshot_region_filter_collection,
                    snapshot_region_filter,
                    scan_constraint_finalized,
                    &mut snapshot_filter_element_scan_plan,
                );
            }

            Self::dispatch_scan_for_snapshot_filter(snapshot_region_filter, &snapshot_filter_element_scan_plan, snapshot_region, element_scan_plan)
        };

        // Perform each scan sequentially over the current result filters.
        for scan_constraint in scan_constraints {
            scan_result_filters = if element_scan_plan.get_is_single_thread_scan() {
                scan_result_filters
                    .iter()
                    .flat_map(|snapshot_region_filter| process_constraint(snapshot_region_filter, scan_constraint))
                    .collect()
            } else {
                scan_result_filters
                    .par_iter()
                    .flat_map_iter(|snapshot_region_filter| process_constraint(snapshot_region_filter, scan_constraint))
                    .collect()
            };

            if scan_result_filters.is_empty() {
                return None;
            }
        }
        if scan_result_filters.is_empty() { None } else { Some(scan_result_filters) }
    }

    fn dispatch_scan_for_snapshot_filter(
        snapshot_region_filter: &SnapshotRegionFilter,
        snapshot_filter_element_scan_plan: &SnapshotFilterElementScanPlan,
        snapshot_region: &SnapshotRegion,
        element_scan_plan: &ElementScanPlan,
    ) -> Vec<SnapshotRegionFilter> {
        // Execute the scanner that corresponds to the mapped parameters.
        let scanner_instance = Self::aquire_scanner_instance(snapshot_filter_element_scan_plan);
        let scan_result_filters = scanner_instance.scan_region(snapshot_region, snapshot_region_filter, snapshot_filter_element_scan_plan);

        // If the debug flag is provided, perform a scalar scan to ensure that our specialized scanner has the same results.
        if element_scan_plan.get_debug_perform_validation_scan() {
            Self::perform_debug_scan(
                scanner_instance,
                &scan_result_filters,
                snapshot_region,
                snapshot_region_filter,
                snapshot_filter_element_scan_plan,
            );
        }
        scan_result_filters
    }

    fn aquire_scanner_instance(snapshot_filter_element_scan_plan: &SnapshotFilterElementScanPlan) -> &'static dyn Scanner {
        // Execute the scanner that corresponds to the mapped parameters.
        match snapshot_filter_element_scan_plan.get_planned_scan_type() {
            PlannedScanType::Invalid() => &ScannerNull {},
            PlannedScanType::Scalar(scan_parameters_scalar) => match scan_parameters_scalar {
                PlannedScanTypeScalar::SingleElement => &ScannerScalarSingleElement {},
                PlannedScanTypeScalar::ScalarIterative => &ScannerScalarIterative {},
            },
            PlannedScanType::Vector(scan_parameters_vector, vectorization_size) => match scan_parameters_vector {
                PlannedScanTypeVector::Overlapping => match vectorization_size {
                    PlannedScanVectorizationSize::Vector16 => &ScannerVectorOverlapping::<16> {},
                    PlannedScanVectorizationSize::Vector32 => &ScannerVectorOverlapping::<32> {},
                    PlannedScanVectorizationSize::Vector64 => &ScannerVectorOverlapping::<64> {},
                },
                PlannedScanTypeVector::Aligned => match vectorization_size {
                    PlannedScanVectorizationSize::Vector16 => &ScannerVectorAligned::<16> {},
                    PlannedScanVectorizationSize::Vector32 => &ScannerVectorAligned::<32> {},
                    PlannedScanVectorizationSize::Vector64 => &ScannerVectorAligned::<64> {},
                },
                PlannedScanTypeVector::Sparse => match vectorization_size {
                    PlannedScanVectorizationSize::Vector16 => &ScannerVectorSparse::<16> {},
                    PlannedScanVectorizationSize::Vector32 => &ScannerVectorSparse::<32> {},
                    PlannedScanVectorizationSize::Vector64 => &ScannerVectorSparse::<64> {},
                },
                PlannedScanTypeVector::OverlappingBytewiseStaggered => match vectorization_size {
                    PlannedScanVectorizationSize::Vector16 => &ScannerVectorOverlapping::<16> {},
                    PlannedScanVectorizationSize::Vector32 => &ScannerVectorOverlapping::<32> {},
                    PlannedScanVectorizationSize::Vector64 => &ScannerVectorOverlapping::<64> {},
                    /*
                    PlannedScanVectorizationSize::Vector16 => &ScannerVectorOverlappingBytewiseStaggered::<16> {},
                    PlannedScanVectorizationSize::Vector32 => &ScannerVectorOverlappingBytewiseStaggered::<32> {},
                    PlannedScanVectorizationSize::Vector64 => &ScannerVectorOverlappingBytewiseStaggered::<64> {},*/
                },
                PlannedScanTypeVector::OverlappingBytewisePeriodic => match vectorization_size {
                    PlannedScanVectorizationSize::Vector16 => &ScannerVectorOverlapping::<16> {},
                    PlannedScanVectorizationSize::Vector32 => &ScannerVectorOverlapping::<32> {},
                    PlannedScanVectorizationSize::Vector64 => &ScannerVectorOverlapping::<64> {},
                    /*
                    PlannedScanVectorizationSize::Vector16 => &ScannerVectorOverlappingBytewisePeriodic::<16> {},
                    PlannedScanVectorizationSize::Vector32 => &ScannerVectorOverlappingBytewisePeriodic::<32> {},
                    PlannedScanVectorizationSize::Vector64 => &ScannerVectorOverlappingBytewisePeriodic::<64> {},*/
                },
            },
            PlannedScanType::ByteArray(scan_parameters_byte_array) => match scan_parameters_byte_array {
                PlannedScanTypeByteArray::ByteArrayBooyerMoore => &ScannerScalarByteArrayBooyerMoore {},
            },
        }
    }

    /// Performs a second scan over the provided snapshot region filter to ensure that the results of a specialized scan match
    /// the results of the scalar scan. This is a way to unit test complex scanner implementations on real world data.
    fn perform_debug_scan(
        scanner_instance: &dyn Scanner,
        comparison_filters: &Vec<SnapshotRegionFilter>,
        snapshot_region: &SnapshotRegion,
        snapshot_region_filter: &SnapshotRegionFilter,
        snapshot_filter_element_scan_plan: &SnapshotFilterElementScanPlan,
    ) {
        let debug_scanner_instance = &ScannerScalarIterative {};
        let debug_filters = debug_scanner_instance.scan_region(snapshot_region, snapshot_region_filter, snapshot_filter_element_scan_plan);
        let has_length_match = debug_filters.len() == comparison_filters.len();

        if !has_length_match {
            log::error!(
                "{}",
                format!(
                    "Specialized scanner produced incorrect number of results: {}",
                    scanner_instance.get_scanner_name()
                )
            );
        }

        for index in 0..(cmp::min(debug_filters.len(), comparison_filters.len())) {
            let debug_filter = &debug_filters[index];
            let filter = &comparison_filters[index];

            if debug_filter != filter {
                log::error!(
                    "{}",
                    format!(
                        "Scanner {} produced mismatch at index: {}. Expected {}:{}, found {}:{}",
                        scanner_instance.get_scanner_name(),
                        index,
                        debug_filter.get_base_address(),
                        debug_filter.get_region_size(),
                        filter.get_base_address(),
                        filter.get_region_size(),
                    )
                );

                break;
            }
        }
    }
}
