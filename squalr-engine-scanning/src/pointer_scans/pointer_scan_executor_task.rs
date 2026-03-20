use crate::pointer_scans::pointer_scan_range_search_kernel::PointerScanRangeSearchKernel;
use crate::pointer_scans::pointer_scan_target_ranges::PointerScanTargetRangeSet;
use crate::scanners::scan_execution_context::ScanExecutionContext;
use crate::scanners::value_collector_task::ValueCollector;
use rayon::prelude::*;
use squalr_engine_api::structures::memory::normalized_module::NormalizedModule;
use squalr_engine_api::structures::pointer_scans::pointer_scan_candidate::PointerScanCandidate;
use squalr_engine_api::structures::pointer_scans::pointer_scan_level::PointerScanLevel;
use squalr_engine_api::structures::pointer_scans::pointer_scan_level_candidates::PointerScanLevelCandidates;
use squalr_engine_api::structures::pointer_scans::pointer_scan_node_type::PointerScanNodeType;
use squalr_engine_api::structures::pointer_scans::pointer_scan_session::PointerScanSession;
use squalr_engine_api::structures::processes::opened_process_info::OpenedProcessInfo;
use squalr_engine_api::structures::scanning::plans::pointer_scan::pointer_scan_parameters::PointerScanParameters;
use squalr_engine_api::structures::snapshots::snapshot::Snapshot;
use squalr_engine_api::structures::snapshots::snapshot_region::SnapshotRegion;
use std::cmp::Ordering;
use std::sync::{Arc, RwLock};
use std::time::Instant;

pub struct PointerScanExecutor;

#[derive(Clone, Debug, Eq, PartialEq)]
struct DiscoveredPointerCandidate {
    pointer_scan_node_type: PointerScanNodeType,
    pointer_address: u64,
    pointer_value: u64,
    module_name: String,
    module_offset: u64,
}

#[derive(Clone, Debug, Default)]
struct DiscoveredPointerLevel {
    static_candidates: Vec<DiscoveredPointerCandidate>,
    heap_candidates: Vec<DiscoveredPointerCandidate>,
}

/// Implementation of a task that discovers pointer chains against the provided snapshot values.
impl PointerScanExecutor {
    pub fn execute_scan(
        process_info: OpenedProcessInfo,
        statics_snapshot: Arc<RwLock<Snapshot>>,
        heaps_snapshot: Arc<RwLock<Snapshot>>,
        pointer_scan_session_id: u64,
        pointer_scan_parameters: PointerScanParameters,
        modules: &[NormalizedModule],
        with_logging: bool,
        scan_execution_context: &ScanExecutionContext,
    ) -> PointerScanSession {
        Self::scan_task(
            process_info,
            statics_snapshot,
            heaps_snapshot,
            pointer_scan_session_id,
            pointer_scan_parameters,
            modules,
            with_logging,
            scan_execution_context,
        )
    }

