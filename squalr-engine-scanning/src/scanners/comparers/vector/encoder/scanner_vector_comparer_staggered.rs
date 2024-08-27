use crate::scanners::comparers::vector::encoder::scanner_vector_comparer::ScannerVectorComparer;
use crate::scanners::comparers::vector::encoder::scanner_vector_comparer::ScannerVectorComparerTrait;
use crate::scanners::comparers::vector::encoder::scanner_vector_comparer::VectorCompareFnDelta128;
use crate::scanners::comparers::vector::encoder::scanner_vector_comparer::VectorCompareFnDelta256;
use crate::scanners::comparers::vector::encoder::scanner_vector_comparer::VectorCompareFnDelta512;
use crate::scanners::comparers::vector::encoder::scanner_vector_comparer::VectorCompareFnImmediate128;
use crate::scanners::comparers::vector::encoder::scanner_vector_comparer::VectorCompareFnImmediate256;
use crate::scanners::comparers::vector::encoder::scanner_vector_comparer::VectorCompareFnImmediate512;
use crate::scanners::comparers::vector::encoder::scanner_vector_comparer::VectorCompareFnRelative128;
use crate::scanners::comparers::vector::encoder::scanner_vector_comparer::VectorCompareFnRelative256;
use crate::scanners::comparers::vector::encoder::scanner_vector_comparer::VectorCompareFnRelative512;
use crate::scanners::parameters::scan_compare_type::ScanCompareType;
use squalr_engine_common::values::data_type::DataType;
use std::sync::Once;

pub struct ScannerVectorComparerStaggered<const VECTOR_SIZE_BITS: usize>;

macro_rules! impl_scanner_vector_comparer_staggered {
    ($vector_bit_size:tt, $fn_type_immediate:ident, $fn_type_relative:ident, $fn_type_delta:ident) => {
        impl ScannerVectorComparerTrait<$vector_bit_size> for ScannerVectorComparerStaggered<$vector_bit_size> {
            type ImmediateCompareFn = $fn_type_immediate;
            type RelativeCompareFn = $fn_type_relative;
            type DeltaCompareFn = $fn_type_delta;

            fn get_immediate_compare_func(
                &self,
                scan_compare_type: ScanCompareType,
                data_type: &DataType,
            ) -> Self::ImmediateCompareFn {
                self.get_immediate_compare_func(scan_compare_type, data_type)
            }

            fn get_relative_compare_func(
                &self,
                scan_compare_type: ScanCompareType,
                data_type: &DataType,
            ) -> Self::RelativeCompareFn {
                self.get_relative_compare_func(scan_compare_type, data_type)
            }

            fn get_relative_delta_compare_func(
                &self,
                scan_compare_type: ScanCompareType,
                data_type: &DataType,
            ) -> Self::DeltaCompareFn {
                self.get_relative_delta_compare_func(scan_compare_type, data_type)
            }
        }

        impl ScannerVectorComparerStaggered<$vector_bit_size> {
            pub fn get_instance() -> &'static ScannerVectorComparerStaggered<$vector_bit_size> {
                static mut INSTANCE: Option<ScannerVectorComparerStaggered<$vector_bit_size>> = None;
                static INIT: Once = Once::new();

                unsafe {
                    INIT.call_once(|| {
                        let instance = ScannerVectorComparerStaggered::<$vector_bit_size>;
                        INSTANCE = Some(instance);
                    });

                    INSTANCE.as_ref().unwrap_unchecked()
                }
            }

            pub fn get_immediate_compare_func(
                &self,
                scan_compare_type: ScanCompareType,
                data_type: &DataType,
            ) -> $fn_type_immediate {
                let base_compare = ScannerVectorComparer::<$vector_bit_size>::get_instance().get_immediate_compare_func(scan_compare_type, data_type);

                return base_compare;
            }

            pub fn get_relative_compare_func(
                &self,
                scan_compare_type: ScanCompareType,
                data_type: &DataType,
            ) -> $fn_type_relative {
                let base_compare = ScannerVectorComparer::<$vector_bit_size>::get_instance().get_relative_compare_func(scan_compare_type, data_type);

                return base_compare;
            }

            pub fn get_relative_delta_compare_func(
                &self,
                scan_compare_type: ScanCompareType,
                data_type: &DataType,
            ) -> $fn_type_delta {
                let base_compare = ScannerVectorComparer::<$vector_bit_size>::get_instance().get_relative_delta_compare_func(scan_compare_type, data_type);

                return base_compare;
            }
        }
    };
}

impl_scanner_vector_comparer_staggered!(128, VectorCompareFnImmediate128, VectorCompareFnRelative128, VectorCompareFnDelta128);
impl_scanner_vector_comparer_staggered!(256, VectorCompareFnImmediate256, VectorCompareFnRelative256, VectorCompareFnDelta256);
impl_scanner_vector_comparer_staggered!(512, VectorCompareFnImmediate512, VectorCompareFnRelative512, VectorCompareFnDelta512);
