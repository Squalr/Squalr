use crate::execution_planner::element_scan::element_scan_execution_rule::ScanParameterMappingRule;
use squalr_engine_api::structures::scanning::{
    comparisons::{scan_compare_type::ScanCompareType, scan_compare_type_immediate::ScanCompareTypeImmediate},
    filters::{snapshot_region_filter::SnapshotRegionFilter, snapshot_region_filter_collection::SnapshotRegionFilterCollection},
    parameters::{element_scan::element_scan_parameters::ElementScanParameters, mapped::mapped_scan_parameters::MappedScanParameters},
};

struct MapUnsignedGreaterThanZeroToNotEqual {}

impl ScanParameterMappingRule for MapUnsignedGreaterThanZeroToNotEqual {
    fn map_parameters(
        snapshot_region_filter_collection: &SnapshotRegionFilterCollection,
        snapshot_region_filter: &SnapshotRegionFilter,
        original_scan_parameters: &ElementScanParameters,
        mapped_parameters: &mut MappedScanParameters,
    ) {
        /*
        if mapped_parameters.get_dynamic_struct() {
            //
        }*/

        match mapped_parameters.get_compare_type() {
            ScanCompareType::Immediate(scan_compare_type_immediate) => match scan_compare_type_immediate {
                ScanCompareTypeImmediate::GreaterThan => {
                    //
                    mapped_parameters.set_compare_type(ScanCompareType::Immediate(ScanCompareTypeImmediate::NotEqual));
                }
                _ => {}
            },
            _ => {}
        }
    }
}

/*

        // First try a single element scanner. This is valid even for cases like array of byte scans, as all data types support basic equality checks.
        if Self::is_single_element_scan(snapshot_region_filter, data_type_ref, memory_alignment) {
            return mapped_params;
        }

        // Try to map the scan value to primitive scans for performance gains.
        // For example, a byte array scan of 2 bytes can be mapped to a u16 scan.
        match Self::try_map_to_primitive(mapped_params.get_compare_type(), &mapped_params.get_data_value()) {
            Some(mapped_data_type_ref) => {
                // Mapping onto a primitive type map was successful. Update our new internal data type, and proceed with this as the new type.
                mapped_params.data_value.remap_data_type(mapped_data_type_ref);
            }
            None => {
                if Self::can_remap_to_byte_array(mapped_params.get_compare_type(), &mapped_params.get_data_value()) {
                    // JIRA: Okay but this breaks if they scan for an array of floats, since float comparisons are actually non-discrete.
                    if mapped_params.data_value.get_data_type().is_floating_point() {
                        log::warn!(
                            "Float array type scans are currently not fully supported! These scans currently lack tolerance checks and perform byte-wise exact comparisons. Scan accuracy may suffer."
                        )
                    }

                    // Perform a byte array scan, since we were unable to map the byte array to a primitive type.
                    // These are the only acceptable options, either the type is a primitive, or its a byte array.
                    mapped_params.mapped_scan_type = MappedScanType::ByteArray(ScanParametersByteArray::ByteArrayBooyerMoore);

                    return mapped_params;
                }
            }
        }

        // Now we decide whether to use a scalar or SIMD scan based on filter region size.
        mapped_params.vectorization_size =
            match Self::create_vectorization_size(snapshot_region_filter, &mapped_params.get_data_type(), mapped_params.memory_alignment) {
                None => {
                    // The filter cannot fit into a vector! Revert to scalar scan.
                    mapped_params.mapped_scan_type = MappedScanType::Scalar(ScanParametersScalar::ScalarIterative);

                    return mapped_params;
                }
                Some(vectorization_size) => vectorization_size,
            };

        let data_type_size = mapped_params.get_data_type().get_size_in_bytes();
        let memory_alignment_size = mapped_params.get_memory_alignment() as u64;

        if data_type_size > memory_alignment_size {
            // For discrete, multi-byte, primitive types (non-floating point), we can fall back on optimized scans if explicitly performing == or != scans.
            if !mapped_params.data_value.get_data_type().is_floating_point()
                && mapped_params.data_value.get_size_in_bytes() > 1
                && Self::is_checking_equal_or_not_equal(&mapped_params.scan_compare_type)
            {
                if let Some(periodicity) = Self::calculate_periodicity(mapped_params.get_data_value(), &mapped_params.scan_compare_type) {
                    mapped_params.periodicity = periodicity;

                    match periodicity {
                        1 => {
                            // Better for debug mode.
                            // mapped_params.mapped_scan_type = MappedScanType::Vector(ScanParametersVector::OverlappingBytewisePeriodic);

                            // Better for release mode.
                            mapped_params.mapped_scan_type = MappedScanType::Vector(ScanParametersVector::OverlappingBytewiseStaggered);

                            return mapped_params;
                        }
                        2 | 4 | 8 => {
                            mapped_params.mapped_scan_type = MappedScanType::Vector(ScanParametersVector::OverlappingBytewiseStaggered);

                            return mapped_params;
                        }
                        _ => {}
                    }
                }
            }
            mapped_params.mapped_scan_type = MappedScanType::Vector(ScanParametersVector::Overlapping);
            mapped_params
        } else if data_type_size < memory_alignment_size {
            mapped_params.mapped_scan_type = MappedScanType::Vector(ScanParametersVector::Sparse);
            mapped_params
        } else {
            mapped_params.mapped_scan_type = MappedScanType::Vector(ScanParametersVector::Aligned);
            mapped_params
        }
*/
