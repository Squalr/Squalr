use crate::registries::scan_rules::element_scan_mapping_rule::ElementScanMappingRule;
use crate::structures::data_types::data_type_ref::DataTypeRef;
use crate::structures::data_values::data_value::DataValue;
use crate::structures::scanning::parameters::mapped::mapped_scan_type::{MappedScanType, ScanParametersVector};
use crate::structures::scanning::{
    comparisons::scan_compare_type::ScanCompareType,
    filters::{snapshot_region_filter::SnapshotRegionFilter, snapshot_region_filter_collection::SnapshotRegionFilterCollection},
    parameters::{element_scan::element_scan_parameters::ElementScanParameters, mapped::mapped_scan_parameters::MappedScanParameters},
};

pub struct MapPeriodicScans {}

impl MapPeriodicScans {
    pub const RULE_ID: &str = "map_periodic_scans";

    fn calculate_periodicity(
        data_value: &DataValue,
        scan_compare_type: &ScanCompareType,
    ) -> Option<u64> {
        match scan_compare_type {
            ScanCompareType::Immediate(_scan_compare_type_immediate) => Some(Self::calculate_periodicity_from_immediate(
                &data_value.get_value_bytes(),
                data_value.get_data_type(),
            )),
            ScanCompareType::Delta(_scan_compare_type_immediate) => Some(Self::calculate_periodicity_from_immediate(
                &data_value.get_value_bytes(),
                data_value.get_data_type(),
            )),
            _ => None,
        }
    }

    /// Calculates the length of repeating byte patterns within a given data type and value combination.
    /// If there are no repeating patterns, the periodicity will be equal to the data type size.
    /// For example, 7C 01 7C 01 has a data typze size of 4, but a periodicity of 2.
    fn calculate_periodicity_from_immediate(
        immediate_value_bytes: &[u8],
        data_type: &DataTypeRef,
    ) -> u64 {
        // Assume optimal periodicity to begin with
        let mut period = 1;
        let data_type_size_bytes = data_type.get_size_in_bytes();

        // Loop through all remaining bytes, and increase the periodicity when we encounter a byte that violates the current assumption.
        for byte_index in 1..data_type_size_bytes as usize {
            if immediate_value_bytes[byte_index] != immediate_value_bytes[byte_index % period] {
                period = byte_index + 1;
            }
        }

        period as u64
    }
}

impl ElementScanMappingRule for MapPeriodicScans {
    fn get_id(&self) -> &str {
        &Self::RULE_ID
    }

    fn map_parameters(
        &self,
        _snapshot_region_filter_collection: &SnapshotRegionFilterCollection,
        _snapshot_region_filter: &SnapshotRegionFilter,
        _original_scan_parameters: &ElementScanParameters,
        mapped_parameters: &mut MappedScanParameters,
    ) {
        if let Some(periodicity) = Self::calculate_periodicity(mapped_parameters.get_data_value(), &mapped_parameters.get_compare_type()) {
            mapped_parameters.set_periodicity(periodicity);

            match periodicity {
                1 => {
                    // Better for debug mode.
                    // mapped_parameters.set_mapped_scan_type(MappedScanType::Vector(ScanParametersVector::OverlappingBytewisePeriodic));

                    // Better for release mode.
                    mapped_parameters.set_mapped_scan_type(MappedScanType::Vector(ScanParametersVector::OverlappingBytewiseStaggered));
                }
                2 | 4 | 8 => {
                    mapped_parameters.set_mapped_scan_type(MappedScanType::Vector(ScanParametersVector::OverlappingBytewiseStaggered));
                }
                _ => {}
            }
        }
    }
}

/*
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
*/
