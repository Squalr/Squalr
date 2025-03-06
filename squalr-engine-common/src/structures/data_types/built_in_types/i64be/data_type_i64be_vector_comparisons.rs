use crate::structures::data_types::built_in_types::i64be::data_type_i64be::DataTypeI64be;
use crate::structures::data_types::comparisons::vector_comparable::VectorComparable;
use crate::structures::data_types::comparisons::vector_comparable::{
    VectorCompareFnDelta16, VectorCompareFnDelta32, VectorCompareFnDelta64, VectorCompareFnImmediate16, VectorCompareFnImmediate32, VectorCompareFnImmediate64,
    VectorCompareFnRelative16, VectorCompareFnRelative32, VectorCompareFnRelative64,
};
use crate::structures::data_types::generics::vector_generics::VectorGenerics;
use crate::structures::scanning::scan_parameters_global::ScanParametersGlobal;
use crate::structures::scanning::scan_parameters_local::ScanParametersLocal;
use std::ops::{Add, Sub};
use std::ptr;
use std::simd::cmp::{SimdPartialEq, SimdPartialOrd};
use std::simd::num::SimdInt;
use std::simd::{LaneCount, Simd, SupportedLaneCount};

type PrimitiveType = i64;

struct DataTypeI64beVector {}

impl DataTypeI64beVector {
    pub fn get_vector_compare_equal<const N: usize>(
        scan_parameters_global: &ScanParametersGlobal,
        scan_parameters_local: &ScanParametersLocal,
    ) -> Option<Box<dyn Fn(*const u8) -> Simd<u8, N>>>
    where
        LaneCount<N>: SupportedLaneCount,
    {
        if let Some(immediate_value) = scan_parameters_global.deanonymize_immediate(scan_parameters_local.get_data_type()) {
            // Optimization: no endian byte swap required for immediate or current values.
            unsafe {
                let immediate_value_ptr = immediate_value.as_ptr();
                let immediate_value = Simd::splat(ptr::read_unaligned(immediate_value_ptr as *const PrimitiveType));

                Some(Box::new(move |current_values_ptr| {
                    let current_values = Simd::from_array(ptr::read_unaligned(current_values_ptr as *const [PrimitiveType; N]));

                    VectorGenerics::transmute_mask::<PrimitiveType, N>(&current_values.simd_eq(immediate_value))
                }))
            }
        } else {
            None
        }
    }

    pub fn get_vector_compare_not_equal<const N: usize>(
        scan_parameters_global: &ScanParametersGlobal,
        scan_parameters_local: &ScanParametersLocal,
    ) -> Option<Box<dyn Fn(*const u8) -> Simd<u8, N>>>
    where
        LaneCount<N>: SupportedLaneCount,
    {
        if let Some(immediate_value) = scan_parameters_global.deanonymize_immediate(scan_parameters_local.get_data_type()) {
            // Optimization: no endian byte swap required for immediate or current values.
            unsafe {
                let immediate_value_ptr = immediate_value.as_ptr();
                let immediate_value = Simd::splat(ptr::read_unaligned(immediate_value_ptr as *const PrimitiveType));

                Some(Box::new(move |current_values_ptr| {
                    let current_values = Simd::from_array(ptr::read_unaligned(current_values_ptr as *const [PrimitiveType; N]));

                    VectorGenerics::transmute_mask::<PrimitiveType, N>(&current_values.simd_ne(immediate_value))
                }))
            }
        } else {
            None
        }
    }

    pub fn get_vector_compare_greater_than<const N: usize>(
        scan_parameters_global: &ScanParametersGlobal,
        scan_parameters_local: &ScanParametersLocal,
    ) -> Option<Box<dyn Fn(*const u8) -> Simd<u8, N>>>
    where
        LaneCount<N>: SupportedLaneCount,
    {
        if let Some(immediate_value) = scan_parameters_global.deanonymize_immediate(scan_parameters_local.get_data_type()) {
            unsafe {
                let immediate_value_ptr = immediate_value.as_ptr();
                let immediate_value = Simd::swap_bytes(Simd::splat(ptr::read_unaligned(immediate_value_ptr as *const PrimitiveType)));

                Some(Box::new(move |current_values_ptr| {
                    let current_values = Simd::swap_bytes(Simd::from_array(ptr::read_unaligned(current_values_ptr as *const [PrimitiveType; N])));

                    VectorGenerics::transmute_mask::<PrimitiveType, N>(&current_values.simd_gt(immediate_value))
                }))
            }
        } else {
            None
        }
    }

