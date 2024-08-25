
use crate::filters::snapshot_region_filter::SnapshotRegionFilter;
use crate::scanners::comparers::snapshot_scanner::Scanner;
use crate::scanners::comparers::vector::encoder::scanner_vector_comparer::ScannerVectorComparer;
use crate::scanners::comparers::vector::encoder::scanner_vector_encoder::ScannerVectorEncoder;
use crate::scanners::parameters::scan_filter_parameters::ScanFilterParameters;
use crate::scanners::parameters::scan_parameters::ScanParameters;
use crate::snapshots::snapshot_region::SnapshotRegion;
use squalr_engine_memory::memory_alignment::MemoryAlignment;
use std::sync::Once;
use std::simd::{u8x16, u8x32, u8x64};

pub struct ScannerVectorStaggered<const VECTOR_SIZE_BITS: usize>;

macro_rules! impl_scanner_vector_staggered {
    ($vector_bit_size:expr, $simd_type:ty, $vector_size_bytes:expr) => {
        impl ScannerVectorStaggered<$vector_bit_size> {
            pub fn get_instance() -> &'static ScannerVectorStaggered<$vector_bit_size> {
                static mut INSTANCE: Option<ScannerVectorStaggered<$vector_bit_size>> = None;
                static INIT: Once = Once::new();

                unsafe {
                    INIT.call_once(|| {
                        let instance = ScannerVectorStaggered::<$vector_bit_size>::new();
                        INSTANCE = Some(instance);
                    });

                    INSTANCE.as_ref().unwrap_unchecked()
                }
            }

            fn new() -> Self {
                Self {}
            }
            
            fn get_staggered_mask(data_type_size: usize, memory_alignment: MemoryAlignment) -> Vec<$simd_type> {
                match (data_type_size, memory_alignment) {
                    // Data type size 2
                    (2, MemoryAlignment::Alignment1) => vec![
                        {
                            let mut mask = [0u8; $vector_size_bytes];
                            for index in (0..$vector_size_bytes).step_by(2) {
                                mask[index] = 0xFF;
                            }
                            <$simd_type>::from_array(mask)
                        },
                        {
                            let mut mask = [0u8; $vector_size_bytes];
                            for index in (1..$vector_size_bytes).step_by(2) {
                                mask[index] = 0xFF;
                            }
                            <$simd_type>::from_array(mask)
                        },
                    ],
                    // Data type size 4
                    (4, MemoryAlignment::Alignment1) => vec![
                        {
                            let mut mask = [0u8; $vector_size_bytes];
                            for index in (0..$vector_size_bytes).step_by(4) {
                                mask[index] = 0xFF;
                            }
                            <$simd_type>::from_array(mask)
                        },
                        {
                            let mut mask = [0u8; $vector_size_bytes];
                            for index in (1..$vector_size_bytes).step_by(4) {
                                mask[index] = 0xFF;
                            }
                            <$simd_type>::from_array(mask)
                        },
                        {
                            let mut mask = [0u8; $vector_size_bytes];
                            for index in (2..$vector_size_bytes).step_by(4) {
                                mask[index] = 0xFF;
                            }
                            <$simd_type>::from_array(mask)
                        },
                        {
                            let mut mask = [0u8; $vector_size_bytes];
                            for index in (3..$vector_size_bytes).step_by(4) {
                                mask[index] = 0xFF;
                            }
                            <$simd_type>::from_array(mask)
                        },
                    ],
                    (4, MemoryAlignment::Alignment2) => vec![
                        {
                            let mut mask = [0u8; $vector_size_bytes];
                            for index in (0..$vector_size_bytes).step_by(4) {
                                mask[index] = 0xFF;
                                mask[index + 1] = 0xFF;
                            }
                            <$simd_type>::from_array(mask)
                        },
                        {
                            let mut mask = [0u8; $vector_size_bytes];
                            for index in (2..$vector_size_bytes).step_by(4) {
                                mask[index] = 0xFF;
                                mask[index + 1] = 0xFF;
                            }
                            <$simd_type>::from_array(mask)
                        },
                    ],
                    // Data type size 8
                    (8, MemoryAlignment::Alignment1) => vec![
                        {
                            let mut mask = [0u8; $vector_size_bytes];
                            for index in (0..$vector_size_bytes).step_by(8) {
                                mask[index] = 0xFF;
                            }
                            <$simd_type>::from_array(mask)
                        },
                        {
                            let mut mask = [0u8; $vector_size_bytes];
                            for index in (1..$vector_size_bytes).step_by(8) {
                                mask[index] = 0xFF;
                            }
                            <$simd_type>::from_array(mask)
                        },
                        {
                            let mut mask = [0u8; $vector_size_bytes];
                            for index in (2..$vector_size_bytes).step_by(8) {
                                mask[index] = 0xFF;
                            }
                            <$simd_type>::from_array(mask)
                        },
                        {
                            let mut mask = [0u8; $vector_size_bytes];
                            for index in (3..$vector_size_bytes).step_by(8) {
                                mask[index] = 0xFF;
                            }
                            <$simd_type>::from_array(mask)
                        },
                        {
                            let mut mask = [0u8; $vector_size_bytes];
                            for index in (4..$vector_size_bytes).step_by(8) {
                                mask[index] = 0xFF;
                            }
                            <$simd_type>::from_array(mask)
                        },
                        {
                            let mut mask = [0u8; $vector_size_bytes];
                            for index in (5..$vector_size_bytes).step_by(8) {
                                mask[index] = 0xFF;
                            }
                            <$simd_type>::from_array(mask)
                        },
                        {
                            let mut mask = [0u8; $vector_size_bytes];
                            for index in (6..$vector_size_bytes).step_by(8) {
                                mask[index] = 0xFF;
                            }
                            <$simd_type>::from_array(mask)
                        },
                        {
                            let mut mask = [0u8; $vector_size_bytes];
                            for index in (7..$vector_size_bytes).step_by(8) {
                                mask[index] = 0xFF;
                            }
                            <$simd_type>::from_array(mask)
                        },
                    ],
                    (8, MemoryAlignment::Alignment2) => vec![
                        {
                            let mut mask = [0u8; $vector_size_bytes];
                            for index in (0..$vector_size_bytes).step_by(8) {
                                mask[index] = 0xFF;
                                mask[index + 1] = 0xFF;
                            }
                            <$simd_type>::from_array(mask)
                        },
                        {
                            let mut mask = [0u8; $vector_size_bytes];
                            for index in (2..$vector_size_bytes).step_by(8) {
                                mask[index] = 0xFF;
                                mask[index + 1] = 0xFF;
                            }
                            <$simd_type>::from_array(mask)
                        },
                        {
                            let mut mask = [0u8; $vector_size_bytes];
                            for index in (4..$vector_size_bytes).step_by(8) {
                                mask[index] = 0xFF;
                                mask[index + 1] = 0xFF;
                            }
                            <$simd_type>::from_array(mask)
                        },
                        {
                            let mut mask = [0u8; $vector_size_bytes];
                            for index in (6..$vector_size_bytes).step_by(8) {
                                mask[index] = 0xFF;
                                mask[index + 1] = 0xFF;
                            }
                            <$simd_type>::from_array(mask)
                        },
                    ],
                    (8, MemoryAlignment::Alignment4) => vec![
                        {
                            let mut mask = [0u8; $vector_size_bytes];
                            for index in (0..$vector_size_bytes).step_by(8) {
                                mask[index] = 0xFF;
                                mask[index + 1] = 0xFF;
                                mask[index + 2] = 0xFF;
                                mask[index + 3] = 0xFF;
                            }
                            <$simd_type>::from_array(mask)
                        },
                        {
                            let mut mask = [0u8; $vector_size_bytes];
                            for index in (4..$vector_size_bytes).step_by(8) {
                                mask[index] = 0xFF;
                                mask[index + 1] = 0xFF;
                                mask[index + 2] = 0xFF;
                                mask[index + 3] = 0xFF;
                            }
                            <$simd_type>::from_array(mask)
                        },
                    ],
                    _ => panic!("Unexpected size/alignment combination.")
                }
            }
        }
    };
}

