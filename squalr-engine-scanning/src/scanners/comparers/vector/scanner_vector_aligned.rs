use crate::filters::snapshot_region_filter::SnapshotRegionFilter;
use crate::scanners::comparers::snapshot_scanner::Scanner;
use crate::scanners::comparers::vector::encoder::scanner_vector_encoder::ScannerVectorEncoder;
use crate::scanners::parameters::scan_filter_parameters::ScanFilterParameters;
use crate::scanners::parameters::scan_parameters::ScanParameters;
use crate::snapshots::snapshot_region::SnapshotRegion;
use std::sync::Once;

pub struct ScannerVectorAligned<const VECTOR_SIZE_BITS: usize>;

macro_rules! impl_scanner_vector_aligned {
    ($bit_width:expr) => {
        impl ScannerVectorAligned<$bit_width> {
            pub fn get_instance() -> &'static ScannerVectorAligned<$bit_width> {
                static mut INSTANCE: Option<ScannerVectorAligned<$bit_width>> = None;
                static INIT: Once = Once::new();

                unsafe {
                    INIT.call_once(|| {
                        let instance = ScannerVectorAligned::<$bit_width>::new();
                        INSTANCE = Some(instance);
                    });

                    INSTANCE.as_ref().unwrap_unchecked()
                }
            }
        }
    };
}

impl_scanner_vector_aligned!(128);
impl_scanner_vector_aligned!(256);
impl_scanner_vector_aligned!(512);

impl<const VECTOR_SIZE_BITS: usize> ScannerVectorAligned<VECTOR_SIZE_BITS> {
    fn new(
    ) -> Self {
        Self { }
    }
}

macro_rules! impl_scanner_for_vector_aligned {
    ($bit_width:expr) => {
        impl Scanner for ScannerVectorAligned<$bit_width> {
            /// Performs a sequential iteration over a region of memory, performing the scan comparison.
            /// A run-length encoding algorithm is used to generate new sub-regions as the scan progresses.
            fn scan_region(
                &self,
                snapshot_region: &SnapshotRegion,
                snapshot_region_filter: &SnapshotRegionFilter,
                scan_parameters: &ScanParameters,
                scan_filter_parameters: &ScanFilterParameters,
            ) -> Vec<SnapshotRegionFilter> {
                let data_type = scan_filter_parameters.get_data_type();
                let data_type_size = data_type.size_in_bytes();
                let memory_alignment = scan_filter_parameters.get_memory_alignment_or_default();
                let aligned_element_count = snapshot_region_filter.get_element_count(memory_alignment, data_type_size);
                let encoder = ScannerVectorEncoder::<$bit_width>::get_instance();
                let current_value_pointer = snapshot_region.get_current_values_pointer(&snapshot_region_filter);
                let previous_value_pointer = snapshot_region.get_previous_values_pointer(&snapshot_region_filter);

                let results = encoder.encode(
                    current_value_pointer,
                    previous_value_pointer,
                    scan_parameters,
                    scan_filter_parameters,
                    snapshot_region_filter.get_base_address(),
                    aligned_element_count
                );

                return results;
            }
        }
    };
}

// Apply the macro to create implementations for 128, 256, and 512 bit widths
impl_scanner_for_vector_aligned!(128);
impl_scanner_for_vector_aligned!(256);
impl_scanner_for_vector_aligned!(512);
