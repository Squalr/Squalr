use crate::scanners::scalar::scanner_scalar_byte_array_booyer_moore::ScannerScalarByteArrayBooyerMoore;
use crate::scanners::scalar::scanner_scalar_iterative::ScannerScalarIterative;
use crate::scanners::scalar::scanner_scalar_single_element::ScannerScalarSingleElement;
use crate::scanners::scanner_null::ScannerNull;
use crate::scanners::snapshot_scanner::Scanner;
use crate::scanners::vector::scanner_vector_aligned::ScannerVectorAligned;
use crate::scanners::vector::scanner_vector_overlapping::ScannerVectorOverlapping;
use crate::scanners::vector::scanner_vector_overlapping_bytewise_periodic::ScannerVectorOverlappingBytewisePeriodic;
use crate::scanners::vector::scanner_vector_overlapping_bytewise_staggered::ScannerVectorOverlappingBytewiseStaggered;
use crate::scanners::vector::scanner_vector_sparse::ScannerVectorSparse;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use squalr_engine_api::registries::scan_rules::element_scan_rule_registry::ElementScanRuleRegistry;
use squalr_engine_api::registries::symbols::symbol_registry::SymbolRegistry;
use squalr_engine_api::structures::scanning::constraints::mapped_scan_type::{
    MappedScanType, ScanParametersByteArray, ScanParametersScalar, ScanParametersVector,
};
use squalr_engine_api::structures::scanning::constraints::optimized_scan_constraint::OptimizedScanConstraint;
use squalr_engine_api::structures::scanning::constraints::scan_constraint::ScanConstraint;
use squalr_engine_api::structures::scanning::constraints::vectorization_size::VectorizationSize;
use squalr_engine_api::structures::scanning::filters::snapshot_region_filter::SnapshotRegionFilter;
use squalr_engine_api::structures::scanning::filters::snapshot_region_filter_collection::SnapshotRegionFilterCollection;
use squalr_engine_api::structures::scanning::parameters::element_scan::element_scan_parameters::ElementScanParameters;
use squalr_engine_api::structures::snapshots::snapshot_region::SnapshotRegion;
use std::cmp;
use std::sync::{Arc, RwLock};

pub struct ElementScanDispatcher {}

/// Implements a scan dispatcher, which picks the best scanner based on the scan constraints and the region being scanned.
/// Choosing the best scanner is critical to maintaining high performance scans.
impl ElementScanDispatcher {
    /// Performs a scan over a provided filter collection, returning a new filter collection with the results.
    pub fn dispatch_scan(
        element_scan_rule_registry: &Arc<RwLock<ElementScanRuleRegistry>>,
        symbol_registry: &Arc<RwLock<SymbolRegistry>>,
        snapshot_region: &SnapshotRegion,
        snapshot_region_filter_collection: &SnapshotRegionFilterCollection,
        element_scan_parameters: &ElementScanParameters,
    ) -> SnapshotRegionFilterCollection {
        // Run the scan either single-threaded or parallel based on settings. Single-thread is not advised unless debugging.
        let result_snapshot_region_filters: Vec<Vec<SnapshotRegionFilter>> = if element_scan_parameters.get_is_single_thread_scan() {
            snapshot_region_filter_collection
                .iter()
                .filter_map(|snapshot_region_filter| {
                    Self::dispatch_scan_for_snapshot_filter(
                        snapshot_region_filter,
                        snapshot_region,
                        element_scan_rule_registry,
                        symbol_registry,
                        snapshot_region_filter_collection,
                        element_scan_parameters,
                    )
                })
                .collect()
        } else {
            snapshot_region_filter_collection
                .par_iter()
                .filter_map(|snapshot_region_filter| {
                    Self::dispatch_scan_for_snapshot_filter(
                        snapshot_region_filter,
                        snapshot_region,
                        element_scan_rule_registry,
                        symbol_registry,
                        snapshot_region_filter_collection,
                        element_scan_parameters,
                    )
                })
                .collect()
        };

        SnapshotRegionFilterCollection::new(
            symbol_registry,
            result_snapshot_region_filters,
            snapshot_region_filter_collection.get_data_type_ref().clone(),
            snapshot_region_filter_collection.get_memory_alignment(),
        )
    }

