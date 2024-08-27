use crate::scanners::comparers::vector::encoder::scanner_vector_comparer::ScannerVectorComparer;
use crate::scanners::comparers::vector::encoder::scanner_vector_comparer::VectorComparer;
use crate::scanners::comparers::vector::types::simd_type::SimdType;
use crate::scanners::parameters::scan_compare_type::ScanCompareType;
use squalr_engine_common::values::data_type::DataType;
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
    ) -> fn(*const u8, *const u8) -> Simd<u8, N> {
        self.get_immediate_compare_func(scan_compare_type, data_type)
    }

    fn get_relative_compare_func(
        &self,
        scan_compare_type: ScanCompareType,
        data_type: &DataType,
    ) -> fn(*const u8, *const u8) -> Simd<u8, N> {
        self.get_relative_compare_func(scan_compare_type, data_type)
    }

    fn get_relative_delta_compare_func(
        &self,
        scan_compare_type: ScanCompareType,
        data_type: &DataType,
    ) -> fn(*const u8, *const u8, *const u8) -> Simd<u8, N> {
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
    pub fn new() -> Self {
        Self {
            inner: ScannerVectorComparer::new(),
        }
    }

    pub fn get_immediate_compare_func(
        &self,
        scan_compare_type: ScanCompareType,
        data_type: &DataType,
    ) -> fn(*const u8, *const u8) -> Simd<u8, N> {
        let base_compare_func = self
            .inner
            .get_immediate_compare_func(scan_compare_type, data_type);

        return base_compare_func;
    }

    pub fn get_relative_compare_func(
        &self,
        scan_compare_type: ScanCompareType,
        data_type: &DataType,
    ) -> fn(*const u8, *const u8) -> Simd<u8, N> {
        let base_compare_func = self
            .inner
            .get_relative_compare_func(scan_compare_type, data_type);

        return base_compare_func;
    }

    pub fn get_relative_delta_compare_func(
        &self,
        scan_compare_type: ScanCompareType,
        data_type: &DataType,
    ) -> fn(*const u8, *const u8, *const u8) -> Simd<u8, N> {
        let base_compare_func = self
            .inner
            .get_relative_delta_compare_func(scan_compare_type, data_type);

        return base_compare_func;
    }
}