    pub fn get_vector_compare_greater_than_or_equal<const N: usize>(
        scan_parameters_global: &ScanParametersGlobal,
        scan_parameters_local: &ScanParametersLocal,
    ) -> Option<Box<dyn Fn(*const u8) -> Simd<u8, N>>>
    where
        LaneCount<N>: SupportedLaneCount,
    {
        if let Some(immediate_value) = scan_parameters_global.deanonymize_immediate(scan_parameters_local.get_data_type()) {
            unsafe {
                let immediate_value_ptr = immediate_value.as_ptr();
                let immediate_value = Simd::swap_bytes(Simd::splat(ptr::read_unaligned(immediate_value_ptr as *const PrimitiveType)));

                Some(Box::new(move |current_values_ptr| {
                    let current_values = Simd::swap_bytes(Simd::from_array(ptr::read_unaligned(current_values_ptr as *const [PrimitiveType; N])));

                    VectorGenerics::transmute_mask::<PrimitiveType, N>(&current_values.simd_ge(immediate_value))
                }))
            }
        } else {
            None
        }
    }

    pub fn get_vector_compare_less_than<const N: usize>(
        scan_parameters_global: &ScanParametersGlobal,
        scan_parameters_local: &ScanParametersLocal,
    ) -> Option<Box<dyn Fn(*const u8) -> Simd<u8, N>>>
    where
        LaneCount<N>: SupportedLaneCount,
    {
        if let Some(immediate_value) = scan_parameters_global.deanonymize_immediate(scan_parameters_local.get_data_type()) {
            unsafe {
                let immediate_value_ptr = immediate_value.as_ptr();
                let immediate_value = Simd::swap_bytes(Simd::splat(ptr::read_unaligned(immediate_value_ptr as *const PrimitiveType)));

                Some(Box::new(move |current_values_ptr| {
                    let current_values = Simd::swap_bytes(Simd::from_array(ptr::read_unaligned(current_values_ptr as *const [PrimitiveType; N])));

                    VectorGenerics::transmute_mask::<PrimitiveType, N>(&current_values.simd_lt(immediate_value))
                }))
            }
        } else {
            None
        }
    }

    pub fn get_vector_compare_less_than_or_equal<const N: usize>(
        scan_parameters_global: &ScanParametersGlobal,
        scan_parameters_local: &ScanParametersLocal,
    ) -> Option<Box<dyn Fn(*const u8) -> Simd<u8, N>>>
    where
        LaneCount<N>: SupportedLaneCount,
    {
        if let Some(immediate_value) = scan_parameters_global.deanonymize_immediate(scan_parameters_local.get_data_type()) {
            unsafe {
                let immediate_value_ptr = immediate_value.as_ptr();
                let immediate_value = Simd::swap_bytes(Simd::splat(ptr::read_unaligned(immediate_value_ptr as *const PrimitiveType)));

                Some(Box::new(move |current_values_ptr| {
                    let current_values = Simd::swap_bytes(Simd::from_array(ptr::read_unaligned(current_values_ptr as *const [PrimitiveType; N])));

                    VectorGenerics::transmute_mask::<PrimitiveType, N>(&current_values.simd_le(immediate_value))
                }))
            }
        } else {
            None
        }
    }

    pub fn get_vector_compare_changed<const N: usize>(
        _scan_parameters_global: &ScanParametersGlobal,
        _scan_parameters_local: &ScanParametersLocal,
    ) -> Option<Box<dyn Fn(*const u8, *const u8) -> Simd<u8, N>>>
    where
        LaneCount<N>: SupportedLaneCount,
    {
        Some(Box::new(move |current_values_ptr, previous_values_ptr| unsafe {
            // Optimization: no endian byte swap required.
            let current_values = Simd::from_array(ptr::read_unaligned(current_values_ptr as *const [PrimitiveType; N]));
            let previous_values = Simd::from_array(ptr::read_unaligned(previous_values_ptr as *const [PrimitiveType; N]));

            VectorGenerics::transmute_mask::<PrimitiveType, N>(&current_values.simd_ne(previous_values))
        }))
    }

