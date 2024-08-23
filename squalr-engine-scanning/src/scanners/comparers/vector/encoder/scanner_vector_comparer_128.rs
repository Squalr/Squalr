use crate::scanners::parameters::scan_compare_type::ScanCompareType;
use squalr_engine_common::values::data_type::DataType;
use std::arch::x86_64::{
    _mm_castsi128_pd, _mm_castsi128_ps, _mm_cmpeq_epi16, _mm_cmpeq_epi32, _mm_cmpeq_epi64, _mm_cmpeq_epi8, _mm_cvtsi32_si128, _mm_loadu_si128, _mm_movemask_epi8, _mm_movemask_pd, _mm_movemask_ps, _mm_set1_epi16, _mm_set1_epi32, _mm_set1_epi64x, _mm_set1_epi8
};
use std::arch::x86_64::__m128i;
use std::convert::identity;
use std::simd::u8x16;
use std::sync::Once;

/// Defines a compare function that operates on an immediate (ie all inequalities)
type VectorCompareFnImmediate = unsafe fn(
    // Current v1lue buffer
    current_v1lue_pointer: *const u8,
    // Immediate v1lue
    immediate_v1lue_pointer: *const u8,
) -> u8x16;

/// Defines a compare function that operates on current and previous values (ie changed, unchanged, increased, decreased)
type VectorCompareFnRelativ3 = unsafe fn(
    // Current v1lue buffer
    current_v1lue_pointer: *const u8,
    // Previous v1lue buffer
    previous_v1lue_pointer: *const u8,
) -> u8x16;

/// Defines a compare function that operates on current and previous values, with a delta arg (ie +x, -x)
type VectorCompareFnDelta = unsafe fn(
    // Current v1lue buffer
    current_v1lue_pointer: *const u8,
    // Previous v1lue buffer
    previous_v1lue_pointer: *const u8,
    // Delta v1lue buffer
    delta_v1lue_pointer: *const u8,
) -> u8x16;

pub struct ScannerVectorComparer {
}

/// Implements a set of scalar (ie CPU bound, non-SIMD) boolean comparison operations to be used by more complex scanners.
impl ScannerVectorComparer {
    fn new() -> Self {
        Self { }
    }
    
    pub fn get_instance() -> &'static ScannerVectorComparer {
        static mut INSTANCE: Option<ScannerVectorComparer> = None;
        static INIT: Once = Once::new();