    fn scan_task(
        process_info: OpenedProcessInfo,
        statics_snapshot: Arc<RwLock<Snapshot>>,
        heaps_snapshot: Arc<RwLock<Snapshot>>,
        pointer_scan_session_id: u64,
        pointer_scan_parameters: PointerScanParameters,
        modules: &[NormalizedModule],
        with_logging: bool,
        scan_execution_context: &ScanExecutionContext,
    ) -> PointerScanSession {
        let total_start_time = Instant::now();

        if with_logging {
            log::info!(
                "Performing pointer scan for target 0x{:X} using {} pointers, max depth {}, and offset {}.",
                pointer_scan_parameters.get_target_address(),
                pointer_scan_parameters.get_pointer_size(),
                pointer_scan_parameters.get_max_depth(),
                pointer_scan_parameters.get_offset_radius(),
            );
        }

        let value_collection_start_time = Instant::now();
        Self::collect_pointer_scan_values(
            process_info.clone(),
            statics_snapshot.clone(),
            heaps_snapshot.clone(),
            with_logging,
            scan_execution_context,
        );
        let value_collection_duration = value_collection_start_time.elapsed();

        let discovery_start_time = Instant::now();
        let pointer_scan_session = if Arc::ptr_eq(&statics_snapshot, &heaps_snapshot) {
            let snapshot_guard = match statics_snapshot.read() {
                Ok(snapshot_guard) => snapshot_guard,
                Err(error) => {
                    if with_logging {
                        log::error!("Failed to acquire read lock on pointer scan snapshot: {}", error);
                    }

                    return Self::create_empty_session(pointer_scan_session_id, &pointer_scan_parameters);
                }
            };

            Self::build_pointer_scan_session(vec![&*snapshot_guard], pointer_scan_session_id, &pointer_scan_parameters, modules, with_logging)
        } else {
            let statics_snapshot_guard = match statics_snapshot.read() {
                Ok(statics_snapshot_guard) => statics_snapshot_guard,
                Err(error) => {
                    if with_logging {
                        log::error!("Failed to acquire read lock on static pointer scan snapshot: {}", error);
                    }

                    return Self::create_empty_session(pointer_scan_session_id, &pointer_scan_parameters);
                }
            };
            let heaps_snapshot_guard = match heaps_snapshot.read() {
                Ok(heaps_snapshot_guard) => heaps_snapshot_guard,
                Err(error) => {
                    if with_logging {
                        log::error!("Failed to acquire read lock on heap pointer scan snapshot: {}", error);
                    }

                    return Self::create_empty_session(pointer_scan_session_id, &pointer_scan_parameters);
                }
            };

            Self::build_pointer_scan_session(
                vec![&*statics_snapshot_guard, &*heaps_snapshot_guard],
                pointer_scan_session_id,
                &pointer_scan_parameters,
                modules,
                with_logging,
            )
        };
        let discovery_duration = discovery_start_time.elapsed();

        if with_logging {
            let pointer_scan_summary = pointer_scan_session.summarize();

            log::info!(
                "Pointer scan complete: roots={}, total_nodes={}, static_nodes={}, heap_nodes={}",
                pointer_scan_summary.get_root_node_count(),
                pointer_scan_summary.get_total_node_count(),
                pointer_scan_summary.get_total_static_node_count(),
                pointer_scan_summary.get_total_heap_node_count(),
            );
            log::info!("Pointer scan value collection time: {:?}", value_collection_duration);
            log::info!("Pointer scan reachability discovery time: {:?}", discovery_duration);
            log::info!("Total pointer scan time: {:?}", total_start_time.elapsed());
        }

        pointer_scan_session
    }

    fn collect_pointer_scan_values(
        process_info: OpenedProcessInfo,
        statics_snapshot: Arc<RwLock<Snapshot>>,
        heaps_snapshot: Arc<RwLock<Snapshot>>,
        with_logging: bool,
        scan_execution_context: &ScanExecutionContext,
    ) {
        if Arc::ptr_eq(&statics_snapshot, &heaps_snapshot) {
            ValueCollector::collect_values(process_info, statics_snapshot, with_logging, scan_execution_context);
        } else {
            ValueCollector::collect_values(process_info.clone(), statics_snapshot, with_logging, scan_execution_context);
            ValueCollector::collect_values(process_info, heaps_snapshot, with_logging, scan_execution_context);
        }
    }

