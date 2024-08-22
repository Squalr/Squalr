use crate::scanners::parameters::scan_compare_type::ScanCompareType;
use squalr_engine_common::values::{data_type::DataType, endian::Endian};
use std::sync::Once;

/// Defines a compare function that operates on an immediate (ie all inequalities)
type ScalarCompareFnImmediate = unsafe fn(
    // Current value pointer
    *const u8,
    // Immediate value pointer
    *const u8,
) -> bool;

/// Defines a compare function that operates on current and previous values (ie changed, unchanged, increased, decreased)
type ScalarCompareFnRelative = unsafe fn(
    // Current value pointer
    *const u8,
    // Previous value pointer
    *const u8,
) -> bool;

/// Defines a compare function that operates on current and previous values, with a delta arg (ie +x, -x)
type ScalarCompareFnDelta = unsafe fn(
    // Current value pointer
    *const u8,
    // Previous value pointer
    *const u8,
    // Delta value pointer
    *const u8,
) -> bool;

pub struct ScannerScalarComparer {
    target_is_little_endian: bool,
}

/// Implements a set of scalar (ie CPU bound, non-SIMD) boolean comparison operations to be used by more complex scanners.
impl ScannerScalarComparer {
    fn new() -> Self {
        Self {
            target_is_little_endian: cfg!(target_endian = "little"),
        }
    }

    pub fn get_instance() -> &'static ScannerScalarComparer {
        static mut INSTANCE: Option<ScannerScalarComparer> = None;
        static INIT: Once = Once::new();