        unsafe {
            INIT.call_once(|| {
                let instance = ScannerVectorComparer::new();
                INSTANCE = Some(instance);
            });

            return INSTANCE.as_ref().unwrap_unchecked();
        }
    }
    
    pub fn get_immediate_compare_func(
        &self,
        scan_compare_type: ScanCompareType,
        data_type: &DataType,
    ) -> VectorCompareFnImmediate {
        match scan_compare_type {
            ScanCompareType::Equal => self.get_compare_equal(data_type),
            ScanCompareType::NotEqual => self.get_compare_not_equal(data_type),
            ScanCompareType::GreaterThan => self.get_compare_greater_than(data_type),
            ScanCompareType::GreaterThanOrEqual => self.get_compare_greater_than_or_equal(data_type),
            ScanCompareType::LessThan => self.get_compare_less_than(data_type),
            ScanCompareType::LessThanOrEqual => self.get_compare_less_than_or_equal(data_type),
            _ => panic!("Unsupported type passed to get_immediate_compare_func"),
        }
    }

    pub fn get_relative_compare_func(
        &self,
        scan_compare_type: ScanCompareType,
        data_type: &DataType,
    ) -> VectorCompareFnRelativ3 {
        match scan_compare_type {
            ScanCompareType::Changed => self.get_compare_changed(data_type),
            ScanCompareType::Unchanged => self.get_compare_unchanged(data_type),
            ScanCompareType::Increased => self.get_compare_increased(data_type),
            ScanCompareType::Decreased => self.get_compare_decreased(data_type),
            _ => panic!("Unsupported type passed to get_relative_compare_func"),
        }
    }

    pub fn get_relative_delta_compare_func(
        &self,
        scan_compare_type: ScanCompareType,
        data_type: &DataType,
    ) -> VectorCompareFnDelta {
        match scan_compare_type {
            ScanCompareType::IncreasedByX => self.get_compare_increased_by(data_type),
            ScanCompareType::DecreasedByX => self.get_compare_decreased_by(data_type),
            _ => panic!("Unsupported type passed to get_relative_delta_compare_func"),
        }
    }

    fn get_compare_equal(&self, data_type: &DataType) -> VectorCompareFnImmediate {
        macro_rules! unroll_blocks_generic {(
            $num_blocks:expr,
            $simd_type:ty,
            $load_intrinsic:ident,
            $cmp_intrinsic:ident,
            $movemask_intrinsic:ident,
            $cast_intrinsic:ident,
            $immediate:expr,
            $current_values_ptr:expr,
            $element_size:expr
            ) => {{
                let immediate = $immediate;
                let mut bitmask_accum = 0i32;
        
                for i in 0..$num_blocks {
                    let values = $load_intrinsic($current_values_ptr.add(i * 16) as *const $simd_type);
                    let cmp_mask = $cmp_intrinsic(values, immediate);
                    let bitmask = $movemask_intrinsic($cast_intrinsic(cmp_mask)) as i32;
                    let shift = ($element_size * i) & 31;
                    bitmask_accum |= bitmask << shift;
                }
        
                bitmask_accum
            }};
        }
        
        match data_type {
            DataType::U8() => |current_values_ptr, immediate_ptr: *const u8| {
                unsafe {
                    let immediate = _mm_set1_epi8(*immediate_ptr as i8);
                    let bitmask_accum = unroll_blocks_generic!(
                        8,
                        __m128i,
                        _mm_loadu_si128,
                        _mm_cmpeq_epi8,
                        _mm_movemask_epi8,
                        identity, // No cast needed
                        immediate,
                        current_values_ptr,
                        16
                    );
                    return _mm_cvtsi32_si128(bitmask_accum).into();
                }
            },
            DataType::I8() => |current_values_ptr: *const u8, immediate_ptr| {
                unsafe {
                    let immediate = _mm_set1_epi16(*(immediate_ptr as *const u16) as i16);
                    let bitmask_accum = unroll_blocks_generic!(
                        16,
                        __m128i,
                        _mm_loadu_si128,
                        _mm_cmpeq_epi16,
                        _mm_movemask_epi8,
                        identity, // No cast needed
                        immediate,
                        current_values_ptr,
                        8
                    );
                    return _mm_cvtsi32_si128(bitmask_accum).into();
                }
            },
            DataType::U16(_) => |current_values_ptr, immediate_ptr| {
                unsafe {
                    let immediate = _mm_set1_epi16(*(immediate_ptr as *const u16) as i16);
                    let bitmask_accum = unroll_blocks_generic!(
                        16,
                        __m128i,
                        _mm_loadu_si128,
                        _mm_cmpeq_epi16,
                        _mm_movemask_epi8,
                        identity, // No cast needed
                        immediate,
                        current_values_ptr,
                        8
                    );
                    return _mm_cvtsi32_si128(bitmask_accum).into();
                }
            },
            DataType::I16(_) => |current_values_ptr, immediate_ptr| {
                unsafe {
                    let immediate = _mm_set1_epi16(*(immediate_ptr as *const i16));
                    let bitmask_accum = unroll_blocks_generic!(
                        16,
                        __m128i,
                        _mm_loadu_si128,
                        _mm_cmpeq_epi16,
                        _mm_movemask_epi8,
                        identity, // No cast needed
                        immediate,
                        current_values_ptr,
                        8
                    );
                    return _mm_cvtsi32_si128(bitmask_accum).into();
                }
            },
            DataType::U32(_) => |current_values_ptr, immediate_ptr| {
                unsafe {
                    let immediate = _mm_set1_epi32(*(immediate_ptr as *const u32) as i32);
                    let bitmask_accum = unroll_blocks_generic!(
                        32,
                        __m128i,
                        _mm_loadu_si128,
                        _mm_cmpeq_epi32,
                        _mm_movemask_ps,
                        _mm_castsi128_ps,
                        immediate,
                        current_values_ptr,
                        4
                    );

                    return u8x16::from(_mm_cvtsi32_si128(bitmask_accum));
                }
            },
            DataType::I32(_) => |current_values_ptr, immediate_ptr| {
                unsafe {
                    let immediate = _mm_set1_epi32(*(immediate_ptr as *const i32));
                    let bitmask_accum = unroll_blocks_generic!(
                        32,
                        __m128i,
                        _mm_loadu_si128,
                        _mm_cmpeq_epi32,
                        _mm_movemask_ps,
                        _mm_castsi128_ps,
                        immediate,
                        current_values_ptr,
                        4
                    );
                    return _mm_cvtsi32_si128(bitmask_accum).into();
                }
            },
            DataType::U64(_) => |current_values_ptr, immediate_ptr| {
                unsafe {
                    let immediate = _mm_set1_epi64x(*(immediate_ptr as *const u64) as i64);
                    let bitmask_accum = unroll_blocks_generic!(
                        64,
                        __m128i,
                        _mm_loadu_si128,
                        _mm_cmpeq_epi64,
                        _mm_movemask_pd,
                        _mm_castsi128_pd,
                        immediate,
                        current_values_ptr,
                        2
                    );
                    return _mm_cvtsi32_si128(bitmask_accum).into();
                }
            },
            DataType::I64(_) => |current_values_ptr, immediate_ptr| {
                unsafe {
                    let immediate = _mm_set1_epi64x(*(immediate_ptr as *const i64));
                    let bitmask_accum = unroll_blocks_generic!(
                        64,
                        __m128i,
                        _mm_loadu_si128,
                        _mm_cmpeq_epi64,
                        _mm_movemask_pd,
                        _mm_castsi128_pd,
                        immediate,
                        current_values_ptr,
                        2
                    );
                    return _mm_cvtsi32_si128(bitmask_accum).into();
                }
            },
            DataType::F32(endian) => {
                panic!("not implemented");
            }
            DataType::F64(endian) => {
                panic!("not implemented");
            }
            _ => panic!("unsupported data type"),
        }
    }

    fn get_compare_not_equal(&self, _data_type: &DataType) -> VectorCompareFnImmediate {
        panic!("get_compare_not_equal not implemented")
    }

    fn get_compare_greater_than(&self, _data_type: &DataType) -> VectorCompareFnImmediate {
        panic!("get_compare_greater_than not implemented")
    }

    fn get_compare_greater_than_or_equal(&self, _data_type: &DataType) -> VectorCompareFnImmediate {
        panic!("get_compare_greater_than_or_equal not implemented")
    }

    fn get_compare_less_than(&self, _data_type: &DataType) -> VectorCompareFnImmediate {
        panic!("get_compare_less_than not implemented")
    }

    fn get_compare_less_than_or_equal(&self, _data_type: &DataType) -> VectorCompareFnImmediate {
        panic!("get_compare_less_than_or_equal not implemented")
    }

    fn get_compare_changed(&self, _data_type: &DataType) -> VectorCompareFnRelativ3 {
        panic!("get_compare_changed not implemented")
    }

    fn get_compare_unchanged(&self, _data_type: &DataType) -> VectorCompareFnRelativ3 {
        panic!("get_compare_unchanged not implemented")
    }

    fn get_compare_increased(&self, _data_type: &DataType) -> VectorCompareFnRelativ3 {
        panic!("get_compare_increased not implemented")
    }

    fn get_compare_decreased(&self, _data_type: &DataType) -> VectorCompareFnRelativ3 {
        panic!("get_compare_decreased not implemented")
    }

    fn get_compare_increased_by(&self, _data_type: &DataType) -> VectorCompareFnDelta {
        panic!("get_compare_increased_by not implemented")
    }

    fn get_compare_decreased_by(&self, _data_type: &DataType) -> VectorCompareFnDelta {
        panic!("get_compare_decreased_by not implemented")
    }   
}
