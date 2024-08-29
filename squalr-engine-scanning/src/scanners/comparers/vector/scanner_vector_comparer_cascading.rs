use crate::scanners::comparers::vector::scanner_vector_comparer::ScannerVectorComparer;
use crate::scanners::comparers::vector::scanner_vector_comparer::VectorComparer;
use crate::scanners::encoders::vector::types::simd_type::SimdType;
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

/// Cascading scans are the single most complex case to handle due to the base addresses not being aligned.
/// It turns out that this problem has been extensively researched under "string search algorithms".
///
/// However, we want to avoid falling back onto a generic search function if we can avoid it. We can pre-analyze the
/// scan data to use more efficient implementations when possible.
///
/// There may be a ton of sub-cases, and this may best be handled by reducing the problem to a several specialized cases.
///
/// A) Periodicity Scans with RLE Discard. This is an algorithm that is optmized for periodic data with repeating 1-4 byte patterns.
///     For 1-periodic scans (all same byte A)
///         Just do a normal SIMD byte scan, and discard all RLEs < data type size
///     For 2-periodic scans (repeating 2 bytes A, B)
///         Create a vector of <A,B,A,B,...> and <B,A,B,A,..>
///         Do 2-byte SIMD comparisons, and OR the results together.
///         Note that the shifted pattern could result in matching up to 2 unwanted bytes at the start/end of the RLE encoding.
///         In the RLE encoder, the first/last bytes need to be manually checked to filter these. Discard RLEs < data size.
///     For 4-periodic scans (repeating 4 bytes A, B, C, D)
///         Create a vector of <A,B,C,D,A,B,C,D,...> <B,C,D,A,B,C,D,A,..> <C,D,A,B,C,D,A,B,..> <D,A,B,C,D,A,B,C,..>
///         As before, we do 4-byte SIMD comparisons. From here we can generalize the RLE trimming.
///         We can use the first byte + size of run length to determine how much we need to trim.
///     For 8-periodic, extrapolate.
///
/// It is very important to realize that even if the user is scanning for a large data type (ie 8 bytes), it can still fall into
/// 1, 2, or 4 periodic! This will give us substantial gains over immediately going for the 8-periodic implementation.
///
/// Similarly, the same is true for byte array scans! If the array of bytes can be decomposed into periodic sequences, periodicty
/// scans will results in substantial savings, given that the array fits into a hardware vector Simd<> type.
///     
/// B) AoB scans via Boyer-Moore or
///
///
///
///
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
