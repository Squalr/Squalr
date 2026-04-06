use crate::data_types::primitive_data_type_24_bit::PrimitiveDataType24Bit;
use squalr_engine_api::structures::memory::endian::Endian;
use squalr_engine_api::structures::scanning::comparisons::scan_function_vector::{VectorCompareFnDelta, VectorCompareFnImmediate, VectorCompareFnRelative};
use squalr_engine_api::structures::scanning::constraints::scan_constraint::ScanConstraint;
use std::simd::Simd;
use std::sync::Arc;

const DATA_TYPE_SIZE_BYTES: usize = 3;
const TWENTY_FOUR_BIT_MASK: u32 = 0x00FF_FFFF;
const TWENTY_FOUR_BIT_SIGN_BIT: u32 = 0x0080_0000;

type ImmediateUnsignedOperation = fn(u32, u32) -> bool;
type ImmediateSignedOperation = fn(i32, i32) -> bool;
type RelativeUnsignedOperation = fn(u32, u32) -> bool;
type RelativeSignedOperation = fn(i32, i32) -> bool;
type DeltaUnsignedOperation = fn(u32, u32) -> Option<u32>;
type DeltaSignedOperation = fn(i32, i32) -> Option<i32>;

fn build_compare_mask<const N: usize>(mut compare_offset: impl FnMut(usize) -> bool) -> Simd<u8, N> {
    let mut compare_mask = [0u8; N];

    for byte_offset in (0..N).step_by(DATA_TYPE_SIZE_BYTES) {
        if compare_offset(byte_offset) {
            compare_mask[byte_offset] = 0xFF;
        }
    }

    Simd::from_array(compare_mask)
}

fn raw_to_signed(raw_value: u32) -> i32 {
    let masked_value = raw_value & TWENTY_FOUR_BIT_MASK;

    if masked_value & TWENTY_FOUR_BIT_SIGN_BIT != 0 {
        (masked_value | !TWENTY_FOUR_BIT_MASK) as i32
    } else {
        masked_value as i32
    }
}

fn signed_to_raw(value: i32) -> u32 {
    (value as u32) & TWENTY_FOUR_BIT_MASK
}

fn normalize_signed(value: i32) -> i32 {
    raw_to_signed(signed_to_raw(value))
}

fn build_vector_compare_immediate_unsigned<const N: usize>(
    immediate_value: u32,
    endian: Endian,
    operation: ImmediateUnsignedOperation,
) -> Option<VectorCompareFnImmediate<N>> {
    Some(Arc::new(move |current_values_ptr| {
        build_compare_mask::<N>(|byte_offset| unsafe {
            let current_value = PrimitiveDataType24Bit::read_unsigned_unchecked(current_values_ptr.add(byte_offset), endian);

            operation(current_value, immediate_value)
        })
    }))
}

fn build_vector_compare_immediate_signed<const N: usize>(
    immediate_value: i32,
    endian: Endian,
    operation: ImmediateSignedOperation,
) -> Option<VectorCompareFnImmediate<N>> {
    Some(Arc::new(move |current_values_ptr| {
        build_compare_mask::<N>(|byte_offset| unsafe {
            let current_value = PrimitiveDataType24Bit::read_signed_unchecked(current_values_ptr.add(byte_offset), endian);

            operation(current_value, immediate_value)
        })
    }))
}

fn build_vector_compare_relative_unsigned<const N: usize>(
    endian: Endian,
    operation: RelativeUnsignedOperation,
) -> Option<VectorCompareFnRelative<N>> {
    Some(Arc::new(move |current_values_ptr, previous_values_ptr| {
        build_compare_mask::<N>(|byte_offset| unsafe {
            let current_value = PrimitiveDataType24Bit::read_unsigned_unchecked(current_values_ptr.add(byte_offset), endian);
            let previous_value = PrimitiveDataType24Bit::read_unsigned_unchecked(previous_values_ptr.add(byte_offset), endian);

            operation(current_value, previous_value)
        })
    }))
}