    pub fn get_vector_compare_unchanged<const N: usize>(
        _scan_parameters_global: &ScanParametersGlobal,
        _scan_parameters_local: &ScanParametersLocal,
    ) -> Option<Box<dyn Fn(*const u8, *const u8) -> Simd<u8, N>>>
    where
        LaneCount<N>: SupportedLaneCount,
    {
        Some(Box::new(move |current_values_ptr, previous_values_ptr| unsafe {
            // Optimization: no endian byte swap required.
            let current_values = Simd::from_array(ptr::read_unaligned(current_values_ptr as *const [PrimitiveType; N]));
            let previous_values = Simd::from_array(ptr::read_unaligned(previous_values_ptr as *const [PrimitiveType; N]));

            VectorGenerics::transmute_mask::<PrimitiveType, N>(&current_values.simd_eq(previous_values))
        }))
    }

    pub fn get_vector_compare_increased<const N: usize>(
        _scan_parameters_global: &ScanParametersGlobal,
        _scan_parameters_local: &ScanParametersLocal,
    ) -> Option<Box<dyn Fn(*const u8, *const u8) -> Simd<u8, N>>>
    where
        LaneCount<N>: SupportedLaneCount,
    {
        Some(Box::new(move |current_values_ptr, previous_values_ptr| unsafe {
            let current_values = Simd::swap_bytes(Simd::from_array(ptr::read_unaligned(current_values_ptr as *const [PrimitiveType; N])));
            let previous_values: Simd<PrimitiveType, N> = Simd::swap_bytes(Simd::splat(ptr::read_unaligned(previous_values_ptr as *const PrimitiveType)));

            VectorGenerics::transmute_mask::<PrimitiveType, N>(&current_values.simd_gt(previous_values))
        }))
    }

    pub fn get_vector_compare_decreased<const N: usize>(
        _scan_parameters_global: &ScanParametersGlobal,
        _scan_parameters_local: &ScanParametersLocal,
    ) -> Option<Box<dyn Fn(*const u8, *const u8) -> Simd<u8, N>>>
    where
        LaneCount<N>: SupportedLaneCount,
    {
        Some(Box::new(move |current_values_ptr, previous_values_ptr| unsafe {
            let current_values = Simd::swap_bytes(Simd::from_array(ptr::read_unaligned(current_values_ptr as *const [PrimitiveType; N])));
            let previous_values: Simd<PrimitiveType, N> = Simd::swap_bytes(Simd::splat(ptr::read_unaligned(previous_values_ptr as *const PrimitiveType)));

            VectorGenerics::transmute_mask::<PrimitiveType, N>(&current_values.simd_lt(previous_values))
        }))
    }

    pub fn get_vector_compare_increased_by<const N: usize>(
        scan_parameters_global: &ScanParametersGlobal,
        scan_parameters_local: &ScanParametersLocal,
    ) -> Option<Box<dyn Fn(*const u8, *const u8) -> Simd<u8, N>>>
    where
        LaneCount<N>: SupportedLaneCount,
    {
        if let Some(delta_value) = scan_parameters_global.deanonymize_immediate(scan_parameters_local.get_data_type()) {
            unsafe {
                let delta_value_ptr = delta_value.as_ptr();
                let delta_value = Simd::swap_bytes(Simd::splat(ptr::read_unaligned(delta_value_ptr as *const PrimitiveType)));

                Some(Box::new(move |current_values_ptr, previous_values_ptr| {
                    let current_values = Simd::swap_bytes(Simd::from_array(ptr::read_unaligned(current_values_ptr as *const [PrimitiveType; N])));
                    let previous_values = Simd::swap_bytes(Simd::from_array(ptr::read_unaligned(previous_values_ptr as *const [PrimitiveType; N])));

                    VectorGenerics::transmute_mask::<PrimitiveType, N>(&current_values.simd_eq(previous_values.add(delta_value)))
                }))
            }
        } else {
            None
        }
    }

    pub fn get_vector_compare_decreased_by<const N: usize>(
        scan_parameters_global: &ScanParametersGlobal,
        scan_parameters_local: &ScanParametersLocal,
    ) -> Option<Box<dyn Fn(*const u8, *const u8) -> Simd<u8, N>>>
    where
        LaneCount<N>: SupportedLaneCount,
    {
        if let Some(delta_value) = scan_parameters_global.deanonymize_immediate(scan_parameters_local.get_data_type()) {
            unsafe {
                let delta_value_ptr = delta_value.as_ptr();
                let delta_value = Simd::swap_bytes(Simd::splat(ptr::read_unaligned(delta_value_ptr as *const PrimitiveType)));

                Some(Box::new(move |current_values_ptr, previous_values_ptr| {
                    let current_values = Simd::swap_bytes(Simd::from_array(ptr::read_unaligned(current_values_ptr as *const [PrimitiveType; N])));
                    let previous_values = Simd::swap_bytes(Simd::from_array(ptr::read_unaligned(previous_values_ptr as *const [PrimitiveType; N])));

                    VectorGenerics::transmute_mask::<PrimitiveType, N>(&current_values.simd_eq(previous_values.sub(delta_value)))
                }))
            }
        } else {
            None
        }
    }
}

