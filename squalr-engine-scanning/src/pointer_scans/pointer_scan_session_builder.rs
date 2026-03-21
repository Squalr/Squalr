use crate::pointer_scans::structures::discovered_pointer_level::DiscoveredPointerLevel;
use squalr_engine_api::structures::memory::normalized_module::NormalizedModule;
use squalr_engine_api::structures::pointer_scans::pointer_scan_candidate::PointerScanCandidate;
use squalr_engine_api::structures::pointer_scans::pointer_scan_level::PointerScanLevel;
use squalr_engine_api::structures::pointer_scans::pointer_scan_level_candidates::PointerScanLevelCandidates;
use squalr_engine_api::structures::pointer_scans::pointer_scan_node_type::PointerScanNodeType;
use squalr_engine_api::structures::pointer_scans::pointer_scan_session::PointerScanSession;
use squalr_engine_api::structures::scanning::plans::pointer_scan::pointer_scan_parameters::PointerScanParameters;

pub(crate) struct PointerScanSessionBuilder;

impl PointerScanSessionBuilder {
    pub(crate) fn build_session(
        pointer_scan_session_id: u64,
        pointer_scan_parameters: &PointerScanParameters,
        modules: &[NormalizedModule],
        discovered_pointer_levels: &[DiscoveredPointerLevel],
        with_logging: bool,
    ) -> PointerScanSession {
        if discovered_pointer_levels.is_empty() {
            if with_logging {
                log::info!("Pointer scan found no reachable pointer nodes.");
            }

            return Self::create_empty_session(pointer_scan_session_id, pointer_scan_parameters);
        }

        let mut pointer_scan_levels = Vec::new();
        let mut pointer_scan_level_candidates = Vec::new();
        let mut next_candidate_id = 1_u64;
        let mut total_static_node_count = 0_u64;
        let mut total_heap_node_count = 0_u64;
        let module_names = modules
            .iter()
            .map(|module| module.get_module_name().to_string())
            .collect::<Vec<_>>();

        for (pointer_level_index, discovered_pointer_level) in discovered_pointer_levels.iter().enumerate() {
            let discovery_depth = pointer_level_index as u64 + 1;
            let level_candidates = Self::build_level_candidates(discovery_depth, discovered_pointer_level, &mut next_candidate_id);

            total_static_node_count = total_static_node_count.saturating_add(level_candidates.get_static_node_count());
            total_heap_node_count = total_heap_node_count.saturating_add(level_candidates.get_heap_node_count());
            pointer_scan_levels.push(PointerScanLevel::new(
                discovery_depth,
                level_candidates.get_node_count(),
                level_candidates.get_static_node_count(),
                level_candidates.get_heap_node_count(),
            ));
            pointer_scan_level_candidates.push(level_candidates);
        }
        let root_node_count = total_static_node_count;

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
        }

        PointerScanSession::new(
            pointer_scan_session_id,
            pointer_scan_parameters.get_target_address(),
            pointer_scan_parameters.get_pointer_size(),
            pointer_scan_parameters.get_max_depth(),
            pointer_scan_parameters.get_offset_radius(),
            module_names,
            pointer_scan_levels,
            pointer_scan_level_candidates,
            root_node_count,
            total_static_node_count,
            total_heap_node_count,
        )
    }

    pub(crate) fn create_empty_session(
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
            Vec::new(),
            0,
            0,
            0,
        )
    }

    fn build_level_candidates(
        discovery_depth: u64,
        discovered_pointer_level: &DiscoveredPointerLevel,
        next_candidate_id: &mut u64,
    ) -> PointerScanLevelCandidates {
        let mut static_candidates = Vec::with_capacity(discovered_pointer_level.static_candidates.len());

        for discovered_pointer_candidate in &discovered_pointer_level.static_candidates {
            static_candidates.push(PointerScanCandidate::new(
                *next_candidate_id,
                discovery_depth,
                PointerScanNodeType::Static,
                discovered_pointer_candidate.pointer_address,
                discovered_pointer_candidate.pointer_value,
                discovered_pointer_candidate.module_index,
                discovered_pointer_candidate.module_offset,
            ));
            *next_candidate_id = next_candidate_id.saturating_add(1);
        }

        let mut heap_candidates = Vec::with_capacity(discovered_pointer_level.heap_candidates.len());

        for discovered_pointer_candidate in &discovered_pointer_level.heap_candidates {
            heap_candidates.push(PointerScanCandidate::new(
                *next_candidate_id,
                discovery_depth,
                PointerScanNodeType::Heap,
                discovered_pointer_candidate.pointer_address,
                discovered_pointer_candidate.pointer_value,
                0,
                0,
            ));
            *next_candidate_id = next_candidate_id.saturating_add(1);
        }

        PointerScanLevelCandidates::new_presorted(discovery_depth, static_candidates, heap_candidates)
    }
}
