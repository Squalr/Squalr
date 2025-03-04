use crate::structures::data_types::comparisons::vector_comparable::{
    VectorCompareFnDelta16, VectorCompareFnDelta32, VectorCompareFnDelta64, VectorCompareFnImmediate16, VectorCompareFnImmediate32, VectorCompareFnImmediate64,
    VectorCompareFnRelative16, VectorCompareFnRelative32, VectorCompareFnRelative64,
};
use crate::structures::data_types::data_type::DataType;
use crate::structures::scanning::scan_compare_type_delta::ScanCompareTypeDelta;
use crate::structures::scanning::scan_compare_type_immediate::ScanCompareTypeImmediate;
use crate::structures::scanning::scan_compare_type_relative::ScanCompareTypeRelative;
use std::simd::{LaneCount, Simd, SupportedLaneCount};

/// A wrapper function to re-genericize vector functions on `DataType` structs for use by scanners.
/// This is necessary because all `DataType` instances need to be implemented by the traits that define them.
/// Due to Rust limitations, these traits cannot have generics, so explicit 64/32/16 byte vector functions are implemented.
/// However, our scanners are generic, so we need to "get back to" generics, and this is how we do it.
pub trait VectorCompare<const N: usize>
where
    LaneCount<N>: SupportedLaneCount,
{
    fn get_vector_compare_func_immediate(
        data_type: &Box<dyn DataType>,
        scan_compare_type_immediate: &ScanCompareTypeImmediate,
    ) -> unsafe fn(*const u8, *const u8) -> Simd<u8, N>;

    fn get_vector_compare_func_relative(
        data_type: &Box<dyn DataType>,
        scan_compare_type_immediate: &ScanCompareTypeRelative,
    ) -> unsafe fn(*const u8, *const u8) -> Simd<u8, N>;

    fn get_vector_compare_func_delta(
        data_type: &Box<dyn DataType>,
        scan_compare_type_immediate: &ScanCompareTypeDelta,
    ) -> unsafe fn(*const u8, *const u8, *const u8) -> Simd<u8, N>;
}

impl VectorCompare<64> for LaneCount<64> {
    fn get_vector_compare_func_immediate(
        data_type: &Box<dyn DataType>,
        scan_compare_type_immediate: &ScanCompareTypeImmediate,
    ) -> VectorCompareFnImmediate64 {
        VectorCompareWrapper64::get_vector_compare_func_immediate(data_type, scan_compare_type_immediate)
    }

    fn get_vector_compare_func_relative(
        data_type: &Box<dyn DataType>,
        scan_compare_type_relative: &ScanCompareTypeRelative,
    ) -> VectorCompareFnRelative64 {
        VectorCompareWrapper64::get_vector_compare_func_relative(data_type, scan_compare_type_relative)
    }

    fn get_vector_compare_func_delta(
        data_type: &Box<dyn DataType>,
        scan_compare_type_delta: &ScanCompareTypeDelta,
    ) -> VectorCompareFnDelta64 {
        VectorCompareWrapper64::get_vector_compare_func_delta(data_type, scan_compare_type_delta)
    }
}

impl VectorCompare<32> for LaneCount<32> {
    fn get_vector_compare_func_immediate(
        data_type: &Box<dyn DataType>,
        scan_compare_type_immediate: &ScanCompareTypeImmediate,
    ) -> VectorCompareFnImmediate32 {
        VectorCompareWrapper32::get_vector_compare_func_immediate(data_type, scan_compare_type_immediate)
    }

    fn get_vector_compare_func_relative(
        data_type: &Box<dyn DataType>,
        scan_compare_type_relative: &ScanCompareTypeRelative,
    ) -> VectorCompareFnRelative32 {
        VectorCompareWrapper32::get_vector_compare_func_relative(data_type, scan_compare_type_relative)
    }

    fn get_vector_compare_func_delta(
        data_type: &Box<dyn DataType>,
        scan_compare_type_delta: &ScanCompareTypeDelta,
    ) -> VectorCompareFnDelta32 {
        VectorCompareWrapper32::get_vector_compare_func_delta(data_type, scan_compare_type_delta)
    }
}

impl VectorCompare<16> for LaneCount<16> {
    fn get_vector_compare_func_immediate(
        data_type: &Box<dyn DataType>,
        scan_compare_type_immediate: &ScanCompareTypeImmediate,
    ) -> VectorCompareFnImmediate16 {
        VectorCompareWrapper16::get_vector_compare_func_immediate(data_type, scan_compare_type_immediate)
    }

