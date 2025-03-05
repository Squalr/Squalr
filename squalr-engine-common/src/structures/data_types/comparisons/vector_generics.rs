use std::simd::SimdElement;
use std::simd::{LaneCount, Simd, SupportedLaneCount, cmp::SimdPartialEq};
use std::{mem, ptr};

pub struct VectorGenerics {}

impl VectorGenerics {
    pub fn transmute_mask<T, const N: usize>(value: &<Simd<T, N> as SimdPartialEq>::Mask) -> Simd<u8, N>
    where
        LaneCount<N>: SupportedLaneCount,
        T: SimdElement,
        Simd<T, N>: SimdPartialEq,
    {
        let mut result_array = [0u8; N];
        let value_ptr = value as *const _ as *const u8;
        unsafe {
            ptr::copy_nonoverlapping(value_ptr, result_array.as_mut_ptr(), mem::size_of_val(value));
        }

        Simd::<u8, N>::from_array(result_array)
    }
}
