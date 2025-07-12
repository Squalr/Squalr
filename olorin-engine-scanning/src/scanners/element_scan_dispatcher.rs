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
use olorin_engine_api::structures::scanning::filters::snapshot_region_filter::SnapshotRegionFilter;
use olorin_engine_api::structures::scanning::filters::snapshot_region_filter_collection::SnapshotRegionFilterCollection;
use olorin_engine_api::structures::scanning::parameters::element_scan::element_scan_parameters::ElementScanParameters;
use olorin_engine_api::structures::scanning::parameters::mapped::mapped_scan_parameters::MappedScanParameters;
use olorin_engine_api::structures::scanning::parameters::mapped::mapped_scan_type::{
    MappedScanType, ScanParametersByteArray, ScanParametersScalar, ScanParametersVector,
};
use olorin_engine_api::structures::scanning::parameters::mapped::vectorization_size::VectorizationSize;
use olorin_engine_api::structures::snapshots::snapshot_region::SnapshotRegion;
use std::cmp;

pub struct ElementScanDispatcher {}

/// Implements a scan dispatcher, which picks the best scanner based on the scan constraints and the region being scanned.
/// Choosing the best scanner is critical to maintaining high performance scans.
impl ElementScanDispatcher {
    /// Performs a scan over a provided filter collection, returning a new filter collection with the results.
    pub fn dispatch_scan(
        snapshot_region: &SnapshotRegion,
        snapshot_region_filter_collection: &SnapshotRegionFilterCollection,
        element_scan_parameters: &ElementScanParameters,
    ) -> SnapshotRegionFilterCollection {
        if !element_scan_parameters.is_valid_for_data_type(snapshot_region_filter_collection.get_data_type()) {
            log::error!("Error in provided scan parameters, unable to start scan!");
            return SnapshotRegionFilterCollection::new(
                vec![],
                snapshot_region_filter_collection.get_data_type().clone(),
                snapshot_region_filter_collection.get_memory_alignment(),
            );
        }

        // The main body of the scan routine performed on a given filter.
        let snapshot_region_scanner = |snapshot_region_filter: &SnapshotRegionFilter| {
            // Map the user scan parameters into an optimized form for improved scanning efficiency.
            let mapped_scan_parameters = ElementScanExecutionPlanner::map(snapshot_region_filter, snapshot_region_filter_collection, element_scan_parameters);

            // Execute the scanner that corresponds to the mapped parameters.
            let scanner_instance = Self::aquire_scanner_instance(&mapped_scan_parameters);
            let filters = scanner_instance.scan_region(snapshot_region, snapshot_region_filter, &mapped_scan_parameters);

            // If the debug flag is provided, perform a scalar scan to ensure that our specialized scanner has the same results.
            if element_scan_parameters.get_debug_perform_validation_scan() {
                Self::perform_debug_scan(scanner_instance, &filters, snapshot_region, snapshot_region_filter, &mapped_scan_parameters);
            }

            if filters.len() > 0 { Some(filters) } else { None }
        };

        // Run the scan either single-threaded or parallel based on settings. Single-thread is not advised unless debugging.
        let single_thread_scan = element_scan_parameters.is_single_thread_scan();
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
            result_snapshot_region_filters,
            snapshot_region_filter_collection.get_data_type().clone(),
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
        filters: &Vec<SnapshotRegionFilter>,
        snapshot_region: &SnapshotRegion,
        snapshot_region_filter: &SnapshotRegionFilter,
        mapped_scan_parameters: &MappedScanParameters,
    ) {
        let debug_scanner_instance = &ScannerScalarIterative {};
        let debug_filters = debug_scanner_instance.scan_region(snapshot_region, snapshot_region_filter, mapped_scan_parameters);
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