/// This algorithm is mostly the same as scanner_vector_aligned. The primary difference is that instead of doing one vector comparison,
/// multiple scans must be done per vector to scan for mis-aligned values. This adds 2 to 8 additional vector scans, based on the alignment
/// and data type. Each of these sub-scans is masked against a stagger mask to create the scan result. For example, scanning for 4-byte integer
/// with a value of 0 with an alignment of 2-bytes against <0, 0, 0, 0, 55, 0, 0, 0, 0, 0> would need to return <255, 0, 0, 0, 255, 255, ..>.
/// This is accomplished by performing a full vector scan, then masking it against the appropriate stagger mask to extract the relevant scan
/// results for that iteration. These sub-scans are OR'd together to get a run-length encoded vector of all scan matches.
macro_rules! impl_scanner_for_vector_staggered {
    ($vector_bit_size:expr, $simd_type:ty) => {
        impl Scanner for ScannerVectorStaggered<$vector_bit_size> {
            fn scan_region(
                &self,
                snapshot_region: &SnapshotRegion,
                snapshot_region_filter: &SnapshotRegionFilter,
                scan_parameters: &ScanParameters,
                scan_filter_parameters: &ScanFilterParameters,
            ) -> Vec<SnapshotRegionFilter> {
                let data_type = scan_filter_parameters.get_data_type();
                let data_type_size = data_type.get_size_in_bytes();
                let memory_alignment = scan_filter_parameters.get_memory_alignment_or_default();
                let encoder = ScannerVectorEncoder::<$vector_bit_size>::get_instance();
                let vector_comparer = ScannerVectorComparer::<$vector_bit_size>::get_instance();

                let results = encoder.encode(
                    snapshot_region.get_current_values_pointer(&snapshot_region_filter),
                    snapshot_region.get_previous_values_pointer(&snapshot_region_filter),
                    scan_parameters,
                    scan_filter_parameters,
                    snapshot_region_filter.get_base_address(),
                    snapshot_region_filter.get_element_count(memory_alignment, data_type_size),
                    vector_comparer,
                    // TODO wrap the comparer's function in a custom class call that augments the scan results using the set of staggered masks.
                    // Until that happens, this class will not work as intended at all.
                    Self::get_staggered_mask(0, memory_alignment)[0],
                );

                return results;
            }
        }
    };
}

// Create implementations for 128, 256, and 512 SIMD vector widths.
impl_scanner_vector_staggered!(128, u8x16, 16);
impl_scanner_vector_staggered!(256, u8x32, 32);
impl_scanner_vector_staggered!(512, u8x64, 64);

impl_scanner_for_vector_staggered!(128, u8x16);
impl_scanner_for_vector_staggered!(256, u8x32);
impl_scanner_for_vector_staggered!(512, u8x64);
