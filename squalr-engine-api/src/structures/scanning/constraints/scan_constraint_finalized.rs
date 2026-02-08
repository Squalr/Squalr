use crate::structures::data_types::data_type_ref::DataTypeRef;
use crate::structures::data_types::generics::vector_comparer::VectorComparer;
use crate::structures::data_types::generics::vector_function::GetVectorFunction;
use crate::structures::data_types::generics::vector_lane_count::VectorLaneCount;
use crate::structures::data_values::data_value::DataValue;
use crate::structures::scanning::comparisons::scan_compare_type::ScanCompareType;
use crate::structures::scanning::comparisons::scan_function_scalar::ScanFunctionScalar;
use crate::structures::scanning::comparisons::scan_function_vector::ScanFunctionVector;
use crate::structures::scanning::constraints::scan_constraint::ScanConstraint;
use crate::{registries::symbols::symbol_registry::SymbolRegistry, structures::data_types::floating_point_tolerance::FloatingPointTolerance};

/// Represents a scan constraint that has finished being processed by rules.
pub struct ScanConstraintFinalized {
    scan_constraint: ScanConstraint,
    periodicity: u64,
    unit_size_bytes: u64,
    scan_function_scalar: Option<ScanFunctionScalar>,
    scan_function_vector_16: Option<ScanFunctionVector<16>>,
    scan_function_vector_32: Option<ScanFunctionVector<32>>,
    scan_function_vector_64: Option<ScanFunctionVector<64>>,
}

impl GetVectorFunction<16> for VectorLaneCount<16> {
    fn get_vector_field<'lifetime>(
        &self,
        scan_constraint_finalized: &'lifetime ScanConstraintFinalized,
    ) -> &'lifetime Option<ScanFunctionVector<16>> {
        &scan_constraint_finalized.scan_function_vector_16
    }
}

impl GetVectorFunction<32> for VectorLaneCount<32> {
    fn get_vector_field<'lifetime>(
        &self,
        scan_constraint_finalized: &'lifetime ScanConstraintFinalized,
    ) -> &'lifetime Option<ScanFunctionVector<32>> {
        &scan_constraint_finalized.scan_function_vector_32
    }
}

impl GetVectorFunction<64> for VectorLaneCount<64> {
    fn get_vector_field<'lifetime>(
        &self,
        scan_constraint_finalized: &'lifetime ScanConstraintFinalized,
    ) -> &'lifetime Option<ScanFunctionVector<64>> {
        &scan_constraint_finalized.scan_function_vector_64
    }
}

impl ScanConstraintFinalized {
    pub fn new(scan_constraint: ScanConstraint) -> Self {
        let symbol_registry = SymbolRegistry::get_instance();
        let periodicity = Self::calculate_periodicity(symbol_registry, &scan_constraint.get_data_value(), &scan_constraint.get_scan_compare_type());
        let unit_size_bytes = symbol_registry.get_unit_size_in_bytes(scan_constraint.get_data_value().get_data_type_ref());
        let scan_function_scalar = Self::build_scan_function_scalar(&scan_constraint);
        let scan_function_vector_16 = Self::build_scan_function_vector::<16>(&scan_constraint);
        let scan_function_vector_32 = Self::build_scan_function_vector::<32>(&scan_constraint);
        let scan_function_vector_64 = Self::build_scan_function_vector::<64>(&scan_constraint);

        Self {
            scan_constraint,
            periodicity,
            unit_size_bytes,
            scan_function_scalar,
            scan_function_vector_16,
            scan_function_vector_32,
            scan_function_vector_64,
        }
    }

    pub fn get_scan_constraint(&self) -> &ScanConstraint {
        &self.scan_constraint
    }

    pub fn get_scan_compare_type(&self) -> ScanCompareType {
        self.scan_constraint.get_scan_compare_type()
    }

    pub fn get_data_value(&self) -> &DataValue {
        &self.scan_constraint.get_data_value()
    }

    pub fn get_floating_point_tolerance(&self) -> FloatingPointTolerance {
        self.scan_constraint.get_floating_point_tolerance()
    }

    pub fn get_periodicity(&self) -> u64 {
        self.periodicity
    }

    pub fn get_unit_size_in_bytes(&self) -> u64 {
        self.unit_size_bytes
    }

    pub fn get_scan_function_scalar(&self) -> &Option<ScanFunctionScalar> {
        &self.scan_function_scalar
    }