impl VectorComparable for DataTypeI64be {
    fn get_vector_compare_equal_64(
        &self,
        scan_parameters_global: &ScanParametersGlobal,
        scan_parameters_local: &ScanParametersLocal,
    ) -> Option<VectorCompareFnImmediate64> {
        DataTypeI64beVector::get_vector_compare_equal(scan_parameters_global, scan_parameters_local)
    }

    fn get_vector_compare_equal_32(
        &self,
        scan_parameters_global: &ScanParametersGlobal,
        scan_parameters_local: &ScanParametersLocal,
    ) -> Option<VectorCompareFnImmediate32> {
        DataTypeI64beVector::get_vector_compare_equal(scan_parameters_global, scan_parameters_local)
    }

    fn get_vector_compare_equal_16(
        &self,
        scan_parameters_global: &ScanParametersGlobal,
        scan_parameters_local: &ScanParametersLocal,
    ) -> Option<VectorCompareFnImmediate16> {
        DataTypeI64beVector::get_vector_compare_equal(scan_parameters_global, scan_parameters_local)
    }

    fn get_vector_compare_not_equal_64(
        &self,
        scan_parameters_global: &ScanParametersGlobal,
        scan_parameters_local: &ScanParametersLocal,
    ) -> Option<VectorCompareFnImmediate64> {
        DataTypeI64beVector::get_vector_compare_not_equal(scan_parameters_global, scan_parameters_local)
    }

    fn get_vector_compare_not_equal_32(
        &self,
        scan_parameters_global: &ScanParametersGlobal,
        scan_parameters_local: &ScanParametersLocal,
    ) -> Option<VectorCompareFnImmediate32> {
        DataTypeI64beVector::get_vector_compare_not_equal(scan_parameters_global, scan_parameters_local)
    }

    fn get_vector_compare_not_equal_16(
        &self,
        scan_parameters_global: &ScanParametersGlobal,
        scan_parameters_local: &ScanParametersLocal,
    ) -> Option<VectorCompareFnImmediate16> {
        DataTypeI64beVector::get_vector_compare_not_equal(scan_parameters_global, scan_parameters_local)
    }

    fn get_vector_compare_greater_than_64(
        &self,
        scan_parameters_global: &ScanParametersGlobal,
        scan_parameters_local: &ScanParametersLocal,
    ) -> Option<VectorCompareFnImmediate64> {
        DataTypeI64beVector::get_vector_compare_greater_than(scan_parameters_global, scan_parameters_local)
    }

    fn get_vector_compare_greater_than_32(
        &self,
        scan_parameters_global: &ScanParametersGlobal,
        scan_parameters_local: &ScanParametersLocal,
    ) -> Option<VectorCompareFnImmediate32> {
        DataTypeI64beVector::get_vector_compare_greater_than(scan_parameters_global, scan_parameters_local)
    }

    fn get_vector_compare_greater_than_16(
        &self,
        scan_parameters_global: &ScanParametersGlobal,
        scan_parameters_local: &ScanParametersLocal,
    ) -> Option<VectorCompareFnImmediate16> {
        DataTypeI64beVector::get_vector_compare_greater_than(scan_parameters_global, scan_parameters_local)
    }

    fn get_vector_compare_greater_than_or_equal_64(
        &self,
        scan_parameters_global: &ScanParametersGlobal,
        scan_parameters_local: &ScanParametersLocal,
    ) -> Option<VectorCompareFnImmediate64> {
        DataTypeI64beVector::get_vector_compare_greater_than_or_equal(scan_parameters_global, scan_parameters_local)
    }

    fn get_vector_compare_greater_than_or_equal_32(
        &self,
        scan_parameters_global: &ScanParametersGlobal,
        scan_parameters_local: &ScanParametersLocal,
    ) -> Option<VectorCompareFnImmediate32> {
        DataTypeI64beVector::get_vector_compare_greater_than_or_equal(scan_parameters_global, scan_parameters_local)
    }