fn build_vector_compare_relative_signed<const N: usize>(
    endian: Endian,
    operation: RelativeSignedOperation,
) -> Option<VectorCompareFnRelative<N>> {
    Some(Arc::new(move |current_values_ptr, previous_values_ptr| {
        build_compare_mask::<N>(|byte_offset| unsafe {
            let current_value = PrimitiveDataType24Bit::read_signed_unchecked(current_values_ptr.add(byte_offset), endian);
            let previous_value = PrimitiveDataType24Bit::read_signed_unchecked(previous_values_ptr.add(byte_offset), endian);

            operation(current_value, previous_value)
        })
    }))
}

fn build_vector_compare_delta_unsigned<const N: usize>(
    delta_value: u32,
    endian: Endian,
    operation: DeltaUnsignedOperation,
) -> Option<VectorCompareFnDelta<N>> {
    Some(Arc::new(move |current_values_ptr, previous_values_ptr| {
        build_compare_mask::<N>(|byte_offset| unsafe {
            let current_value = PrimitiveDataType24Bit::read_unsigned_unchecked(current_values_ptr.add(byte_offset), endian);
            let previous_value = PrimitiveDataType24Bit::read_unsigned_unchecked(previous_values_ptr.add(byte_offset), endian);

            operation(previous_value, delta_value)
                .map(|target_value| current_value == target_value)
                .unwrap_or(false)
        })
    }))
}

fn build_vector_compare_delta_signed<const N: usize>(
    delta_value: i32,
    endian: Endian,
    operation: DeltaSignedOperation,
) -> Option<VectorCompareFnDelta<N>> {
    Some(Arc::new(move |current_values_ptr, previous_values_ptr| {
        build_compare_mask::<N>(|byte_offset| unsafe {
            let current_value = PrimitiveDataType24Bit::read_signed_unchecked(current_values_ptr.add(byte_offset), endian);
            let previous_value = PrimitiveDataType24Bit::read_signed_unchecked(previous_values_ptr.add(byte_offset), endian);

            operation(previous_value, delta_value)
                .map(|target_value| current_value == target_value)
                .unwrap_or(false)
        })
    }))
}

pub fn get_vector_compare_equal_unsigned<const N: usize>(
    scan_constraint: &ScanConstraint,
    endian: Endian,
) -> Option<VectorCompareFnImmediate<N>> {
    let immediate_value = PrimitiveDataType24Bit::read_unsigned(scan_constraint.get_data_value().get_value_bytes(), endian).ok()?;

    build_vector_compare_immediate_unsigned(immediate_value, endian, |current_value, target_value| current_value == target_value)
}

pub fn get_vector_compare_not_equal_unsigned<const N: usize>(
    scan_constraint: &ScanConstraint,
    endian: Endian,
) -> Option<VectorCompareFnImmediate<N>> {
    let immediate_value = PrimitiveDataType24Bit::read_unsigned(scan_constraint.get_data_value().get_value_bytes(), endian).ok()?;

    build_vector_compare_immediate_unsigned(immediate_value, endian, |current_value, target_value| current_value != target_value)
}

pub fn get_vector_compare_greater_than_unsigned<const N: usize>(
    scan_constraint: &ScanConstraint,
    endian: Endian,
) -> Option<VectorCompareFnImmediate<N>> {
    let immediate_value = PrimitiveDataType24Bit::read_unsigned(scan_constraint.get_data_value().get_value_bytes(), endian).ok()?;

    build_vector_compare_immediate_unsigned(immediate_value, endian, |current_value, target_value| current_value > target_value)
}

pub fn get_vector_compare_greater_than_or_equal_unsigned<const N: usize>(
    scan_constraint: &ScanConstraint,
    endian: Endian,
) -> Option<VectorCompareFnImmediate<N>> {
    let immediate_value = PrimitiveDataType24Bit::read_unsigned(scan_constraint.get_data_value().get_value_bytes(), endian).ok()?;

    build_vector_compare_immediate_unsigned(immediate_value, endian, |current_value, target_value| current_value >= target_value)
}