    fn build_pointer_scan_session(
        snapshots: Vec<&Snapshot>,
        pointer_scan_session_id: u64,
        pointer_scan_parameters: &PointerScanParameters,
        modules: &[NormalizedModule],
        with_logging: bool,
    ) -> PointerScanSession {
        let discovered_pointer_levels = Self::discover_pointer_levels(&snapshots, pointer_scan_parameters, modules, with_logging);

        if discovered_pointer_levels.is_empty() {
            if with_logging {
                log::info!("Pointer scan found no reachable pointer nodes.");
            }

            return Self::create_empty_session(pointer_scan_session_id, pointer_scan_parameters);
        }

        let mut pointer_scan_levels = Vec::new();
        let mut all_pointer_scan_level_candidates = Vec::new();
        let mut next_candidate_id = 1_u64;
        let mut total_static_node_count = 0_u64;
        let mut total_heap_node_count = 0_u64;

        for (pointer_level_index, discovered_pointer_level) in discovered_pointer_levels.iter().enumerate() {
            let discovery_depth = pointer_level_index as u64 + 1;
            let static_candidates = discovered_pointer_level
                .static_candidates
                .iter()
                .map(|discovered_pointer_candidate| {
                    let pointer_scan_candidate = PointerScanCandidate::new(
                        next_candidate_id,
                        discovery_depth,
                        PointerScanNodeType::Static,
                        discovered_pointer_candidate.pointer_address,
                        discovered_pointer_candidate.pointer_value,
                        discovered_pointer_candidate.module_name.clone(),
                        discovered_pointer_candidate.module_offset,
                    );
                    next_candidate_id = next_candidate_id.saturating_add(1);

                    pointer_scan_candidate
                })
                .collect::<Vec<_>>();
            let heap_candidates = discovered_pointer_level
                .heap_candidates
                .iter()
                .map(|discovered_pointer_candidate| {
                    let pointer_scan_candidate = PointerScanCandidate::new(
                        next_candidate_id,
                        discovery_depth,
                        PointerScanNodeType::Heap,
                        discovered_pointer_candidate.pointer_address,
                        discovered_pointer_candidate.pointer_value,
                        String::new(),
                        0,
                    );
                    next_candidate_id = next_candidate_id.saturating_add(1);

                    pointer_scan_candidate
                })
                .collect::<Vec<_>>();
            let discovered_level_candidates = PointerScanLevelCandidates::new(discovery_depth, static_candidates, heap_candidates);

            total_static_node_count = total_static_node_count.saturating_add(discovered_level_candidates.get_static_node_count());
            total_heap_node_count = total_heap_node_count.saturating_add(discovered_level_candidates.get_heap_node_count());

            pointer_scan_levels.push(PointerScanLevel::new(
                discovery_depth,
                discovered_level_candidates.get_node_count(),
                discovered_level_candidates.get_static_node_count(),
                discovered_level_candidates.get_heap_node_count(),
            ));
            all_pointer_scan_level_candidates.push(discovered_level_candidates);
        }
        let root_count_start_time = Instant::now();
        let root_node_count = Self::count_root_nodes(&all_pointer_scan_level_candidates, pointer_scan_parameters.get_offset_radius());

        if with_logging {
            for pointer_scan_level in &pointer_scan_levels {
                log::info!(
                    "Pointer scan level {} retained {} unique nodes (static {} / heap {}).",
                    pointer_scan_level.get_depth(),
                    pointer_scan_level.get_node_count(),
                    pointer_scan_level.get_static_node_count(),
                    pointer_scan_level.get_heap_node_count(),
                );
            }
            log::info!("Pointer scan root summarization time: {:?}", root_count_start_time.elapsed());
        }

        PointerScanSession::new(
            pointer_scan_session_id,
            pointer_scan_parameters.get_target_address(),
            pointer_scan_parameters.get_pointer_size(),
            pointer_scan_parameters.get_max_depth(),
            pointer_scan_parameters.get_offset_radius(),
            pointer_scan_levels,
            all_pointer_scan_level_candidates,
            root_node_count,
            total_static_node_count,
            total_heap_node_count,
        )
    }

