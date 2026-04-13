use crate::pointer_scans::structures::pointer_scan_collected_level::PointerScanCollectedLevel;
use squalr_engine_api::structures::memory::normalized_module::NormalizedModule;
use squalr_engine_api::structures::pointer_scans::pointer_scan_address_space::PointerScanAddressSpace;
use squalr_engine_api::structures::pointer_scans::pointer_scan_candidate::PointerScanCandidate;
use squalr_engine_api::structures::pointer_scans::pointer_scan_level::PointerScanLevel;
use squalr_engine_api::structures::pointer_scans::pointer_scan_level_candidates::PointerScanLevelCandidates;
use squalr_engine_api::structures::pointer_scans::pointer_scan_node_type::PointerScanNodeType;
use squalr_engine_api::structures::pointer_scans::pointer_scan_results::PointerScanResults;
use squalr_engine_api::structures::pointer_scans::pointer_scan_target_descriptor::PointerScanTargetDescriptor;
use squalr_engine_api::structures::scanning::plans::pointer_scan::pointer_scan_parameters::PointerScanParameters;

pub(crate) struct PointerScanResultsBuilder;

impl PointerScanResultsBuilder {
    pub(crate) fn build_results(
        pointer_scan_results_id: u64,
        pointer_scan_parameters: &PointerScanParameters,
        target_descriptor: PointerScanTargetDescriptor,
        target_addresses: Vec<u64>,
        address_space: PointerScanAddressSpace,
        modules: &[NormalizedModule],
        collected_pointer_levels: &[PointerScanCollectedLevel],
        with_logging: bool,
    ) -> PointerScanResults {
        let module_names = modules
            .iter()
            .map(|module| module.get_module_name().to_string())
            .collect::<Vec<_>>();

        Self::build_results_with_module_names(
            pointer_scan_results_id,
            pointer_scan_parameters,
            target_descriptor,
            target_addresses,
            address_space,
            module_names,
            collected_pointer_levels,
            with_logging,
        )
    }

    pub(crate) fn build_results_with_module_names(
        pointer_scan_results_id: u64,
        pointer_scan_parameters: &PointerScanParameters,
        target_descriptor: PointerScanTargetDescriptor,
        target_addresses: Vec<u64>,
        address_space: PointerScanAddressSpace,
        module_names: Vec<String>,
        collected_pointer_levels: &[PointerScanCollectedLevel],
        with_logging: bool,
    ) -> PointerScanResults {
        if collected_pointer_levels.is_empty() {
            if with_logging {
                log::info!("Pointer scan found no reachable pointer nodes.");
            }

            return Self::create_empty_results(
                pointer_scan_results_id,
                pointer_scan_parameters,
                target_descriptor,
                target_addresses,
                address_space,
            );
        }

        let mut pointer_scan_levels = Vec::new();
        let mut pointer_scan_level_candidates = Vec::new();
        let mut next_candidate_id = 1_u64;
        let mut total_static_node_count = 0_u64;
        let mut total_heap_node_count = 0_u64;

        for (pointer_level_index, collected_pointer_level) in collected_pointer_levels.iter().enumerate() {
            let discovery_depth = pointer_level_index as u64 + 1;
            let level_candidates = Self::build_level_candidates(discovery_depth, collected_pointer_level, &mut next_candidate_id);

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

        PointerScanResults::new(
            pointer_scan_results_id,
            target_descriptor,
            target_addresses,
            address_space,
            pointer_scan_parameters.get_pointer_size(),
            pointer_scan_parameters.get_max_depth(),
            pointer_scan_parameters.get_offset_radius(),
            module_names,
            pointer_scan_levels,
            pointer_scan_level_candidates,
            total_static_node_count,
            total_heap_node_count,
        )
    }

    pub(crate) fn create_empty_results(
        pointer_scan_results_id: u64,
        pointer_scan_parameters: &PointerScanParameters,
        target_descriptor: PointerScanTargetDescriptor,
        target_addresses: Vec<u64>,
        address_space: PointerScanAddressSpace,
    ) -> PointerScanResults {
        PointerScanResults::new(
            pointer_scan_results_id,
            target_descriptor,
            target_addresses,
            address_space,
            pointer_scan_parameters.get_pointer_size(),
            pointer_scan_parameters.get_max_depth(),
            pointer_scan_parameters.get_offset_radius(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            0,
            0,
        )
    }

    fn build_level_candidates(
        discovery_depth: u64,
        collected_pointer_level: &PointerScanCollectedLevel,
        next_candidate_id: &mut u64,
    ) -> PointerScanLevelCandidates {
        let mut static_candidates = Vec::with_capacity(collected_pointer_level.static_candidates.len());

        for collected_pointer_candidate in &collected_pointer_level.static_candidates {
            static_candidates.push(PointerScanCandidate::new(
                *next_candidate_id,
                discovery_depth,
                PointerScanNodeType::Static,
                collected_pointer_candidate.pointer_address,
                collected_pointer_candidate.pointer_value,
                collected_pointer_candidate.module_index,
                collected_pointer_candidate.module_offset,
            ));
            *next_candidate_id = next_candidate_id.saturating_add(1);
        }

        let mut heap_candidates = Vec::with_capacity(collected_pointer_level.heap_candidates.len());

        for collected_pointer_candidate in &collected_pointer_level.heap_candidates {
            heap_candidates.push(PointerScanCandidate::new(
                *next_candidate_id,
                discovery_depth,
                PointerScanNodeType::Heap,
                collected_pointer_candidate.pointer_address,
                collected_pointer_candidate.pointer_value,
                0,
                0,
            ));
            *next_candidate_id = next_candidate_id.saturating_add(1);
        }

        PointerScanLevelCandidates::new_presorted(discovery_depth, static_candidates, heap_candidates)
    }
}