pub fn get_vector_compare_less_than_unsigned<const N: usize>(
    scan_constraint: &ScanConstraint,
    endian: Endian,
) -> Option<VectorCompareFnImmediate<N>> {
    let immediate_value = PrimitiveDataType24Bit::read_unsigned(scan_constraint.get_data_value().get_value_bytes(), endian).ok()?;

    build_vector_compare_immediate_unsigned(immediate_value, endian, |current_value, target_value| current_value < target_value)
}

pub fn get_vector_compare_less_than_or_equal_unsigned<const N: usize>(
    scan_constraint: &ScanConstraint,
    endian: Endian,
) -> Option<VectorCompareFnImmediate<N>> {
    let immediate_value = PrimitiveDataType24Bit::read_unsigned(scan_constraint.get_data_value().get_value_bytes(), endian).ok()?;

    build_vector_compare_immediate_unsigned(immediate_value, endian, |current_value, target_value| current_value <= target_value)
}

pub fn get_vector_compare_changed_unsigned<const N: usize>(
    _scan_constraint: &ScanConstraint,
    endian: Endian,
) -> Option<VectorCompareFnRelative<N>> {
    build_vector_compare_relative_unsigned(endian, |current_value, previous_value| current_value != previous_value)
}

pub fn get_vector_compare_unchanged_unsigned<const N: usize>(
    _scan_constraint: &ScanConstraint,
    endian: Endian,
) -> Option<VectorCompareFnRelative<N>> {
    build_vector_compare_relative_unsigned(endian, |current_value, previous_value| current_value == previous_value)
}

pub fn get_vector_compare_increased_unsigned<const N: usize>(
    _scan_constraint: &ScanConstraint,
    endian: Endian,
) -> Option<VectorCompareFnRelative<N>> {
    build_vector_compare_relative_unsigned(endian, |current_value, previous_value| current_value > previous_value)
}

pub fn get_vector_compare_decreased_unsigned<const N: usize>(
    _scan_constraint: &ScanConstraint,
    endian: Endian,
) -> Option<VectorCompareFnRelative<N>> {
    build_vector_compare_relative_unsigned(endian, |current_value, previous_value| current_value < previous_value)
}

pub fn get_vector_compare_increased_by_unsigned<const N: usize>(
    scan_constraint: &ScanConstraint,
    endian: Endian,
) -> Option<VectorCompareFnDelta<N>> {
    let delta_value = PrimitiveDataType24Bit::read_unsigned(scan_constraint.get_data_value().get_value_bytes(), endian).ok()?;

    build_vector_compare_delta_unsigned(delta_value, endian, |previous_value, delta_value| {
        Some(previous_value.wrapping_add(delta_value) & TWENTY_FOUR_BIT_MASK)
    })
}

pub fn get_vector_compare_decreased_by_unsigned<const N: usize>(
    scan_constraint: &ScanConstraint,
    endian: Endian,
) -> Option<VectorCompareFnDelta<N>> {
    let delta_value = PrimitiveDataType24Bit::read_unsigned(scan_constraint.get_data_value().get_value_bytes(), endian).ok()?;

    build_vector_compare_delta_unsigned(delta_value, endian, |previous_value, delta_value| {
        Some(previous_value.wrapping_sub(delta_value) & TWENTY_FOUR_BIT_MASK)
    })
}

pub fn get_vector_compare_multiplied_by_unsigned<const N: usize>(
    scan_constraint: &ScanConstraint,
    endian: Endian,
) -> Option<VectorCompareFnDelta<N>> {
    let delta_value = PrimitiveDataType24Bit::read_unsigned(scan_constraint.get_data_value().get_value_bytes(), endian).ok()?;

    build_vector_compare_delta_unsigned(delta_value, endian, |previous_value, delta_value| {
        Some(previous_value.wrapping_mul(delta_value) & TWENTY_FOUR_BIT_MASK)
    })
}

