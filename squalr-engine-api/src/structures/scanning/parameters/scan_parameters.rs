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
use std::sync::atomic::{AtomicU64, Ordering};

// Contains all parameters that define a scan over a region of memory.
// This includes global scan parameters, localized scan parameters for this particular region, and any optimization metadata.
pub struct ScanParameters<'a> {
    scan_parameters_global: &'a ScanParametersGlobal,
    scan_parameters_local: &'a ScanParametersLocal,
    scan_parameter_optimizations: &'a ScanParameterOptimizations,

    // This value applies byte rotation to any retrieved data values.
    // Stored using atomics as a hack to grant mutability to non-mutable references.
    immediate_rotation: AtomicU64,
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
            immediate_rotation: AtomicU64::new(0),
        }
    }

    pub fn get_compare_type(&self) -> ScanCompareType {
        self.scan_parameters_global.get_compare_type()
    }

    pub fn get_data_value(&self) -> Option<DataValue> {
        if let Some(mut data_value) = self
            .scan_parameters_global
            .get_data_value(self.scan_parameters_local, self.scan_parameter_optimizations)
        {
            let immediate_rotation = self.immediate_rotation.load(Ordering::Acquire);

            // Apply any rotations to the immediate value vector if specified. Otherwise, just return the unaltered data value.
            if immediate_rotation != 0 {
                let mut rotated_bytes = data_value.take_value_bytes();

                rotated_bytes.rotate_left(immediate_rotation as usize);

                Some(DataValue::new(data_value.get_data_type_id(), rotated_bytes))
            } else {
                Some(data_value)
            }
        } else {
            None
        }
    }

    pub fn set_immediate_rotation(
        &self,
        immediate_rotation: u64,
    ) {
        self.immediate_rotation
            .store(immediate_rotation, Ordering::SeqCst);
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
        immediate_rotation: u64,
        use_optimized_data_type: bool,
    ) -> Option<ScalarCompareFnImmediate> {
        self.set_immediate_rotation(immediate_rotation);

        let result = if use_optimized_data_type {
            self.get_optimized_data_type()
                .get_scalar_compare_func_immediate(scan_compare_type, self)
        } else {
            self.get_original_data_type()
                .get_scalar_compare_func_immediate(scan_compare_type, self)
        };

        self.set_immediate_rotation(0);

        result
    }

    pub fn get_scalar_compare_func_relative(
        &self,
        scan_compare_type: &ScanCompareTypeRelative,
        immediate_rotation: u64,
        use_optimized_data_type: bool,
    ) -> Option<ScalarCompareFnRelative> {
        self.set_immediate_rotation(immediate_rotation);

        let result = if use_optimized_data_type {
            self.get_optimized_data_type()
                .get_scalar_compare_func_relative(scan_compare_type, self)
        } else {
            self.get_original_data_type()
                .get_scalar_compare_func_relative(scan_compare_type, self)
        };

        self.set_immediate_rotation(0);

        result
    }

    pub fn get_scalar_compare_func_delta(
        &self,
        scan_compare_type: &ScanCompareTypeDelta,
        immediate_rotation: u64,
        use_optimized_data_type: bool,
    ) -> Option<ScalarCompareFnDelta> {
        self.set_immediate_rotation(immediate_rotation);

        let result = if use_optimized_data_type {
            self.get_optimized_data_type()
                .get_scalar_compare_func_delta(scan_compare_type, self)
        } else {
            self.get_original_data_type()
                .get_scalar_compare_func_delta(scan_compare_type, self)
        };

        self.set_immediate_rotation(0);

        result
    }

    pub fn get_vector_compare_func_immediate<const N: usize>(
        &self,
        scan_compare_type_immediate: &ScanCompareTypeImmediate,
        immediate_rotation: u64,
        use_optimized_data_type: bool,
    ) -> Option<Box<dyn Fn(*const u8) -> Simd<u8, N>>>
    where
        LaneCount<N>: SupportedLaneCount + VectorComparer<N>,
    {
        self.set_immediate_rotation(immediate_rotation);

        let result = if use_optimized_data_type {
            self.get_optimized_data_type()
                .get_vector_compare_func_immediate(scan_compare_type_immediate, self)
        } else {
            self.get_original_data_type()
                .get_vector_compare_func_immediate(scan_compare_type_immediate, self)
        };

        self.set_immediate_rotation(0);

        result
    }

    pub fn get_vector_compare_func_relative<const N: usize>(
        &self,
        scan_compare_type_relative: &ScanCompareTypeRelative,
        immediate_rotation: u64,
        use_optimized_data_type: bool,
    ) -> Option<Box<dyn Fn(*const u8, *const u8) -> Simd<u8, N>>>
    where
        LaneCount<N>: SupportedLaneCount + VectorComparer<N>,
    {
        self.set_immediate_rotation(immediate_rotation);

        let result = if use_optimized_data_type {
            self.get_optimized_data_type()
                .get_vector_compare_func_relative(scan_compare_type_relative, self)
        } else {
            self.get_original_data_type()
                .get_vector_compare_func_relative(scan_compare_type_relative, self)
        };

        self.set_immediate_rotation(0);

        result
    }

    pub fn get_vector_compare_func_delta<const N: usize>(
        &self,
        scan_compare_type_delta: &ScanCompareTypeDelta,
        immediate_rotation: u64,
        use_optimized_data_type: bool,
    ) -> Option<Box<dyn Fn(*const u8, *const u8) -> Simd<u8, N>>>
    where
        LaneCount<N>: SupportedLaneCount + VectorComparer<N>,
    {
        self.set_immediate_rotation(immediate_rotation);

        let result = if use_optimized_data_type {
            self.get_optimized_data_type()
                .get_vector_compare_func_delta(scan_compare_type_delta, self)
        } else {
            self.get_original_data_type()
                .get_vector_compare_func_delta(scan_compare_type_delta, self)
        };

        self.set_immediate_rotation(0);

        result
    }
}
