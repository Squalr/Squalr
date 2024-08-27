use crate::scanners::comparers::vector::encoder::scanner_vector_comparer::ScannerVectorComparer;
use crate::scanners::comparers::vector::encoder::scanner_vector_comparer::VectorComparer;
use crate::scanners::comparers::vector::types::simd_type::SimdType;
use crate::scanners::parameters::scan_compare_type::ScanCompareType;
use squalr_engine_common::values::data_type::DataType;
use squalr_engine_memory::memory_alignment::MemoryAlignment;
use std::simd::cmp::SimdPartialEq;
use std::simd::{LaneCount, Simd, SimdElement, SupportedLaneCount};

pub struct ScannerVectorComparerCascading<T: SimdElement + SimdType + PartialEq, const N: usize>
where
    T: SimdElement + SimdType + PartialEq,
    LaneCount<N>: SupportedLaneCount,
    LaneCount<{ N / 2 }>: SupportedLaneCount,
    LaneCount<{ N / 4 }>: SupportedLaneCount,
    LaneCount<{ N / 8 }>: SupportedLaneCount,
    Simd<T, N>: SimdPartialEq,
{
    inner: ScannerVectorComparer<T, N>,
    memory_alignment: MemoryAlignment,
}

impl<T, const N: usize> VectorComparer<T, N> for ScannerVectorComparerCascading<T, N>
where
    T: SimdElement + SimdType + PartialEq,
    LaneCount<N>: SupportedLaneCount,
    LaneCount<{ N / 2 }>: SupportedLaneCount,
    LaneCount<{ N / 4 }>: SupportedLaneCount,
    LaneCount<{ N / 8 }>: SupportedLaneCount,
    Simd<T, N>: SimdPartialEq,
{
    fn get_immediate_compare_func(
        &self,
        scan_compare_type: ScanCompareType,
        data_type: &DataType,
    ) -> impl Fn(*const u8, *const u8) -> Simd<u8, N> {
        self.get_immediate_compare_func(scan_compare_type, data_type)
    }

    fn get_relative_compare_func(
        &self,
        scan_compare_type: ScanCompareType,
        data_type: &DataType,
    ) -> impl Fn(*const u8, *const u8) -> Simd<u8, N> {
        self.get_relative_compare_func(scan_compare_type, data_type)
    }

    fn get_relative_delta_compare_func(
        &self,
        scan_compare_type: ScanCompareType,
        data_type: &DataType,
    ) -> impl Fn(*const u8, *const u8, *const u8) -> Simd<u8, N> {
        self.get_relative_delta_compare_func(scan_compare_type, data_type)
    }
}

/// This vector comparer allows for comparing vectors of data where the memory alignment is smaller than the data type size.
/// This works by over-reading each SIMD vector by exactly 1 element worth of bytes. For example, if the hardware
/// vector size is 256 bits (32 bytes), and we want to compare a sequence of 8 byte integers, but the scan alignment
/// is 2 bytes, we need to step forward in increments of 2 bytes 4 times, causing us to over-read memory. This turns our
/// 32 byte scan into a 40 byte scan! We OR all of the sub-steps together to get the final scan result.
impl<T: SimdElement + SimdType, const N: usize> ScannerVectorComparerCascading<T, N>
where
    T: SimdElement + SimdType + PartialEq,
    LaneCount<N>: SupportedLaneCount,
    LaneCount<{ N / 2 }>: SupportedLaneCount,
    LaneCount<{ N / 4 }>: SupportedLaneCount,
    LaneCount<{ N / 8 }>: SupportedLaneCount,
    Simd<T, N>: SimdPartialEq,
{
    pub fn new(memory_alignment: MemoryAlignment) -> Self {
        Self {
            inner: ScannerVectorComparer::new(),
            memory_alignment: memory_alignment,
        }
    }

    pub fn get_immediate_compare_func(
        &self,
        scan_compare_type: ScanCompareType,
        data_type: &DataType,
    ) -> impl Fn(*const u8, *const u8) -> Simd<u8, N> {
        let base_compare_func = self
            .inner
            .get_immediate_compare_func(scan_compare_type, data_type);
        let memory_alignment = self.memory_alignment as usize;
        let data_type_size = data_type.get_size_in_bytes() as usize;
        let cascade_compare_func = move |current_values_ptr: *const u8, immediate_ptr: *const u8| {
            let mut result = base_compare_func(current_values_ptr, immediate_ptr);
            for cascade_offset in (memory_alignment..data_type_size).step_by(memory_alignment) {
                let current_values_ptr = unsafe { current_values_ptr.add(cascade_offset) };
                result |= base_compare_func(current_values_ptr, immediate_ptr);
            }
            return result;
        };

        return cascade_compare_func;
    }

    pub fn get_relative_compare_func(
        &self,
        scan_compare_type: ScanCompareType,
        data_type: &DataType,
    ) -> impl Fn(*const u8, *const u8) -> Simd<u8, N> {
        let base_compare_func = self
            .inner
            .get_relative_compare_func(scan_compare_type, data_type);
        let memory_alignment = self.memory_alignment as usize;
        let data_type_size = data_type.get_size_in_bytes() as usize;
        let cascade_compare_func = move |current_values_ptr: *const u8, previous_values_ptr: *const u8| {
            let mut result = base_compare_func(current_values_ptr, previous_values_ptr);
            for cascade_offset in (memory_alignment..data_type_size).step_by(memory_alignment) {
                let current_values_ptr = unsafe { current_values_ptr.add(cascade_offset) };
                let previous_values_ptr = unsafe { previous_values_ptr.add(cascade_offset) };
                result |= base_compare_func(current_values_ptr, previous_values_ptr);
            }
            return result;
        };

        return cascade_compare_func;
    }

    pub fn get_relative_delta_compare_func(
        &self,
        scan_compare_type: ScanCompareType,
        data_type: &DataType,
    ) -> impl Fn(*const u8, *const u8, *const u8) -> Simd<u8, N> {
        let base_compare_func = self
            .inner
            .get_relative_delta_compare_func(scan_compare_type, data_type);
        let memory_alignment = self.memory_alignment as usize;
        let data_type_size = data_type.get_size_in_bytes() as usize;
        let cascade_compare_func = move |current_values_ptr: *const u8, previous_values_ptr: *const u8, immediate_ptr: *const u8| {
            let mut result = base_compare_func(current_values_ptr, previous_values_ptr, immediate_ptr);
            for cascade_offset in (memory_alignment..data_type_size).step_by(memory_alignment) {
                let current_values_ptr = unsafe { current_values_ptr.add(cascade_offset) };
                let previous_values_ptr = unsafe { previous_values_ptr.add(cascade_offset) };
                result |= base_compare_func(current_values_ptr, previous_values_ptr, immediate_ptr);
            }
            return result;
        };

        return cascade_compare_func;
    }
}
