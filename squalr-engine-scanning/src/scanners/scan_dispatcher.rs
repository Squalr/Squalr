use std::cmp;

use crate::scanners::scalar::scanner_scalar_byte_array_booyer_moore::ScannerScalarByteArrayBooyerMoore;
use crate::scanners::scalar::scanner_scalar_iterative::ScannerScalarIterative;
use crate::scanners::scalar::scanner_scalar_single_element::ScannerScalarSingleElement;
use crate::scanners::snapshot_scanner::Scanner;
use crate::scanners::vector::scanner_vector_aligned::ScannerVectorAligned;
use crate::scanners::vector::scanner_vector_overlapping::ScannerVectorOverlapping;
use crate::scanners::vector::scanner_vector_overlapping_bytewise_periodic::ScannerVectorOverlappingBytewisePeriodic;
use crate::scanners::vector::scanner_vector_overlapping_bytewise_staggered::ScannerVectorOverlappingBytewiseStaggered;
use crate::scanners::vector::scanner_vector_sparse::ScannerVectorSparse;
use crate::snapshots::snapshot_region::SnapshotRegion;
use rayon::iter::ParallelIterator;
use squalr_engine_api::structures::scanning::filters::snapshot_region_filter::SnapshotRegionFilter;
use squalr_engine_api::structures::scanning::filters::snapshot_region_filter_collection::SnapshotRegionFilterCollection;
use squalr_engine_api::structures::scanning::parameters::mapped::mapped_scan_parameters::MappedScanParameters;
use squalr_engine_api::structures::scanning::parameters::mapped::mapped_scan_type::{
    MappedScanType, ScanParametersByteArray, ScanParametersScalar, ScanParametersVector,
};
use squalr_engine_api::structures::scanning::parameters::mapped::vectorization_size::VectorizationSize;
use squalr_engine_api::structures::scanning::parameters::user::user_scan_parameters_global::UserScanParametersGlobal;

pub struct ScanDispatcher {}

/// Implements a scan dispatcher, which picks the best scanner based on the scan constraints and the region being scanned.
/// Choosing the best scanner is critical to maintaining high performance scans.
impl ScanDispatcher {
    /// Performs a scan over a provided filter collection, returning a new filter collection with the results.
    pub fn dispatch_scan(
        snapshot_region: &SnapshotRegion,
        snapshot_region_filter_collection: &SnapshotRegionFilterCollection,
        user_scan_parameters_global: &UserScanParametersGlobal,
    ) -> SnapshotRegionFilterCollection {
        let user_scan_parameters_local = snapshot_region_filter_collection.get_user_scan_parameters_local();

        if !user_scan_parameters_global.is_valid() {
            log::error!("Error in provided scan parameters, unable to start scan!");
            return SnapshotRegionFilterCollection::new(vec![], user_scan_parameters_local.clone());
        }

        // The main body of the scan routine performed on a given filter.
        let snapshot_region_scanner = |snapshot_region_filter| {
            // Combine the global and local parameters into a single container that optimizes the parameters for selecting the best scanner implementation.
            let scan_parameters = MappedScanParameters::new(snapshot_region_filter, user_scan_parameters_global, user_scan_parameters_local);

            // Execute the scanner that corresponds to the mapped parameters.
            let scanner_instance = Self::aquire_scanner_instance(&scan_parameters);
            let filters = scanner_instance.scan_region(snapshot_region, snapshot_region_filter, &scan_parameters);

            // If the debug flag is provided, perform a scalar scan to ensure that our specialized scanner has the same results.
            if user_scan_parameters_global.get_perform_debug_shadow_scan() {
                Self::perform_debug_scan(scanner_instance, &filters, snapshot_region, snapshot_region_filter, &scan_parameters);
            }

            if filters.len() > 0 { Some(filters) } else { None }
        };

        // Run the scan either single-threaded or parallel based on settings. Single-thread is not advised unless debugging.
        let single_thread_scan = user_scan_parameters_global.is_single_thread_scan();
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
            snapshot_region_filter_collection
                .get_user_scan_parameters_local()
                .clone(),
        )
    }

    fn aquire_scanner_instance(scan_parameters: &MappedScanParameters) -> &'static dyn Scanner {
        // Execute the scanner that corresponds to the mapped parameters.
        match scan_parameters.get_mapped_scan_type() {
            MappedScanType::Scalar(scan_parameters_scalar) => match scan_parameters_scalar {
                ScanParametersScalar::SingleElement => &ScannerScalarSingleElement {},
                ScanParametersScalar::ScalarIterative => &ScannerScalarIterative {},
            },
            MappedScanType::Vector(scan_parameters_vector) => match scan_parameters_vector {
                ScanParametersVector::Overlapping => match scan_parameters.get_vectorization_size() {
                    VectorizationSize::Vector16 => &ScannerVectorOverlapping::<16> {},
                    VectorizationSize::Vector32 => &ScannerVectorOverlapping::<32> {},
                    VectorizationSize::Vector64 => &ScannerVectorOverlapping::<64> {},
                },
                ScanParametersVector::Aligned => match scan_parameters.get_vectorization_size() {
                    VectorizationSize::Vector16 => &ScannerVectorAligned::<16> {},
                    VectorizationSize::Vector32 => &ScannerVectorAligned::<32> {},
                    VectorizationSize::Vector64 => &ScannerVectorAligned::<64> {},
                },
                ScanParametersVector::Sparse => match scan_parameters.get_vectorization_size() {
                    VectorizationSize::Vector16 => &ScannerVectorSparse::<16> {},
                    VectorizationSize::Vector32 => &ScannerVectorSparse::<32> {},
                    VectorizationSize::Vector64 => &ScannerVectorSparse::<64> {},
                },
                ScanParametersVector::OverlappingBytewiseStaggered => match scan_parameters.get_vectorization_size() {
                    VectorizationSize::Vector16 => &ScannerVectorOverlapping::<16> {},
                    VectorizationSize::Vector32 => &ScannerVectorOverlapping::<32> {},
                    VectorizationSize::Vector64 => &ScannerVectorOverlapping::<64> {},
                    /*
                    VectorizationSize::Vector16 => &ScannerVectorOverlappingBytewiseStaggered::<16> {},
                    VectorizationSize::Vector32 => &ScannerVectorOverlappingBytewiseStaggered::<32> {},
                    VectorizationSize::Vector64 => &ScannerVectorOverlappingBytewiseStaggered::<64> {},*/
                },
                ScanParametersVector::OverlappingBytewisePeriodic => match scan_parameters.get_vectorization_size() {
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
        scan_parameters: &MappedScanParameters,
    ) {
        let debug_scanner_instance = &ScannerScalarIterative {};
        let debug_filters = debug_scanner_instance.scan_region(snapshot_region, snapshot_region_filter, scan_parameters);
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
