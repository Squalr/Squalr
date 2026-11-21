use super::{pointer_scan_level_candidates::PointerScanLevelCandidates, pointer_scan_results::PointerScanResults};
use anyhow::{Result, bail, ensure};
use std::{
    collections::{BTreeMap, HashSet},
    ops::Range,
};

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

/// Compares roots from one module at one exact depth. Building the index visits each level's static roots once.
pub struct PairedPointerWalk<'a> {
    previous_levels: &'a mut [PointerScanLevelCandidates],
    current_levels: &'a mut [PointerScanLevelCandidates],
    previous_roots: BTreeMap<u64, u64>,
    current_roots: BTreeMap<u64, u64>,
    previous_target: u64,
    current_target: u64,
    radius: u64,
    depth: usize,
}

impl<'a> PairedPointerWalk<'a> {
    pub fn new(
        previous: &'a mut PointerScanResults,
        current: &'a mut PointerScanResults,
        module: &str,
        depth: usize,
    ) -> Result<Self> {
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
        let previous_roots = module_roots(previous, module, depth)?;
        let current_roots = module_roots(current, module, depth)?;
        Ok(Self {
            previous_levels: previous.get_pointer_scan_level_candidates_mut(),
            current_levels: current.get_pointer_scan_level_candidates_mut(),
            previous_roots,
            current_roots,
            previous_target,
            current_target,
            radius,
            depth,
        })
    }

    pub fn baseline_root_offsets(&self) -> impl Iterator<Item = u64> + '_ {
        self.previous_roots.keys().copied()
    }

    /// Returns the first common sequence in root-to-target order. Each traversal step costs one unit.
    /// A work-limit result is inconclusive and must be retained when pruning.
    pub fn walk_root(
        &mut self,
        module_offset: u64,
        work_limit: usize,
    ) -> PairedWalkResult {
        let (Some(&previous_value), Some(&current_value)) = (self.previous_roots.get(&module_offset), self.current_roots.get(&module_offset)) else {
            return PairedWalkResult::NoMatch;
        };
        if work_limit == 0 {
            return PairedWalkResult::WorkLimit;
        }
        let root = State {
            depth: self.depth,
            previous: previous_value,
            current: current_value,
        };
        let mut stack = vec![Frame::new(root, self.previous_levels, self.radius)];
        // Only completed failures are reusable across branches within this root.
        let mut exhausted = HashSet::new();
        let mut remaining_work = work_limit;
        while let Some(frame) = stack.last_mut() {
            if remaining_work == 0 {
                return PairedWalkResult::WorkLimit;
            }
            remaining_work -= 1;
            let state = frame.state;
            if state.depth == 1 {
                let offset = i128::from(self.previous_target) - i128::from(state.previous);
                if self.previous_target.abs_diff(state.previous) <= self.radius && offset == i128::from(self.current_target) - i128::from(state.current) {
                    let mut offsets: Vec<_> = stack[..stack.len() - 1]
                        .iter()
                        .map(|ancestor| ancestor.offset)
                        .collect();
                    offsets.push(offset);
                    return PairedWalkResult::Matched(offsets);
                }
            } else {
                let child_level = state.depth - 2;
                if let Some(current_slot) = frame.current_matches.next() {
                    let next = State {
                        depth: state.depth - 1,
                        previous: frame.previous_child,
                        current: self.current_levels[child_level].heap_slot_value(current_slot),
                    };
                    if !exhausted.contains(&next) {
                        stack.push(Frame::new(next, self.previous_levels, self.radius));
                    }
                    continue;
                }
                if let Some(previous_slot) = frame.previous_slots.next() {
                    let level = &self.previous_levels[child_level];
                    frame.offset = i128::from(level.heap_address(previous_slot)) - i128::from(state.previous);
                    frame.previous_child = level.heap_slot_value(previous_slot);
                    if let Ok(current_address) = u64::try_from(i128::from(state.current) + frame.offset) {
                        frame.current_matches = self.current_levels[child_level].heap_candidate_range(current_address, current_address);
                    }
                    continue;
                }
            }
            exhausted.insert(state);
            stack.pop();
        }
        PairedWalkResult::NoMatch
    }
}

fn module_roots(
    graph: &PointerScanResults,
    module: &str,
    depth: usize,
) -> Result<BTreeMap<u64, u64>> {
    let mut roots = BTreeMap::new();
    let mut matching = graph
        .get_module_names()
        .iter()
        .enumerate()
        .filter(|(_, name)| name.as_str() == module);
    let Some((module_index, _)) = matching.next() else {
        bail!("module missing from capture: {module}")
    };
    ensure!(matching.next().is_none(), "ambiguous module name: {module}");
    if let Some(level) = graph.get_pointer_scan_level_candidates().get(depth - 1) {
        for root in level
            .get_static_candidates()
            .iter()
            .filter(|root| root.get_module_index() == module_index)
        {
            ensure!(
                roots
                    .insert(root.get_module_offset(), root.get_pointer_value())
                    .is_none(),
                "ambiguous module-relative root"
            );
        }
    }
    Ok(roots)
}

#[cfg(test)]
mod tests;