    fn discover_pointer_levels(
        snapshots: &[&Snapshot],
        pointer_scan_parameters: &PointerScanParameters,
        modules: &[NormalizedModule],
        with_logging: bool,
    ) -> Vec<DiscoveredPointerLevel> {
        let discovery_start_time = Instant::now();
        let max_depth = pointer_scan_parameters.get_max_depth();

        if max_depth == 0 {
            return Vec::new();
        }

        let target_address = pointer_scan_parameters.get_target_address();
        let mut frontier_target_addresses = vec![target_address];
        let mut discovered_pointer_levels = Vec::new();
        let total_snapshot_region_count = snapshots
            .iter()
            .map(|snapshot| snapshot.get_snapshot_regions().len())
            .sum::<usize>();

        for pointer_chain_depth in 0..max_depth {
            let level_number = pointer_chain_depth.saturating_add(1);
            let is_terminal_level = level_number >= max_depth;
            frontier_target_addresses.sort_unstable();
            frontier_target_addresses.dedup();

            if frontier_target_addresses.is_empty() {
                if with_logging {
                    log::info!(
                        "Pointer scan stopped after level {} because no frontier targets remained.",
                        level_number.saturating_sub(1)
                    );
                }

                break;
            }

            let frontier_target_ranges =
                PointerScanTargetRangeSet::from_target_addresses(&frontier_target_addresses, pointer_scan_parameters.get_offset_radius());
            let range_search_kernel = PointerScanRangeSearchKernel::new(&frontier_target_ranges, pointer_scan_parameters.get_pointer_size());
            let level_start_time = Instant::now();

            if with_logging {
                log::info!(
                    "Pointer scan level {}/{}: scanning {} snapshot regions for {} frontier targets merged into {} ranges with {} kernel.",
                    level_number,
                    max_depth,
                    total_snapshot_region_count,
                    frontier_target_addresses.len(),
                    frontier_target_ranges.get_range_count(),
                    range_search_kernel.get_name(),
                );
            }

            let discovered_pointer_level = Self::scan_snapshots_for_pointer_targets(snapshots, &range_search_kernel, modules, !is_terminal_level);
            let level_duration = level_start_time.elapsed();

            if with_logging {
                log::info!(
                    "Pointer scan level {}/{} complete in {:?}: retained {} static nodes and {} heap nodes, and produced {} next frontier targets.",
                    level_number,
                    max_depth,
                    level_duration,
                    discovered_pointer_level.static_candidates.len(),
                    discovered_pointer_level.heap_candidates.len(),
                    if is_terminal_level {
                        0
                    } else {
                        discovered_pointer_level.heap_candidates.len()
                    },
                );
            }

            if discovered_pointer_level.static_candidates.is_empty() && discovered_pointer_level.heap_candidates.is_empty() {
                if with_logging {
                    log::info!(
                        "Pointer scan stopped after level {} because no deeper pointer candidates were found.",
                        level_number
                    );
                }

                break;
            }

            frontier_target_addresses = discovered_pointer_level
                .heap_candidates
                .iter()
                .map(|discovered_pointer_candidate| discovered_pointer_candidate.pointer_address)
                .collect();
            discovered_pointer_levels.push(discovered_pointer_level);
        }

        if with_logging {
            let discovered_pointer_node_count = discovered_pointer_levels
                .iter()
                .map(|discovered_pointer_level| discovered_pointer_level.static_candidates.len() + discovered_pointer_level.heap_candidates.len())
                .sum::<usize>();

            log::info!(
                "Pointer scan discovered {} unique reachable pointer nodes across {} levels before any tree expansion.",
                discovered_pointer_node_count,
                discovered_pointer_levels.len(),
            );
            log::info!("Pointer scan reachability levels built in: {:?}", discovery_start_time.elapsed());
        }

        discovered_pointer_levels
    }

    fn scan_snapshots_for_pointer_targets(
        snapshots: &[&Snapshot],
        range_search_kernel: &PointerScanRangeSearchKernel<'_>,
        modules: &[NormalizedModule],
        retain_heap_candidates: bool,
    ) -> DiscoveredPointerLevel {
        if range_search_kernel.is_empty() {
            return DiscoveredPointerLevel::default();
        }

        let snapshot_regions = snapshots
            .iter()
            .flat_map(|snapshot| snapshot.get_snapshot_regions().iter())
            .collect::<Vec<_>>();
        let mut discovered_pointer_level = snapshot_regions
            .par_iter()
            .map(|snapshot_region| Self::scan_snapshot_region_for_pointer_targets(snapshot_region, range_search_kernel, modules, retain_heap_candidates))
            .reduce(DiscoveredPointerLevel::default, Self::merge_discovered_pointer_levels);

        discovered_pointer_level
            .static_candidates
            .sort_by(Self::compare_discovered_pointer_candidates);
        discovered_pointer_level.static_candidates.dedup();
        discovered_pointer_level
            .heap_candidates
            .sort_by(Self::compare_discovered_pointer_candidates);
        discovered_pointer_level.heap_candidates.dedup();

        discovered_pointer_level
    }

