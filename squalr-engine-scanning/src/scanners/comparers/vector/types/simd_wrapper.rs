use std::marker::PhantomData;
use std::simd::{cmp::{SimdPartialEq, SimdPartialOrd}, LaneCount, Mask, Simd, SimdElement, SupportedLaneCount};

pub struct SimdWrapper<T, const N: usize> {
    _marker: PhantomData<T>,
}

impl<T, const N: usize> SimdWrapper<T, N>
where
    T: SimdElement,
    LaneCount<N>: SupportedLaneCount,
{
    pub fn from_array(array: [T; N]) -> Simd<T, N> {
        Simd::<T, N>::from_array(array)
    }

    pub fn splat(value: T) -> Simd<T, N> {
        Simd::<T, N>::splat(value)
    }

    /// Convert a mask to a `Simd<u8, N>`, where `true` is `255` and `false` is `0`.
    fn mask_to_u8(mask: Mask<i8, N>) -> Simd<u8, N> {
        mask.select(Simd::splat(255), Simd::splat(0))
    }

    pub fn eq(a: Simd<T, N>, b: Simd<T, N>) -> Simd<u8, N>
    where
        Simd<T, N>: SimdPartialEq<Mask = Mask<i8, N>>,
    {
        Self::mask_to_u8(a.simd_eq(b))
    }

    pub fn ne(a: Simd<T, N>, b: Simd<T, N>) -> Simd<u8, N>
    where
        Simd<T, N>: SimdPartialEq<Mask = Mask<i8, N>>,
    {
        Self::mask_to_u8(a.simd_ne(b))
    }

    pub fn gt(a: Simd<T, N>, b: Simd<T, N>) -> Simd<u8, N>
    where
        Simd<T, N>: SimdPartialOrd<Mask = Mask<i8, N>>,
    {
        Self::mask_to_u8(a.simd_gt(b))
    }

    pub fn ge(a: Simd<T, N>, b: Simd<T, N>) -> Simd<u8, N>
    where
        Simd<T, N>: SimdPartialOrd<Mask = Mask<i8, N>>,
    {
        Self::mask_to_u8(a.simd_ge(b))
    }

    pub fn lt(a: Simd<T, N>, b: Simd<T, N>) -> Simd<u8, N>
    where
        Simd<T, N>: SimdPartialOrd<Mask = Mask<i8, N>>,
    {
        Self::mask_to_u8(a.simd_lt(b))
    }

    pub fn le(a: Simd<T, N>, b: Simd<T, N>) -> Simd<u8, N>
    where
        Simd<T, N>: SimdPartialOrd<Mask = Mask<i8, N>>,
    {
        Self::mask_to_u8(a.simd_le(b))
    }
}
