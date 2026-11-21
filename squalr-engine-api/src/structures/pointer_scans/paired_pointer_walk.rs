use super::{pointer_scan_level_candidates::PointerScanLevelCandidates, pointer_scan_results::PointerScanResults};
use anyhow::{Result, bail, ensure};
use std::{collections::HashSet, ops::Range};

/// Result for one module-relative root at one exact depth in two address-target graphs.
#[derive(Debug, PartialEq, Eq)]
pub enum PairedWalkResult {
    Matched(Vec<i128>),
    NoMatch,
    WorkLimit,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct State {
    depth: usize,
    previous: u64,
    current: u64,
}

struct Frame {
    state: State,
    previous_slots: Range<usize>,
    current_matches: Range<usize>,
    previous_child: u64,
    offset: i128,
}

impl Frame {
    fn new(
        state: State,
        previous: &mut [PointerScanLevelCandidates],
        radius: u64,
    ) -> Self {
        let previous_slots = if state.depth == 1 {
            0..0
        } else {
            previous[state.depth - 2].heap_candidate_range(state.previous.saturating_sub(radius), state.previous.saturating_add(radius))
        };
        Self {
            state,
            previous_slots,
            current_matches: 0..0,
            previous_child: 0,
            offset: 0,
        }
    }
}

/// Looks for one identical signed offset sequence across two captures. Each loop step consumes one unit of work.
/// Matches roots by module name and relative address at the requested depth. Returned offsets are in root-to-target order.
/// Each heap lookup uses the compact graph. A work-limit result is inconclusive and must be retained when pruning.
pub fn walk_paired_root(
    previous: &mut PointerScanResults,
    current: &mut PointerScanResults,
    module: &str,
    module_offset: u64,
    depth: usize,
    work_limit: usize,
) -> Result<PairedWalkResult> {
    ensure!(
        depth > 0 && depth as u64 <= previous.get_max_depth().min(current.get_max_depth()),
        "depth must be within both scans"
    );
    ensure!(
        previous.get_pointer_size() == current.get_pointer_size() && previous.get_address_space() == current.get_address_space(),
        "incompatible pointer size or address space"
    );
    let (Some(previous_target), Some(current_target)) = (
        previous.get_target_descriptor().get_target_address(),
        current.get_target_descriptor().get_target_address(),
    ) else {
        bail!("paired walking requires one address target per capture");
    };
    let radius = previous.get_offset_radius().min(current.get_offset_radius());
    let (Some(previous_value), Some(current_value)) = (
        root_value(previous, module, module_offset, depth)?,
        root_value(current, module, module_offset, depth)?,
    ) else {
        return Ok(PairedWalkResult::NoMatch);
    };
    if work_limit == 0 {
        return Ok(PairedWalkResult::WorkLimit);
    }
    let previous_levels = previous.get_pointer_scan_level_candidates_mut();
    let current_levels = current.get_pointer_scan_level_candidates_mut();
    let root = State {
        depth,
        previous: previous_value,
        current: current_value,
    };
    let mut stack = vec![Frame::new(root, previous_levels, radius)];
    // Only completed failures are reusable across branches within this root.
    let mut exhausted = HashSet::new();
    let mut remaining_work = work_limit;
    while let Some(frame) = stack.last_mut() {
        if remaining_work == 0 {
            return Ok(PairedWalkResult::WorkLimit);
        }
        remaining_work -= 1;
        let state = frame.state;
        if state.depth == 1 {
            let offset = i128::from(previous_target) - i128::from(state.previous);
            if previous_target.abs_diff(state.previous) <= radius && offset == i128::from(current_target) - i128::from(state.current) {
                let mut offsets: Vec<_> = stack[..stack.len() - 1]
                    .iter()
                    .map(|ancestor| ancestor.offset)
                    .collect();
                offsets.push(offset);
                return Ok(PairedWalkResult::Matched(offsets));
            }
        } else {
            let child_level = state.depth - 2;
            if let Some(current_slot) = frame.current_matches.next() {
                let next = State {
                    depth: state.depth - 1,
                    previous: frame.previous_child,
                    current: current_levels[child_level].heap_slot_value(current_slot),
                };
                if !exhausted.contains(&next) {
                    stack.push(Frame::new(next, previous_levels, radius));
                }
                continue;
            }
            if let Some(previous_slot) = frame.previous_slots.next() {
                let level = &previous_levels[child_level];
                frame.offset = i128::from(level.heap_address(previous_slot)) - i128::from(state.previous);
                frame.previous_child = level.heap_slot_value(previous_slot);
                if let Ok(current_address) = u64::try_from(i128::from(state.current) + frame.offset) {
                    frame.current_matches = current_levels[child_level].heap_candidate_range(current_address, current_address);
                }
                continue;
            }
        }
        exhausted.insert(state);
        stack.pop();
    }
    Ok(PairedWalkResult::NoMatch)
}

fn root_value(
    graph: &PointerScanResults,
    module: &str,
    module_offset: u64,
    depth: usize,
) -> Result<Option<u64>> {
    let mut matching_modules = graph
        .get_module_names()
        .iter()
        .enumerate()
        .filter(|(_, name)| name.as_str() == module);
    let Some((module_index, _)) = matching_modules.next() else { return Ok(None) };
    ensure!(matching_modules.next().is_none(), "ambiguous module name: {module}");
    let Some(level) = graph.get_pointer_scan_level_candidates().get(depth - 1) else {
        return Ok(None);
    };
    let roots = level.get_static_candidates();
    let root_key = (module_index, module_offset);
    let root_index = roots.partition_point(|root| (root.get_module_index(), root.get_module_offset()) < root_key);
    let Some(root) = roots
        .get(root_index)
        .filter(|root| (root.get_module_index(), root.get_module_offset()) == root_key)
    else {
        return Ok(None);
    };
    ensure!(
        roots
            .get(root_index + 1)
            .is_none_or(|next| (next.get_module_index(), next.get_module_offset()) != root_key),
        "ambiguous module-relative root"
    );
    Ok(Some(root.get_pointer_value()))
}

#[cfg(test)]
mod tests;
