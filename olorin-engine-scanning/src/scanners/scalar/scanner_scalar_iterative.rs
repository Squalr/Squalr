use std::sync::{Arc, RwLock};

use crate::scanners::snapshot_scanner::Scanner;
use crate::scanners::structures::snapshot_region_filter_run_length_encoder::SnapshotRegionFilterRunLengthEncoder;
use olorin_engine_api::registries::symbols::symbol_registry::SymbolRegistry;
use olorin_engine_api::structures::scanning::comparisons::scan_function_scalar::ScanFunctionScalar;
use olorin_engine_api::structures::scanning::filters::snapshot_region_filter::SnapshotRegionFilter;
use olorin_engine_api::structures::scanning::parameters::mapped::mapped_scan_parameters::MappedScanParameters;
use olorin_engine_api::structures::snapshots::snapshot_region::SnapshotRegion;

pub struct ScannerScalarIterative {}

impl ScannerScalarIterative {}

/// Implements a scalar (ie CPU bound, non-SIMD) region scanning algorithm. This simply iterates over a region of memory,
/// comparing each element based on the provided parameters. Elements that pass the scan are grouped into filter ranges and returned.
impl Scanner for ScannerScalarIterative {
    fn get_scanner_name(&self) -> &'static str {
        &"Scalar Iterative"
    }

    /// Performs a sequential iteration over a region of memory, performing the scan comparison. A run-length encoding algorithm
    /// is used to generate new sub-regions as the scan progresses.
    fn scan_region(
        &self,
        symbol_registry: &Arc<RwLock<SymbolRegistry>>,
        snapshot_region: &SnapshotRegion,
        snapshot_region_filter: &SnapshotRegionFilter,
        mapped_scan_parameters: &MappedScanParameters,
    ) -> Vec<SnapshotRegionFilter> {
        let symbol_registry_guard = match symbol_registry.read() {
            Ok(registry) => registry,
            Err(error) => {
                log::error!("Failed to acquire read lock on SymbolRegistry: {}", error);

                return vec![];
            }
        };
        let base_address = snapshot_region_filter.get_base_address();
        let memory_alignment = mapped_scan_parameters.get_memory_alignment();
        let memory_alignment_size = memory_alignment as u64;
        let data_type_ref = mapped_scan_parameters.get_data_type_ref();
        let data_type_size = symbol_registry_guard.get_unit_size_in_bytes(data_type_ref);
        let data_type_size_padding = data_type_size.saturating_sub(memory_alignment_size);
        let element_count = snapshot_region_filter.get_element_count(symbol_registry, data_type_ref, memory_alignment);
        let current_value_pointer = snapshot_region.get_current_values_filter_pointer(&snapshot_region_filter);
        let previous_value_pointer = snapshot_region.get_previous_values_filter_pointer(&snapshot_region_filter);
        let mut run_length_encoder = SnapshotRegionFilterRunLengthEncoder::new(base_address);

        if let Some(scalar_compare_func) = mapped_scan_parameters.get_scan_function_scalar(symbol_registry) {
            match scalar_compare_func {
                ScanFunctionScalar::Immediate(compare_func) => {
                    for index in 0..element_count {
                        let current_value_pointer = unsafe { current_value_pointer.add((index * memory_alignment_size) as usize) };
                        let compare_result = compare_func(current_value_pointer);

                        if compare_result {
                            run_length_encoder.encode_range(memory_alignment_size);
                        } else {
                            run_length_encoder.finalize_current_encode_with_padding(memory_alignment_size, data_type_size_padding);
                        }
                    }
                }
                ScanFunctionScalar::RelativeOrDelta(compare_func) => {
                    for index in 0..element_count {
                        let current_value_pointer = unsafe { current_value_pointer.add((index * memory_alignment_size) as usize) };
                        let previous_value_pointer = unsafe { previous_value_pointer.add((index * memory_alignment_size) as usize) };
                        let compare_result = compare_func(current_value_pointer, previous_value_pointer);

                        if compare_result {
                            run_length_encoder.encode_range(memory_alignment_size);
                        } else {
                            run_length_encoder.finalize_current_encode_with_padding(memory_alignment_size, data_type_size_padding);
                        }
                    }
                }
            }
        }

        run_length_encoder.finalize_current_encode_with_padding(memory_alignment_size, data_type_size_padding);
        run_length_encoder.take_result_regions()
    }
}