        unsafe {
            INIT.call_once(|| {
                let instance = ScannerScalarComparer::new();
                INSTANCE = Some(instance);
            });

            return INSTANCE.as_ref().unwrap_unchecked();
        }
    }

    pub fn get_immediate_compare_func(
        &self,
        scan_compare_type: ScanCompareType,
        data_type: &DataType,
    ) -> ScalarCompareFnImmediate {
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
    ) -> ScalarCompareFnRelative {
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
    ) -> ScalarCompareFnDelta {
        match scan_compare_type {
            ScanCompareType::IncreasedByX => self.get_compare_increased_by(data_type),
            ScanCompareType::DecreasedByX => self.get_compare_decreased_by(data_type),
            _ => panic!("Unsupported type passed to get_relative_delta_compare_func"),
        }
    }

    fn check_endian(&self, endian: &Endian) -> bool {
        (self.target_is_little_endian && *endian == Endian::Little)
            || (!self.target_is_little_endian && *endian == Endian::Big)
    }
    fn get_compare_equal(&self, data_type: &DataType) -> ScalarCompareFnImmediate {
        match data_type {
            // Note that as an optimization, endian resolving is not required for integer equality comparisons.
            DataType::U8() => |a, b| unsafe { std::ptr::read_unaligned(a as *const u8) == std::ptr::read_unaligned(b as *const u8) },
            DataType::I8() => |a, b| unsafe { std::ptr::read_unaligned(a as *const i8) == std::ptr::read_unaligned(b as *const i8) },
            DataType::U16(_) => |a, b| unsafe { std::ptr::read_unaligned(a as *const u16) == std::ptr::read_unaligned(b as *const u16) },
            DataType::I16(_) => |a, b| unsafe { std::ptr::read_unaligned(a as *const i16) == std::ptr::read_unaligned(b as *const i16) },
            DataType::U32(_) => |a, b| unsafe { std::ptr::read_unaligned(a as *const u32) == std::ptr::read_unaligned(b as *const u32) },
            DataType::I32(_) => |a, b| unsafe { std::ptr::read_unaligned(a as *const i32) == std::ptr::read_unaligned(b as *const i32) },
            DataType::U64(_) => |a, b| unsafe { std::ptr::read_unaligned(a as *const u64) == std::ptr::read_unaligned(b as *const u64) },
            DataType::I64(_) => |a, b| unsafe { std::ptr::read_unaligned(a as *const i64) == std::ptr::read_unaligned(b as *const i64) },
            // TODO: FLOATING POINT TOLERANCE SUPPORT
            DataType::F32(endian) => {
                if self.check_endian(endian) {
                    |a, b| unsafe { std::ptr::read_unaligned(a as *const f32) == std::ptr::read_unaligned(b as *const f32) }
                } else {
                    |a, b| unsafe {
                        let a_val = std::ptr::read_unaligned(a as *const u32).swap_bytes();
                        let b_val = std::ptr::read_unaligned(b as *const u32).swap_bytes();
                        *(a_val as *const f32) == *(b_val as *const f32)
                    }
                }
            }
            // TODO: FLOATING POINT TOLERANCE SUPPORT
            DataType::F64(endian) => {
                if self.check_endian(endian) {
                    |a, b| unsafe { std::ptr::read_unaligned(a as *const f64) == std::ptr::read_unaligned(b as *const f64) }
                } else {
                    |a, b| unsafe {
                        let a_val = std::ptr::read_unaligned(a as *const u64).swap_bytes();
                        let b_val = std::ptr::read_unaligned(b as *const u64).swap_bytes();
                        *(a_val as *const f64) == *(b_val as *const f64)
                    }
                }
            }
            _ => panic!("unsupported data type"),
        }
    }

    fn get_compare_not_equal(&self, data_type: &DataType) -> ScalarCompareFnImmediate {
        // Note that as an optimization, endian resolving is not required for integer inequality comparisons.
        match data_type {
            DataType::U8() => |a, b| unsafe { std::ptr::read_unaligned(a as *const u8) != std::ptr::read_unaligned(b as *const u8) },
            DataType::I8() => |a, b| unsafe { std::ptr::read_unaligned(a as *const i8) != std::ptr::read_unaligned(b as *const i8) },
            DataType::U16(_) => |a, b| unsafe { std::ptr::read_unaligned(a as *const u16) != std::ptr::read_unaligned(b as *const u16) },
            DataType::I16(_) => |a, b| unsafe { std::ptr::read_unaligned(a as *const i16) != std::ptr::read_unaligned(b as *const i16) },
            DataType::U32(_) => |a, b| unsafe { std::ptr::read_unaligned(a as *const u32) != std::ptr::read_unaligned(b as *const u32) },
            DataType::I32(_) => |a, b| unsafe { std::ptr::read_unaligned(a as *const i32) != std::ptr::read_unaligned(b as *const i32) },
            DataType::U64(_) => |a, b| unsafe { std::ptr::read_unaligned(a as *const u64) != std::ptr::read_unaligned(b as *const u64) },
            DataType::I64(_) => |a, b| unsafe { std::ptr::read_unaligned(a as *const i64) != std::ptr::read_unaligned(b as *const i64) },
            // TODO: FLOATING POINT TOLERANCE SUPPORT
            DataType::F32(endian) => {
                if self.check_endian(endian) {
                    |a, b| unsafe { std::ptr::read_unaligned(a as *const f32) != std::ptr::read_unaligned(b as *const f32) }
                } else {
                    |a, b| unsafe {
                        let a_val = std::ptr::read_unaligned(a as *const u32).swap_bytes();
                        let b_val = std::ptr::read_unaligned(b as *const u32).swap_bytes();
                        *(a_val as *const f32) != *(b_val as *const f32)
                    }
                }
            }
            // TODO: FLOATING POINT TOLERANCE SUPPORT
            DataType::F64(endian) => {
                if self.check_endian(endian) {
                    |a, b| unsafe { std::ptr::read_unaligned(a as *const f64) != std::ptr::read_unaligned(b as *const f64) }
                } else {
                    |a, b| unsafe {
                        let a_val = std::ptr::read_unaligned(a as *const u64).swap_bytes();
                        let b_val = std::ptr::read_unaligned(b as *const u64).swap_bytes();
                        *(a_val as *const f64) != *(b_val as *const f64)
                    }
                }
            }
            _ => panic!("unsupported data type"),
        }
    }

    fn get_compare_greater_than(&self, data_type: &DataType) -> ScalarCompareFnImmediate {
        match data_type {
            DataType::U8() => |a, b| unsafe { *(a as *const u8) > *(b as *const u8) },
            DataType::I8() => |a, b| unsafe { *(a as *const i8) > *(b as *const i8) },
            DataType::U16(endian) => {
                if self.check_endian(endian) {
                    |a, b| unsafe { *(a as *const u16) > *(b as *const u16) }
                } else {
                    |a, b| unsafe { (*(a as *const u16)).swap_bytes() > (*(b as *const u16)).swap_bytes() }
                }
            }
            DataType::I16(endian) => {
                if self.check_endian(endian) {
                    |a, b| unsafe { *(a as *const i16) > *(b as *const i16) }
                } else {
                    |a, b| unsafe { (*(a as *const i16)).swap_bytes() > (*(b as *const i16)).swap_bytes() }
                }
            }
            DataType::U32(endian) => {
                if self.check_endian(endian) {
                    |a, b| unsafe { *(a as *const u32) > *(b as *const u32) }
                } else {
                    |a, b| unsafe { (*(a as *const u32)).swap_bytes() > (*(b as *const u32)).swap_bytes() }
                }
            }
            DataType::I32(endian) => {
                if self.check_endian(endian) {
                    |a, b| unsafe { *(a as *const i32) > *(b as *const i32) }
                } else {
                    |a, b| unsafe { (*(a as *const i32)).swap_bytes() > (*(b as *const i32)).swap_bytes() }
                }
            }
            DataType::U64(endian) => {
                if self.check_endian(endian) {
                    |a, b| unsafe { *(a as *const u64) > *(b as *const u64) }
                } else {
                    |a, b| unsafe { (*(a as *const u64)).swap_bytes() > (*(b as *const u64)).swap_bytes() }
                }
            }
            DataType::I64(endian) => {
                if self.check_endian(endian) {
                    |a, b| unsafe { *(a as *const i64) > *(b as *const i64) }
                } else {
                    |a, b| unsafe { (*(a as *const i64)).swap_bytes() > (*(b as *const i64)).swap_bytes() }
                }
            }
            DataType::F32(endian) => {
                if self.check_endian(endian) {
                    |a, b| unsafe { *(a as *const f32) > *(b as *const f32) }
                } else {
                    |a, b| unsafe {
                        let a_val = (*(a as *const u32)).swap_bytes();
                        let b_val = (*(b as *const u32)).swap_bytes();
                        *(a_val as *const f32) > *(b_val as *const f32)
                    }
                }
            }
            DataType::F64(endian) => {
                if self.check_endian(endian) {
                    |a, b| unsafe { *(a as *const f64) > *(b as *const f64) }
                } else {
                    |a, b| unsafe {
                        let a_val = (*(a as *const u64)).swap_bytes();
                        let b_val = (*(b as *const u64)).swap_bytes();
                        *(a_val as *const f64) > *(b_val as *const f64)
                    }
                }
            }
            _ => panic!("unsupported data type"),
        }
    }    
    
    fn get_compare_greater_than_or_equal(&self, data_type: &DataType) -> ScalarCompareFnImmediate {
        match data_type {
            DataType::U8() => |a, b| unsafe { std::ptr::read_unaligned(a as *const u8) >= std::ptr::read_unaligned(b as *const u8) },
            DataType::I8() => |a, b| unsafe { std::ptr::read_unaligned(a as *const i8) >= std::ptr::read_unaligned(b as *const i8) },
            DataType::U16(endian) => {
                if self.check_endian(endian) {
                    |a, b| unsafe { std::ptr::read_unaligned(a as *const u16) >= std::ptr::read_unaligned(b as *const u16) }
                } else {
                    |a, b| unsafe { std::ptr::read_unaligned(a as *const u16).swap_bytes() >= std::ptr::read_unaligned(b as *const u16).swap_bytes() }
                }
            }
            DataType::I16(endian) => {
                if self.check_endian(endian) {
                    |a, b| unsafe { std::ptr::read_unaligned(a as *const i16) >= std::ptr::read_unaligned(b as *const i16) }
                } else {
                    |a, b| unsafe { std::ptr::read_unaligned(a as *const i16).swap_bytes() >= std::ptr::read_unaligned(b as *const i16).swap_bytes() }
                }
            }
            DataType::U32(endian) => {
                if self.check_endian(endian) {
                    |a, b| unsafe { std::ptr::read_unaligned(a as *const u32) >= std::ptr::read_unaligned(b as *const u32) }
                } else {
                    |a, b| unsafe { std::ptr::read_unaligned(a as *const u32).swap_bytes() >= std::ptr::read_unaligned(b as *const u32).swap_bytes() }
                }
            }
            DataType::I32(endian) => {
                if self.check_endian(endian) {
                    |a, b| unsafe { std::ptr::read_unaligned(a as *const i32) >= std::ptr::read_unaligned(b as *const i32) }
                } else {
                    |a, b| unsafe { std::ptr::read_unaligned(a as *const i32).swap_bytes() >= std::ptr::read_unaligned(b as *const i32).swap_bytes() }
                }
            }
            DataType::U64(endian) => {
                if self.check_endian(endian) {
                    |a, b| unsafe { std::ptr::read_unaligned(a as *const u64) >= std::ptr::read_unaligned(b as *const u64) }
                } else {
                    |a, b| unsafe { std::ptr::read_unaligned(a as *const u64).swap_bytes() >= std::ptr::read_unaligned(b as *const u64).swap_bytes() }
                }
            }
            DataType::I64(endian) => {
                if self.check_endian(endian) {
                    |a, b| unsafe { std::ptr::read_unaligned(a as *const i64) >= std::ptr::read_unaligned(b as *const i64) }
                } else {
                    |a, b| unsafe { std::ptr::read_unaligned(a as *const i64).swap_bytes() >= std::ptr::read_unaligned(b as *const i64).swap_bytes() }
                }
            }
            DataType::F32(endian) => {
                if self.check_endian(endian) {
                    |a, b| unsafe { std::ptr::read_unaligned(a as *const f32) >= std::ptr::read_unaligned(b as *const f32) }
                } else {
                    |a, b| unsafe {
                        let a_val = std::ptr::read_unaligned(a as *const u32).swap_bytes();
                        let b_val = std::ptr::read_unaligned(b as *const u32).swap_bytes();
                        *(a_val as *const f32) >= *(b_val as *const f32)
                    }
                }
            }
            DataType::F64(endian) => {
                if self.check_endian(endian) {
                    |a, b| unsafe { std::ptr::read_unaligned(a as *const f64) >= std::ptr::read_unaligned(b as *const f64) }
                } else {
                    |a, b| unsafe {
                        let a_val = std::ptr::read_unaligned(a as *const u64).swap_bytes();
                        let b_val = std::ptr::read_unaligned(b as *const u64).swap_bytes();
                        *(a_val as *const f64) >= *(b_val as *const f64)
                    }
                }
            }
            _ => panic!("unsupported data type"),
        }
    }
    
    fn get_compare_less_than(&self, data_type: &DataType) -> ScalarCompareFnImmediate {
        match data_type {
            DataType::U8() => |a, b| unsafe { std::ptr::read_unaligned(a as *const u8) < std::ptr::read_unaligned(b as *const u8) },
            DataType::I8() => |a, b| unsafe { std::ptr::read_unaligned(a as *const i8) < std::ptr::read_unaligned(b as *const i8) },
            DataType::U16(endian) => {
                if self.check_endian(endian) {
                    |a, b| unsafe { std::ptr::read_unaligned(a as *const u16) < std::ptr::read_unaligned(b as *const u16) }
                } else {
                    |a, b| unsafe { std::ptr::read_unaligned(a as *const u16).swap_bytes() < std::ptr::read_unaligned(b as *const u16).swap_bytes() }
                }
            }
            DataType::I16(endian) => {
                if self.check_endian(endian) {
                    |a, b| unsafe { std::ptr::read_unaligned(a as *const i16) < std::ptr::read_unaligned(b as *const i16) }
                } else {
                    |a, b| unsafe { std::ptr::read_unaligned(a as *const i16).swap_bytes() < std::ptr::read_unaligned(b as *const i16).swap_bytes() }
                }
            }
            DataType::U32(endian) => {
                if self.check_endian(endian) {
                    |a, b| unsafe { std::ptr::read_unaligned(a as *const u32) < std::ptr::read_unaligned(b as *const u32) }
                } else {
                    |a, b| unsafe { std::ptr::read_unaligned(a as *const u32).swap_bytes() < std::ptr::read_unaligned(b as *const u32).swap_bytes() }
                }
            }
            DataType::I32(endian) => {
                if self.check_endian(endian) {
                    |a, b| unsafe { std::ptr::read_unaligned(a as *const i32) < std::ptr::read_unaligned(b as *const i32) }
                } else {
                    |a, b| unsafe { std::ptr::read_unaligned(a as *const i32).swap_bytes() < std::ptr::read_unaligned(b as *const i32).swap_bytes() }
                }
            }
            DataType::U64(endian) => {
                if self.check_endian(endian) {
                    |a, b| unsafe { std::ptr::read_unaligned(a as *const u64) < std::ptr::read_unaligned(b as *const u64) }
                } else {
                    |a, b| unsafe { std::ptr::read_unaligned(a as *const u64).swap_bytes() < std::ptr::read_unaligned(b as *const u64).swap_bytes() }
                }
            }
            DataType::I64(endian) => {
                if self.check_endian(endian) {
                    |a, b| unsafe { std::ptr::read_unaligned(a as *const i64) < std::ptr::read_unaligned(b as *const i64) }
                } else {
                    |a, b| unsafe { std::ptr::read_unaligned(a as *const i64).swap_bytes() < std::ptr::read_unaligned(b as *const i64).swap_bytes() }
                }
            }
            DataType::F32(endian) => {
                if self.check_endian(endian) {
                    |a, b| unsafe { std::ptr::read_unaligned(a as *const f32) < std::ptr::read_unaligned(b as *const f32) }
                } else {
                    |a, b| unsafe {
                        let a_val = std::ptr::read_unaligned(a as *const u32).swap_bytes();
                        let b_val = std::ptr::read_unaligned(b as *const u32).swap_bytes();
                        *(a_val as *const f32) < *(b_val as *const f32)
                    }
                }
            }
            DataType::F64(endian) => {
                if self.check_endian(endian) {
                    |a, b| unsafe { std::ptr::read_unaligned(a as *const f64) < std::ptr::read_unaligned(b as *const f64) }
                } else {
                    |a, b| unsafe {
                        let a_val = std::ptr::read_unaligned(a as *const u64).swap_bytes();
                        let b_val = std::ptr::read_unaligned(b as *const u64).swap_bytes();
                        *(a_val as *const f64) < *(b_val as *const f64)
                    }
                }
            }
            _ => panic!("unsupported data type"),
        }
    }
    
    fn get_compare_less_than_or_equal(&self, data_type: &DataType) -> ScalarCompareFnImmediate {
        match data_type {
            DataType::U8() => |a, b| unsafe { std::ptr::read_unaligned(a as *const u8) <= std::ptr::read_unaligned(b as *const u8) },
            DataType::I8() => |a, b| unsafe { std::ptr::read_unaligned(a as *const i8) <= std::ptr::read_unaligned(b as *const i8) },
            DataType::U16(endian) => {
                if self.check_endian(endian) {
                    |a, b| unsafe { std::ptr::read_unaligned(a as *const u16) <= std::ptr::read_unaligned(b as *const u16) }
                } else {
                    |a, b| unsafe { std::ptr::read_unaligned(a as *const u16).swap_bytes() <= std::ptr::read_unaligned(b as *const u16).swap_bytes() }
                }
            }
            DataType::I16(endian) => {
                if self.check_endian(endian) {
                    |a, b| unsafe { std::ptr::read_unaligned(a as *const i16) <= std::ptr::read_unaligned(b as *const i16) }
                } else {
                    |a, b| unsafe { std::ptr::read_unaligned(a as *const i16).swap_bytes() <= std::ptr::read_unaligned(b as *const i16).swap_bytes() }
                }
            }
            DataType::U32(endian) => {
                if self.check_endian(endian) {
                    |a, b| unsafe { std::ptr::read_unaligned(a as *const u32) <= std::ptr::read_unaligned(b as *const u32) }
                } else {
                    |a, b| unsafe { std::ptr::read_unaligned(a as *const u32).swap_bytes() <= std::ptr::read_unaligned(b as *const u32).swap_bytes() }
                }
            }
            DataType::I32(endian) => {
                if self.check_endian(endian) {
                    |a, b| unsafe { std::ptr::read_unaligned(a as *const i32) <= std::ptr::read_unaligned(b as *const i32) }
                } else {
                    |a, b| unsafe { std::ptr::read_unaligned(a as *const i32).swap_bytes() <= std::ptr::read_unaligned(b as *const i32).swap_bytes() }
                }
            }
            DataType::U64(endian) => {
                if self.check_endian(endian) {
                    |a, b| unsafe { std::ptr::read_unaligned(a as *const u64) <= std::ptr::read_unaligned(b as *const u64) }
                } else {
                    |a, b| unsafe { std::ptr::read_unaligned(a as *const u64).swap_bytes() <= std::ptr::read_unaligned(b as *const u64).swap_bytes() }
                }
            }
            DataType::I64(endian) => {
                if self.check_endian(endian) {
                    |a, b| unsafe { std::ptr::read_unaligned(a as *const i64) <= std::ptr::read_unaligned(b as *const i64) }
                } else {
                    |a, b| unsafe { std::ptr::read_unaligned(a as *const i64).swap_bytes() <= std::ptr::read_unaligned(b as *const i64).swap_bytes() }
                }
            }
            DataType::F32(endian) => {
                if self.check_endian(endian) {
                    |a, b| unsafe { std::ptr::read_unaligned(a as *const f32) <= std::ptr::read_unaligned(b as *const f32) }
                } else {
                    |a, b| unsafe {
                        let a_val = std::ptr::read_unaligned(a as *const u32).swap_bytes();
                        let b_val = std::ptr::read_unaligned(b as *const u32).swap_bytes();
                        *(a_val as *const f32) <= *(b_val as *const f32)
                    }
                }
            }
            DataType::F64(endian) => {
                if self.check_endian(endian) {
                    |a, b| unsafe { std::ptr::read_unaligned(a as *const f64) <= std::ptr::read_unaligned(b as *const f64) }
                } else {
                    |a, b| unsafe {
                        let a_val = std::ptr::read_unaligned(a as *const u64).swap_bytes();
                        let b_val = std::ptr::read_unaligned(b as *const u64).swap_bytes();
                        *(a_val as *const f64) <= *(b_val as *const f64)
                    }
                }
            }
            _ => panic!("unsupported data type"),
        }
    }
    
    fn get_compare_changed(&self, data_type: &DataType) -> ScalarCompareFnRelative {
        match data_type {
            DataType::U8() => |a, b| unsafe { std::ptr::read_unaligned(a as *const u8) != std::ptr::read_unaligned(b as *const u8) },
            DataType::I8() => |a, b| unsafe { std::ptr::read_unaligned(a as *const i8) != std::ptr::read_unaligned(b as *const i8) },
            DataType::U16(endian) => {
                if self.check_endian(endian) {
                    |a, b| unsafe { std::ptr::read_unaligned(a as *const u16) != std::ptr::read_unaligned(b as *const u16) }
                } else {
                    |a, b| unsafe { std::ptr::read_unaligned(a as *const u16).swap_bytes() != std::ptr::read_unaligned(b as *const u16).swap_bytes() }
                }
            }
            DataType::I16(endian) => {
                if self.check_endian(endian) {
                    |a, b| unsafe { std::ptr::read_unaligned(a as *const i16) != std::ptr::read_unaligned(b as *const i16) }
                } else {
                    |a, b| unsafe { std::ptr::read_unaligned(a as *const i16).swap_bytes() != std::ptr::read_unaligned(b as *const i16).swap_bytes() }
                }
            }
            DataType::U32(endian) => {
                if self.check_endian(endian) {
                    |a, b| unsafe { std::ptr::read_unaligned(a as *const u32) != std::ptr::read_unaligned(b as *const u32) }
                } else {
                    |a, b| unsafe { std::ptr::read_unaligned(a as *const u32).swap_bytes() != std::ptr::read_unaligned(b as *const u32).swap_bytes() }
                }
            }
            DataType::I32(endian) => {
                if self.check_endian(endian) {
                    |a, b| unsafe { std::ptr::read_unaligned(a as *const i32) != std::ptr::read_unaligned(b as *const i32) }
                } else {
                    |a, b| unsafe { std::ptr::read_unaligned(a as *const i32).swap_bytes() != std::ptr::read_unaligned(b as *const i32).swap_bytes() }
                }
            }
            DataType::U64(endian) => {
                if self.check_endian(endian) {
                    |a, b| unsafe { std::ptr::read_unaligned(a as *const u64) != std::ptr::read_unaligned(b as *const u64) }
                } else {
                    |a, b| unsafe { std::ptr::read_unaligned(a as *const u64).swap_bytes() != std::ptr::read_unaligned(b as *const u64).swap_bytes() }
                }
            }
            DataType::I64(endian) => {
                if self.check_endian(endian) {
                    |a, b| unsafe { std::ptr::read_unaligned(a as *const i64) != std::ptr::read_unaligned(b as *const i64) }
                } else {
                    |a, b| unsafe { std::ptr::read_unaligned(a as *const i64).swap_bytes() != std::ptr::read_unaligned(b as *const i64).swap_bytes() }
                }
            }
            DataType::F32(endian) => {
                if self.check_endian(endian) {
                    |a, b| unsafe { std::ptr::read_unaligned(a as *const f32) != std::ptr::read_unaligned(b as *const f32) }
                } else {
                    |a, b| unsafe {
                        let a_val = std::ptr::read_unaligned(a as *const u32).swap_bytes();
                        let b_val = std::ptr::read_unaligned(b as *const u32).swap_bytes();
                        *(a_val as *const f32) != *(b_val as *const f32)
                    }
                }
            }
            DataType::F64(endian) => {
                if self.check_endian(endian) {
                    |a, b| unsafe { std::ptr::read_unaligned(a as *const f64) != std::ptr::read_unaligned(b as *const f64) }
                } else {
                    |a, b| unsafe {
                        let a_val = std::ptr::read_unaligned(a as *const u64).swap_bytes();
                        let b_val = std::ptr::read_unaligned(b as *const u64).swap_bytes();
                        *(a_val as *const f64) != *(b_val as *const f64)
                    }
                }
            }
            _ => panic!("unsupported data type"),
        }
    }
    
    fn get_compare_unchanged(&self, data_type: &DataType) -> ScalarCompareFnRelative {
        match data_type {
            DataType::U8() => |a, b| unsafe { std::ptr::read_unaligned(a as *const u8) == std::ptr::read_unaligned(b as *const u8) },
            DataType::I8() => |a, b| unsafe { std::ptr::read_unaligned(a as *const i8) == std::ptr::read_unaligned(b as *const i8) },
            DataType::U16(endian) => {
                if self.check_endian(endian) {
                    |a, b| unsafe { std::ptr::read_unaligned(a as *const u16) == std::ptr::read_unaligned(b as *const u16) }
                } else {
                    |a, b| unsafe { std::ptr::read_unaligned(a as *const u16).swap_bytes() == std::ptr::read_unaligned(b as *const u16).swap_bytes() }
                }
            }
            DataType::I16(endian) => {
                if self.check_endian(endian) {
                    |a, b| unsafe { std::ptr::read_unaligned(a as *const i16) == std::ptr::read_unaligned(b as *const i16) }
                } else {
                    |a, b| unsafe { std::ptr::read_unaligned(a as *const i16).swap_bytes() == std::ptr::read_unaligned(b as *const i16).swap_bytes() }
                }
            }
            DataType::U32(endian) => {
                if self.check_endian(endian) {
                    |a, b| unsafe { std::ptr::read_unaligned(a as *const u32) == std::ptr::read_unaligned(b as *const u32) }
                } else {
                    |a, b| unsafe { std::ptr::read_unaligned(a as *const u32).swap_bytes() == std::ptr::read_unaligned(b as *const u32).swap_bytes() }
                }
            }
            DataType::I32(endian) => {
                if self.check_endian(endian) {
                    |a, b| unsafe { std::ptr::read_unaligned(a as *const i32) == std::ptr::read_unaligned(b as *const i32) }
                } else {
                    |a, b| unsafe { std::ptr::read_unaligned(a as *const i32).swap_bytes() == std::ptr::read_unaligned(b as *const i32).swap_bytes() }
                }
            }
            DataType::U64(endian) => {
                if self.check_endian(endian) {
                    |a, b| unsafe { std::ptr::read_unaligned(a as *const u64) == std::ptr::read_unaligned(b as *const u64) }
                } else {
                    |a, b| unsafe { std::ptr::read_unaligned(a as *const u64).swap_bytes() == std::ptr::read_unaligned(b as *const u64).swap_bytes() }
                }
            }
            DataType::I64(endian) => {
                if self.check_endian(endian) {
                    |a, b| unsafe { std::ptr::read_unaligned(a as *const i64) == std::ptr::read_unaligned(b as *const i64) }
                } else {
                    |a, b| unsafe { std::ptr::read_unaligned(a as *const i64).swap_bytes() == std::ptr::read_unaligned(b as *const i64).swap_bytes() }
                }
            }
            DataType::F32(endian) => {
                if self.check_endian(endian) {
                    |a, b| unsafe { std::ptr::read_unaligned(a as *const f32) == std::ptr::read_unaligned(b as *const f32) }
                } else {
                    |a, b| unsafe {
                        let a_val = std::ptr::read_unaligned(a as *const u32).swap_bytes();
                        let b_val = std::ptr::read_unaligned(b as *const u32).swap_bytes();
                        *(a_val as *const f32) == *(b_val as *const f32)
                    }
                }
            }
            DataType::F64(endian) => {
                if self.check_endian(endian) {
                    |a, b| unsafe { std::ptr::read_unaligned(a as *const f64) == std::ptr::read_unaligned(b as *const f64) }
                } else {
                    |a, b| unsafe {
                        let a_val = std::ptr::read_unaligned(a as *const u64).swap_bytes();
                        let b_val = std::ptr::read_unaligned(b as *const u64).swap_bytes();
                        *(a_val as *const f64) == *(b_val as *const f64)
                    }
                }
            }
            _ => panic!("unsupported data type"),
        }
    }
    
    fn get_compare_increased(&self, data_type: &DataType) -> ScalarCompareFnRelative {
        match data_type {
            DataType::U8() => |a, b| unsafe { std::ptr::read_unaligned(a as *const u8) > std::ptr::read_unaligned(b as *const u8) },
            DataType::I8() => |a, b| unsafe { std::ptr::read_unaligned(a as *const i8) > std::ptr::read_unaligned(b as *const i8) },
            DataType::U16(endian) => {
                if self.check_endian(endian) {
                    |a, b| unsafe { std::ptr::read_unaligned(a as *const u16) > std::ptr::read_unaligned(b as *const u16) }
                } else {
                    |a, b| unsafe { std::ptr::read_unaligned(a as *const u16).swap_bytes() > std::ptr::read_unaligned(b as *const u16).swap_bytes() }
                }
            }
            DataType::I16(endian) => {
                if self.check_endian(endian) {
                    |a, b| unsafe { std::ptr::read_unaligned(a as *const i16) > std::ptr::read_unaligned(b as *const i16) }
                } else {
                    |a, b| unsafe { std::ptr::read_unaligned(a as *const i16).swap_bytes() > std::ptr::read_unaligned(b as *const i16).swap_bytes() }
                }
            }
            DataType::U32(endian) => {
                if self.check_endian(endian) {
                    |a, b| unsafe { std::ptr::read_unaligned(a as *const u32) > std::ptr::read_unaligned(b as *const u32) }
                } else {
                    |a, b| unsafe { std::ptr::read_unaligned(a as *const u32).swap_bytes() > std::ptr::read_unaligned(b as *const u32).swap_bytes() }
                }
            }
            DataType::I32(endian) => {
                if self.check_endian(endian) {
                    |a, b| unsafe { std::ptr::read_unaligned(a as *const i32) > std::ptr::read_unaligned(b as *const i32) }
                } else {
                    |a, b| unsafe { std::ptr::read_unaligned(a as *const i32).swap_bytes() > std::ptr::read_unaligned(b as *const i32).swap_bytes() }
                }
            }
            DataType::U64(endian) => {
                if self.check_endian(endian) {
                    |a, b| unsafe { std::ptr::read_unaligned(a as *const u64) > std::ptr::read_unaligned(b as *const u64) }
                } else {
                    |a, b| unsafe { std::ptr::read_unaligned(a as *const u64).swap_bytes() > std::ptr::read_unaligned(b as *const u64).swap_bytes() }
                }
            }
            DataType::I64(endian) => {
                if self.check_endian(endian) {
                    |a, b| unsafe { std::ptr::read_unaligned(a as *const i64) > std::ptr::read_unaligned(b as *const i64) }
                } else {
                    |a, b| unsafe { std::ptr::read_unaligned(a as *const i64).swap_bytes() > std::ptr::read_unaligned(b as *const i64).swap_bytes() }
                }
            }
            DataType::F32(endian) => {
                if self.check_endian(endian) {
                    |a, b| unsafe { std::ptr::read_unaligned(a as *const f32) > std::ptr::read_unaligned(b as *const f32) }
                } else {
                    |a, b| unsafe {
                        let a_val = std::ptr::read_unaligned(a as *const u32).swap_bytes();
                        let b_val = std::ptr::read_unaligned(b as *const u32).swap_bytes();
                        *(a_val as *const f32) > *(b_val as *const f32)
                    }
                }
            }
            DataType::F64(endian) => {
                if self.check_endian(endian) {
                    |a, b| unsafe { std::ptr::read_unaligned(a as *const f64) > std::ptr::read_unaligned(b as *const f64) }
                } else {
                    |a, b| unsafe {
                        let a_val = std::ptr::read_unaligned(a as *const u64).swap_bytes();
                        let b_val = std::ptr::read_unaligned(b as *const u64).swap_bytes();
                        *(a_val as *const f64) > *(b_val as *const f64)
                    }
                }
            }
            _ => panic!("unsupported data type"),
        }
    }
    
    fn get_compare_decreased(&self, data_type: &DataType) -> ScalarCompareFnRelative {
        match data_type {
            DataType::U8() => |a, b| unsafe { std::ptr::read_unaligned(a as *const u8) < std::ptr::read_unaligned(b as *const u8) },
            DataType::I8() => |a, b| unsafe { std::ptr::read_unaligned(a as *const i8) < std::ptr::read_unaligned(b as *const i8) },
            DataType::U16(endian) => {
                if self.check_endian(endian) {
                    |a, b| unsafe { std::ptr::read_unaligned(a as *const u16) < std::ptr::read_unaligned(b as *const u16) }
                } else {
                    |a, b| unsafe { std::ptr::read_unaligned(a as *const u16).swap_bytes() < std::ptr::read_unaligned(b as *const u16).swap_bytes() }
                }
            }
            DataType::I16(endian) => {
                if self.check_endian(endian) {
                    |a, b| unsafe { std::ptr::read_unaligned(a as *const i16) < std::ptr::read_unaligned(b as *const i16) }
                } else {
                    |a, b| unsafe { std::ptr::read_unaligned(a as *const i16).swap_bytes() < std::ptr::read_unaligned(b as *const i16).swap_bytes() }
                }
            }
            DataType::U32(endian) => {
                if self.check_endian(endian) {
                    |a, b| unsafe { std::ptr::read_unaligned(a as *const u32) < std::ptr::read_unaligned(b as *const u32) }
                } else {
                    |a, b| unsafe { std::ptr::read_unaligned(a as *const u32).swap_bytes() < std::ptr::read_unaligned(b as *const u32).swap_bytes() }
                }
            }
            DataType::I32(endian) => {
                if self.check_endian(endian) {
                    |a, b| unsafe { std::ptr::read_unaligned(a as *const i32) < std::ptr::read_unaligned(b as *const i32) }
                } else {
                    |a, b| unsafe { std::ptr::read_unaligned(a as *const i32).swap_bytes() < std::ptr::read_unaligned(b as *const i32).swap_bytes() }
                }
            }
            DataType::U64(endian) => {
                if self.check_endian(endian) {
                    |a, b| unsafe { std::ptr::read_unaligned(a as *const u64) < std::ptr::read_unaligned(b as *const u64) }
                } else {
                    |a, b| unsafe { std::ptr::read_unaligned(a as *const u64).swap_bytes() < std::ptr::read_unaligned(b as *const u64).swap_bytes() }
                }
            }
            DataType::I64(endian) => {
                if self.check_endian(endian) {
                    |a, b| unsafe { std::ptr::read_unaligned(a as *const i64) < std::ptr::read_unaligned(b as *const i64) }
                } else {
                    |a, b| unsafe { std::ptr::read_unaligned(a as *const i64).swap_bytes() < std::ptr::read_unaligned(b as *const i64).swap_bytes() }
                }
            }
            DataType::F32(endian) => {
                if self.check_endian(endian) {
                    |a, b| unsafe { std::ptr::read_unaligned(a as *const f32) < std::ptr::read_unaligned(b as *const f32) }
                } else {
                    |a, b| unsafe {
                        let a_val = std::ptr::read_unaligned(a as *const u32).swap_bytes();
                        let b_val = std::ptr::read_unaligned(b as *const u32).swap_bytes();
                        *(a_val as *const f32) < *(b_val as *const f32)
                    }
                }
            }
            DataType::F64(endian) => {
                if self.check_endian(endian) {
                    |a, b| unsafe { std::ptr::read_unaligned(a as *const f64) < std::ptr::read_unaligned(b as *const f64) }
                } else {
                    |a, b| unsafe {
                        let a_val = std::ptr::read_unaligned(a as *const u64).swap_bytes();
                        let b_val = std::ptr::read_unaligned(b as *const u64).swap_bytes();
                        *(a_val as *const f64) < *(b_val as *const f64)
                    }
                }
            }
            _ => panic!("unsupported data type"),
        }
    }
    
    fn get_compare_increased_by(&self, data_type: &DataType) -> ScalarCompareFnDelta {
        match data_type {
            DataType::U8() => |a, b, delta| unsafe {
                std::ptr::read_unaligned(a as *const u8) == std::ptr::read_unaligned(b as *const u8).wrapping_add(std::ptr::read_unaligned(delta as *const u8))
            },
            DataType::I8() => |a, b, delta| unsafe {
                std::ptr::read_unaligned(a as *const i8) == std::ptr::read_unaligned(b as *const i8).wrapping_add(std::ptr::read_unaligned(delta as *const i8))
            },
            DataType::U16(endian) => {
                if self.check_endian(endian) {
                    |a, b, delta| unsafe {
                        std::ptr::read_unaligned(a as *const u16) == std::ptr::read_unaligned(b as *const u16).wrapping_add(std::ptr::read_unaligned(delta as *const u16))
                    }
                } else {
                    |a, b, delta| unsafe {
                        std::ptr::read_unaligned(a as *const u16).swap_bytes()
                            == std::ptr::read_unaligned(b as *const u16).swap_bytes().wrapping_add(std::ptr::read_unaligned(delta as *const u16).swap_bytes())
                    }
                }
            }
            DataType::I16(endian) => {
                if self.check_endian(endian) {
                    |a, b, delta| unsafe {
                        std::ptr::read_unaligned(a as *const i16) == std::ptr::read_unaligned(b as *const i16).wrapping_add(std::ptr::read_unaligned(delta as *const i16))
                    }
                } else {
                    |a, b, delta| unsafe {
                        std::ptr::read_unaligned(a as *const i16).swap_bytes()
                            == std::ptr::read_unaligned(b as *const i16).swap_bytes().wrapping_add(std::ptr::read_unaligned(delta as *const i16).swap_bytes())
                    }
                }
            }
            DataType::U32(endian) => {
                if self.check_endian(endian) {
                    |a, b, delta| unsafe {
                        std::ptr::read_unaligned(a as *const u32) == std::ptr::read_unaligned(b as *const u32).wrapping_add(std::ptr::read_unaligned(delta as *const u32))
                    }
                } else {
                    |a, b, delta| unsafe {
                        std::ptr::read_unaligned(a as *const u32).swap_bytes()
                            == std::ptr::read_unaligned(b as *const u32).swap_bytes().wrapping_add(std::ptr::read_unaligned(delta as *const u32).swap_bytes())
                    }
                }
            }
            DataType::I32(endian) => {
                if self.check_endian(endian) {
                    |a, b, delta| unsafe {
                        std::ptr::read_unaligned(a as *const i32) == std::ptr::read_unaligned(b as *const i32).wrapping_add(std::ptr::read_unaligned(delta as *const i32))
                    }
                } else {
                    |a, b, delta| unsafe {
                        std::ptr::read_unaligned(a as *const i32).swap_bytes()
                            == std::ptr::read_unaligned(b as *const i32).swap_bytes().wrapping_add(std::ptr::read_unaligned(delta as *const i32).swap_bytes())
                    }
                }
            }
            DataType::U64(endian) => {
                if self.check_endian(endian) {
                    |a, b, delta| unsafe {
                        std::ptr::read_unaligned(a as *const u64) == std::ptr::read_unaligned(b as *const u64).wrapping_add(std::ptr::read_unaligned(delta as *const u64))
                    }
                } else {
                    |a, b, delta| unsafe {
                        std::ptr::read_unaligned(a as *const u64).swap_bytes()
                            == std::ptr::read_unaligned(b as *const u64).swap_bytes().wrapping_add(std::ptr::read_unaligned(delta as *const u64).swap_bytes())
                    }
                }
            }
            DataType::I64(endian) => {
                if self.check_endian(endian) {
                    |a, b, delta| unsafe {
                        std::ptr::read_unaligned(a as *const i64) == std::ptr::read_unaligned(b as *const i64).wrapping_add(std::ptr::read_unaligned(delta as *const i64))
                    }
                } else {
                    |a, b, delta| unsafe {
                        std::ptr::read_unaligned(a as *const i64).swap_bytes()
                            == std::ptr::read_unaligned(b as *const i64).swap_bytes().wrapping_add(std::ptr::read_unaligned(delta as *const i64).swap_bytes())
                    }
                }
            }
            DataType::F32(endian) => {
                if self.check_endian(endian) {
                    |a, b, delta| unsafe {
                        std::ptr::read_unaligned(a as *const f32) == std::ptr::read_unaligned(b as *const f32) + std::ptr::read_unaligned(delta as *const f32)
                    }
                } else {
                    |a, b, delta| unsafe {
                        let a_val = std::ptr::read_unaligned(a as *const u32).swap_bytes();
                        let b_val = std::ptr::read_unaligned(b as *const u32).swap_bytes();
                        *(a_val as *const f32) == *(b_val as *const f32) + std::ptr::read_unaligned(delta as *const f32)
                    }
                }
            }
            DataType::F64(endian) => {
                if self.check_endian(endian) {
                    |a, b, delta| unsafe {
                        std::ptr::read_unaligned(a as *const f64) == std::ptr::read_unaligned(b as *const f64) + std::ptr::read_unaligned(delta as *const f64)
                    }
                } else {
                    |a, b, delta| unsafe {
                        let a_val = std::ptr::read_unaligned(a as *const u64).swap_bytes();
                        let b_val = std::ptr::read_unaligned(b as *const u64).swap_bytes();
                        *(a_val as *const f64) == *(b_val as *const f64) + std::ptr::read_unaligned(delta as *const f64)
                    }
                }
            }
            _ => panic!("unsupported data type"),
        }
    }
    
    fn get_compare_decreased_by(&self, data_type: &DataType) -> ScalarCompareFnDelta {
        match data_type {
            DataType::U8() => |a, b, delta| unsafe {
                std::ptr::read_unaligned(a as *const u8) == std::ptr::read_unaligned(b as *const u8).wrapping_sub(std::ptr::read_unaligned(delta as *const u8))
            },
            DataType::I8() => |a, b, delta| unsafe {
                std::ptr::read_unaligned(a as *const i8) == std::ptr::read_unaligned(b as *const i8).wrapping_sub(std::ptr::read_unaligned(delta as *const i8))
            },
            DataType::U16(endian) => {
                if self.check_endian(endian) {
                    |a, b, delta| unsafe {
                        std::ptr::read_unaligned(a as *const u16) == std::ptr::read_unaligned(b as *const u16).wrapping_sub(std::ptr::read_unaligned(delta as *const u16))
                    }
                } else {
                    |a, b, delta| unsafe {
                        std::ptr::read_unaligned(a as *const u16).swap_bytes()
                            == std::ptr::read_unaligned(b as *const u16).swap_bytes().wrapping_sub(std::ptr::read_unaligned(delta as *const u16).swap_bytes())
                    }
                }
            }
            DataType::I16(endian) => {
                if self.check_endian(endian) {
                    |a, b, delta| unsafe {
                        std::ptr::read_unaligned(a as *const i16) == std::ptr::read_unaligned(b as *const i16).wrapping_sub(std::ptr::read_unaligned(delta as *const i16))
                    }
                } else {
                    |a, b, delta| unsafe {
                        std::ptr::read_unaligned(a as *const i16).swap_bytes()
                            == std::ptr::read_unaligned(b as *const i16).swap_bytes().wrapping_sub(std::ptr::read_unaligned(delta as *const i16).swap_bytes())
                    }
                }
            }
            DataType::U32(endian) => {
                if self.check_endian(endian) {
                    |a, b, delta| unsafe {
                        std::ptr::read_unaligned(a as *const u32) == std::ptr::read_unaligned(b as *const u32).wrapping_sub(std::ptr::read_unaligned(delta as *const u32))
                    }
                } else {
                    |a, b, delta| unsafe {
                        std::ptr::read_unaligned(a as *const u32).swap_bytes()
                            == std::ptr::read_unaligned(b as *const u32).swap_bytes().wrapping_sub(std::ptr::read_unaligned(delta as *const u32).swap_bytes())
                    }
                }
            }
            DataType::I32(endian) => {
                if self.check_endian(endian) {
                    |a, b, delta| unsafe {
                        std::ptr::read_unaligned(a as *const i32) == std::ptr::read_unaligned(b as *const i32).wrapping_sub(std::ptr::read_unaligned(delta as *const i32))
                    }
                } else {
                    |a, b, delta| unsafe {
                        std::ptr::read_unaligned(a as *const i32).swap_bytes()
                            == std::ptr::read_unaligned(b as *const i32).swap_bytes().wrapping_sub(std::ptr::read_unaligned(delta as *const i32).swap_bytes())
                    }
                }
            }
            DataType::U64(endian) => {
                if self.check_endian(endian) {
                    |a, b, delta| unsafe {
                        std::ptr::read_unaligned(a as *const u64) == std::ptr::read_unaligned(b as *const u64).wrapping_sub(std::ptr::read_unaligned(delta as *const u64))
                    }
                } else {
                    |a, b, delta| unsafe {
                        std::ptr::read_unaligned(a as *const u64).swap_bytes()
                            == std::ptr::read_unaligned(b as *const u64).swap_bytes().wrapping_sub(std::ptr::read_unaligned(delta as *const u64).swap_bytes())
                    }
                }
            }
            DataType::I64(endian) => {
                if self.check_endian(endian) {
                    |a, b, delta| unsafe {
                        std::ptr::read_unaligned(a as *const i64) == std::ptr::read_unaligned(b as *const i64).wrapping_sub(std::ptr::read_unaligned(delta as *const i64))
                    }
                } else {
                    |a, b, delta| unsafe {
                        std::ptr::read_unaligned(a as *const i64).swap_bytes()
                            == std::ptr::read_unaligned(b as *const i64).swap_bytes().wrapping_sub(std::ptr::read_unaligned(delta as *const i64).swap_bytes())
                    }
                }
            }
            DataType::F32(endian) => {
                if self.check_endian(endian) {
                    |a, b, delta| unsafe {
                        std::ptr::read_unaligned(a as *const f32) == std::ptr::read_unaligned(b as *const f32) - std::ptr::read_unaligned(delta as *const f32)
                    }
                } else {
                    |a, b, delta| unsafe {
                        let a_val = std::ptr::read_unaligned(a as *const u32).swap_bytes();
                        let b_val = std::ptr::read_unaligned(b as *const u32).swap_bytes();
                        *(a_val as *const f32) == *(b_val as *const f32) - std::ptr::read_unaligned(delta as *const f32)
                    }
                }
            }
            DataType::F64(endian) => {
                if self.check_endian(endian) {
                    |a, b, delta| unsafe {
                        std::ptr::read_unaligned(a as *const f64) == std::ptr::read_unaligned(b as *const f64) - std::ptr::read_unaligned(delta as *const f64)
                    }
                } else {
                    |a, b, delta| unsafe {
                        let a_val = std::ptr::read_unaligned(a as *const u64).swap_bytes();
                        let b_val = std::ptr::read_unaligned(b as *const u64).swap_bytes();
                        *(a_val as *const f64) == *(b_val as *const f64) - std::ptr::read_unaligned(delta as *const f64)
                    }
                }
            }
            _ => panic!("unsupported data type"),
        }
    }    
}
