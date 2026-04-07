use crate::pointer_scans::search_kernels::pointer_scan_pointer_value_reader::{read_pointer_lane_values_u32, read_pointer_lane_values_u64};
use crate::pointer_scans::search_kernels::pointer_scan_scalar_search_kernel::scan_region_scalar;
use crate::pointer_scans::structures::pointer_scan_region_match::PointerScanRegionMatch;
use crate::pointer_scans::structures::pointer_scan_target_ranges::PointerScanTargetRangeSet;
use squalr_engine_api::structures::pointer_scans::pointer_scan_pointer_size::PointerScanPointerSize;
use std::mem::size_of;
use std::simd::Simd;
use std::simd::cmp::SimdPartialOrd;

pub(crate) fn scan_region_simd_linear<VisitMatch>(
    base_address: u64,
    current_values: &[u8],
    start_offset: usize,
    pointer_size: PointerScanPointerSize,
    target_range_set: &PointerScanTargetRangeSet,
    visit_match: &mut VisitMatch,
) where
    VisitMatch: FnMut(PointerScanRegionMatch),
{
    match pointer_size {
        PointerScanPointerSize::Pointer24 | PointerScanPointerSize::Pointer24be => scan_region_scalar(
            base_address,
            current_values,
            start_offset,
            pointer_size,
            |pointer_value| target_range_set.contains_value_linear(pointer_value),
            visit_match,
        ),
        PointerScanPointerSize::Pointer32 | PointerScanPointerSize::Pointer32be => {
            scan_region_simd_linear_u32(base_address, current_values, start_offset, pointer_size, target_range_set, visit_match)
        }
        PointerScanPointerSize::Pointer64 | PointerScanPointerSize::Pointer64be => {
            scan_region_simd_linear_u64(base_address, current_values, start_offset, pointer_size, target_range_set, visit_match)
        }
    }
}

fn scan_region_simd_linear_u32<VisitMatch>(
    base_address: u64,
    current_values: &[u8],
    start_offset: usize,
    pointer_size: PointerScanPointerSize,
    target_range_set: &PointerScanTargetRangeSet,
    visit_match: &mut VisitMatch,
) where
    VisitMatch: FnMut(PointerScanRegionMatch),
{
    const SIMD_LANE_COUNT: usize = 16;
    let pointer_size_in_bytes = size_of::<u32>();
    let current_values_ptr = current_values.as_ptr();
    let mut pointer_value_offset = start_offset;

    while pointer_value_offset.saturating_add(pointer_size_in_bytes * SIMD_LANE_COUNT) <= current_values.len() {
        // The loop guard guarantees a full SIMD-width unaligned load.
        let lane_values = unsafe { read_pointer_lane_values_u32::<SIMD_LANE_COUNT>(current_values_ptr.add(pointer_value_offset), pointer_size) };
        let pointer_values = Simd::<u32, SIMD_LANE_COUNT>::from_array(lane_values);
        let mut matching_lane_mask = 0_u64;

        for target_range in target_range_set.get_target_ranges() {
            let Ok(lower_bound) = u32::try_from(target_range.get_base_address()) else {
                continue;
            };
            let upper_bound = u32::try_from(target_range.get_end_address()).unwrap_or(u32::MAX);
            let lower_bound_vector = Simd::splat(lower_bound);
            let upper_bound_vector = Simd::splat(upper_bound);

            matching_lane_mask |= (pointer_values.simd_ge(lower_bound_vector) & pointer_values.simd_le(upper_bound_vector)).to_bitmask();
        }

        if matching_lane_mask != 0 {
            for lane_index in 0..SIMD_LANE_COUNT {
                if matching_lane_mask & (1_u64 << lane_index) == 0 {
                    continue;
                }

                visit_match(PointerScanRegionMatch::new(
                    base_address.saturating_add((pointer_value_offset + lane_index * pointer_size_in_bytes) as u64),
                    lane_values[lane_index] as u64,
                ));
            }
        }

        pointer_value_offset = pointer_value_offset.saturating_add(pointer_size_in_bytes * SIMD_LANE_COUNT);
    }

    scan_region_scalar(
        base_address,
        current_values,
        pointer_value_offset,
        pointer_size,
        |pointer_value| target_range_set.contains_value_linear(pointer_value),
        visit_match,
    );
}

fn scan_region_simd_linear_u64<VisitMatch>(
    base_address: u64,
    current_values: &[u8],
    start_offset: usize,
    pointer_size: PointerScanPointerSize,
    target_range_set: &PointerScanTargetRangeSet,
    visit_match: &mut VisitMatch,
) where
    VisitMatch: FnMut(PointerScanRegionMatch),
{
    const SIMD_LANE_COUNT: usize = 8;
    let pointer_size_in_bytes = size_of::<u64>();
    let current_values_ptr = current_values.as_ptr();
    let mut pointer_value_offset = start_offset;

    while pointer_value_offset.saturating_add(pointer_size_in_bytes * SIMD_LANE_COUNT) <= current_values.len() {
        // The loop guard guarantees a full SIMD-width unaligned load.
        let lane_values = unsafe { read_pointer_lane_values_u64::<SIMD_LANE_COUNT>(current_values_ptr.add(pointer_value_offset), pointer_size) };
        let pointer_values = Simd::<u64, SIMD_LANE_COUNT>::from_array(lane_values);
        let mut matching_lane_mask = 0_u64;

        for target_range in target_range_set.get_target_ranges() {
            let lower_bound_vector = Simd::splat(target_range.get_base_address());
            let upper_bound_vector = Simd::splat(target_range.get_end_address());

            matching_lane_mask |= (pointer_values.simd_ge(lower_bound_vector) & pointer_values.simd_le(upper_bound_vector)).to_bitmask();
        }

        if matching_lane_mask != 0 {
            for lane_index in 0..SIMD_LANE_COUNT {
                if matching_lane_mask & (1_u64 << lane_index) == 0 {
                    continue;
                }

                visit_match(PointerScanRegionMatch::new(
                    base_address.saturating_add((pointer_value_offset + lane_index * pointer_size_in_bytes) as u64),
                    lane_values[lane_index],
                ));
            }
        }

        pointer_value_offset = pointer_value_offset.saturating_add(pointer_size_in_bytes * SIMD_LANE_COUNT);
    }

    scan_region_scalar(
        base_address,
        current_values,
        pointer_value_offset,
        pointer_size,
        |pointer_value| target_range_set.contains_value_linear(pointer_value),
        visit_match,
    );
}
