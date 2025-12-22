use crate::execution_planner::element_scan::element_scan_execution_planner::ElementScanExecutionPlanner;
use crate::scanners::scalar::scanner_scalar_byte_array_booyer_moore::ScannerScalarByteArrayBooyerMoore;
use crate::scanners::scalar::scanner_scalar_iterative::ScannerScalarIterative;
use crate::scanners::scalar::scanner_scalar_single_element::ScannerScalarSingleElement;
use crate::scanners::snapshot_scanner::Scanner;
use crate::scanners::vector::scanner_vector_aligned::ScannerVectorAligned;
use crate::scanners::vector::scanner_vector_overlapping::ScannerVectorOverlapping;
use crate::scanners::vector::scanner_vector_overlapping_bytewise_periodic::ScannerVectorOverlappingBytewisePeriodic;
use crate::scanners::vector::scanner_vector_overlapping_bytewise_staggered::ScannerVectorOverlappingBytewiseStaggered;
use crate::scanners::vector::scanner_vector_sparse::ScannerVectorSparse;
use rayon::iter::ParallelIterator;
use squalr_engine_api::registries::scan_rules::element_scan_rule_registry::ElementScanRuleRegistry;
use squalr_engine_api::registries::symbols::symbol_registry::SymbolRegistry;
use squalr_engine_api::structures::scanning::filters::snapshot_region_filter::SnapshotRegionFilter;
use squalr_engine_api::structures::scanning::filters::snapshot_region_filter_collection::SnapshotRegionFilterCollection;
use squalr_engine_api::structures::scanning::parameters::element_scan::element_scan_parameters::ElementScanParameters;
use squalr_engine_api::structures::scanning::parameters::mapped::mapped_scan_parameters::MappedScanParameters;
use squalr_engine_api::structures::scanning::parameters::mapped::mapped_scan_type::{
    MappedScanType, ScanParametersByteArray, ScanParametersScalar, ScanParametersVector,
};
use squalr_engine_api::structures::scanning::parameters::mapped::vectorization_size::VectorizationSize;
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
        let symbol_registry_guard = match symbol_registry.read() {
            Ok(registry) => registry,
            Err(error) => {
                log::error!("Failed to acquire read lock on SymbolRegistry: {}", error);

                return SnapshotRegionFilterCollection::new(
                    symbol_registry,
                    vec![],
                    snapshot_region_filter_collection.get_data_type_ref().clone(),
                    snapshot_region_filter_collection.get_memory_alignment(),
                );
            }
        };

        /*
        if !element_scan_parameters.is_valid_for_data_type(snapshot_region_filter_collection.get_data_type_ref()) {
            log::error!("Error in provided scan parameters, unable to start scan!");
            return SnapshotRegionFilterCollection::new(
                symbol_registry,
                vec![],
                snapshot_region_filter_collection.get_data_type_ref().clone(),
                snapshot_region_filter_collection.get_memory_alignment(),
            );
        }*/

        // The main body of the scan routine performed on a given filter.
        let parametrized_snapshot_region_scanner = |snapshot_region_filter: &SnapshotRegionFilter, mapped_scan_parameters: &MappedScanParameters| {
            // Execute the scanner that corresponds to the mapped parameters.
            let scanner_instance = Self::aquire_scanner_instance(&mapped_scan_parameters);
            let filters = scanner_instance.scan_region(symbol_registry, snapshot_region, snapshot_region_filter, &mapped_scan_parameters);

            // If the debug flag is provided, perform a scalar scan to ensure that our specialized scanner has the same results.
            if element_scan_parameters.get_debug_perform_validation_scan() {
                Self::perform_debug_scan(
                    scanner_instance,
                    symbol_registry,
                    &filters,
                    snapshot_region,
                    snapshot_region_filter,
                    &mapped_scan_parameters,
                );
            }

            filters
        };

        // The orchestrator that allows multiple scan parameters to combine when scanning a single snapshot region.
        // Currently, this assumes only AND operations are supported, ie value >= 2000 && value <= 5000.
        // This works by simply running constraints sequentially to produce filters.
        let snapshot_region_scanner = |snapshot_region_filter: &SnapshotRegionFilter| {
            // Map the user scan parameters into an optimized form for improved scanning efficiency.
            let mapped_scan_parameters_vec = ElementScanExecutionPlanner::map(
                element_scan_rule_registry,
                symbol_registry,
                snapshot_region_filter,
                snapshot_region_filter_collection,
                element_scan_parameters,
            );

            if mapped_scan_parameters_vec.is_empty() {
                return None;
            }

            // Start with the input filter as the initial set.
            let mut scan_result_filters: Vec<SnapshotRegionFilter> = vec![snapshot_region_filter.clone()];

            for mapped_scan_parameters in mapped_scan_parameters_vec {
                // Iteratively apply each mapped scan parameters in order to update our filters.
                scan_result_filters = scan_result_filters
                    .iter()
                    .flat_map(|filter| parametrized_snapshot_region_scanner(filter, &mapped_scan_parameters))
                    .collect();
            }

            if !scan_result_filters.is_empty() { Some(scan_result_filters) } else { None }
        };

        // Run the scan either single-threaded or parallel based on settings. Single-thread is not advised unless debugging.
        let single_thread_scan = element_scan_parameters.get_is_single_thread_scan();
        let result_snapshot_region_filters = if single_thread_scan {
            snapshot_region_filter_collection
                .iter()
                .filter_map(snapshot_region_scanner)
                .collect()
        } else {
            snapshot_region_filter_collection
                .par_iter()
                .filter_map(snapshot_region_scanner)
                .collect()
        };

        SnapshotRegionFilterCollection::new(
            symbol_registry,
            result_snapshot_region_filters,
            snapshot_region_filter_collection.get_data_type_ref().clone(),
            snapshot_region_filter_collection.get_memory_alignment(),
        )
    }

    fn aquire_scanner_instance(mapped_scan_parameters: &MappedScanParameters) -> &'static dyn Scanner {
        // Execute the scanner that corresponds to the mapped parameters.
        match mapped_scan_parameters.get_mapped_scan_type() {
            MappedScanType::Scalar(scan_parameters_scalar) => match scan_parameters_scalar {
                ScanParametersScalar::SingleElement => &ScannerScalarSingleElement {},
                ScanParametersScalar::ScalarIterative => &ScannerScalarIterative {},
            },
            MappedScanType::Vector(scan_parameters_vector) => match scan_parameters_vector {
                ScanParametersVector::Overlapping => match mapped_scan_parameters.get_vectorization_size() {
                    VectorizationSize::Vector16 => &ScannerVectorOverlapping::<16> {},
                    VectorizationSize::Vector32 => &ScannerVectorOverlapping::<32> {},
                    VectorizationSize::Vector64 => &ScannerVectorOverlapping::<64> {},
                },
                ScanParametersVector::Aligned => match mapped_scan_parameters.get_vectorization_size() {
                    VectorizationSize::Vector16 => &ScannerVectorAligned::<16> {},
                    VectorizationSize::Vector32 => &ScannerVectorAligned::<32> {},
                    VectorizationSize::Vector64 => &ScannerVectorAligned::<64> {},
                },
                ScanParametersVector::Sparse => match mapped_scan_parameters.get_vectorization_size() {
                    VectorizationSize::Vector16 => &ScannerVectorSparse::<16> {},
                    VectorizationSize::Vector32 => &ScannerVectorSparse::<32> {},
                    VectorizationSize::Vector64 => &ScannerVectorSparse::<64> {},
                },
                ScanParametersVector::OverlappingBytewiseStaggered => match mapped_scan_parameters.get_vectorization_size() {
                    VectorizationSize::Vector16 => &ScannerVectorOverlapping::<16> {},
                    VectorizationSize::Vector32 => &ScannerVectorOverlapping::<32> {},
                    VectorizationSize::Vector64 => &ScannerVectorOverlapping::<64> {},
                    /*
                    VectorizationSize::Vector16 => &ScannerVectorOverlappingBytewiseStaggered::<16> {},
                    VectorizationSize::Vector32 => &ScannerVectorOverlappingBytewiseStaggered::<32> {},
                    VectorizationSize::Vector64 => &ScannerVectorOverlappingBytewiseStaggered::<64> {},*/
                },
                ScanParametersVector::OverlappingBytewisePeriodic => match mapped_scan_parameters.get_vectorization_size() {
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

    /// Performs a second scan over the provided snapshot region filter to ensure that the results of a specialized scan match
    /// the results of the scalar scan. This is a way to unit test complex scanner implementations on real world data.
    fn perform_debug_scan(
        scanner_instance: &dyn Scanner,
        symbol_registry: &Arc<RwLock<SymbolRegistry>>,
        filters: &Vec<SnapshotRegionFilter>,
        snapshot_region: &SnapshotRegion,
        snapshot_region_filter: &SnapshotRegionFilter,
        mapped_scan_parameters: &MappedScanParameters,
    ) {
        let debug_scanner_instance = &ScannerScalarIterative {};
        let debug_filters = debug_scanner_instance.scan_region(symbol_registry, snapshot_region, snapshot_region_filter, mapped_scan_parameters);
        let has_length_match = debug_filters.len() == filters.len();

        if !has_length_match {
            log::error!(
                "{}",
                format!(
                    "Specialized scanner produced incorrect number of results: {}",
                    scanner_instance.get_scanner_name()
                )
            );
        }

        for index in 0..(cmp::min(debug_filters.len(), filters.len())) {
            let debug_filter = &debug_filters[index];
            let filter = &filters[index];

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
