use crate::registries::symbols::symbol_registry::SymbolRegistry;
use crate::structures::data_types::data_type_ref::DataTypeRef;
use crate::structures::data_types::floating_point_tolerance::FloatingPointTolerance;
use crate::structures::data_types::generics::vector_comparer::VectorComparer;
use crate::structures::data_values::data_value::DataValue;
use crate::structures::memory::memory_alignment::MemoryAlignment;
use crate::structures::scanning::comparisons::scan_compare_type::ScanCompareType;
use crate::structures::scanning::comparisons::scan_function_scalar::ScanFunctionScalar;
use crate::structures::scanning::comparisons::scan_function_vector::ScanFunctionVector;
use crate::structures::scanning::plans::plan_types::planned_scan_type::PlannedScanType;
use std::simd::LaneCount;
use std::simd::SupportedLaneCount;
use std::sync::Arc;
use std::sync::RwLock;

/// Represents processed scan parameters derived from user provided scan parameters.
pub struct SnapshotFilterElementScanPlan<'lifetime> {
    data_value: &'lifetime DataValue,
    memory_alignment: MemoryAlignment,
    scan_compare_type: ScanCompareType,
    floating_point_tolerance: FloatingPointTolerance,
    periodicity: u64,
    planned_scan_type: PlannedScanType,
    scan_function_scalar: Option<ScanFunctionScalar>,
}

impl<'lifetime> SnapshotFilterElementScanPlan<'lifetime> {
    /// Creates optimized scan paramaters for a given snapshot region filter, given user provided scan parameters.
    /// Internally, the user parameters are processed into more optimal parameters that help select the most optimal scan implementation.
    pub fn new(
        data_value: &'lifetime DataValue,
        memory_alignment: MemoryAlignment,
        scan_compare_type: ScanCompareType,
        floating_point_tolerance: FloatingPointTolerance,
    ) -> Self {
        let symbol_registry = SymbolRegistry::get_instance();

        let mut instance = Self {
            data_value,
            memory_alignment,
            scan_compare_type,
            floating_point_tolerance,
            periodicity: 0,
            planned_scan_type: PlannedScanType::Invalid(),
            scan_function_scalar: None,
        };

        let scan_function_scalar = match scan_compare_type {
            ScanCompareType::Immediate(scan_compare_type_immediate) => {
                if let Some(compare_func) = symbol_registry.get_scalar_compare_func_immediate(&scan_compare_type_immediate, &instance) {
                    Some(ScanFunctionScalar::Immediate(compare_func))
                } else {
                    None
                }
            }
            ScanCompareType::Relative(scan_compare_type_relative) => {
                if let Some(compare_func) = symbol_registry.get_scalar_compare_func_relative(&scan_compare_type_relative, &instance) {
                    Some(ScanFunctionScalar::RelativeOrDelta(compare_func))
                } else {
                    None
                }
            }
            ScanCompareType::Delta(scan_compare_type_delta) => {
                if let Some(compare_func) = symbol_registry.get_scalar_compare_func_delta(&scan_compare_type_delta, &instance) {
                    Some(ScanFunctionScalar::RelativeOrDelta(compare_func))
                } else {
                    None
                }
            }
        };

        instance.scan_function_scalar = scan_function_scalar;

        instance
    }

    pub fn get_data_value(&self) -> &DataValue {
        &self.data_value
    }

    pub fn get_data_type_ref(&self) -> &DataTypeRef {
        &self.get_data_value().get_data_type_ref()
    }

    pub fn get_memory_alignment(&self) -> MemoryAlignment {
        self.memory_alignment
    }

    pub fn get_compare_type(&self) -> &ScanCompareType {
        &self.scan_compare_type
    }

    pub fn set_compare_type(
        &mut self,
        scan_compare_type: ScanCompareType,
    ) {
        self.scan_compare_type = scan_compare_type;
    }

    pub fn get_floating_point_tolerance(&self) -> FloatingPointTolerance {
        self.floating_point_tolerance
    }

    pub fn get_periodicity(&self) -> u64 {
        self.periodicity
    }

    pub fn set_periodicity(
        &mut self,
        periodicity: u64,
    ) {
        self.periodicity = periodicity;
    }

    pub fn get_planned_scan_type(&self) -> &PlannedScanType {
        &self.planned_scan_type
    }

    pub fn set_planned_scan_type(
        &mut self,
        planned_scan_type: PlannedScanType,
    ) {
        self.planned_scan_type = planned_scan_type;
    }

    pub fn get_scan_function_scalar(&self) -> &Option<ScanFunctionScalar> {
        &self.scan_function_scalar
    }

    pub fn get_scan_function_vector<const N: usize>(
        &self,
        symbol_registry: &Arc<RwLock<SymbolRegistry>>,
    ) -> Option<ScanFunctionVector<N>>
    where
        LaneCount<N>: SupportedLaneCount + VectorComparer<N>,
    {
        /*
        let symbol_registry_guard = match symbol_registry.read() {
            Ok(registry) => registry,
            Err(error) => {
                log::error!("Failed to acquire read lock on SymbolRegistry: {}", error);

                return None;
            }
        };*/
        let symbol_registry = SymbolRegistry::get_instance();

        match self.get_compare_type() {
            ScanCompareType::Immediate(scan_compare_type_immediate) => {
                if let Some(compare_func) = symbol_registry.get_vector_compare_func_immediate(&scan_compare_type_immediate, &self) {
                    return Some(ScanFunctionVector::Immediate(compare_func));
                }
            }
            ScanCompareType::Relative(scan_compare_type_relative) => {
                if let Some(compare_func) = symbol_registry.get_vector_compare_func_relative(&scan_compare_type_relative, &self) {
                    return Some(ScanFunctionVector::RelativeOrDelta(compare_func));
                }
            }
            ScanCompareType::Delta(scan_compare_type_delta) => {
                if let Some(compare_func) = symbol_registry.get_vector_compare_func_delta(&scan_compare_type_delta, &self) {
                    return Some(ScanFunctionVector::RelativeOrDelta(compare_func));
                }
            }
        }

        None
    }
}
