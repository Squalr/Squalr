use crate::structures::data_types::generics::vector_generics::VectorGenerics;
use crate::structures::scanning::comparisons::scan_function_vector::{VectorCompareFnDelta, VectorCompareFnImmediate, VectorCompareFnRelative};
use crate::structures::scanning::constraints::scan_constraint::ScanConstraint;
use std::ops::{Add, BitAnd, BitOr, BitXor, Div, Mul, Rem, Shl, Shr, Sub};
use std::ptr;
use std::simd::cmp::{SimdPartialEq, SimdPartialOrd};
use std::simd::num::{SimdInt, SimdUint};
use std::simd::{Simd, SimdElement};
use std::sync::Arc;

pub struct VectorComparisonsIntegerBigEndian {}

impl VectorComparisonsIntegerBigEndian {
    pub fn get_vector_compare_equal<const N: usize, const E: usize, PrimitiveType: SimdElement + Send + Sync + 'static>(
        scan_constraint: &ScanConstraint
    ) -> Option<VectorCompareFnImmediate<N>>
    where
        Simd<PrimitiveType, E>: SimdPartialEq,
    {
        // Optimization: no endian byte swap required for immediate or current values.
        let immediate_value = scan_constraint.get_data_value();
        let immediate_value_ptr = immediate_value.as_ptr();
        let immediate_value = Simd::splat(unsafe { ptr::read_unaligned(immediate_value_ptr as *const PrimitiveType) });

        Some(Arc::new(move |current_values_ptr| {
            let current_values = Simd::from_array(unsafe { ptr::read_unaligned(current_values_ptr as *const [PrimitiveType; E]) });

            VectorGenerics::transmute_mask(current_values.simd_eq(immediate_value))
        }))
    }

    pub fn get_vector_compare_not_equal<const N: usize, const E: usize, PrimitiveType: SimdElement + Send + Sync + 'static>(
        scan_constraint: &ScanConstraint
    ) -> Option<VectorCompareFnImmediate<N>>
    where
        Simd<PrimitiveType, E>: SimdPartialEq,
    {
        // Optimization: no endian byte swap required for immediate or current values.
        let immediate_value = scan_constraint.get_data_value();
        let immediate_value_ptr = immediate_value.as_ptr();
        let immediate_value = Simd::splat(unsafe { ptr::read_unaligned(immediate_value_ptr as *const PrimitiveType) });

        Some(Arc::new(move |current_values_ptr| {
            let current_values = Simd::from_array(unsafe { ptr::read_unaligned(current_values_ptr as *const [PrimitiveType; E]) });

            VectorGenerics::transmute_mask(current_values.simd_ne(immediate_value))
        }))
    }

    pub fn get_vector_compare_greater_than<const N: usize, const E: usize, PrimitiveType: SimdElement + Send + Sync + 'static>(
        scan_constraint: &ScanConstraint
    ) -> Option<VectorCompareFnImmediate<N>>
    where
        Simd<PrimitiveType, E>: SimdPartialOrd + SimdInt,
    {
        let immediate_value = scan_constraint.get_data_value();
        let immediate_value_ptr = immediate_value.as_ptr();
        let immediate_value = SimdInt::swap_bytes(Simd::splat(unsafe { ptr::read_unaligned(immediate_value_ptr as *const PrimitiveType) }));

        Some(Arc::new(move |current_values_ptr| {
            let current_values = SimdInt::swap_bytes(Simd::from_array(unsafe {
                ptr::read_unaligned(current_values_ptr as *const [PrimitiveType; E])
            }));

            VectorGenerics::transmute_mask(current_values.simd_gt(immediate_value))
        }))
    }

    pub fn get_vector_compare_greater_than_unsigned<const N: usize, const E: usize, PrimitiveType: SimdElement + Send + Sync + 'static>(
        scan_constraint: &ScanConstraint
    ) -> Option<VectorCompareFnImmediate<N>>
    where
        Simd<PrimitiveType, E>: SimdPartialOrd + SimdUint,
    {
        let immediate_value = scan_constraint.get_data_value();
        let immediate_value_ptr = immediate_value.as_ptr();
        let immediate_value = SimdUint::swap_bytes(Simd::splat(unsafe { ptr::read_unaligned(immediate_value_ptr as *const PrimitiveType) }));

        Some(Arc::new(move |current_values_ptr| {
            let current_values = SimdUint::swap_bytes(Simd::from_array(unsafe {
                ptr::read_unaligned(current_values_ptr as *const [PrimitiveType; E])
            }));

            VectorGenerics::transmute_mask(current_values.simd_gt(immediate_value))
        }))
    }

    pub fn get_vector_compare_greater_than_or_equal<const N: usize, const E: usize, PrimitiveType: SimdElement + Send + Sync + 'static>(
        scan_constraint: &ScanConstraint
    ) -> Option<VectorCompareFnImmediate<N>>
    where
        Simd<PrimitiveType, E>: SimdPartialOrd + SimdInt,
    {
        let immediate_value = scan_constraint.get_data_value();
        let immediate_value_ptr = immediate_value.as_ptr();
        let immediate_value = SimdInt::swap_bytes(Simd::splat(unsafe { ptr::read_unaligned(immediate_value_ptr as *const PrimitiveType) }));

        Some(Arc::new(move |current_values_ptr| {
            let current_values = SimdInt::swap_bytes(Simd::from_array(unsafe {
                ptr::read_unaligned(current_values_ptr as *const [PrimitiveType; E])
            }));

            VectorGenerics::transmute_mask(current_values.simd_ge(immediate_value))
        }))
    }

    pub fn get_vector_compare_greater_than_or_equal_unsigned<const N: usize, const E: usize, PrimitiveType: SimdElement + Send + Sync + 'static>(
        scan_constraint: &ScanConstraint
    ) -> Option<VectorCompareFnImmediate<N>>
    where
        Simd<PrimitiveType, E>: SimdPartialOrd + SimdUint,
    {
        let immediate_value = scan_constraint.get_data_value();
        let immediate_value_ptr = immediate_value.as_ptr();
        let immediate_value = SimdUint::swap_bytes(Simd::splat(unsafe { ptr::read_unaligned(immediate_value_ptr as *const PrimitiveType) }));

        Some(Arc::new(move |current_values_ptr| {
            let current_values = SimdUint::swap_bytes(Simd::from_array(unsafe {
                ptr::read_unaligned(current_values_ptr as *const [PrimitiveType; E])
            }));

            VectorGenerics::transmute_mask(current_values.simd_ge(immediate_value))
        }))
    }

    pub fn get_vector_compare_less_than<const N: usize, const E: usize, PrimitiveType: SimdElement + Send + Sync + 'static>(
        scan_constraint: &ScanConstraint
    ) -> Option<VectorCompareFnImmediate<N>>
    where
        Simd<PrimitiveType, E>: SimdPartialOrd + SimdInt,
    {
        let immediate_value = scan_constraint.get_data_value();
        let immediate_value_ptr = immediate_value.as_ptr();
        let immediate_value = SimdInt::swap_bytes(Simd::splat(unsafe { ptr::read_unaligned(immediate_value_ptr as *const PrimitiveType) }));

        Some(Arc::new(move |current_values_ptr| {
            let current_values = SimdInt::swap_bytes(Simd::from_array(unsafe {
                ptr::read_unaligned(current_values_ptr as *const [PrimitiveType; E])
            }));

            VectorGenerics::transmute_mask(current_values.simd_lt(immediate_value))
        }))
    }

    pub fn get_vector_compare_less_than_unsigned<const N: usize, const E: usize, PrimitiveType: SimdElement + Send + Sync + 'static>(
        scan_constraint: &ScanConstraint
    ) -> Option<VectorCompareFnImmediate<N>>
    where
        Simd<PrimitiveType, E>: SimdPartialOrd + SimdUint,
    {
        let immediate_value = scan_constraint.get_data_value();
        let immediate_value_ptr = immediate_value.as_ptr();
        let immediate_value = SimdUint::swap_bytes(Simd::splat(unsafe { ptr::read_unaligned(immediate_value_ptr as *const PrimitiveType) }));

        Some(Arc::new(move |current_values_ptr| {
            let current_values = SimdUint::swap_bytes(Simd::from_array(unsafe {
                ptr::read_unaligned(current_values_ptr as *const [PrimitiveType; E])
            }));

            VectorGenerics::transmute_mask(current_values.simd_lt(immediate_value))
        }))
    }

    pub fn get_vector_compare_less_than_or_equal<const N: usize, const E: usize, PrimitiveType: SimdElement + Send + Sync + 'static>(
        scan_constraint: &ScanConstraint
    ) -> Option<VectorCompareFnImmediate<N>>
    where
        Simd<PrimitiveType, E>: SimdPartialOrd + SimdInt,
    {
        let immediate_value = scan_constraint.get_data_value();
        let immediate_value_ptr = immediate_value.as_ptr();
        let immediate_value = SimdInt::swap_bytes(Simd::splat(unsafe { ptr::read_unaligned(immediate_value_ptr as *const PrimitiveType) }));

        Some(Arc::new(move |current_values_ptr| {
            let current_values = SimdInt::swap_bytes(Simd::from_array(unsafe {
                ptr::read_unaligned(current_values_ptr as *const [PrimitiveType; E])
            }));

            VectorGenerics::transmute_mask(current_values.simd_le(immediate_value))
        }))
    }

    pub fn get_vector_compare_less_than_or_equal_unsigned<const N: usize, const E: usize, PrimitiveType: SimdElement + Send + Sync + 'static>(
        scan_constraint: &ScanConstraint
    ) -> Option<VectorCompareFnImmediate<N>>
    where
        Simd<PrimitiveType, E>: SimdPartialOrd + SimdUint,
    {
        let immediate_value = scan_constraint.get_data_value();
        let immediate_value_ptr = immediate_value.as_ptr();
        let immediate_value = SimdUint::swap_bytes(Simd::splat(unsafe { ptr::read_unaligned(immediate_value_ptr as *const PrimitiveType) }));

        Some(Arc::new(move |current_values_ptr| {
            let current_values = SimdUint::swap_bytes(Simd::from_array(unsafe {
                ptr::read_unaligned(current_values_ptr as *const [PrimitiveType; E])
            }));

            VectorGenerics::transmute_mask(current_values.simd_le(immediate_value))
        }))
    }

    pub fn get_vector_compare_changed<const N: usize, const E: usize, PrimitiveType: SimdElement + Send + Sync + 'static>(
        _scan_constraint: &ScanConstraint
    ) -> Option<VectorCompareFnRelative<N>>
    where
        Simd<PrimitiveType, E>: SimdPartialEq,
    {
        Some(Arc::new(move |current_values_ptr, previous_values_ptr| {
            // Optimization: no endian byte swap required.
            let current_values = Simd::from_array(unsafe { ptr::read_unaligned(current_values_ptr as *const [PrimitiveType; E]) });
            let previous_values = Simd::from_array(unsafe { ptr::read_unaligned(previous_values_ptr as *const [PrimitiveType; E]) });

            VectorGenerics::transmute_mask(current_values.simd_ne(previous_values))
        }))
    }

    pub fn get_vector_compare_unchanged<const N: usize, const E: usize, PrimitiveType: SimdElement + Send + Sync + 'static>(
        _scan_constraint: &ScanConstraint
    ) -> Option<VectorCompareFnRelative<N>>
    where
        Simd<PrimitiveType, E>: SimdPartialEq,
    {
        Some(Arc::new(move |current_values_ptr, previous_values_ptr| {
            // Optimization: no endian byte swap required.
            let current_values = Simd::from_array(unsafe { ptr::read_unaligned(current_values_ptr as *const [PrimitiveType; E]) });
            let previous_values = Simd::from_array(unsafe { ptr::read_unaligned(previous_values_ptr as *const [PrimitiveType; E]) });

            VectorGenerics::transmute_mask(current_values.simd_eq(previous_values))
        }))
    }

    pub fn get_vector_compare_increased<const N: usize, const E: usize, PrimitiveType: SimdElement + Send + Sync + 'static>(
        _scan_constraint: &ScanConstraint
    ) -> Option<VectorCompareFnRelative<N>>
    where
        Simd<PrimitiveType, E>: SimdPartialOrd + SimdInt,
    {
        Some(Arc::new(move |current_values_ptr, previous_values_ptr| {
            let current_values = SimdInt::swap_bytes(Simd::from_array(unsafe {
                ptr::read_unaligned(current_values_ptr as *const [PrimitiveType; E])
            }));
            let previous_values: Simd<PrimitiveType, E> =
                SimdInt::swap_bytes(Simd::splat(unsafe { ptr::read_unaligned(previous_values_ptr as *const PrimitiveType) }));

            VectorGenerics::transmute_mask(current_values.simd_gt(previous_values))
        }))
    }

    pub fn get_vector_compare_increased_unsigned<const N: usize, const E: usize, PrimitiveType: SimdElement + Send + Sync + 'static>(
        _scan_constraint: &ScanConstraint
    ) -> Option<VectorCompareFnRelative<N>>
    where
        Simd<PrimitiveType, E>: SimdPartialOrd + SimdUint,
    {
        Some(Arc::new(move |current_values_ptr, previous_values_ptr| {
            let current_values = SimdUint::swap_bytes(Simd::from_array(unsafe {
                ptr::read_unaligned(current_values_ptr as *const [PrimitiveType; E])
            }));
            let previous_values: Simd<PrimitiveType, E> =
                SimdUint::swap_bytes(Simd::splat(unsafe { ptr::read_unaligned(previous_values_ptr as *const PrimitiveType) }));

            VectorGenerics::transmute_mask(current_values.simd_gt(previous_values))
        }))
    }

    pub fn get_vector_compare_decreased<const N: usize, const E: usize, PrimitiveType: SimdElement + Send + Sync + 'static>(
        _scan_constraint: &ScanConstraint
    ) -> Option<VectorCompareFnRelative<N>>
    where
        Simd<PrimitiveType, E>: SimdPartialOrd + SimdInt,
    {
        Some(Arc::new(move |current_values_ptr, previous_values_ptr| {
            let current_values = SimdInt::swap_bytes(Simd::from_array(unsafe {
                ptr::read_unaligned(current_values_ptr as *const [PrimitiveType; E])
            }));
            let previous_values: Simd<PrimitiveType, E> =
                SimdInt::swap_bytes(Simd::splat(unsafe { ptr::read_unaligned(previous_values_ptr as *const PrimitiveType) }));

            VectorGenerics::transmute_mask(current_values.simd_lt(previous_values))
        }))
    }

    pub fn get_vector_compare_decreased_unsigned<const N: usize, const E: usize, PrimitiveType: SimdElement + Send + Sync + 'static>(
        _scan_constraint: &ScanConstraint
    ) -> Option<VectorCompareFnRelative<N>>
    where
        Simd<PrimitiveType, E>: SimdPartialOrd + SimdUint,
    {
        Some(Arc::new(move |current_values_ptr, previous_values_ptr| {
            let current_values = SimdUint::swap_bytes(Simd::from_array(unsafe {
                ptr::read_unaligned(current_values_ptr as *const [PrimitiveType; E])
            }));
            let previous_values: Simd<PrimitiveType, E> =
                SimdUint::swap_bytes(Simd::splat(unsafe { ptr::read_unaligned(previous_values_ptr as *const PrimitiveType) }));

            VectorGenerics::transmute_mask(current_values.simd_lt(previous_values))
        }))
    }

    pub fn get_vector_compare_increased_by<const N: usize, const E: usize, PrimitiveType: SimdElement + Send + Sync + 'static>(
        scan_constraint: &ScanConstraint
    ) -> Option<VectorCompareFnDelta<N>>
    where
        Simd<PrimitiveType, E>: SimdPartialEq + SimdInt + Add<Simd<PrimitiveType, E>, Output = Simd<PrimitiveType, E>>,
    {
        let immediate_value = scan_constraint.get_data_value();
        let delta_value_ptr = immediate_value.as_ptr();
        let delta_value = SimdInt::swap_bytes(Simd::splat(unsafe { ptr::read_unaligned(delta_value_ptr as *const PrimitiveType) }));

        Some(Arc::new(move |current_values_ptr, previous_values_ptr| {
            let current_values = SimdInt::swap_bytes(Simd::from_array(unsafe {
                ptr::read_unaligned(current_values_ptr as *const [PrimitiveType; E])
            }));
            let previous_values = SimdInt::swap_bytes(Simd::from_array(unsafe {
                ptr::read_unaligned(previous_values_ptr as *const [PrimitiveType; E])
            }));
            let target_values = previous_values.add(delta_value);

            VectorGenerics::transmute_mask(current_values.simd_eq(target_values))
        }))
    }

    pub fn get_vector_compare_increased_by_unsigned<const N: usize, const E: usize, PrimitiveType: SimdElement + Send + Sync + 'static>(
        scan_constraint: &ScanConstraint
    ) -> Option<VectorCompareFnDelta<N>>
    where
        Simd<PrimitiveType, E>: SimdPartialEq + SimdUint + Add<Simd<PrimitiveType, E>, Output = Simd<PrimitiveType, E>>,
    {
        let immediate_value = scan_constraint.get_data_value();
        let delta_value_ptr = immediate_value.as_ptr();
        let delta_value = SimdUint::swap_bytes(Simd::splat(unsafe { ptr::read_unaligned(delta_value_ptr as *const PrimitiveType) }));

        Some(Arc::new(move |current_values_ptr, previous_values_ptr| {
            let current_values = SimdUint::swap_bytes(Simd::from_array(unsafe {
                ptr::read_unaligned(current_values_ptr as *const [PrimitiveType; E])
            }));
            let previous_values = SimdUint::swap_bytes(Simd::from_array(unsafe {
                ptr::read_unaligned(previous_values_ptr as *const [PrimitiveType; E])
            }));
            let target_values = previous_values.add(delta_value);

            VectorGenerics::transmute_mask(current_values.simd_eq(target_values))
        }))
    }

    pub fn get_vector_compare_decreased_by<const N: usize, const E: usize, PrimitiveType: SimdElement + Send + Sync + 'static>(
        scan_constraint: &ScanConstraint
    ) -> Option<VectorCompareFnDelta<N>>
    where
        Simd<PrimitiveType, E>: SimdPartialEq + SimdInt + Sub<Simd<PrimitiveType, E>, Output = Simd<PrimitiveType, E>>,
    {
        let immediate_value = scan_constraint.get_data_value();
        let delta_value_ptr = immediate_value.as_ptr();
        let delta_value = SimdInt::swap_bytes(Simd::splat(unsafe { ptr::read_unaligned(delta_value_ptr as *const PrimitiveType) }));

        Some(Arc::new(move |current_values_ptr, previous_values_ptr| {
            let current_values = SimdInt::swap_bytes(Simd::from_array(unsafe {
                ptr::read_unaligned(current_values_ptr as *const [PrimitiveType; E])
            }));
            let previous_values = SimdInt::swap_bytes(Simd::from_array(unsafe {
                ptr::read_unaligned(previous_values_ptr as *const [PrimitiveType; E])
            }));
            let target_values = previous_values.sub(delta_value);

            VectorGenerics::transmute_mask(current_values.simd_eq(target_values))
        }))
    }

    pub fn get_vector_compare_decreased_by_unsigned<const N: usize, const E: usize, PrimitiveType: SimdElement + Send + Sync + 'static>(
        scan_constraint: &ScanConstraint
    ) -> Option<VectorCompareFnDelta<N>>
    where
        Simd<PrimitiveType, E>: SimdPartialEq + SimdUint + Sub<Simd<PrimitiveType, E>, Output = Simd<PrimitiveType, E>>,
    {
        let immediate_value = scan_constraint.get_data_value();
        let delta_value_ptr = immediate_value.as_ptr();
        let delta_value = SimdUint::swap_bytes(Simd::splat(unsafe { ptr::read_unaligned(delta_value_ptr as *const PrimitiveType) }));

        Some(Arc::new(move |current_values_ptr, previous_values_ptr| {
            let current_values = SimdUint::swap_bytes(Simd::from_array(unsafe {
                ptr::read_unaligned(current_values_ptr as *const [PrimitiveType; E])
            }));
            let previous_values = SimdUint::swap_bytes(Simd::from_array(unsafe {
                ptr::read_unaligned(previous_values_ptr as *const [PrimitiveType; E])
            }));
            let target_values = previous_values.sub(delta_value);

            VectorGenerics::transmute_mask(current_values.simd_eq(target_values))
        }))
    }

    pub fn get_vector_compare_multiplied_by<const N: usize, const E: usize, PrimitiveType: SimdElement + Send + Sync + 'static>(
        scan_constraint: &ScanConstraint
    ) -> Option<VectorCompareFnDelta<N>>
    where
        Simd<PrimitiveType, E>: SimdPartialEq + SimdInt + Mul<Simd<PrimitiveType, E>, Output = Simd<PrimitiveType, E>>,
    {
        let immediate_value = scan_constraint.get_data_value();
        let delta_value_ptr = immediate_value.as_ptr();
        let delta_value = SimdInt::swap_bytes(Simd::splat(unsafe { ptr::read_unaligned(delta_value_ptr as *const PrimitiveType) }));

        Some(Arc::new(move |current_values_ptr, previous_values_ptr| {
            let current_values = SimdInt::swap_bytes(Simd::from_array(unsafe {
                ptr::read_unaligned(current_values_ptr as *const [PrimitiveType; E])
            }));
            let previous_values = SimdInt::swap_bytes(Simd::from_array(unsafe {
                ptr::read_unaligned(previous_values_ptr as *const [PrimitiveType; E])
            }));
            let target_values = previous_values.mul(delta_value);

            VectorGenerics::transmute_mask(current_values.simd_eq(target_values))
        }))
    }

    pub fn get_vector_compare_multiplied_by_unsigned<const N: usize, const E: usize, PrimitiveType: SimdElement + Send + Sync + 'static>(
        scan_constraint: &ScanConstraint
    ) -> Option<VectorCompareFnDelta<N>>
    where
        Simd<PrimitiveType, E>: SimdPartialEq + SimdUint + Mul<Simd<PrimitiveType, E>, Output = Simd<PrimitiveType, E>>,
    {
        let immediate_value = scan_constraint.get_data_value();
        let delta_value_ptr = immediate_value.as_ptr();
        let delta_value = SimdUint::swap_bytes(Simd::splat(unsafe { ptr::read_unaligned(delta_value_ptr as *const PrimitiveType) }));

        Some(Arc::new(move |current_values_ptr, previous_values_ptr| {
            let current_values = SimdUint::swap_bytes(Simd::from_array(unsafe {
                ptr::read_unaligned(current_values_ptr as *const [PrimitiveType; E])
            }));
            let previous_values = SimdUint::swap_bytes(Simd::from_array(unsafe {
                ptr::read_unaligned(previous_values_ptr as *const [PrimitiveType; E])
            }));
            let target_values = previous_values.mul(delta_value);

            VectorGenerics::transmute_mask(current_values.simd_eq(target_values))
        }))
    }

    pub fn get_vector_compare_divided_by<const N: usize, const E: usize, PrimitiveType: SimdElement + PartialEq + Default + Send + Sync + 'static>(
        scan_constraint: &ScanConstraint
    ) -> Option<VectorCompareFnDelta<N>>
    where
        Simd<PrimitiveType, E>: SimdPartialEq + SimdInt + Div<Simd<PrimitiveType, E>, Output = Simd<PrimitiveType, E>>,
    {
        let immediate_value = scan_constraint.get_data_value();
        let delta_value_ptr = immediate_value.as_ptr();
        let primitive_value = unsafe { ptr::read_unaligned(delta_value_ptr as *const PrimitiveType) };

        // Disallow divide by zero.
        if primitive_value == PrimitiveType::default() {
            return None;
        }

        let delta_value = SimdInt::swap_bytes(Simd::splat(primitive_value));

        Some(Arc::new(move |current_values_ptr, previous_values_ptr| {
            let current_values = SimdInt::swap_bytes(Simd::from_array(unsafe {
                ptr::read_unaligned(current_values_ptr as *const [PrimitiveType; E])
            }));
            let previous_values = SimdInt::swap_bytes(Simd::from_array(unsafe {
                ptr::read_unaligned(previous_values_ptr as *const [PrimitiveType; E])
            }));
            let target_values = previous_values.div(delta_value);

            VectorGenerics::transmute_mask(current_values.simd_eq(target_values))
        }))
    }

    pub fn get_vector_compare_divided_by_unsigned<const N: usize, const E: usize, PrimitiveType: SimdElement + PartialEq + Default + Send + Sync + 'static>(
        scan_constraint: &ScanConstraint
    ) -> Option<VectorCompareFnDelta<N>>
    where
        Simd<PrimitiveType, E>: SimdPartialEq + SimdUint + Div<Simd<PrimitiveType, E>, Output = Simd<PrimitiveType, E>>,
    {
        let immediate_value = scan_constraint.get_data_value();
        let delta_value_ptr = immediate_value.as_ptr();
        let primitive_value = unsafe { ptr::read_unaligned(delta_value_ptr as *const PrimitiveType) };

        // Disallow divide by zero.
        if primitive_value == PrimitiveType::default() {
            return None;
        }

        let delta_value = SimdUint::swap_bytes(Simd::splat(primitive_value));

        Some(Arc::new(move |current_values_ptr, previous_values_ptr| {
            let current_values = SimdUint::swap_bytes(Simd::from_array(unsafe {
                ptr::read_unaligned(current_values_ptr as *const [PrimitiveType; E])
            }));
            let previous_values = SimdUint::swap_bytes(Simd::from_array(unsafe {
                ptr::read_unaligned(previous_values_ptr as *const [PrimitiveType; E])
            }));
            let target_values = previous_values.div(delta_value);

            VectorGenerics::transmute_mask(current_values.simd_eq(target_values))
        }))
    }

    pub fn get_vector_compare_modulo_by<const N: usize, const E: usize, PrimitiveType: SimdElement + PartialEq + Default + Send + Sync + 'static>(
        scan_constraint: &ScanConstraint
    ) -> Option<VectorCompareFnDelta<N>>
    where
        Simd<PrimitiveType, E>: SimdPartialEq + SimdInt + Rem<Simd<PrimitiveType, E>, Output = Simd<PrimitiveType, E>>,
    {
        let immediate_value = scan_constraint.get_data_value();
        let delta_value_ptr = immediate_value.as_ptr();
        let primitive_value = unsafe { ptr::read_unaligned(delta_value_ptr as *const PrimitiveType) };

        // Disallow divide by zero.
        if primitive_value == PrimitiveType::default() {
            return None;
        }

        let delta_value = SimdInt::swap_bytes(Simd::splat(primitive_value));

        Some(Arc::new(move |current_values_ptr, previous_values_ptr| {
            let current_values = SimdInt::swap_bytes(Simd::from_array(unsafe {
                ptr::read_unaligned(current_values_ptr as *const [PrimitiveType; E])
            }));
            let previous_values = SimdInt::swap_bytes(Simd::from_array(unsafe {
                ptr::read_unaligned(previous_values_ptr as *const [PrimitiveType; E])
            }));
            let target_values = previous_values.rem(delta_value);

            VectorGenerics::transmute_mask(current_values.simd_eq(target_values))
        }))
    }

    pub fn get_vector_compare_modulo_by_unsigned<const N: usize, const E: usize, PrimitiveType: SimdElement + PartialEq + Default + Send + Sync + 'static>(
        scan_constraint: &ScanConstraint
    ) -> Option<VectorCompareFnDelta<N>>
    where
        Simd<PrimitiveType, E>: SimdPartialEq + SimdUint + Rem<Simd<PrimitiveType, E>, Output = Simd<PrimitiveType, E>>,
    {
        let immediate_value = scan_constraint.get_data_value();
        let delta_value_ptr = immediate_value.as_ptr();
        let primitive_value = unsafe { ptr::read_unaligned(delta_value_ptr as *const PrimitiveType) };

        // Disallow divide by zero.
        if primitive_value == PrimitiveType::default() {
            return None;
        }

        let delta_value = SimdUint::swap_bytes(Simd::splat(primitive_value));

        Some(Arc::new(move |current_values_ptr, previous_values_ptr| {
            let current_values = SimdUint::swap_bytes(Simd::from_array(unsafe {
                ptr::read_unaligned(current_values_ptr as *const [PrimitiveType; E])
            }));
            let previous_values = SimdUint::swap_bytes(Simd::from_array(unsafe {
                ptr::read_unaligned(previous_values_ptr as *const [PrimitiveType; E])
            }));
            let target_values = previous_values.rem(delta_value);

            VectorGenerics::transmute_mask(current_values.simd_eq(target_values))
        }))
    }

    pub fn get_vector_compare_shift_left_by<const N: usize, const E: usize, PrimitiveType: SimdElement + Send + Sync + 'static>(
        scan_constraint: &ScanConstraint
    ) -> Option<VectorCompareFnDelta<N>>
    where
        Simd<PrimitiveType, E>: SimdPartialEq + SimdInt + Shl<Simd<PrimitiveType, E>, Output = Simd<PrimitiveType, E>>,
    {
        let immediate_value = scan_constraint.get_data_value();
        let delta_value_ptr = immediate_value.as_ptr();
        let delta_value = SimdInt::swap_bytes(Simd::splat(unsafe { ptr::read_unaligned(delta_value_ptr as *const PrimitiveType) }));

        Some(Arc::new(move |current_values_ptr, previous_values_ptr| {
            let current_values = SimdInt::swap_bytes(Simd::from_array(unsafe {
                ptr::read_unaligned(current_values_ptr as *const [PrimitiveType; E])
            }));
            let previous_values = SimdInt::swap_bytes(Simd::from_array(unsafe {
                ptr::read_unaligned(previous_values_ptr as *const [PrimitiveType; E])
            }));
            let target_values = previous_values.shl(delta_value);

            VectorGenerics::transmute_mask(current_values.simd_eq(target_values))
        }))
    }

    pub fn get_vector_compare_shift_left_by_unsigned<const N: usize, const E: usize, PrimitiveType: SimdElement + Send + Sync + 'static>(
        scan_constraint: &ScanConstraint
    ) -> Option<VectorCompareFnDelta<N>>
    where
        Simd<PrimitiveType, E>: SimdPartialEq + SimdUint + Shl<Simd<PrimitiveType, E>, Output = Simd<PrimitiveType, E>>,
    {
        let immediate_value = scan_constraint.get_data_value();
        let delta_value_ptr = immediate_value.as_ptr();
        let delta_value = SimdUint::swap_bytes(Simd::splat(unsafe { ptr::read_unaligned(delta_value_ptr as *const PrimitiveType) }));

        Some(Arc::new(move |current_values_ptr, previous_values_ptr| {
            let current_values = SimdUint::swap_bytes(Simd::from_array(unsafe {
                ptr::read_unaligned(current_values_ptr as *const [PrimitiveType; E])
            }));
            let previous_values = SimdUint::swap_bytes(Simd::from_array(unsafe {
                ptr::read_unaligned(previous_values_ptr as *const [PrimitiveType; E])
            }));
            let target_values = previous_values.shl(delta_value);

            VectorGenerics::transmute_mask(current_values.simd_eq(target_values))
        }))
    }

    pub fn get_vector_compare_shift_right_by<const N: usize, const E: usize, PrimitiveType: SimdElement + Send + Sync + 'static>(
        scan_constraint: &ScanConstraint
    ) -> Option<VectorCompareFnDelta<N>>
    where
        Simd<PrimitiveType, E>: SimdPartialEq + SimdInt + Shr<Simd<PrimitiveType, E>, Output = Simd<PrimitiveType, E>>,
    {
        let immediate_value = scan_constraint.get_data_value();
        let delta_value_ptr = immediate_value.as_ptr();
        let delta_value = SimdInt::swap_bytes(Simd::splat(unsafe { ptr::read_unaligned(delta_value_ptr as *const PrimitiveType) }));

        Some(Arc::new(move |current_values_ptr, previous_values_ptr| {
            let current_values = SimdInt::swap_bytes(Simd::from_array(unsafe {
                ptr::read_unaligned(current_values_ptr as *const [PrimitiveType; E])
            }));
            let previous_values = SimdInt::swap_bytes(Simd::from_array(unsafe {
                ptr::read_unaligned(previous_values_ptr as *const [PrimitiveType; E])
            }));
            let target_values = previous_values.shr(delta_value);

            VectorGenerics::transmute_mask(current_values.simd_eq(target_values))
        }))
    }

    pub fn get_vector_compare_shift_right_by_unsigned<const N: usize, const E: usize, PrimitiveType: SimdElement + Send + Sync + 'static>(
        scan_constraint: &ScanConstraint
    ) -> Option<VectorCompareFnDelta<N>>
    where
        Simd<PrimitiveType, E>: SimdPartialEq + SimdUint + Shr<Simd<PrimitiveType, E>, Output = Simd<PrimitiveType, E>>,
    {
        let immediate_value = scan_constraint.get_data_value();
        let delta_value_ptr = immediate_value.as_ptr();
        let delta_value = SimdUint::swap_bytes(Simd::splat(unsafe { ptr::read_unaligned(delta_value_ptr as *const PrimitiveType) }));

        Some(Arc::new(move |current_values_ptr, previous_values_ptr| {
            let current_values = SimdUint::swap_bytes(Simd::from_array(unsafe {
                ptr::read_unaligned(current_values_ptr as *const [PrimitiveType; E])
            }));
            let previous_values = SimdUint::swap_bytes(Simd::from_array(unsafe {
                ptr::read_unaligned(previous_values_ptr as *const [PrimitiveType; E])
            }));
            let target_values = previous_values.shr(delta_value);

            VectorGenerics::transmute_mask(current_values.simd_eq(target_values))
        }))
    }

    pub fn get_vector_compare_logical_and_by<const N: usize, const E: usize, PrimitiveType: SimdElement + Send + Sync + 'static>(
        scan_constraint: &ScanConstraint
    ) -> Option<VectorCompareFnDelta<N>>
    where
        Simd<PrimitiveType, E>: SimdPartialEq + SimdInt + BitAnd<Simd<PrimitiveType, E>, Output = Simd<PrimitiveType, E>>,
    {
        let immediate_value = scan_constraint.get_data_value();
        let delta_value_ptr = immediate_value.as_ptr();
        let delta_value = SimdInt::swap_bytes(Simd::splat(unsafe { ptr::read_unaligned(delta_value_ptr as *const PrimitiveType) }));

        Some(Arc::new(move |current_values_ptr, previous_values_ptr| {
            let current_values = SimdInt::swap_bytes(Simd::from_array(unsafe {
                ptr::read_unaligned(current_values_ptr as *const [PrimitiveType; E])
            }));
            let previous_values = SimdInt::swap_bytes(Simd::from_array(unsafe {
                ptr::read_unaligned(previous_values_ptr as *const [PrimitiveType; E])
            }));
            let target_values = previous_values.bitand(delta_value);

            VectorGenerics::transmute_mask(current_values.simd_eq(target_values))
        }))
    }

    pub fn get_vector_compare_logical_and_by_unsigned<const N: usize, const E: usize, PrimitiveType: SimdElement + Send + Sync + 'static>(
        scan_constraint: &ScanConstraint
    ) -> Option<VectorCompareFnDelta<N>>
    where
        Simd<PrimitiveType, E>: SimdPartialEq + SimdUint + BitAnd<Simd<PrimitiveType, E>, Output = Simd<PrimitiveType, E>>,
    {
        let immediate_value = scan_constraint.get_data_value();
        let delta_value_ptr = immediate_value.as_ptr();
        let delta_value = SimdUint::swap_bytes(Simd::splat(unsafe { ptr::read_unaligned(delta_value_ptr as *const PrimitiveType) }));

        Some(Arc::new(move |current_values_ptr, previous_values_ptr| {
            let current_values = SimdUint::swap_bytes(Simd::from_array(unsafe {
                ptr::read_unaligned(current_values_ptr as *const [PrimitiveType; E])
            }));
            let previous_values = SimdUint::swap_bytes(Simd::from_array(unsafe {
                ptr::read_unaligned(previous_values_ptr as *const [PrimitiveType; E])
            }));
            let target_values = previous_values.bitand(delta_value);

            VectorGenerics::transmute_mask(current_values.simd_eq(target_values))
        }))
    }

    pub fn get_vector_compare_logical_or_by<const N: usize, const E: usize, PrimitiveType: SimdElement + Send + Sync + 'static>(
        scan_constraint: &ScanConstraint
    ) -> Option<VectorCompareFnDelta<N>>
    where
        Simd<PrimitiveType, E>: SimdPartialEq + SimdInt + BitOr<Simd<PrimitiveType, E>, Output = Simd<PrimitiveType, E>>,
    {
        let immediate_value = scan_constraint.get_data_value();
        let delta_value_ptr = immediate_value.as_ptr();
        let delta_value = SimdInt::swap_bytes(Simd::splat(unsafe { ptr::read_unaligned(delta_value_ptr as *const PrimitiveType) }));

        Some(Arc::new(move |current_values_ptr, previous_values_ptr| {
            let current_values = SimdInt::swap_bytes(Simd::from_array(unsafe {
                ptr::read_unaligned(current_values_ptr as *const [PrimitiveType; E])
            }));
            let previous_values = SimdInt::swap_bytes(Simd::from_array(unsafe {
                ptr::read_unaligned(previous_values_ptr as *const [PrimitiveType; E])
            }));
            let target_values = previous_values.bitor(delta_value);

            VectorGenerics::transmute_mask(current_values.simd_eq(target_values))
        }))
    }

    pub fn get_vector_compare_logical_or_by_unsigned<const N: usize, const E: usize, PrimitiveType: SimdElement + Send + Sync + 'static>(
        scan_constraint: &ScanConstraint
    ) -> Option<VectorCompareFnDelta<N>>
    where
        Simd<PrimitiveType, E>: SimdPartialEq + SimdUint + BitOr<Simd<PrimitiveType, E>, Output = Simd<PrimitiveType, E>>,
    {
        let immediate_value = scan_constraint.get_data_value();
        let delta_value_ptr = immediate_value.as_ptr();
        let delta_value = SimdUint::swap_bytes(Simd::splat(unsafe { ptr::read_unaligned(delta_value_ptr as *const PrimitiveType) }));

        Some(Arc::new(move |current_values_ptr, previous_values_ptr| {
            let current_values = SimdUint::swap_bytes(Simd::from_array(unsafe {
                ptr::read_unaligned(current_values_ptr as *const [PrimitiveType; E])
            }));
            let previous_values = SimdUint::swap_bytes(Simd::from_array(unsafe {
                ptr::read_unaligned(previous_values_ptr as *const [PrimitiveType; E])
            }));
            let target_values = previous_values.bitor(delta_value);

            VectorGenerics::transmute_mask(current_values.simd_eq(target_values))
        }))
    }

    pub fn get_vector_compare_logical_xor_by<const N: usize, const E: usize, PrimitiveType: SimdElement + Send + Sync + 'static>(
        scan_constraint: &ScanConstraint
    ) -> Option<VectorCompareFnDelta<N>>
    where
        Simd<PrimitiveType, E>: SimdPartialEq + SimdInt + BitXor<Simd<PrimitiveType, E>, Output = Simd<PrimitiveType, E>>,
    {
        let immediate_value = scan_constraint.get_data_value();
        let delta_value_ptr = immediate_value.as_ptr();
        let delta_value = SimdInt::swap_bytes(Simd::splat(unsafe { ptr::read_unaligned(delta_value_ptr as *const PrimitiveType) }));

        Some(Arc::new(move |current_values_ptr, previous_values_ptr| {
            let current_values = SimdInt::swap_bytes(Simd::from_array(unsafe {
                ptr::read_unaligned(current_values_ptr as *const [PrimitiveType; E])
            }));
            let previous_values = SimdInt::swap_bytes(Simd::from_array(unsafe {
                ptr::read_unaligned(previous_values_ptr as *const [PrimitiveType; E])
            }));
            let target_values = previous_values.bitxor(delta_value);

            VectorGenerics::transmute_mask(current_values.simd_eq(target_values))
        }))
    }

    pub fn get_vector_compare_logical_xor_by_unsigned<const N: usize, const E: usize, PrimitiveType: SimdElement + Send + Sync + 'static>(
        scan_constraint: &ScanConstraint
    ) -> Option<VectorCompareFnDelta<N>>
    where
        Simd<PrimitiveType, E>: SimdPartialEq + SimdUint + BitXor<Simd<PrimitiveType, E>, Output = Simd<PrimitiveType, E>>,
    {
        let immediate_value = scan_constraint.get_data_value();
        let delta_value_ptr = immediate_value.as_ptr();
        let delta_value = SimdUint::swap_bytes(Simd::splat(unsafe { ptr::read_unaligned(delta_value_ptr as *const PrimitiveType) }));

        Some(Arc::new(move |current_values_ptr, previous_values_ptr| {
            let current_values = SimdUint::swap_bytes(Simd::from_array(unsafe {
                ptr::read_unaligned(current_values_ptr as *const [PrimitiveType; E])
            }));
            let previous_values = SimdUint::swap_bytes(Simd::from_array(unsafe {
                ptr::read_unaligned(previous_values_ptr as *const [PrimitiveType; E])
            }));
            let target_values = previous_values.bitxor(delta_value);

            VectorGenerics::transmute_mask(current_values.simd_eq(target_values))
        }))
    }
}