pub fn get_vector_compare_divided_by_unsigned<const N: usize>(
    scan_constraint: &ScanConstraint,
    endian: Endian,
) -> Option<VectorCompareFnDelta<N>> {
    let delta_value = PrimitiveDataType24Bit::read_unsigned(scan_constraint.get_data_value().get_value_bytes(), endian).ok()?;

    build_vector_compare_delta_unsigned(
        delta_value,
        endian,
        |previous_value, delta_value| {
            if delta_value == 0 { None } else { Some(previous_value / delta_value) }
        },
    )
}

pub fn get_vector_compare_modulo_by_unsigned<const N: usize>(
    scan_constraint: &ScanConstraint,
    endian: Endian,
) -> Option<VectorCompareFnDelta<N>> {
    let delta_value = PrimitiveDataType24Bit::read_unsigned(scan_constraint.get_data_value().get_value_bytes(), endian).ok()?;

    build_vector_compare_delta_unsigned(
        delta_value,
        endian,
        |previous_value, delta_value| {
            if delta_value == 0 { None } else { Some(previous_value % delta_value) }
        },
    )
}

pub fn get_vector_compare_shift_left_by_unsigned<const N: usize>(
    scan_constraint: &ScanConstraint,
    endian: Endian,
) -> Option<VectorCompareFnDelta<N>> {
    let delta_value = PrimitiveDataType24Bit::read_unsigned(scan_constraint.get_data_value().get_value_bytes(), endian).ok()?;

    build_vector_compare_delta_unsigned(delta_value, endian, |previous_value, delta_value| {
        if delta_value >= 24 {
            None
        } else {
            Some((previous_value << delta_value) & TWENTY_FOUR_BIT_MASK)
        }
    })
}

pub fn get_vector_compare_shift_right_by_unsigned<const N: usize>(
    scan_constraint: &ScanConstraint,
    endian: Endian,
) -> Option<VectorCompareFnDelta<N>> {
    let delta_value = PrimitiveDataType24Bit::read_unsigned(scan_constraint.get_data_value().get_value_bytes(), endian).ok()?;

    build_vector_compare_delta_unsigned(
        delta_value,
        endian,
        |previous_value, delta_value| {
            if delta_value >= 24 { None } else { Some(previous_value >> delta_value) }
        },
    )
}

pub fn get_vector_compare_logical_and_by_unsigned<const N: usize>(
    scan_constraint: &ScanConstraint,
    endian: Endian,
) -> Option<VectorCompareFnDelta<N>> {
    let delta_value = PrimitiveDataType24Bit::read_unsigned(scan_constraint.get_data_value().get_value_bytes(), endian).ok()?;

    build_vector_compare_delta_unsigned(delta_value, endian, |previous_value, delta_value| Some(previous_value & delta_value))
}

pub fn get_vector_compare_logical_or_by_unsigned<const N: usize>(
    scan_constraint: &ScanConstraint,
    endian: Endian,
) -> Option<VectorCompareFnDelta<N>> {
    let delta_value = PrimitiveDataType24Bit::read_unsigned(scan_constraint.get_data_value().get_value_bytes(), endian).ok()?;

    build_vector_compare_delta_unsigned(delta_value, endian, |previous_value, delta_value| {
        Some((previous_value | delta_value) & TWENTY_FOUR_BIT_MASK)
    })
}

pub fn get_vector_compare_logical_xor_by_unsigned<const N: usize>(
    scan_constraint: &ScanConstraint,
    endian: Endian,
) -> Option<VectorCompareFnDelta<N>> {
    let delta_value = PrimitiveDataType24Bit::read_unsigned(scan_constraint.get_data_value().get_value_bytes(), endian).ok()?;

    build_vector_compare_delta_unsigned(delta_value, endian, |previous_value, delta_value| {
        Some((previous_value ^ delta_value) & TWENTY_FOUR_BIT_MASK)
    })
}