    fn get_vector_compare_greater_than_or_equal_16(
        &self,
        scan_parameters_global: &ScanParametersGlobal,
        scan_parameters_local: &ScanParametersLocal,
    ) -> Option<VectorCompareFnImmediate16> {
        DataTypeI64beVector::get_vector_compare_greater_than_or_equal(scan_parameters_global, scan_parameters_local)
    }

    fn get_vector_compare_less_than_64(
        &self,
        scan_parameters_global: &ScanParametersGlobal,
        scan_parameters_local: &ScanParametersLocal,
    ) -> Option<VectorCompareFnImmediate64> {
        DataTypeI64beVector::get_vector_compare_less_than(scan_parameters_global, scan_parameters_local)
    }

    fn get_vector_compare_less_than_32(
        &self,
        scan_parameters_global: &ScanParametersGlobal,
        scan_parameters_local: &ScanParametersLocal,
    ) -> Option<VectorCompareFnImmediate32> {
        DataTypeI64beVector::get_vector_compare_less_than(scan_parameters_global, scan_parameters_local)
    }

    fn get_vector_compare_less_than_16(
        &self,
        scan_parameters_global: &ScanParametersGlobal,
        scan_parameters_local: &ScanParametersLocal,
    ) -> Option<VectorCompareFnImmediate16> {
        DataTypeI64beVector::get_vector_compare_less_than(scan_parameters_global, scan_parameters_local)
    }

    fn get_vector_compare_less_than_or_equal_64(
        &self,
        scan_parameters_global: &ScanParametersGlobal,
        scan_parameters_local: &ScanParametersLocal,
    ) -> Option<VectorCompareFnImmediate64> {
        DataTypeI64beVector::get_vector_compare_less_than_or_equal(scan_parameters_global, scan_parameters_local)
    }

    fn get_vector_compare_less_than_or_equal_32(
        &self,
        scan_parameters_global: &ScanParametersGlobal,
        scan_parameters_local: &ScanParametersLocal,
    ) -> Option<VectorCompareFnImmediate32> {
        DataTypeI64beVector::get_vector_compare_less_than_or_equal(scan_parameters_global, scan_parameters_local)
    }

    fn get_vector_compare_less_than_or_equal_16(
        &self,
        scan_parameters_global: &ScanParametersGlobal,
        scan_parameters_local: &ScanParametersLocal,
    ) -> Option<VectorCompareFnImmediate16> {
        DataTypeI64beVector::get_vector_compare_less_than_or_equal(scan_parameters_global, scan_parameters_local)
    }

    fn get_vector_compare_changed_64(
        &self,
        scan_parameters_global: &ScanParametersGlobal,
        scan_parameters_local: &ScanParametersLocal,
    ) -> Option<VectorCompareFnRelative64> {
        DataTypeI64beVector::get_vector_compare_changed(scan_parameters_global, scan_parameters_local)
    }

    fn get_vector_compare_changed_32(
        &self,
        scan_parameters_global: &ScanParametersGlobal,
        scan_parameters_local: &ScanParametersLocal,
    ) -> Option<VectorCompareFnRelative32> {
        DataTypeI64beVector::get_vector_compare_changed(scan_parameters_global, scan_parameters_local)
    }

    fn get_vector_compare_changed_16(
        &self,
        scan_parameters_global: &ScanParametersGlobal,
        scan_parameters_local: &ScanParametersLocal,
    ) -> Option<VectorCompareFnRelative16> {
        DataTypeI64beVector::get_vector_compare_changed(scan_parameters_global, scan_parameters_local)
    }

    fn get_vector_compare_unchanged_64(
        &self,
        scan_parameters_global: &ScanParametersGlobal,
        scan_parameters_local: &ScanParametersLocal,
    ) -> Option<VectorCompareFnRelative64> {
        DataTypeI64beVector::get_vector_compare_unchanged(scan_parameters_global, scan_parameters_local)
    }

    fn get_vector_compare_unchanged_32(
        &self,
        scan_parameters_global: &ScanParametersGlobal,
        scan_parameters_local: &ScanParametersLocal,
    ) -> Option<VectorCompareFnRelative32> {
        DataTypeI64beVector::get_vector_compare_unchanged(scan_parameters_global, scan_parameters_local)
    }

