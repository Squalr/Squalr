use crate::registries::scan_rules::element_scan_mapping_rule::ElementScanMappingRule;
use crate::structures::scanning::{
    comparisons::{scan_compare_type::ScanCompareType, scan_compare_type_immediate::ScanCompareTypeImmediate},
    filters::{snapshot_region_filter::SnapshotRegionFilter, snapshot_region_filter_collection::SnapshotRegionFilterCollection},
    parameters::{element_scan::element_scan_parameters::ElementScanParameters, mapped::mapped_scan_parameters::MappedScanParameters},
};

pub struct MapUnsignedGreaterThanZeroToNotEqual {}

impl MapUnsignedGreaterThanZeroToNotEqual {
    pub const RULE_ID: &str = "map_unsigned_greater_than_zero_to_not_equal";
}

impl ElementScanMappingRule for MapUnsignedGreaterThanZeroToNotEqual {
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
        let is_signed = mapped_parameters.get_data_value().get_data_type().is_signed();
        let is_floating_point = mapped_parameters
            .get_data_value()
            .get_data_type()
            .is_floating_point();
        let is_all_zero = mapped_parameters
            .get_data_value()
            .get_value_bytes()
            .iter()
            .all(|byte| *byte == 0);

        // Remap unsigned scans that are checking "> 0" to be "!= 0" since this is actually faster internally.
        if !is_signed && !is_floating_point && is_all_zero {
            match mapped_parameters.get_compare_type() {
                ScanCompareType::Immediate(scan_compare_type_immediate) => match scan_compare_type_immediate {
                    ScanCompareTypeImmediate::GreaterThan => {
                        mapped_parameters.set_compare_type(ScanCompareType::Immediate(ScanCompareTypeImmediate::NotEqual));
                    }
                    _ => {}
                },
                _ => {}
            }
        }
    }
}
