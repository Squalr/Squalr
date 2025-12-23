use std::sync::{Arc, RwLock};

use crate::registries::symbols::symbol_registry::SymbolRegistry;
use crate::structures::data_types::built_in_types::u8::data_type_u8::DataTypeU8;
use crate::structures::data_types::built_in_types::u16be::data_type_u16be::DataTypeU16be;
use crate::structures::data_types::built_in_types::u32be::data_type_u32be::DataTypeU32be;
use crate::structures::data_types::built_in_types::u64be::data_type_u64be::DataTypeU64be;
use crate::structures::data_types::data_type_ref::DataTypeRef;
use crate::structures::scanning::comparisons::scan_compare_type::ScanCompareType;
use crate::structures::scanning::rules::element_scan_mapping_rule::ElementScanMappingRule;
use crate::structures::scanning::{
    filters::{snapshot_region_filter::SnapshotRegionFilter, snapshot_region_filter_collection::SnapshotRegionFilterCollection},
    parameters::{element_scan::element_scan_parameters::ElementScanParameters, mapped::mapped_scan_parameters::MappedScanParameters},
};
use crate::structures::snapshots::snapshot_region::SnapshotRegion;

pub struct RuleMapToPrimitiveType {}

impl RuleMapToPrimitiveType {
    pub const RULE_ID: &str = "map_to_primitive_type";
}

impl ElementScanMappingRule for RuleMapToPrimitiveType {
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
        let symbol_registry_guard = match symbol_registry.read() {
            Ok(registry) => registry,
            Err(error) => {
                log::error!("Failed to acquire read lock on SymbolRegistry: {}", error);

                return;
            }
        };
        let data_value = mapped_parameters.get_data_value();
        let data_type_ref = data_value.get_data_type_ref();

        // Only immediate scans can be remapped, if the scan is relative, then the original data type is crucial.
        match mapped_parameters.get_compare_type() {
            ScanCompareType::Relative(_) | ScanCompareType::Delta(_) => return,
            ScanCompareType::Immediate(_) => {}
        };

        // Non discrete / floating point types cannot be remapped. For example, if we have an array of two f32 values,
        // we absolutely cannot remap this to a single u64 (nor an f64) since these require tolerance comparisons.
        if symbol_registry_guard.is_floating_point(data_type_ref) {
            return;
        }

        let data_type_size = data_value.get_size_in_bytes();
        let data_type_default_size = symbol_registry_guard.get_unit_size_in_bytes(data_type_ref);

        // If the data type size is the default for that type, and its already a valid primitive size,
        // there is no need to perform a remapping. We do this check to avoid meaningless remappings,
        // such as remapping i16 to u16, even though this appears acceptable at a surface level.
        // These remappings must be avoided to avoid subtle problems, such as remapping a ">= 0" scan from signed to unsigned.
        // This would potentially allow for poisoned future optimizations, such as remapping ">= 0" to "!= 0",
        // which is perfectly valid for unsigned scans, but not valid at all for signed scans.
        if data_type_size == data_type_default_size {
            match data_type_size {
                1 | 2 | 4 | 8 => return,
                _ => {}
            };
        }

        // If applicable, try to reinterpret array of byte scans as a primitive type of the same size.
        // These are much more efficient than array of byte scans, so for scans of these sizes performance will be improved greatly.
        match data_type_size {
            8 => mapped_parameters
                .get_data_value_mut()
                .set_data_type(DataTypeRef::new(DataTypeU64be::get_data_type_id())),
            4 => mapped_parameters
                .get_data_value_mut()
                .set_data_type(DataTypeRef::new(DataTypeU32be::get_data_type_id())),
            2 => mapped_parameters
                .get_data_value_mut()
                .set_data_type(DataTypeRef::new(DataTypeU16be::get_data_type_id())),
            1 => mapped_parameters
                .get_data_value_mut()
                .set_data_type(DataTypeRef::new(DataTypeU8::get_data_type_id())),
            _ => {}
        }
    }
}
