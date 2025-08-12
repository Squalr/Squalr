use std::sync::{Arc, RwLock};

use crate::registries::symbols::symbol_registry::SymbolRegistry;
use crate::structures::data_types::data_type_ref::DataTypeRef;
use crate::structures::data_values::data_value::DataValue;
use crate::structures::scanning::rules::element_scan_mapping_rule::ElementScanMappingRule;
use crate::structures::scanning::{
    comparisons::scan_compare_type::ScanCompareType,
    filters::{snapshot_region_filter::SnapshotRegionFilter, snapshot_region_filter_collection::SnapshotRegionFilterCollection},
    parameters::{element_scan::element_scan_parameters::ElementScanParameters, mapped::mapped_scan_parameters::MappedScanParameters},
};

pub struct MapPeriodicScans {}

impl MapPeriodicScans {
    pub const RULE_ID: &str = "map_periodic_scans";

    fn calculate_periodicity(
        symbol_registry: &Arc<RwLock<SymbolRegistry>>,
        data_value: &DataValue,
        scan_compare_type: &ScanCompareType,
    ) -> Option<u64> {
        match scan_compare_type {
            ScanCompareType::Immediate(_scan_compare_type_immediate) => Some(Self::calculate_periodicity_from_immediate(
                symbol_registry,
                &data_value.get_value_bytes(),
                data_value.get_data_type_ref(),
            )),
            ScanCompareType::Delta(_scan_compare_type_immediate) => Some(Self::calculate_periodicity_from_immediate(
                symbol_registry,
                &data_value.get_value_bytes(),
                data_value.get_data_type_ref(),
            )),
            _ => None,
        }
    }

    /// Calculates the length of repeating byte patterns within a given data type and value combination.
    /// If there are no repeating patterns, the periodicity will be equal to the data type size.
    /// For example, 7C 01 7C 01 has a data typze size of 4, but a periodicity of 2.
    fn calculate_periodicity_from_immediate(
        symbol_registry: &Arc<RwLock<SymbolRegistry>>,
        immediate_value_bytes: &[u8],
        data_type_ref: &DataTypeRef,
    ) -> u64 {
        // Assume optimal periodicity to begin with
        let mut period = 1;
        let symbol_registry_guard = match symbol_registry.read() {
            Ok(registry) => registry,
            Err(error) => {
                log::error!("Failed to acquire read lock on SymbolRegistry: {}", error);

                return period;
            }
        };
        let data_type_size_bytes = symbol_registry_guard.get_unit_size_in_bytes(data_type_ref);

        // Loop through all remaining bytes, and increase the periodicity when we encounter a byte that violates the current assumption.
        for byte_index in 1..data_type_size_bytes as usize {
            if immediate_value_bytes[byte_index] != immediate_value_bytes[byte_index % period as usize] {
                period = byte_index as u64 + 1;
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
        symbol_registry: &Arc<RwLock<SymbolRegistry>>,
        _snapshot_region_filter_collection: &SnapshotRegionFilterCollection,
        _snapshot_region_filter: &SnapshotRegionFilter,
        _original_scan_parameters: &ElementScanParameters,
        mapped_parameters: &mut MappedScanParameters,
    ) {
        if let Some(periodicity) = Self::calculate_periodicity(symbol_registry, mapped_parameters.get_data_value(), &mapped_parameters.get_compare_type()) {
            mapped_parameters.set_periodicity(periodicity);
        }
    }
}
