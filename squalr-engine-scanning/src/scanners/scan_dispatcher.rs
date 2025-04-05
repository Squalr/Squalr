use crate::scanners::scalar::scanner_scalar_byte_array_booyer_moore::ScannerScalarByteArrayBooyerMoore;
use crate::scanners::scalar::scanner_scalar_iterative::ScannerScalarIterative;
use crate::scanners::scalar::scanner_scalar_single_element::ScannerScalarSingleElement;
use crate::scanners::snapshot_scanner::Scanner;
use crate::scanners::vector::scanner_vector_aligned::ScannerVectorAligned;
use crate::scanners::vector::scanner_vector_overlapping_bytewise_periodic::ScannerVectorOverlappingBytewisePeriodic;
use crate::scanners::vector::scanner_vector_overlapping_bytewise_staggered::ScannerVectorOverlappingBytewiseStaggered;
use crate::scanners::vector::scanner_vector_sparse::ScannerVectorSparse;
use crate::snapshots::snapshot_region::SnapshotRegion;
use rayon::iter::ParallelIterator;
use squalr_engine_api::structures::scanning::filters::snapshot_region_filter_collection::SnapshotRegionFilterCollection;
use squalr_engine_api::structures::scanning::parameters::mapped_scan_parameters::{
    MappedScanParameters, ScanParametersByteArray, ScanParametersScalar, ScanParametersVector, VectorizationSize,
};
use squalr_engine_api::structures::scanning::parameters::user_scan_parameters_global::UserScanParametersGlobal;

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
        // The main body of the scan routine performed on a given filter.
        let snapshot_region_scanner = |snapshot_region_filter| {
            let user_scan_parameters_local = snapshot_region_filter_collection.get_user_scan_parameters_local();

            // Combine the global and local parameters into a single container that optimizes the parameters for selecting the best scanner implementation.
            let scan_parameters = MappedScanParameters::new(snapshot_region_filter, user_scan_parameters_global, user_scan_parameters_local);

            // Execute the scanner that corresponds to the mapped parameters.
            let filters = match scan_parameters {
                MappedScanParameters::Scalar(scan_parameters_scalar) => match scan_parameters_scalar {
                    ScanParametersScalar::SingleElement(scan_parameters) => {
                        ScannerScalarSingleElement::scan_region(snapshot_region, snapshot_region_filter, &scan_parameters)
                    }
                    ScanParametersScalar::ScalarIterative(scan_parameters) => {
                        ScannerScalarIterative::scan_region(snapshot_region, snapshot_region_filter, &scan_parameters)
                    }
                },
                MappedScanParameters::Vector(scan_parameters_vector) => match scan_parameters_vector {
                    ScanParametersVector::Aligned(scan_parameters) => match scan_parameters.get_vectorization_size() {
                        VectorizationSize::Vector16 => ScannerVectorAligned::<16>::scan_region(snapshot_region, snapshot_region_filter, &scan_parameters),
                        VectorizationSize::Vector32 => ScannerVectorAligned::<32>::scan_region(snapshot_region, snapshot_region_filter, &scan_parameters),
                        VectorizationSize::Vector64 => ScannerVectorAligned::<64>::scan_region(snapshot_region, snapshot_region_filter, &scan_parameters),
                    },
                    ScanParametersVector::Sparse(scan_parameters) => match scan_parameters.get_vectorization_size() {
                        VectorizationSize::Vector16 => ScannerVectorSparse::<16>::scan_region(snapshot_region, snapshot_region_filter, &scan_parameters),
                        VectorizationSize::Vector32 => ScannerVectorSparse::<32>::scan_region(snapshot_region, snapshot_region_filter, &scan_parameters),
                        VectorizationSize::Vector64 => ScannerVectorSparse::<64>::scan_region(snapshot_region, snapshot_region_filter, &scan_parameters),
                    },
                    ScanParametersVector::OverlappingBytewiseStaggered(scan_parameters) => match scan_parameters.get_vectorization_size() {
                        VectorizationSize::Vector16 => {
                            ScannerVectorOverlappingBytewiseStaggered::<16>::scan_region(snapshot_region, snapshot_region_filter, &scan_parameters)
                        }
                        VectorizationSize::Vector32 => {
                            ScannerVectorOverlappingBytewiseStaggered::<32>::scan_region(snapshot_region, snapshot_region_filter, &scan_parameters)
                        }
                        VectorizationSize::Vector64 => {
                            ScannerVectorOverlappingBytewiseStaggered::<64>::scan_region(snapshot_region, snapshot_region_filter, &scan_parameters)
                        }
                    },
                    ScanParametersVector::OverlappingBytewisePeriodic(scan_parameters) => match scan_parameters.get_vectorization_size() {
                        VectorizationSize::Vector16 => {
                            ScannerVectorOverlappingBytewisePeriodic::<16>::scan_region(snapshot_region, snapshot_region_filter, &scan_parameters)
                        }
                        VectorizationSize::Vector32 => {
                            ScannerVectorOverlappingBytewisePeriodic::<32>::scan_region(snapshot_region, snapshot_region_filter, &scan_parameters)
                        }
                        VectorizationSize::Vector64 => {
                            ScannerVectorOverlappingBytewisePeriodic::<64>::scan_region(snapshot_region, snapshot_region_filter, &scan_parameters)
                        }
                    },
                },
                MappedScanParameters::ByteArray(scan_parameters_byte_array) => match scan_parameters_byte_array {
                    ScanParametersByteArray::ByteArrayBooyerMoore(scan_parameters) => {
                        ScannerScalarByteArrayBooyerMoore::scan_region(snapshot_region, snapshot_region_filter, &scan_parameters)
                    }
                },
            };

            if filters.len() > 0 { Some(filters) } else { None }
        };

        // Run the scan either single-threaded or parallel based on settings. Single-thread is not advised unless debugging.
        let result_snapshot_region_filters = if user_scan_parameters_global.is_single_thread_scan() {
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
}