    fn get_vector_compare_func_relative(
        data_type: &Box<dyn DataType>,
        scan_compare_type_relative: &ScanCompareTypeRelative,
    ) -> VectorCompareFnRelative16 {
        VectorCompareWrapper16::get_vector_compare_func_relative(data_type, scan_compare_type_relative)
    }

    fn get_vector_compare_func_delta(
        data_type: &Box<dyn DataType>,
        scan_compare_type_delta: &ScanCompareTypeDelta,
    ) -> VectorCompareFnDelta16 {
        VectorCompareWrapper16::get_vector_compare_func_delta(data_type, scan_compare_type_delta)
    }
}

trait VectorCompareWrapper<const N: usize>
where
    LaneCount<N>: SupportedLaneCount,
{
    fn get_vector_compare_func_immediate(
        data_type: &Box<dyn DataType>,
        compare_type_immediate: &ScanCompareTypeImmediate,
    ) -> unsafe fn(*const u8, *const u8) -> Simd<u8, N>;

    fn get_vector_compare_func_relative(
        data_type: &Box<dyn DataType>,
        scan_compare_type_relative: &ScanCompareTypeRelative,
    ) -> unsafe fn(*const u8, *const u8) -> Simd<u8, N>;

    fn get_vector_compare_func_delta(
        data_type: &Box<dyn DataType>,
        scan_compare_type_delta: &ScanCompareTypeDelta,
    ) -> unsafe fn(*const u8, *const u8, *const u8) -> Simd<u8, N>;
}
struct VectorCompareWrapper64 {}

impl VectorCompareWrapper<64> for VectorCompareWrapper64 {
    fn get_vector_compare_func_immediate(
        data_type: &Box<dyn DataType>,
        scan_compare_type_immediate: &ScanCompareTypeImmediate,
    ) -> VectorCompareFnImmediate64 {
        data_type.get_vector_compare_func_immediate_64(scan_compare_type_immediate)
    }

    fn get_vector_compare_func_relative(
        data_type: &Box<dyn DataType>,
        scan_compare_type_relative: &ScanCompareTypeRelative,
    ) -> VectorCompareFnRelative64 {
        data_type.get_vector_compare_func_relative_64(scan_compare_type_relative)
    }

    fn get_vector_compare_func_delta(
        data_type: &Box<dyn DataType>,
        scan_compare_type_delta: &ScanCompareTypeDelta,
    ) -> VectorCompareFnDelta64 {
        data_type.get_vector_compare_func_delta_64(scan_compare_type_delta)
    }
}

struct VectorCompareWrapper32 {}

impl VectorCompareWrapper<32> for VectorCompareWrapper32 {
    fn get_vector_compare_func_immediate(
        data_type: &Box<dyn DataType>,
        scan_compare_type_immediate: &ScanCompareTypeImmediate,
    ) -> VectorCompareFnImmediate32 {
        data_type.get_vector_compare_func_immediate_32(scan_compare_type_immediate)
    }

    fn get_vector_compare_func_relative(
        data_type: &Box<dyn DataType>,
        scan_compare_type_relative: &ScanCompareTypeRelative,
    ) -> VectorCompareFnRelative32 {
        data_type.get_vector_compare_func_relative_32(scan_compare_type_relative)
    }

    fn get_vector_compare_func_delta(
        data_type: &Box<dyn DataType>,
        scan_compare_type_delta: &ScanCompareTypeDelta,
    ) -> VectorCompareFnDelta32 {
        data_type.get_vector_compare_func_delta_32(scan_compare_type_delta)
    }
}

struct VectorCompareWrapper16 {}

impl VectorCompareWrapper<16> for VectorCompareWrapper16 {
    fn get_vector_compare_func_immediate(
        data_type: &Box<dyn DataType>,
        scan_compare_type_immediate: &ScanCompareTypeImmediate,
    ) -> VectorCompareFnImmediate16 {
        data_type.get_vector_compare_func_immediate_16(scan_compare_type_immediate)
    }

    fn get_vector_compare_func_relative(
        data_type: &Box<dyn DataType>,
        scan_compare_type_relative: &ScanCompareTypeRelative,
    ) -> VectorCompareFnRelative16 {
        data_type.get_vector_compare_func_relative_16(scan_compare_type_relative)
    }

    fn get_vector_compare_func_delta(
        data_type: &Box<dyn DataType>,
        scan_compare_type_delta: &ScanCompareTypeDelta,
    ) -> VectorCompareFnDelta16 {
        data_type.get_vector_compare_func_delta_16(scan_compare_type_delta)
    }
}