    fn get_vector_compare_unchanged_16(
        &self,
        scan_parameters_global: &ScanParametersGlobal,
        scan_parameters_local: &ScanParametersLocal,
    ) -> Option<VectorCompareFnRelative16> {
        DataTypeI64beVector::get_vector_compare_unchanged(scan_parameters_global, scan_parameters_local)
    }

    fn get_vector_compare_increased_64(
        &self,
        scan_parameters_global: &ScanParametersGlobal,
        scan_parameters_local: &ScanParametersLocal,
    ) -> Option<VectorCompareFnRelative64> {
        DataTypeI64beVector::get_vector_compare_increased(scan_parameters_global, scan_parameters_local)
    }

    fn get_vector_compare_increased_32(
        &self,
        scan_parameters_global: &ScanParametersGlobal,
        scan_parameters_local: &ScanParametersLocal,
    ) -> Option<VectorCompareFnRelative32> {
        DataTypeI64beVector::get_vector_compare_increased(scan_parameters_global, scan_parameters_local)
    }

    fn get_vector_compare_increased_16(
        &self,
        scan_parameters_global: &ScanParametersGlobal,
        scan_parameters_local: &ScanParametersLocal,
    ) -> Option<VectorCompareFnRelative16> {
        DataTypeI64beVector::get_vector_compare_increased(scan_parameters_global, scan_parameters_local)
    }

    fn get_vector_compare_decreased_64(
        &self,
        scan_parameters_global: &ScanParametersGlobal,
        scan_parameters_local: &ScanParametersLocal,
    ) -> Option<VectorCompareFnRelative64> {
        DataTypeI64beVector::get_vector_compare_decreased(scan_parameters_global, scan_parameters_local)
    }

    fn get_vector_compare_decreased_32(
        &self,
        scan_parameters_global: &ScanParametersGlobal,
        scan_parameters_local: &ScanParametersLocal,
    ) -> Option<VectorCompareFnRelative32> {
        DataTypeI64beVector::get_vector_compare_decreased(scan_parameters_global, scan_parameters_local)
    }

    fn get_vector_compare_decreased_16(
        &self,
        scan_parameters_global: &ScanParametersGlobal,
        scan_parameters_local: &ScanParametersLocal,
    ) -> Option<VectorCompareFnRelative16> {
        DataTypeI64beVector::get_vector_compare_decreased(scan_parameters_global, scan_parameters_local)
    }

    fn get_vector_compare_increased_by_64(
        &self,
        scan_parameters_global: &ScanParametersGlobal,
        scan_parameters_local: &ScanParametersLocal,
    ) -> Option<VectorCompareFnDelta64> {
        DataTypeI64beVector::get_vector_compare_increased_by(scan_parameters_global, scan_parameters_local)
    }

    fn get_vector_compare_increased_by_32(
        &self,
        scan_parameters_global: &ScanParametersGlobal,
        scan_parameters_local: &ScanParametersLocal,
    ) -> Option<VectorCompareFnDelta32> {
        DataTypeI64beVector::get_vector_compare_increased_by(scan_parameters_global, scan_parameters_local)
    }

    fn get_vector_compare_increased_by_16(
        &self,
        scan_parameters_global: &ScanParametersGlobal,
        scan_parameters_local: &ScanParametersLocal,
    ) -> Option<VectorCompareFnDelta16> {
        DataTypeI64beVector::get_vector_compare_increased_by(scan_parameters_global, scan_parameters_local)
    }

    fn get_vector_compare_decreased_by_64(
        &self,
        scan_parameters_global: &ScanParametersGlobal,
        scan_parameters_local: &ScanParametersLocal,
    ) -> Option<VectorCompareFnDelta64> {
        DataTypeI64beVector::get_vector_compare_decreased_by(scan_parameters_global, scan_parameters_local)
    }

    fn get_vector_compare_decreased_by_32(
        &self,
        scan_parameters_global: &ScanParametersGlobal,
        scan_parameters_local: &ScanParametersLocal,
    ) -> Option<VectorCompareFnDelta32> {
        DataTypeI64beVector::get_vector_compare_decreased_by(scan_parameters_global, scan_parameters_local)
    }

    fn get_vector_compare_decreased_by_16(
        &self,
        scan_parameters_global: &ScanParametersGlobal,
        scan_parameters_local: &ScanParametersLocal,
    ) -> Option<VectorCompareFnDelta16> {
        DataTypeI64beVector::get_vector_compare_decreased_by(scan_parameters_global, scan_parameters_local)
    }
}