    pub fn get_scan_function_vector<const N: usize>(&self) -> &Option<ScanFunctionVector<N>>
    where
        VectorLaneCount<N>: VectorComparer<N> + GetVectorFunction<N>,
    {
        VectorLaneCount::<N> {}.get_vector_field(self)
    }

    fn calculate_periodicity(
        symbol_registry: &SymbolRegistry,
        data_value: &DataValue,
        scan_compare_type: &ScanCompareType,
    ) -> u64 {
        match scan_compare_type {
            ScanCompareType::Immediate(_scan_compare_type_immediate) => {
                Self::calculate_periodicity_from_immediate(symbol_registry, &data_value.get_value_bytes(), data_value.get_data_type_ref())
            }
            ScanCompareType::Delta(_scan_compare_type_immediate) => {
                Self::calculate_periodicity_from_immediate(symbol_registry, &data_value.get_value_bytes(), data_value.get_data_type_ref())
            }
            _ => 0,
        }
    }

    /// Calculates the length of repeating byte patterns within a given data type and value combination.
    /// If there are no repeating patterns, the periodicity will be equal to the data type size.
    /// For example, 7C 01 7C 01 has a data typze size of 4, but a periodicity of 2.
    fn calculate_periodicity_from_immediate(
        symbol_registry: &SymbolRegistry,
        immediate_value_bytes: &[u8],
        data_type_ref: &DataTypeRef,
    ) -> u64 {
        // Assume optimal periodicity to begin with
        let mut period = 1;
        let data_type_size_bytes = symbol_registry.get_unit_size_in_bytes(data_type_ref);

        // Loop through all remaining bytes, and increase the periodicity when we encounter a byte that violates the current assumption.
        for byte_index in 1..data_type_size_bytes as usize {
            if immediate_value_bytes[byte_index] != immediate_value_bytes[byte_index % period as usize] {
                period = byte_index as u64 + 1;
            }
        }

        period as u64
    }

    fn build_scan_function_scalar(scan_constraint: &ScanConstraint) -> Option<ScanFunctionScalar> {
        let symbol_registry = SymbolRegistry::get_instance();

        match scan_constraint.get_scan_compare_type() {
            ScanCompareType::Immediate(scan_compare_type_immediate) => {
                if let Some(compare_func) = symbol_registry.get_scalar_compare_func_immediate(&scan_compare_type_immediate, scan_constraint) {
                    return Some(ScanFunctionScalar::Immediate(compare_func));
                }
            }
            ScanCompareType::Relative(scan_compare_type_relative) => {
                if let Some(compare_func) = symbol_registry.get_scalar_compare_func_relative(&scan_compare_type_relative, scan_constraint) {
                    return Some(ScanFunctionScalar::RelativeOrDelta(compare_func));
                }
            }
            ScanCompareType::Delta(scan_compare_type_delta) => {
                if let Some(compare_func) = symbol_registry.get_scalar_compare_func_delta(&scan_compare_type_delta, scan_constraint) {
                    return Some(ScanFunctionScalar::RelativeOrDelta(compare_func));
                }
            }
        }

        None
    }

    pub fn build_scan_function_vector<const N: usize>(scan_constraint: &ScanConstraint) -> Option<ScanFunctionVector<N>>
    where
        VectorLaneCount<N>: VectorComparer<N> + GetVectorFunction<N>,
    {
        let symbol_registry = SymbolRegistry::get_instance();

        match scan_constraint.get_scan_compare_type() {
            ScanCompareType::Immediate(scan_compare_type_immediate) => {
                if let Some(compare_func) = symbol_registry.get_vector_compare_func_immediate(&scan_compare_type_immediate, scan_constraint) {
                    return Some(ScanFunctionVector::Immediate(compare_func));
                }
            }
            ScanCompareType::Relative(scan_compare_type_relative) => {
                if let Some(compare_func) = symbol_registry.get_vector_compare_func_relative(&scan_compare_type_relative, scan_constraint) {
                    return Some(ScanFunctionVector::RelativeOrDelta(compare_func));
                }
            }
            ScanCompareType::Delta(scan_compare_type_delta) => {
                if let Some(compare_func) = symbol_registry.get_vector_compare_func_delta(&scan_compare_type_delta, scan_constraint) {
                    return Some(ScanFunctionVector::RelativeOrDelta(compare_func));
                }
            }
        }

        None
    }
}
