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

    fn get_cascade_size(
        &self,
        data_type: &DataType,
    ) -> usize {
        return data_type.get_size_in_bytes() as usize / self.memory_alignment as usize;
    }

    pub fn get_immediate_compare_func(
        &self,
        scan_compare_type: ScanCompareType,
        data_type: &DataType,
    ) -> impl Fn(*const u8, *const u8) -> Simd<u8, N> {
        let base_compare_func = self
            .inner
            .get_immediate_compare_func(scan_compare_type, data_type);
        let cascade_size = self.get_cascade_size(data_type);
        let cascade_compare_func = move |ptr1: *const u8, ptr2: *const u8| {
            let mut result = base_compare_func(ptr1, ptr2);
            for cascade_index in 1..cascade_size {
                let offset_ptr1 = unsafe { ptr1.add(cascade_index) };
                let offset_ptr2 = unsafe { ptr2.add(cascade_index) };
                result |= base_compare_func(offset_ptr1, offset_ptr2);
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
        let cascade_size = self.get_cascade_size(data_type);
        let cascade_compare_func = move |ptr1: *const u8, ptr2: *const u8| {
            let mut result = base_compare_func(ptr1, ptr2);
            for cascade_index in 1..cascade_size {
                let offset_ptr1 = unsafe { ptr1.add(cascade_index) };
                let offset_ptr2 = unsafe { ptr2.add(cascade_index) };
                result |= base_compare_func(offset_ptr1, offset_ptr2);
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
        let cascade_size = self.get_cascade_size(data_type);
        let cascade_compare_func = move |ptr1: *const u8, ptr2: *const u8, ptr3: *const u8| {
            let mut result = base_compare_func(ptr1, ptr2, ptr3);
            for cascade_index in 1..cascade_size {
                let offset_ptr1 = unsafe { ptr1.add(cascade_index) };
                let offset_ptr2 = unsafe { ptr2.add(cascade_index) };
                let offset_ptr3 = unsafe { ptr3.add(cascade_index) };
                result |= base_compare_func(offset_ptr1, offset_ptr2, offset_ptr3);
            }

            return result;
        };

        return cascade_compare_func;
    }
}