pub fn get_vector_compare_equal_signed<const N: usize>(
    scan_constraint: &ScanConstraint,
    endian: Endian,
) -> Option<VectorCompareFnImmediate<N>> {
    let immediate_value = PrimitiveDataType24Bit::read_signed(scan_constraint.get_data_value().get_value_bytes(), endian).ok()?;

    build_vector_compare_immediate_signed(immediate_value, endian, |current_value, target_value| current_value == target_value)
}

pub fn get_vector_compare_not_equal_signed<const N: usize>(
    scan_constraint: &ScanConstraint,
    endian: Endian,
) -> Option<VectorCompareFnImmediate<N>> {
    let immediate_value = PrimitiveDataType24Bit::read_signed(scan_constraint.get_data_value().get_value_bytes(), endian).ok()?;

    build_vector_compare_immediate_signed(immediate_value, endian, |current_value, target_value| current_value != target_value)
}

pub fn get_vector_compare_greater_than_signed<const N: usize>(
    scan_constraint: &ScanConstraint,
    endian: Endian,
) -> Option<VectorCompareFnImmediate<N>> {
    let immediate_value = PrimitiveDataType24Bit::read_signed(scan_constraint.get_data_value().get_value_bytes(), endian).ok()?;

    build_vector_compare_immediate_signed(immediate_value, endian, |current_value, target_value| current_value > target_value)
}

pub fn get_vector_compare_greater_than_or_equal_signed<const N: usize>(
    scan_constraint: &ScanConstraint,
    endian: Endian,
) -> Option<VectorCompareFnImmediate<N>> {
    let immediate_value = PrimitiveDataType24Bit::read_signed(scan_constraint.get_data_value().get_value_bytes(), endian).ok()?;

    build_vector_compare_immediate_signed(immediate_value, endian, |current_value, target_value| current_value >= target_value)
}

pub fn get_vector_compare_less_than_signed<const N: usize>(
    scan_constraint: &ScanConstraint,
    endian: Endian,
) -> Option<VectorCompareFnImmediate<N>> {
    let immediate_value = PrimitiveDataType24Bit::read_signed(scan_constraint.get_data_value().get_value_bytes(), endian).ok()?;

    build_vector_compare_immediate_signed(immediate_value, endian, |current_value, target_value| current_value < target_value)
}

pub fn get_vector_compare_less_than_or_equal_signed<const N: usize>(
    scan_constraint: &ScanConstraint,
    endian: Endian,
) -> Option<VectorCompareFnImmediate<N>> {
    let immediate_value = PrimitiveDataType24Bit::read_signed(scan_constraint.get_data_value().get_value_bytes(), endian).ok()?;

    build_vector_compare_immediate_signed(immediate_value, endian, |current_value, target_value| current_value <= target_value)
}

pub fn get_vector_compare_changed_signed<const N: usize>(
    _scan_constraint: &ScanConstraint,
    endian: Endian,
) -> Option<VectorCompareFnRelative<N>> {
    build_vector_compare_relative_signed(endian, |current_value, previous_value| current_value != previous_value)
}

pub fn get_vector_compare_unchanged_signed<const N: usize>(
    _scan_constraint: &ScanConstraint,
    endian: Endian,
) -> Option<VectorCompareFnRelative<N>> {
    build_vector_compare_relative_signed(endian, |current_value, previous_value| current_value == previous_value)
}

pub fn get_vector_compare_increased_signed<const N: usize>(
    _scan_constraint: &ScanConstraint,
    endian: Endian,
) -> Option<VectorCompareFnRelative<N>> {
    build_vector_compare_relative_signed(endian, |current_value, previous_value| current_value > previous_value)
}

pub fn get_vector_compare_decreased_signed<const N: usize>(
    _scan_constraint: &ScanConstraint,
    endian: Endian,
) -> Option<VectorCompareFnRelative<N>> {
    build_vector_compare_relative_signed(endian, |current_value, previous_value| current_value < previous_value)
}

