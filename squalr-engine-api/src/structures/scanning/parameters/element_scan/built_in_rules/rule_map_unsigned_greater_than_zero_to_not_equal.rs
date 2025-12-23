use std::sync::{Arc, RwLock};

use crate::registries::symbols::symbol_registry::SymbolRegistry;
use crate::structures::scanning::rules::element_scan_mapping_rule::ElementScanMappingRule;
use crate::structures::scanning::{
    comparisons::{scan_compare_type::ScanCompareType, scan_compare_type_immediate::ScanCompareTypeImmediate},
    filters::{snapshot_region_filter::SnapshotRegionFilter, snapshot_region_filter_collection::SnapshotRegionFilterCollection},
    parameters::{element_scan::element_scan_parameters::ElementScanParameters, mapped::mapped_scan_parameters::MappedScanParameters},
};
use crate::structures::snapshots::snapshot_region::SnapshotRegion;

/// Defines a mapping rule that converts > 0 scans for unsigned non-floating-point values into != 0.
/// This optimization allows for better vectorization, resulting in faster scans.
pub struct RuleMapUnsignedGreaterThanZeroToNotEqual {}

impl RuleMapUnsignedGreaterThanZeroToNotEqual {
    pub const RULE_ID: &str = "map_unsigned_greater_than_zero_to_not_equal";
}

impl ElementScanMappingRule for RuleMapUnsignedGreaterThanZeroToNotEqual {
    fn get_id(&self) -> &str {
        &Self::RULE_ID
    }

    fn map_parameters(
        &self,
        symbol_registry: &Arc<RwLock<SymbolRegistry>>,
        _snapshot_region: &SnapshotRegion,
        _snapshot_region_filter_collection: &SnapshotRegionFilterCollection,
        _snapshot_region_filter: &SnapshotRegionFilter,
        _original_scan_parameters: &ElementScanParameters,
        mapped_parameters: &mut MappedScanParameters,
    ) {
        let data_type_ref = mapped_parameters.get_data_value().get_data_type_ref();
        let symbol_registry_guard = match symbol_registry.read() {
            Ok(registry) => registry,
            Err(error) => {
                log::error!("Failed to acquire read lock on SymbolRegistry: {}", error);

                return;
            }
        };
        let is_signed = symbol_registry_guard.is_signed(data_type_ref);
        let is_floating_point = symbol_registry_guard.is_floating_point(data_type_ref);
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
