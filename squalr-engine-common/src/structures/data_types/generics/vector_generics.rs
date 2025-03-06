use std::simd::SimdElement;
use std::simd::{LaneCount, Simd, SupportedLaneCount, cmp::SimdPartialEq};
use std::{mem, ptr};

pub struct VectorGenerics {}

impl VectorGenerics {
    pub fn transmute<Src, Dst, const N: usize>(value: &Simd<Src, N>) -> Simd<Dst, N>
    where
        LaneCount<N>: SupportedLaneCount,
        Src: SimdElement,
        Dst: SimdElement,
        Simd<Dst, N>: SimdPartialEq,
    {
        unsafe {
            let mut result = mem::MaybeUninit::<Simd<Dst, N>>::uninit();
            ptr::copy_nonoverlapping(
                value as *const Simd<Src, N> as *const u8,
                result.as_mut_ptr() as *mut u8,
                mem::size_of::<Simd<Dst, N>>(),
            );
            result.assume_init()
        }
    }

    pub fn transmute_mask<Src, const N: usize>(value: &<Simd<Src, N> as SimdPartialEq>::Mask) -> Simd<u8, N>
    where
        LaneCount<N>: SupportedLaneCount,
        Src: SimdElement,
        Simd<Src, N>: SimdPartialEq,
    {
        let mut result_array = [0u8; N];
        let value_ptr = value as *const _ as *const u8;
        unsafe {
            ptr::copy_nonoverlapping(value_ptr, result_array.as_mut_ptr(), mem::size_of_val(value));
        }

        Simd::<u8, N>::from_array(result_array)
    }
}