pub fn get_vector_compare_increased_by_signed<const N: usize>(
    scan_constraint: &ScanConstraint,
    endian: Endian,
) -> Option<VectorCompareFnDelta<N>> {
    let delta_value = PrimitiveDataType24Bit::read_signed(scan_constraint.get_data_value().get_value_bytes(), endian).ok()?;

    build_vector_compare_delta_signed(delta_value, endian, |previous_value, delta_value| {
        Some(normalize_signed(previous_value.wrapping_add(delta_value)))
    })
}

pub fn get_vector_compare_decreased_by_signed<const N: usize>(
    scan_constraint: &ScanConstraint,
    endian: Endian,
) -> Option<VectorCompareFnDelta<N>> {
    let delta_value = PrimitiveDataType24Bit::read_signed(scan_constraint.get_data_value().get_value_bytes(), endian).ok()?;

    build_vector_compare_delta_signed(delta_value, endian, |previous_value, delta_value| {
        Some(normalize_signed(previous_value.wrapping_sub(delta_value)))
    })
}

pub fn get_vector_compare_multiplied_by_signed<const N: usize>(
    scan_constraint: &ScanConstraint,
    endian: Endian,
) -> Option<VectorCompareFnDelta<N>> {
    let delta_value = PrimitiveDataType24Bit::read_signed(scan_constraint.get_data_value().get_value_bytes(), endian).ok()?;

    build_vector_compare_delta_signed(delta_value, endian, |previous_value, delta_value| {
        Some(normalize_signed(previous_value.wrapping_mul(delta_value)))
    })
}

pub fn get_vector_compare_divided_by_signed<const N: usize>(
    scan_constraint: &ScanConstraint,
    endian: Endian,
) -> Option<VectorCompareFnDelta<N>> {
    let delta_value = PrimitiveDataType24Bit::read_signed(scan_constraint.get_data_value().get_value_bytes(), endian).ok()?;

    build_vector_compare_delta_signed(delta_value, endian, |previous_value, delta_value| {
        if delta_value == 0 {
            None
        } else {
            Some(normalize_signed(previous_value / delta_value))
        }
    })
}

pub fn get_vector_compare_modulo_by_signed<const N: usize>(
    scan_constraint: &ScanConstraint,
    endian: Endian,
) -> Option<VectorCompareFnDelta<N>> {
    let delta_value = PrimitiveDataType24Bit::read_signed(scan_constraint.get_data_value().get_value_bytes(), endian).ok()?;

    build_vector_compare_delta_signed(delta_value, endian, |previous_value, delta_value| {
        if delta_value == 0 {
            None
        } else {
            Some(normalize_signed(previous_value % delta_value))
        }
    })
}

pub fn get_vector_compare_shift_left_by_signed<const N: usize>(
    scan_constraint: &ScanConstraint,
    endian: Endian,
) -> Option<VectorCompareFnDelta<N>> {
    let delta_value = PrimitiveDataType24Bit::read_signed(scan_constraint.get_data_value().get_value_bytes(), endian).ok()?;

    build_vector_compare_delta_signed(delta_value, endian, |previous_value, delta_value| {
        if delta_value < 0 || delta_value >= 24 {
            None
        } else {
            Some(normalize_signed(previous_value << delta_value))
        }
    })
}

pub fn get_vector_compare_shift_right_by_signed<const N: usize>(
    scan_constraint: &ScanConstraint,
    endian: Endian,
) -> Option<VectorCompareFnDelta<N>> {
    let delta_value = PrimitiveDataType24Bit::read_signed(scan_constraint.get_data_value().get_value_bytes(), endian).ok()?;

    build_vector_compare_delta_signed(delta_value, endian, |previous_value, delta_value| {
        if delta_value < 0 || delta_value >= 24 {
            None
        } else {
            Some(normalize_signed(previous_value >> delta_value))
        }
    })
}