    fn scan_snapshot_region_for_pointer_targets(
        snapshot_region: &SnapshotRegion,
        range_search_kernel: &PointerScanRangeSearchKernel<'_>,
        modules: &[NormalizedModule],
        retain_heap_candidates: bool,
    ) -> DiscoveredPointerLevel {
        let mut discovered_pointer_level = DiscoveredPointerLevel::default();

        for pointer_match in range_search_kernel.scan_region(snapshot_region.get_base_address(), snapshot_region.get_current_values()) {
            let (pointer_scan_node_type, module_name, module_offset) = Self::classify_pointer_address(pointer_match.get_pointer_address(), modules);
            let discovered_pointer_candidate = DiscoveredPointerCandidate {
                pointer_scan_node_type,
                pointer_address: pointer_match.get_pointer_address(),
                pointer_value: pointer_match.get_pointer_value(),
                module_name,
                module_offset,
            };

            match pointer_scan_node_type {
                PointerScanNodeType::Static => discovered_pointer_level
                    .static_candidates
                    .push(discovered_pointer_candidate),
                PointerScanNodeType::Heap => {
                    if retain_heap_candidates {
                        discovered_pointer_level
                            .heap_candidates
                            .push(discovered_pointer_candidate);
                    }
                }
            }
        }

        discovered_pointer_level
    }

    fn merge_discovered_pointer_levels(
        mut left_discovered_pointer_level: DiscoveredPointerLevel,
        mut right_discovered_pointer_level: DiscoveredPointerLevel,
    ) -> DiscoveredPointerLevel {
        left_discovered_pointer_level
            .static_candidates
            .append(&mut right_discovered_pointer_level.static_candidates);
        left_discovered_pointer_level
            .heap_candidates
            .append(&mut right_discovered_pointer_level.heap_candidates);

        left_discovered_pointer_level
    }

    fn classify_pointer_address(
        pointer_address: u64,
        modules: &[NormalizedModule],
    ) -> (PointerScanNodeType, String, u64) {
        if let Some(module) = modules
            .iter()
            .find(|module| module.contains_address(pointer_address))
        {
            (
                PointerScanNodeType::Static,
                module.get_module_name().to_string(),
                pointer_address.saturating_sub(module.get_base_address()),
            )
        } else {
            (PointerScanNodeType::Heap, String::new(), 0)
        }
    }

    fn count_root_nodes(
        pointer_scan_levels: &[PointerScanLevelCandidates],
        offset_radius: u64,
    ) -> u64 {
        let mut root_node_count = 0_u64;

        for pointer_scan_level_candidates in pointer_scan_levels.iter().rev() {
            let child_target_ranges = pointer_scan_level_candidates
                .get_discovery_depth()
                .checked_sub(2)
                .and_then(|child_level_index| pointer_scan_levels.get(child_level_index as usize))
                .map(|child_pointer_scan_level_candidates| {
                    PointerScanTargetRangeSet::from_target_addresses(
                        &child_pointer_scan_level_candidates
                            .get_heap_candidates()
                            .iter()
                            .map(PointerScanCandidate::get_pointer_address)
                            .collect::<Vec<_>>(),
                        offset_radius,
                    )
                });

            for static_candidate in pointer_scan_level_candidates.get_static_candidates() {
                if static_candidate.get_discovery_depth() <= 1 {
                    root_node_count = root_node_count.saturating_add(1);
                    continue;
                }

                let has_matching_child = child_target_ranges
                    .as_ref()
                    .map(|child_target_ranges| child_target_ranges.contains_value_binary(static_candidate.get_pointer_value()))
                    .unwrap_or(false);

                if has_matching_child {
                    root_node_count = root_node_count.saturating_add(1);
                }
            }
        }

        root_node_count
    }

