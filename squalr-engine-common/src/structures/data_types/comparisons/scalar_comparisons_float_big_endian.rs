use crate::structures::data_types::comparisons::scalar_comparable::{ScalarCompareFnDelta, ScalarCompareFnImmediate, ScalarCompareFnRelative};
use crate::structures::scanning::scan_parameters_global::ScanParametersGlobal;
use crate::structures::scanning::scan_parameters_local::ScanParametersLocal;
use num_traits::Float;
use std::ops::{Add, Sub};
use std::{mem, ptr};

pub trait ReadFloatBigEndian: Sized {
    fn read_float_be(value_ptr: *const u8) -> Self;
}

impl ReadFloatBigEndian for f32 {
    fn read_float_be(value_ptr: *const u8) -> Self {
        unsafe { mem::transmute::<u32, f32>(u32::swap_bytes(ptr::read_unaligned(value_ptr as *const u32))) }
    }
}

impl ReadFloatBigEndian for f64 {
    fn read_float_be(value_ptr: *const u8) -> Self {
        unsafe { mem::transmute::<u64, f64>(u64::swap_bytes(ptr::read_unaligned(value_ptr as *const u64))) }
    }
}

pub struct ScalarComparisonsFloatBigEndian {}

impl ScalarComparisonsFloatBigEndian {
    pub fn get_compare_equal<PrimitiveType: PartialEq + Float + Sub<Output = PrimitiveType> + ReadFloatBigEndian + 'static>(
        scan_parameters_global: &ScanParametersGlobal,
        scan_parameters_local: &ScanParametersLocal,
    ) -> Option<ScalarCompareFnImmediate> {
        if let Some(immediate_value) = scan_parameters_global.deanonymize_immediate(scan_parameters_local.get_data_type()) {
            // Note: The typical integer optimization of leaving the values unswapped does not work for floating points, so we must swap as normal.
            let tolerance = scan_parameters_global
                .get_floating_point_tolerance()
                .get_value();
            let immediate_value_ptr = immediate_value.as_ptr();
            let immediate_value: PrimitiveType = ReadFloatBigEndian::read_float_be(immediate_value_ptr);

            Some(Box::new(move |current_value_ptr| {
                let current_value: PrimitiveType = ReadFloatBigEndian::read_float_be(current_value_ptr);

                // Equality between the current and immediate value is determined by being within the given tolerance.
                current_value.sub(immediate_value).abs() <= tolerance
            }))
        } else {
            None
        }
    }

    pub fn get_compare_not_equal<PrimitiveType: PartialEq + Float + Sub<Output = PrimitiveType> + ReadFloatBigEndian + 'static>(
        scan_parameters_global: &ScanParametersGlobal,
        scan_parameters_local: &ScanParametersLocal,
    ) -> Option<ScalarCompareFnImmediate> {
        // Note: The typical integer optimization of leaving the values unswapped does not work for floating points, so we must swap as normal.
        if let Some(immediate_value) = scan_parameters_global.deanonymize_immediate(scan_parameters_local.get_data_type()) {
            let tolerance = scan_parameters_global
                .get_floating_point_tolerance()
                .get_value();
            let immediate_value_ptr = immediate_value.as_ptr();
            let immediate_value: PrimitiveType = ReadFloatBigEndian::read_float_be(immediate_value_ptr);

            Some(Box::new(move |current_value_ptr| {
                let current_value: PrimitiveType = ReadFloatBigEndian::read_float_be(current_value_ptr);

                // Inequality between the current and immediate value is determined by being outside the given tolerance.
                current_value.sub(immediate_value).abs() > tolerance
            }))
        } else {
            None
        }
    }

    pub fn get_compare_greater_than<PrimitiveType: PartialOrd + ReadFloatBigEndian + 'static>(
        scan_parameters_global: &ScanParametersGlobal,
        scan_parameters_local: &ScanParametersLocal,
    ) -> Option<ScalarCompareFnImmediate> {
        if let Some(immediate_value) = scan_parameters_global.deanonymize_immediate(scan_parameters_local.get_data_type()) {
            let immediate_value_ptr = immediate_value.as_ptr();
            let immediate_value: PrimitiveType = ReadFloatBigEndian::read_float_be(immediate_value_ptr);

            Some(Box::new(move |current_value_ptr| {
                let current_value: PrimitiveType = ReadFloatBigEndian::read_float_be(current_value_ptr);

                // No checks tolerance required.
                current_value > immediate_value
            }))
        } else {
            None
        }
    }

    pub fn get_compare_greater_than_or_equal<PrimitiveType: PartialOrd + ReadFloatBigEndian + 'static>(
        scan_parameters_global: &ScanParametersGlobal,
        scan_parameters_local: &ScanParametersLocal,
    ) -> Option<ScalarCompareFnImmediate> {
        if let Some(immediate_value) = scan_parameters_global.deanonymize_immediate(scan_parameters_local.get_data_type()) {
            let immediate_value_ptr = immediate_value.as_ptr();
            let immediate_value: PrimitiveType = ReadFloatBigEndian::read_float_be(immediate_value_ptr);

            Some(Box::new(move |current_value_ptr| {
                let current_value: PrimitiveType = ReadFloatBigEndian::read_float_be(current_value_ptr);

                // No checks tolerance required.
                current_value >= immediate_value
            }))
        } else {
            None
        }
    }

    pub fn get_compare_less_than<PrimitiveType: PartialOrd + ReadFloatBigEndian + 'static>(
        scan_parameters_global: &ScanParametersGlobal,
        scan_parameters_local: &ScanParametersLocal,
    ) -> Option<ScalarCompareFnImmediate> {
        if let Some(immediate_value) = scan_parameters_global.deanonymize_immediate(scan_parameters_local.get_data_type()) {
            let immediate_value_ptr = immediate_value.as_ptr();
            let immediate_value: PrimitiveType = ReadFloatBigEndian::read_float_be(immediate_value_ptr);

            Some(Box::new(move |current_value_ptr| {
                let current_value: PrimitiveType = ReadFloatBigEndian::read_float_be(current_value_ptr);

                // No checks tolerance required.
                current_value < immediate_value
            }))
        } else {
            None
        }
    }

    pub fn get_compare_less_than_or_equal<PrimitiveType: PartialOrd + ReadFloatBigEndian + 'static>(
        scan_parameters_global: &ScanParametersGlobal,
        scan_parameters_local: &ScanParametersLocal,
    ) -> Option<ScalarCompareFnImmediate> {
        if let Some(immediate_value) = scan_parameters_global.deanonymize_immediate(scan_parameters_local.get_data_type()) {
            let immediate_value_ptr = immediate_value.as_ptr();
            let immediate_value: PrimitiveType = ReadFloatBigEndian::read_float_be(immediate_value_ptr);

            Some(Box::new(move |current_value_ptr| {
                let current_value: PrimitiveType = ReadFloatBigEndian::read_float_be(current_value_ptr);

                // No checks tolerance required.
                current_value <= immediate_value
            }))
        } else {
            None
        }
    }

    pub fn get_compare_changed<PrimitiveType: PartialEq + ReadFloatBigEndian + 'static>(
        _scan_parameters_global: &ScanParametersGlobal,
        _scan_parameters_local: &ScanParametersLocal,
    ) -> Option<ScalarCompareFnRelative> {
        Some(Box::new(move |current_value_ptr, previous_value_ptr| unsafe {
            // Optimization: no endian byte swaps required for current or previous values.
            let current_value = ptr::read_unaligned(current_value_ptr as *const PrimitiveType);
            let previous_value = ptr::read_unaligned(previous_value_ptr as *const PrimitiveType);

            // No checks tolerance required.
            current_value != previous_value
        }))
    }

    pub fn get_compare_unchanged<PrimitiveType: PartialEq + ReadFloatBigEndian + 'static>(
        _scan_parameters_global: &ScanParametersGlobal,
        _scan_parameters_local: &ScanParametersLocal,
    ) -> Option<ScalarCompareFnRelative> {
        Some(Box::new(move |current_value_ptr, previous_value_ptr| unsafe {
            // Optimization: no endian byte swaps required for current or previous values.
            let current_value = ptr::read_unaligned(current_value_ptr as *const PrimitiveType);
            let previous_value = ptr::read_unaligned(previous_value_ptr as *const PrimitiveType);

            // No checks tolerance required.
            current_value == previous_value
        }))
    }

    pub fn get_compare_increased<PrimitiveType: PartialOrd + ReadFloatBigEndian + 'static>(
        _scan_parameters_global: &ScanParametersGlobal,
        _scan_parameters_local: &ScanParametersLocal,
    ) -> Option<ScalarCompareFnRelative> {
        Some(Box::new(move |current_value_ptr, previous_value_ptr| {
            let current_value: PrimitiveType = ReadFloatBigEndian::read_float_be(current_value_ptr);
            let previous_value: PrimitiveType = ReadFloatBigEndian::read_float_be(previous_value_ptr);

            // No checks tolerance required.
            current_value > previous_value
        }))
    }

    pub fn get_compare_decreased<PrimitiveType: PartialOrd + ReadFloatBigEndian + 'static>(
        _scan_parameters_global: &ScanParametersGlobal,
        _scan_parameters_local: &ScanParametersLocal,
    ) -> Option<ScalarCompareFnRelative> {
        Some(Box::new(move |current_value_ptr, previous_value_ptr| {
            let current_value: PrimitiveType = ReadFloatBigEndian::read_float_be(current_value_ptr);
            let previous_value: PrimitiveType = ReadFloatBigEndian::read_float_be(previous_value_ptr);

            // No checks tolerance required.
            current_value < previous_value
        }))
    }

    pub fn get_compare_increased_by<
        PrimitiveType: Copy + PartialEq + Float + Add<Output = PrimitiveType> + Sub<Output = PrimitiveType> + ReadFloatBigEndian + 'static,
    >(
        scan_parameters_global: &ScanParametersGlobal,
        scan_parameters_local: &ScanParametersLocal,
    ) -> Option<ScalarCompareFnDelta> {
        if let Some(delta_value) = scan_parameters_global.deanonymize_immediate(scan_parameters_local.get_data_type()) {
            // Note: The typical integer optimization of leaving the values unswapped does not work for floating points, so we must swap as normal.
            let tolerance = scan_parameters_global
                .get_floating_point_tolerance()
                .get_value();
            let delta_value_ptr = delta_value.as_ptr();
            let delta_value: PrimitiveType = ReadFloatBigEndian::read_float_be(delta_value_ptr);

            Some(Box::new(move |current_value_ptr, previous_value_ptr| {
                let current_value: PrimitiveType = ReadFloatBigEndian::read_float_be(current_value_ptr);
                let previous_value: PrimitiveType = ReadFloatBigEndian::read_float_be(previous_value_ptr);
                let target_value = previous_value.add(delta_value);

                // Equality between the current and target value is determined by being within the given tolerance.
                current_value.sub(target_value).abs() <= tolerance
            }))
        } else {
            None
        }
    }

    pub fn get_compare_decreased_by<
        PrimitiveType: Copy + PartialEq + Float + Add<Output = PrimitiveType> + Sub<Output = PrimitiveType> + ReadFloatBigEndian + 'static,
    >(
        scan_parameters_global: &ScanParametersGlobal,
        scan_parameters_local: &ScanParametersLocal,
    ) -> Option<ScalarCompareFnDelta> {
        if let Some(delta_value) = scan_parameters_global.deanonymize_immediate(scan_parameters_local.get_data_type()) {
            // Note: The typical integer optimization of leaving the values unswapped does not work for floating points, so we must swap as normal.
            let tolerance = scan_parameters_global
                .get_floating_point_tolerance()
                .get_value();
            let delta_value_ptr = delta_value.as_ptr();
            let delta_value: PrimitiveType = ReadFloatBigEndian::read_float_be(delta_value_ptr);

            Some(Box::new(move |current_value_ptr, previous_value_ptr| {
                let current_value: PrimitiveType = ReadFloatBigEndian::read_float_be(current_value_ptr);
                let previous_value: PrimitiveType = ReadFloatBigEndian::read_float_be(previous_value_ptr);
                let target_value = previous_value.add(delta_value);

                // Equality between the current and target value is determined by being within the given tolerance.
                current_value.sub(target_value).abs() <= tolerance
            }))
        } else {
            None
        }
    }
}