pub fn get_vector_compare_logical_and_by_signed<const N: usize>(
    scan_constraint: &ScanConstraint,
    endian: Endian,
) -> Option<VectorCompareFnDelta<N>> {
    let delta_value = PrimitiveDataType24Bit::read_signed(scan_constraint.get_data_value().get_value_bytes(), endian).ok()?;

    build_vector_compare_delta_signed(delta_value, endian, |previous_value, delta_value| {
        Some(raw_to_signed(signed_to_raw(previous_value) & signed_to_raw(delta_value)))
    })
}

pub fn get_vector_compare_logical_or_by_signed<const N: usize>(
    scan_constraint: &ScanConstraint,
    endian: Endian,
) -> Option<VectorCompareFnDelta<N>> {
    let delta_value = PrimitiveDataType24Bit::read_signed(scan_constraint.get_data_value().get_value_bytes(), endian).ok()?;

    build_vector_compare_delta_signed(delta_value, endian, |previous_value, delta_value| {
        Some(raw_to_signed(
            (signed_to_raw(previous_value) | signed_to_raw(delta_value)) & TWENTY_FOUR_BIT_MASK,
        ))
    })
}

pub fn get_vector_compare_logical_xor_by_signed<const N: usize>(
    scan_constraint: &ScanConstraint,
    endian: Endian,
) -> Option<VectorCompareFnDelta<N>> {
    let delta_value = PrimitiveDataType24Bit::read_signed(scan_constraint.get_data_value().get_value_bytes(), endian).ok()?;

    build_vector_compare_delta_signed(delta_value, endian, |previous_value, delta_value| {
        Some(raw_to_signed(
            (signed_to_raw(previous_value) ^ signed_to_raw(delta_value)) & TWENTY_FOUR_BIT_MASK,
        ))
    })
}

#[cfg(test)]
mod tests {
    use super::{get_vector_compare_equal_signed, get_vector_compare_equal_unsigned};
    use crate::data_types::i24::data_type_i24::DataTypeI24;
    use crate::data_types::i24be::data_type_i24be::DataTypeI24be;
    use crate::data_types::primitive_data_type_24_bit::PrimitiveDataType24Bit;
    use squalr_engine_api::registries::symbols::symbol_registry::SymbolRegistry;
    use squalr_engine_api::structures::data_types::data_type_ref::DataTypeRef;
    use squalr_engine_api::structures::data_types::floating_point_tolerance::FloatingPointTolerance;
    use squalr_engine_api::structures::memory::{endian::Endian, memory_alignment::MemoryAlignment, normalized_region::NormalizedRegion};
    use squalr_engine_api::structures::scanning::comparisons::{scan_compare_type::ScanCompareType, scan_compare_type_immediate::ScanCompareTypeImmediate};
    use squalr_engine_api::structures::scanning::constraints::{scan_constraint::ScanConstraint, scan_constraint_finalized::ScanConstraintFinalized};
    use squalr_engine_api::structures::scanning::filters::{
        snapshot_region_filter::SnapshotRegionFilter, snapshot_region_filter_collection::SnapshotRegionFilterCollection,
    };
    use squalr_engine_api::structures::scanning::plans::{
        element_scan::snapshot_filter_element_scan_plan::SnapshotFilterElementScanPlan, plan_types::planned_scan_type::PlannedScanType,
    };
    use squalr_engine_api::structures::scanning::rules::element_scan::built_in_filter_rules::filter_rule_map_scan_type::RuleMapScanType;
    use squalr_engine_api::structures::scanning::rules::element_scan_filter_rule::ElementScanFilterRule;
    use squalr_engine_api::structures::snapshots::snapshot_region::SnapshotRegion;
    use std::sync::Arc;

    #[test]
    fn unsigned_equal_marks_matching_phase_offsets() {
        let scan_constraint = ScanConstraint::new(
            ScanCompareType::Immediate(ScanCompareTypeImmediate::Equal),
            crate::data_types::u24::data_type_u24::DataTypeU24::get_value_from_primitive(3),
            FloatingPointTolerance::default(),
        );
        let compare_func = get_vector_compare_equal_unsigned::<16>(&scan_constraint, Endian::Little).expect("Expected u24 equal vector compare function.");
        let compare_bytes = [3u8, 0, 0, 1, 0, 0, 3, 0, 0, 2, 0, 0, 3, 0, 0, 9, 9, 9];

        let compare_mask = compare_func(compare_bytes.as_ptr()).to_array();

        assert_eq!(compare_mask, [0xFF, 0, 0, 0, 0, 0, 0xFF, 0, 0, 0, 0, 0, 0xFF, 0, 0, 0]);
    }

