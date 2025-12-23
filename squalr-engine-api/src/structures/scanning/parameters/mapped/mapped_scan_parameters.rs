use crate::registries::symbols::symbol_registry::SymbolRegistry;
use crate::structures::data_types::data_type_ref::DataTypeRef;
use crate::structures::data_types::floating_point_tolerance::FloatingPointTolerance;
use crate::structures::data_types::generics::vector_comparer::VectorComparer;
use crate::structures::data_values::data_value::DataValue;
use crate::structures::memory::memory_alignment::MemoryAlignment;
use crate::structures::scanning::comparisons::scan_compare_type::ScanCompareType;
use crate::structures::scanning::comparisons::scan_function_scalar::ScanFunctionScalar;
use crate::structures::scanning::comparisons::scan_function_vector::ScanFunctionVector;
use crate::structures::scanning::parameters::mapped::mapped_scan_type::MappedScanType;
use std::simd::LaneCount;
use std::simd::SupportedLaneCount;
use std::sync::Arc;
use std::sync::RwLock;

/// Represents processed scan parameters derived from user provided scan parameters.
#[derive(Debug, Clone)]
pub struct MappedScanParameters {
    data_value: DataValue,
    memory_alignment: MemoryAlignment,
    scan_compare_type: ScanCompareType,
    floating_point_tolerance: FloatingPointTolerance,
    periodicity: u64,
    mapped_scan_type: MappedScanType,
}

impl MappedScanParameters {
    /// Creates optimized scan paramaters for a given snapshot region filter, given user provided scan parameters.
    /// Internally, the user parameters are processed into more optimal parameters that help select the most optimal scan implementation.
    pub fn new(
        data_value: DataValue,
        memory_alignment: MemoryAlignment,
        scan_compare_type: ScanCompareType,
        floating_point_tolerance: FloatingPointTolerance,
    ) -> Self {
        Self {
            data_value,
            memory_alignment,
            scan_compare_type,
            floating_point_tolerance,
            periodicity: 0,
            mapped_scan_type: MappedScanType::Invalid(),
        }
    }

    pub fn get_data_value(&self) -> &DataValue {
        &self.data_value
    }

    pub fn get_data_value_mut(&mut self) -> &mut DataValue {
        &mut self.data_value
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

    pub fn get_mapped_scan_type(&self) -> &MappedScanType {
        &self.mapped_scan_type
    }

    pub fn set_mapped_scan_type(
        &mut self,
        mapped_scan_type: MappedScanType,
    ) {
        self.mapped_scan_type = mapped_scan_type;
    }

    pub fn get_scan_function_scalar(
        &self,
        symbol_registry: &Arc<RwLock<SymbolRegistry>>,
    ) -> Option<ScanFunctionScalar> {
        let symbol_registry_guard = match symbol_registry.read() {
            Ok(registry) => registry,
            Err(error) => {
                log::error!("Failed to acquire read lock on SymbolRegistry: {}", error);

                return None;
            }
        };

        match self.get_compare_type() {
            ScanCompareType::Immediate(scan_compare_type_immediate) => {
                if let Some(compare_func) =
                    symbol_registry_guard.get_scalar_compare_func_immediate(self.get_data_type_ref(), &scan_compare_type_immediate, &self)
                {
                    return Some(ScanFunctionScalar::Immediate(compare_func));
                }
            }
            ScanCompareType::Relative(scan_compare_type_relative) => {
                if let Some(compare_func) = symbol_registry_guard.get_scalar_compare_func_relative(self.get_data_type_ref(), &scan_compare_type_relative, &self)
                {
                    return Some(ScanFunctionScalar::RelativeOrDelta(compare_func));
                }
            }
            ScanCompareType::Delta(scan_compare_type_delta) => {
                if let Some(compare_func) = symbol_registry_guard.get_scalar_compare_func_delta(self.get_data_type_ref(), &scan_compare_type_delta, &self) {
                    return Some(ScanFunctionScalar::RelativeOrDelta(compare_func));
                }
            }
        }

        None
    }

    pub fn get_scan_function_vector<const N: usize>(
        &self,
        symbol_registry: &Arc<RwLock<SymbolRegistry>>,
    ) -> Option<ScanFunctionVector<N>>
    where
        LaneCount<N>: SupportedLaneCount + VectorComparer<N>,
    {
        let symbol_registry_guard = match symbol_registry.read() {
            Ok(registry) => registry,
            Err(error) => {
                log::error!("Failed to acquire read lock on SymbolRegistry: {}", error);

                return None;
            }
        };

        match self.get_compare_type() {
            ScanCompareType::Immediate(scan_compare_type_immediate) => {
                if let Some(compare_func) =
                    symbol_registry_guard.get_vector_compare_func_immediate(self.get_data_type_ref(), &scan_compare_type_immediate, &self)
                {
                    return Some(ScanFunctionVector::Immediate(compare_func));
                }
            }
            ScanCompareType::Relative(scan_compare_type_relative) => {
                if let Some(compare_func) = symbol_registry_guard.get_vector_compare_func_relative(self.get_data_type_ref(), &scan_compare_type_relative, &self)
                {
                    return Some(ScanFunctionVector::RelativeOrDelta(compare_func));
                }
            }
            ScanCompareType::Delta(scan_compare_type_delta) => {
                if let Some(compare_func) = symbol_registry_guard.get_vector_compare_func_delta(self.get_data_type_ref(), &scan_compare_type_delta, &self) {
                    return Some(ScanFunctionVector::RelativeOrDelta(compare_func));
                }
            }
        }

        None
    }
}