    // This method orchestrates multiple scan parameters to combine when scanning a single snapshot region.
    // Currently, this assumes only AND operations are supported, ie value >= 2000 && value <= 5000.
    // This works by simply running constraints sequentially to produce filters.
    fn dispatch_scan_for_snapshot_filter(
        snapshot_region_filter: &SnapshotRegionFilter,
        snapshot_region: &SnapshotRegion,
        element_scan_rule_registry: &Arc<RwLock<ElementScanRuleRegistry>>,
        symbol_registry: &Arc<RwLock<SymbolRegistry>>,
        snapshot_region_filter_collection: &SnapshotRegionFilterCollection,
        element_scan_parameters: &ElementScanParameters,
    ) -> Option<Vec<SnapshotRegionFilter>> {
        let mut scan_result_filters = vec![snapshot_region_filter.clone()];

        // Helper function to map the given element scan parameters to optimized mapped parameters for the given filter.
        let process_constraint = |snapshot_region_filter: &SnapshotRegionFilter, scan_constraint: &ScanConstraint| {
            let optimized_scan_constraint = Self::optimize_scan_constraint(
                element_scan_rule_registry,
                symbol_registry,
                snapshot_region,
                snapshot_region_filter,
                snapshot_region_filter_collection,
                element_scan_parameters,
                scan_constraint,
            );
            optimized_scan_constraint
                .map(|optimized_scan_constraint| {
                    Self::perform_scan_for_snapshot_filter(
                        snapshot_region_filter,
                        &optimized_scan_constraint,
                        snapshot_region,
                        symbol_registry,
                        element_scan_parameters,
                    )
                })
                .unwrap_or_default()
                .into_iter()
        };

        // Perform each scan sequentially over the current result filters.
        for scan_constraint in element_scan_parameters.get_scan_constraints() {
            scan_result_filters = if element_scan_parameters.get_is_single_thread_scan() {
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

    fn perform_scan_for_snapshot_filter(
        snapshot_region_filter: &SnapshotRegionFilter,
        optimized_scan_constraint: &OptimizedScanConstraint,
        snapshot_region: &SnapshotRegion,
        symbol_registry: &Arc<RwLock<SymbolRegistry>>,
        element_scan_parameters: &ElementScanParameters,
    ) -> Vec<SnapshotRegionFilter> {
        // Execute the scanner that corresponds to the mapped parameters.
        let scanner_instance = Self::aquire_scanner_instance(optimized_scan_constraint);
        let scan_result_filters = scanner_instance.scan_region(symbol_registry, snapshot_region, snapshot_region_filter, optimized_scan_constraint);

        // If the debug flag is provided, perform a scalar scan to ensure that our specialized scanner has the same results.
        if element_scan_parameters.get_debug_perform_validation_scan() {
            Self::perform_debug_scan(
                scanner_instance,
                symbol_registry,
                &scan_result_filters,
                snapshot_region,
                snapshot_region_filter,
                optimized_scan_constraint,
            );
        }
        scan_result_filters
    }

    fn aquire_scanner_instance(optimized_scan_constraint: &OptimizedScanConstraint) -> &'static dyn Scanner {
        // Execute the scanner that corresponds to the mapped parameters.
        match optimized_scan_constraint.get_mapped_scan_type() {
            MappedScanType::Invalid() => &ScannerNull {},
            MappedScanType::Scalar(scan_parameters_scalar) => match scan_parameters_scalar {
                ScanParametersScalar::SingleElement => &ScannerScalarSingleElement {},
                ScanParametersScalar::ScalarIterative => &ScannerScalarIterative {},
            },
            MappedScanType::Vector(scan_parameters_vector, vectorization_size) => match scan_parameters_vector {
                ScanParametersVector::Overlapping => match vectorization_size {
                    VectorizationSize::Vector16 => &ScannerVectorOverlapping::<16> {},
                    VectorizationSize::Vector32 => &ScannerVectorOverlapping::<32> {},
                    VectorizationSize::Vector64 => &ScannerVectorOverlapping::<64> {},
                },
                ScanParametersVector::Aligned => match vectorization_size {
                    VectorizationSize::Vector16 => &ScannerVectorAligned::<16> {},
                    VectorizationSize::Vector32 => &ScannerVectorAligned::<32> {},
                    VectorizationSize::Vector64 => &ScannerVectorAligned::<64> {},
                },
                ScanParametersVector::Sparse => match vectorization_size {
                    VectorizationSize::Vector16 => &ScannerVectorSparse::<16> {},
                    VectorizationSize::Vector32 => &ScannerVectorSparse::<32> {},
                    VectorizationSize::Vector64 => &ScannerVectorSparse::<64> {},
                },
                ScanParametersVector::OverlappingBytewiseStaggered => match vectorization_size {
                    VectorizationSize::Vector16 => &ScannerVectorOverlapping::<16> {},
                    VectorizationSize::Vector32 => &ScannerVectorOverlapping::<32> {},
                    VectorizationSize::Vector64 => &ScannerVectorOverlapping::<64> {},
                    /*
                    VectorizationSize::Vector16 => &ScannerVectorOverlappingBytewiseStaggered::<16> {},
                    VectorizationSize::Vector32 => &ScannerVectorOverlappingBytewiseStaggered::<32> {},
                    VectorizationSize::Vector64 => &ScannerVectorOverlappingBytewiseStaggered::<64> {},*/
                },
                ScanParametersVector::OverlappingBytewisePeriodic => match vectorization_size {
                    VectorizationSize::Vector16 => &ScannerVectorOverlapping::<16> {},
                    VectorizationSize::Vector32 => &ScannerVectorOverlapping::<32> {},
                    VectorizationSize::Vector64 => &ScannerVectorOverlapping::<64> {},
                    /*
                    VectorizationSize::Vector16 => &ScannerVectorOverlappingBytewisePeriodic::<16> {},
                    VectorizationSize::Vector32 => &ScannerVectorOverlappingBytewisePeriodic::<32> {},
                    VectorizationSize::Vector64 => &ScannerVectorOverlappingBytewisePeriodic::<64> {},*/
                },
            },
            MappedScanType::ByteArray(scan_parameters_byte_array) => match scan_parameters_byte_array {
                ScanParametersByteArray::ByteArrayBooyerMoore => &ScannerScalarByteArrayBooyerMoore {},
            },
        }
    }

    pub fn optimize_scan_constraint(
        element_scan_rule_registry: &Arc<RwLock<ElementScanRuleRegistry>>,
        symbol_registry: &Arc<RwLock<SymbolRegistry>>,
        snapshot_region: &SnapshotRegion,
        snapshot_region_filter: &SnapshotRegionFilter,
        snapshot_region_filter_collection: &SnapshotRegionFilterCollection,
        element_scan_parameters: &ElementScanParameters,
        scan_constraint: &ScanConstraint,
    ) -> Option<OptimizedScanConstraint> {
        let element_scan_rule_registry_guard = match element_scan_rule_registry.read() {
            Ok(registry) => registry,
            Err(error) => {
                log::error!("Failed to acquire read lock on ElementScanRuleRegistry: {}", error);
                return None;
            }
        };

        let mut optimized_scan_constraint = OptimizedScanConstraint::new(
            scan_constraint.get_data_value().clone(),
            element_scan_parameters.get_memory_alignment(),
            scan_constraint.get_compare_type(),
            element_scan_parameters.get_floating_point_tolerance(),
        );

        // Apply all scan rules to the mapped parameters.
        for (_id, rule) in element_scan_rule_registry_guard.get_registry().iter() {
            rule.map_parameters(
                symbol_registry,
                snapshot_region,
                snapshot_region_filter_collection,
                snapshot_region_filter,
                element_scan_parameters,
                &mut optimized_scan_constraint,
            );
        }

        Some(optimized_scan_constraint)
    }

    /// Performs a second scan over the provided snapshot region filter to ensure that the results of a specialized scan match
    /// the results of the scalar scan. This is a way to unit test complex scanner implementations on real world data.
    fn perform_debug_scan(
        scanner_instance: &dyn Scanner,
        symbol_registry: &Arc<RwLock<SymbolRegistry>>,
        comparison_filters: &Vec<SnapshotRegionFilter>,
        snapshot_region: &SnapshotRegion,
        snapshot_region_filter: &SnapshotRegionFilter,
        optimized_scan_constraint: &OptimizedScanConstraint,
    ) {
        let debug_scanner_instance = &ScannerScalarIterative {};
        let debug_filters = debug_scanner_instance.scan_region(symbol_registry, snapshot_region, snapshot_region_filter, optimized_scan_constraint);
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