    #[test]
    fn signed_big_endian_equal_sign_extends_values() {
        let scan_constraint = ScanConstraint::new(
            ScanCompareType::Immediate(ScanCompareTypeImmediate::Equal),
            DataTypeI24be::get_value_from_primitive(-2),
            FloatingPointTolerance::default(),
        );
        let compare_func = get_vector_compare_equal_signed::<16>(&scan_constraint, Endian::Big).expect("Expected i24be equal vector compare function.");
        let mut compare_bytes = Vec::new();
        compare_bytes.extend_from_slice(&PrimitiveDataType24Bit::get_signed_value_bytes(-2, Endian::Big));
        compare_bytes.extend_from_slice(&PrimitiveDataType24Bit::get_signed_value_bytes(1, Endian::Big));
        compare_bytes.extend_from_slice(&PrimitiveDataType24Bit::get_signed_value_bytes(-2, Endian::Big));
        compare_bytes.extend_from_slice(&PrimitiveDataType24Bit::get_signed_value_bytes(7, Endian::Big));
        compare_bytes.extend_from_slice(&PrimitiveDataType24Bit::get_signed_value_bytes(-2, Endian::Big));
        compare_bytes.extend_from_slice(&PrimitiveDataType24Bit::get_signed_value_bytes(0, Endian::Big));

        let compare_mask = compare_func(compare_bytes.as_ptr()).to_array();

        assert_eq!(compare_mask, [0xFF, 0, 0, 0, 0, 0, 0xFF, 0, 0, 0, 0, 0, 0xFF, 0, 0, 0]);
    }

    #[test]
    fn planner_keeps_i24_on_vector_path_when_vector_compare_exists() {
        let symbol_registry = SymbolRegistry::new();
        assert!(symbol_registry.register_data_type(Arc::new(DataTypeI24)));

        let data_type_ref = DataTypeRef::new(DataTypeI24::DATA_TYPE_ID);
        let scan_constraint = ScanConstraint::new(
            ScanCompareType::Immediate(ScanCompareTypeImmediate::Equal),
            DataTypeI24::get_value_from_primitive(3),
            FloatingPointTolerance::default(),
        );
        let scan_constraint_finalized = ScanConstraintFinalized::new(&symbol_registry, scan_constraint);
        assert!(
            scan_constraint_finalized
                .get_scan_function_vector::<64>()
                .is_some()
        );

        let mut snapshot_region = SnapshotRegion::new(NormalizedRegion::new(0x1000, 128), vec![]);
        snapshot_region.current_values = vec![0u8; 128];

        let snapshot_region_filter = SnapshotRegionFilter::new(0x1000, 128);
        let snapshot_region_filter_collection = SnapshotRegionFilterCollection::new(
            &symbol_registry,
            vec![vec![snapshot_region_filter.clone()]],
            data_type_ref,
            MemoryAlignment::Alignment1,
        );
        let mut snapshot_filter_element_scan_plan =
            SnapshotFilterElementScanPlan::new(&scan_constraint_finalized, MemoryAlignment::Alignment1, FloatingPointTolerance::default());

        RuleMapScanType {}.map_parameters(
            &symbol_registry,
            &snapshot_region,
            &snapshot_region_filter_collection,
            &snapshot_region_filter,
            &scan_constraint_finalized,
            &mut snapshot_filter_element_scan_plan,
        );

        assert!(matches!(
            snapshot_filter_element_scan_plan.get_planned_scan_type(),
            PlannedScanType::Vector(_, _)
        ));
    }
}