    fn compare_discovered_pointer_candidates(
        left_pointer_node: &DiscoveredPointerCandidate,
        right_pointer_node: &DiscoveredPointerCandidate,
    ) -> Ordering {
        left_pointer_node
            .pointer_address
            .cmp(&right_pointer_node.pointer_address)
            .then_with(|| {
                left_pointer_node
                    .pointer_value
                    .cmp(&right_pointer_node.pointer_value)
            })
            .then_with(|| {
                left_pointer_node
                    .module_name
                    .cmp(&right_pointer_node.module_name)
            })
            .then_with(|| {
                left_pointer_node
                    .module_offset
                    .cmp(&right_pointer_node.module_offset)
            })
            .then_with(|| {
                let left_node_class = match left_pointer_node.pointer_scan_node_type {
                    PointerScanNodeType::Heap => 0_u8,
                    PointerScanNodeType::Static => 1_u8,
                };
                let right_node_class = match right_pointer_node.pointer_scan_node_type {
                    PointerScanNodeType::Heap => 0_u8,
                    PointerScanNodeType::Static => 1_u8,
                };

                left_node_class.cmp(&right_node_class)
            })
    }

    fn create_empty_session(
        pointer_scan_session_id: u64,
        pointer_scan_parameters: &PointerScanParameters,
    ) -> PointerScanSession {
        PointerScanSession::new(
            pointer_scan_session_id,
            pointer_scan_parameters.get_target_address(),
            pointer_scan_parameters.get_pointer_size(),
            pointer_scan_parameters.get_max_depth(),
            pointer_scan_parameters.get_offset_radius(),
            Vec::new(),
            Vec::new(),
            0,
            0,
            0,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::PointerScanExecutor;
    use crate::scanners::scan_execution_context::ScanExecutionContext;
    use squalr_engine_api::structures::memory::bitness::Bitness;
    use squalr_engine_api::structures::memory::normalized_module::NormalizedModule;
    use squalr_engine_api::structures::memory::normalized_region::NormalizedRegion;
    use squalr_engine_api::structures::pointer_scans::pointer_scan_node_type::PointerScanNodeType;
    use squalr_engine_api::structures::pointer_scans::pointer_scan_pointer_size::PointerScanPointerSize;
    use squalr_engine_api::structures::processes::opened_process_info::OpenedProcessInfo;
    use squalr_engine_api::structures::scanning::plans::pointer_scan::pointer_scan_parameters::PointerScanParameters;
    use squalr_engine_api::structures::snapshots::snapshot::Snapshot;
    use squalr_engine_api::structures::snapshots::snapshot_region::SnapshotRegion;
    use std::collections::HashMap;
    use std::sync::{Arc, RwLock};

    #[test]
    fn execute_scan_builds_pointer_chains_and_classifies_static_nodes() {
        let memory_map = Arc::new(build_pointer_scan_memory_map());
        let scan_execution_context = ScanExecutionContext::new(
            None,
            None,
            Some(Arc::new({
                let memory_map = memory_map.clone();

                move |_opened_process_info, address, values| read_memory_from_map(&memory_map, address, values)
            })),
        );
        let snapshot = Arc::new(RwLock::new(build_pointer_scan_snapshot()));
        let pointer_scan_parameters = PointerScanParameters::new(0x3010, PointerScanPointerSize::Pointer64, 0x20, 3, true, false);
        let mut pointer_scan_session = PointerScanExecutor::execute_scan(
            OpenedProcessInfo::new(7, "pointer-test".to_string(), 0, Bitness::Bit64, None),
            snapshot.clone(),
            snapshot,
            41,
            pointer_scan_parameters,
            &[NormalizedModule::new("game.exe", 0x1000, 0x100)],
            false,
            &scan_execution_context,
        );

        assert_eq!(pointer_scan_session.get_session_id(), 41);
        assert_eq!(pointer_scan_session.get_root_node_count(), 2);
        assert_eq!(pointer_scan_session.get_total_node_count(), 3);
        assert_eq!(pointer_scan_session.get_total_static_node_count(), 2);
        assert_eq!(pointer_scan_session.get_total_heap_node_count(), 1);

        let pointer_scan_levels = pointer_scan_session.get_pointer_scan_levels();
        assert_eq!(pointer_scan_levels.len(), 2);
        assert_eq!(pointer_scan_levels[0].get_depth(), 1);
        assert_eq!(pointer_scan_levels[0].get_node_count(), 2);
        assert_eq!(pointer_scan_levels[0].get_static_node_count(), 1);
        assert_eq!(pointer_scan_levels[0].get_heap_node_count(), 1);
        assert_eq!(pointer_scan_levels[1].get_depth(), 2);
        assert_eq!(pointer_scan_levels[1].get_node_count(), 1);
        assert_eq!(pointer_scan_levels[1].get_static_node_count(), 1);
        assert_eq!(pointer_scan_levels[1].get_heap_node_count(), 0);

        let root_nodes = pointer_scan_session.get_expanded_nodes(None);
        assert_eq!(root_nodes.len(), 2);

        let static_chain_root = root_nodes
            .iter()
            .find(|pointer_scan_node| pointer_scan_node.get_pointer_address() == 0x1010)
            .expect("Expected the rooted static pointer chain.");
        let direct_static_root = root_nodes
            .iter()
            .find(|pointer_scan_node| pointer_scan_node.get_pointer_address() == 0x1030)
            .expect("Expected the direct static pointer chain.");

        assert_eq!(static_chain_root.get_pointer_scan_node_type(), PointerScanNodeType::Static);
        assert_eq!(static_chain_root.get_depth(), 1);
        assert_eq!(static_chain_root.get_module_name(), "game.exe");
        assert_eq!(static_chain_root.get_module_offset(), 0x10);
        assert_eq!(static_chain_root.get_resolved_target_address(), 0x1FF0);
        assert_eq!(static_chain_root.get_pointer_offset(), 0);
        assert!(static_chain_root.has_children());

        let child_nodes = pointer_scan_session.get_expanded_nodes(Some(static_chain_root.get_node_id()));
        assert_eq!(child_nodes.len(), 1);
        assert_eq!(child_nodes[0].get_pointer_address(), 0x1010);
        assert_eq!(child_nodes[0].get_pointer_scan_node_type(), PointerScanNodeType::Static);
        assert_eq!(child_nodes[0].get_depth(), 2);
        assert_eq!(child_nodes[0].get_resolved_target_address(), 0x2000);
        assert_eq!(child_nodes[0].get_pointer_offset(), 0x10);
        assert!(child_nodes[0].has_children());

        let grandchild_nodes = pointer_scan_session.get_expanded_nodes(Some(child_nodes[0].get_node_id()));
        assert_eq!(grandchild_nodes.len(), 1);
        assert_eq!(grandchild_nodes[0].get_pointer_address(), 0x2000);
        assert_eq!(grandchild_nodes[0].get_pointer_scan_node_type(), PointerScanNodeType::Heap);
        assert_eq!(grandchild_nodes[0].get_depth(), 3);
        assert_eq!(grandchild_nodes[0].get_resolved_target_address(), 0x3010);
        assert_eq!(grandchild_nodes[0].get_pointer_offset(), 0x10);
        assert!(!grandchild_nodes[0].has_children());

        assert_eq!(direct_static_root.get_pointer_scan_node_type(), PointerScanNodeType::Static);
        assert_eq!(direct_static_root.get_depth(), 1);
        assert_eq!(direct_static_root.get_module_name(), "game.exe");
        assert_eq!(direct_static_root.get_module_offset(), 0x30);
        assert_eq!(direct_static_root.get_resolved_target_address(), 0x3010);
        assert_eq!(direct_static_root.get_pointer_offset(), -0x10);
        assert!(!direct_static_root.has_children());
    }

    #[test]
    fn execute_scan_omits_terminal_heap_candidates() {
        let memory_map = Arc::new(build_terminal_heap_pointer_scan_memory_map());
        let scan_execution_context = ScanExecutionContext::new(
            None,
            None,
            Some(Arc::new({
                let memory_map = memory_map.clone();

                move |_opened_process_info, address, values| read_memory_from_map(&memory_map, address, values)
            })),
        );
        let snapshot = Arc::new(RwLock::new(build_terminal_heap_pointer_scan_snapshot()));
        let pointer_scan_parameters = PointerScanParameters::new(0x3010, PointerScanPointerSize::Pointer64, 0x20, 2, true, false);
        let pointer_scan_session = PointerScanExecutor::execute_scan(
            OpenedProcessInfo::new(8, "pointer-terminal-heap-test".to_string(), 0, Bitness::Bit64, None),
            snapshot.clone(),
            snapshot,
            42,
            pointer_scan_parameters,
            &[NormalizedModule::new("game.exe", 0x1000, 0x100)],
            false,
            &scan_execution_context,
        );

        let pointer_scan_levels = pointer_scan_session.get_pointer_scan_levels();

        assert_eq!(pointer_scan_levels.len(), 2);
        assert_eq!(pointer_scan_levels[0].get_heap_node_count(), 1);
        assert_eq!(pointer_scan_levels[1].get_static_node_count(), 1);
        assert_eq!(pointer_scan_levels[1].get_heap_node_count(), 0);
        assert_eq!(pointer_scan_session.get_total_heap_node_count(), 1);
    }

    fn build_pointer_scan_snapshot() -> Snapshot {
        let mut snapshot = Snapshot::new();

        snapshot.set_snapshot_regions(vec![
            SnapshotRegion::new(NormalizedRegion::new(0x1000, 0x40), Vec::new()),
            SnapshotRegion::new(NormalizedRegion::new(0x2000, 0x40), Vec::new()),
            SnapshotRegion::new(NormalizedRegion::new(0x3000, 0x40), Vec::new()),
        ]);

        snapshot
    }

    fn build_terminal_heap_pointer_scan_snapshot() -> Snapshot {
        let mut snapshot = Snapshot::new();

        snapshot.set_snapshot_regions(vec![
            SnapshotRegion::new(NormalizedRegion::new(0x1000, 0x40), Vec::new()),
            SnapshotRegion::new(NormalizedRegion::new(0x2000, 0x40), Vec::new()),
            SnapshotRegion::new(NormalizedRegion::new(0x4000, 0x40), Vec::new()),
        ]);

        snapshot
    }

    fn build_pointer_scan_memory_map() -> HashMap<u64, u8> {
        let mut memory_map = HashMap::new();

        write_pointer_bytes(&mut memory_map, 0x1010, 0x1FF0_u64);
        write_pointer_bytes(&mut memory_map, 0x1030, 0x3020_u64);
        write_pointer_bytes(&mut memory_map, 0x2000, 0x3000_u64);

        memory_map
    }

    fn build_terminal_heap_pointer_scan_memory_map() -> HashMap<u64, u8> {
        let mut memory_map = HashMap::new();

        write_pointer_bytes(&mut memory_map, 0x1030, 0x3020_u64);
        write_pointer_bytes(&mut memory_map, 0x1020, 0x2000_u64);
        write_pointer_bytes(&mut memory_map, 0x2000, 0x3000_u64);
        write_pointer_bytes(&mut memory_map, 0x4000, 0x2000_u64);

        memory_map
    }

    fn write_pointer_bytes(
        memory_map: &mut HashMap<u64, u8>,
        address: u64,
        value: u64,
    ) {
        for (byte_index, byte_value) in value.to_le_bytes().iter().enumerate() {
            memory_map.insert(address.saturating_add(byte_index as u64), *byte_value);
        }
    }

    fn read_memory_from_map(
        memory_map: &HashMap<u64, u8>,
        address: u64,
        values: &mut [u8],
    ) -> bool {
        for (byte_index, value) in values.iter_mut().enumerate() {
            *value = *memory_map
                .get(&address.saturating_add(byte_index as u64))
                .unwrap_or(&0);
        }

        true
    }
}
