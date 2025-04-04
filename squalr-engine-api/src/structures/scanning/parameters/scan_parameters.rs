use crate::structures::data_types::comparisons::scalar_comparable::{ScalarCompareFnDelta, ScalarCompareFnImmediate, ScalarCompareFnRelative};
use crate::structures::data_types::data_type_ref::DataTypeRef;
use crate::structures::data_types::floating_point_tolerance::FloatingPointTolerance;
use crate::structures::data_types::generics::vector_comparer::VectorComparer;
use crate::structures::data_values::data_value::DataValue;
use crate::structures::memory::memory_alignment::MemoryAlignment;
use crate::structures::scanning::comparisons::scan_compare_type::ScanCompareType;
use crate::structures::scanning::comparisons::scan_compare_type_delta::ScanCompareTypeDelta;
use crate::structures::scanning::comparisons::scan_compare_type_immediate::ScanCompareTypeImmediate;
use crate::structures::scanning::comparisons::scan_compare_type_relative::ScanCompareTypeRelative;
use crate::structures::scanning::parameters::scan_parameter_optimizations::ScanParameterOptimizations;
use crate::structures::scanning::parameters::scan_parameters_global::ScanParametersGlobal;
use crate::structures::scanning::parameters::scan_parameters_local::ScanParametersLocal;
use std::simd::{LaneCount, Simd, SupportedLaneCount};

// Contains all parameters that define a scan over a region of memory.
// This includes global scan parameters, localized scan parameters for this particular region, and any optimization metadata.
pub struct ScanParameters<'a> {
    scan_parameters_global: &'a ScanParametersGlobal,
    scan_parameters_local: &'a ScanParametersLocal,
    scan_parameter_optimizations: &'a ScanParameterOptimizations,
}

impl<'a> ScanParameters<'a> {
    pub fn new(
        scan_parameters_global: &'a ScanParametersGlobal,
        scan_parameters_local: &'a ScanParametersLocal,
        scan_parameter_optimizations: &'a ScanParameterOptimizations,
    ) -> Self {
        Self {
            scan_parameters_global,
            scan_parameters_local,
            scan_parameter_optimizations,
        }
    }

    pub fn get_compare_type(&self) -> ScanCompareType {
        self.scan_parameters_global.get_compare_type()
    }

    pub fn get_data_value(&self) -> Option<DataValue> {
        if let Some(data_value) = self
            .scan_parameters_global
            .get_data_value(self.scan_parameters_local, self.scan_parameter_optimizations)
        {
            Some(data_value)
        } else {
            None
        }
    }

    pub fn get_floating_point_tolerance(&self) -> FloatingPointTolerance {
        self.scan_parameters_global.get_floating_point_tolerance()
    }

    pub fn get_optimized_data_type(&self) -> &DataTypeRef {
        if let Some(data_type) = self.scan_parameter_optimizations.get_data_type_override() {
            data_type
        } else {
            self.scan_parameters_local.get_data_type()
        }
    }

    pub fn get_original_data_type(&self) -> &DataTypeRef {
        self.scan_parameters_local.get_data_type()
    }

    pub fn get_memory_alignment_or_default(&self) -> MemoryAlignment {
        self.scan_parameters_local.get_memory_alignment_or_default()
    }

    pub fn get_scalar_compare_func_immediate(
        &self,
        scan_compare_type: &ScanCompareTypeImmediate,
        use_optimized_data_type: bool,
    ) -> Option<ScalarCompareFnImmediate> {
        let result = if use_optimized_data_type {
            self.get_optimized_data_type()
                .get_scalar_compare_func_immediate(scan_compare_type, self)
        } else {
            self.get_original_data_type()
                .get_scalar_compare_func_immediate(scan_compare_type, self)
        };

        result
    }

    pub fn get_scalar_compare_func_relative(
        &self,
        scan_compare_type: &ScanCompareTypeRelative,
        use_optimized_data_type: bool,
    ) -> Option<ScalarCompareFnRelative> {
        let result = if use_optimized_data_type {
            self.get_optimized_data_type()
                .get_scalar_compare_func_relative(scan_compare_type, self)
        } else {
            self.get_original_data_type()
                .get_scalar_compare_func_relative(scan_compare_type, self)
        };

        result
    }

    pub fn get_scalar_compare_func_delta(
        &self,
        scan_compare_type: &ScanCompareTypeDelta,
        use_optimized_data_type: bool,
    ) -> Option<ScalarCompareFnDelta> {
        let result = if use_optimized_data_type {
            self.get_optimized_data_type()
                .get_scalar_compare_func_delta(scan_compare_type, self)
        } else {
            self.get_original_data_type()
                .get_scalar_compare_func_delta(scan_compare_type, self)
        };

        result
    }

    pub fn get_vector_compare_func_immediate<const N: usize>(
        &self,
        scan_compare_type_immediate: &ScanCompareTypeImmediate,
        use_optimized_data_type: bool,
    ) -> Option<Box<dyn Fn(*const u8) -> Simd<u8, N>>>
    where
        LaneCount<N>: SupportedLaneCount + VectorComparer<N>,
    {
        let result = if use_optimized_data_type {
            self.get_optimized_data_type()
                .get_vector_compare_func_immediate(scan_compare_type_immediate, self)
        } else {
            self.get_original_data_type()
                .get_vector_compare_func_immediate(scan_compare_type_immediate, self)
        };

        result
    }

    pub fn get_vector_compare_func_relative<const N: usize>(
        &self,
        scan_compare_type_relative: &ScanCompareTypeRelative,
        use_optimized_data_type: bool,
    ) -> Option<Box<dyn Fn(*const u8, *const u8) -> Simd<u8, N>>>
    where
        LaneCount<N>: SupportedLaneCount + VectorComparer<N>,
    {
        let result = if use_optimized_data_type {
            self.get_optimized_data_type()
                .get_vector_compare_func_relative(scan_compare_type_relative, self)
        } else {
            self.get_original_data_type()
                .get_vector_compare_func_relative(scan_compare_type_relative, self)
        };

        result
    }

    pub fn get_vector_compare_func_delta<const N: usize>(
        &self,
        scan_compare_type_delta: &ScanCompareTypeDelta,
        use_optimized_data_type: bool,
    ) -> Option<Box<dyn Fn(*const u8, *const u8) -> Simd<u8, N>>>
    where
        LaneCount<N>: SupportedLaneCount + VectorComparer<N>,
    {
        let result = if use_optimized_data_type {
            self.get_optimized_data_type()
                .get_vector_compare_func_delta(scan_compare_type_delta, self)
        } else {
            self.get_original_data_type()
                .get_vector_compare_func_delta(scan_compare_type_delta, self)
        };

        result
    }
}
