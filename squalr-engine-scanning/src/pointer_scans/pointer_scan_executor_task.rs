use crate::pointer_scans::pointer_scan_level_collector::PointerScanLevelCollector;
use crate::pointer_scans::pointer_scan_results_builder::PointerScanResultsBuilder;
use squalr_engine_api::structures::memory::normalized_module::NormalizedModule;
use squalr_engine_api::structures::pointer_scans::pointer_scan_address_space::PointerScanAddressSpace;
use squalr_engine_api::structures::pointer_scans::pointer_scan_results::PointerScanResults;
use squalr_engine_api::structures::pointer_scans::pointer_scan_target_descriptor::PointerScanTargetDescriptor;
use squalr_engine_api::structures::scanning::plans::pointer_scan::pointer_scan_parameters::PointerScanParameters;
use squalr_engine_api::structures::snapshots::snapshot::Snapshot;
use std::sync::{Arc, RwLock};

pub struct PointerScanExecutor;

/// Performs an initial pointer scan against already-collected snapshot values.
impl PointerScanExecutor {
    pub fn execute_scan(
        statics_snapshot: Arc<RwLock<Snapshot>>,
        heaps_snapshot: Arc<RwLock<Snapshot>>,
        pointer_scan_session_id: u64,
        pointer_scan_parameters: PointerScanParameters,
        target_descriptor: PointerScanTargetDescriptor,
        target_addresses: Vec<u64>,
        address_space: PointerScanAddressSpace,
        modules: &[NormalizedModule],
        with_logging: bool,
    ) -> PointerScanResults {
        Self::build_session_from_collected_values(
            statics_snapshot,
            heaps_snapshot,
            pointer_scan_session_id,
            pointer_scan_parameters,
            target_descriptor,
            target_addresses,
            address_space,
            modules,
            with_logging,
        )
    }

    fn build_session_from_collected_values(
        statics_snapshot: Arc<RwLock<Snapshot>>,
        heaps_snapshot: Arc<RwLock<Snapshot>>,
        pointer_scan_session_id: u64,
        pointer_scan_parameters: PointerScanParameters,
        target_descriptor: PointerScanTargetDescriptor,
        target_addresses: Vec<u64>,
        address_space: PointerScanAddressSpace,
        modules: &[NormalizedModule],
        with_logging: bool,
    ) -> PointerScanResults {
        let empty_target_descriptor = target_descriptor.clone();
        let empty_target_addresses = target_addresses.clone();

        Self::with_pointer_scan_snapshots(
            statics_snapshot,
            heaps_snapshot,
            &pointer_scan_parameters,
            pointer_scan_session_id,
            &empty_target_descriptor,
            &empty_target_addresses,
            address_space,
            modules,
            with_logging,
            |snapshots| {
                let discovered_pointer_levels =
                    PointerScanLevelCollector::discover_pointer_levels(snapshots, &target_addresses, &pointer_scan_parameters, modules, with_logging);

                PointerScanResultsBuilder::build_results(
                    pointer_scan_session_id,
                    &pointer_scan_parameters,
                    target_descriptor,
                    target_addresses,
                    address_space,
                    modules,
                    &discovered_pointer_levels,
                    with_logging,
                )
            },
        )
    }

    fn with_pointer_scan_snapshots<BuildSession>(
        statics_snapshot: Arc<RwLock<Snapshot>>,
        heaps_snapshot: Arc<RwLock<Snapshot>>,
        pointer_scan_parameters: &PointerScanParameters,
        pointer_scan_session_id: u64,
        target_descriptor: &PointerScanTargetDescriptor,
        target_addresses: &[u64],
        address_space: PointerScanAddressSpace,
        modules: &[NormalizedModule],
        with_logging: bool,
        build_session: BuildSession,
    ) -> PointerScanResults
    where
        BuildSession: FnOnce(&[&Snapshot]) -> PointerScanResults,
    {
        if Arc::ptr_eq(&statics_snapshot, &heaps_snapshot) {
            let snapshot_guard = match statics_snapshot.read() {
                Ok(snapshot_guard) => snapshot_guard,
                Err(error) => {
                    if with_logging {
                        log::error!("Failed to acquire read lock on pointer scan snapshot: {}", error);
                    }

                    return PointerScanResultsBuilder::create_empty_results(
                        pointer_scan_session_id,
                        pointer_scan_parameters,
                        target_descriptor.clone(),
                        target_addresses.to_vec(),
                        address_space,
                    );
                }
            };
            let snapshots = [&*snapshot_guard];

            return build_session(&snapshots);
        }

        let statics_snapshot_guard = match statics_snapshot.read() {
            Ok(statics_snapshot_guard) => statics_snapshot_guard,
            Err(error) => {
                if with_logging {
                    log::error!("Failed to acquire read lock on static pointer scan snapshot: {}", error);
                }

                return PointerScanResultsBuilder::create_empty_results(
                    pointer_scan_session_id,
                    pointer_scan_parameters,
                    target_descriptor.clone(),
                    target_addresses.to_vec(),
                    address_space,
                );
            }
        };
        let heaps_snapshot_guard = match heaps_snapshot.read() {
            Ok(heaps_snapshot_guard) => heaps_snapshot_guard,
            Err(error) => {
                if with_logging {
                    log::error!("Failed to acquire read lock on heap pointer scan snapshot: {}", error);
                }

                return PointerScanResultsBuilder::create_empty_results(
                    pointer_scan_session_id,
                    pointer_scan_parameters,
                    target_descriptor.clone(),
                    target_addresses.to_vec(),
                    address_space,
                );
            }
        };
        let snapshots = [&*statics_snapshot_guard, &*heaps_snapshot_guard];

        if modules.is_empty() && with_logging {
            log::debug!("Pointer scan is executing without any module metadata for static classification.");
        }

        build_session(&snapshots)
    }
}

#[cfg(test)]
mod tests {
    use super::PointerScanExecutor;
    use squalr_engine_api::structures::memory::normalized_module::NormalizedModule;
    use squalr_engine_api::structures::memory::normalized_region::NormalizedRegion;
    use squalr_engine_api::structures::pointer_scans::pointer_scan_address_space::PointerScanAddressSpace;
    use squalr_engine_api::structures::pointer_scans::pointer_scan_node_type::PointerScanNodeType;
    use squalr_engine_api::structures::pointer_scans::pointer_scan_pointer_size::PointerScanPointerSize;
    use squalr_engine_api::structures::pointer_scans::pointer_scan_target_descriptor::PointerScanTargetDescriptor;
    use squalr_engine_api::structures::scanning::plans::pointer_scan::pointer_scan_parameters::PointerScanParameters;
    use squalr_engine_api::structures::snapshots::snapshot::Snapshot;
    use squalr_engine_api::structures::snapshots::snapshot_region::SnapshotRegion;
    use std::collections::HashMap;
    use std::sync::{Arc, RwLock};

    #[test]
    fn execute_scan_builds_pointer_chains_and_classifies_static_nodes() {
        let memory_map = build_pointer_scan_memory_map();
        let snapshot = Arc::new(RwLock::new(build_pointer_scan_snapshot(&memory_map)));
        let pointer_scan_parameters = PointerScanParameters::new(PointerScanPointerSize::Pointer64, 0x20, 3, true, false);
        let mut pointer_scan_results = PointerScanExecutor::execute_scan(
            snapshot.clone(),
            snapshot,
            41,
            pointer_scan_parameters,
            PointerScanTargetDescriptor::address(0x3010),
            vec![0x3010],
            PointerScanAddressSpace::EmulatorMemory,
            &[NormalizedModule::new("game.exe", 0x1000, 0x100)],
            false,
        );

        assert_eq!(pointer_scan_results.get_session_id(), 41);
        assert_eq!(pointer_scan_results.get_root_node_count(), 2);
        assert_eq!(pointer_scan_results.get_total_node_count(), 3);
        assert_eq!(pointer_scan_results.get_total_static_node_count(), 2);
        assert_eq!(pointer_scan_results.get_total_heap_node_count(), 1);

        let pointer_scan_levels = pointer_scan_results.get_pointer_scan_levels();
        assert_eq!(pointer_scan_levels.len(), 2);
        assert_eq!(pointer_scan_levels[0].get_depth(), 1);
        assert_eq!(pointer_scan_levels[0].get_node_count(), 2);
        assert_eq!(pointer_scan_levels[0].get_static_node_count(), 1);
        assert_eq!(pointer_scan_levels[0].get_heap_node_count(), 1);
        assert_eq!(pointer_scan_levels[1].get_depth(), 2);
        assert_eq!(pointer_scan_levels[1].get_node_count(), 1);
        assert_eq!(pointer_scan_levels[1].get_static_node_count(), 1);
        assert_eq!(pointer_scan_levels[1].get_heap_node_count(), 0);

        let root_nodes = pointer_scan_results.get_expanded_nodes(None);
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

        let child_nodes = pointer_scan_results.get_expanded_nodes(Some(static_chain_root.get_node_id()));
        assert_eq!(child_nodes.len(), 1);
        assert_eq!(child_nodes[0].get_pointer_address(), 0x1010);
        assert_eq!(child_nodes[0].get_pointer_scan_node_type(), PointerScanNodeType::Static);
        assert_eq!(child_nodes[0].get_depth(), 2);
        assert_eq!(child_nodes[0].get_resolved_target_address(), 0x2000);
        assert_eq!(child_nodes[0].get_pointer_offset(), 0x10);
        assert!(child_nodes[0].has_children());

        let grandchild_nodes = pointer_scan_results.get_expanded_nodes(Some(child_nodes[0].get_node_id()));
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
        let memory_map = build_terminal_heap_pointer_scan_memory_map();
        let snapshot = Arc::new(RwLock::new(build_terminal_heap_pointer_scan_snapshot(&memory_map)));
        let pointer_scan_parameters = PointerScanParameters::new(PointerScanPointerSize::Pointer64, 0x20, 2, true, false);
        let pointer_scan_results = PointerScanExecutor::execute_scan(
            snapshot.clone(),
            snapshot,
            42,
            pointer_scan_parameters,
            PointerScanTargetDescriptor::address(0x3010),
            vec![0x3010],
            PointerScanAddressSpace::EmulatorMemory,
            &[NormalizedModule::new("game.exe", 0x1000, 0x100)],
            false,
        );

        let pointer_scan_levels = pointer_scan_results.get_pointer_scan_levels();

        assert_eq!(pointer_scan_levels.len(), 2);
        assert_eq!(pointer_scan_levels[0].get_heap_node_count(), 1);
        assert_eq!(pointer_scan_levels[1].get_static_node_count(), 1);
        assert_eq!(pointer_scan_levels[1].get_heap_node_count(), 0);
        assert_eq!(pointer_scan_results.get_total_heap_node_count(), 1);
    }

    fn build_pointer_scan_snapshot(memory_map: &HashMap<u64, u8>) -> Snapshot {
        let mut snapshot = Snapshot::new();

        snapshot.set_snapshot_regions(vec![
            build_snapshot_region(NormalizedRegion::new(0x1000, 0x40), memory_map),
            build_snapshot_region(NormalizedRegion::new(0x2000, 0x40), memory_map),
            build_snapshot_region(NormalizedRegion::new(0x3000, 0x40), memory_map),
        ]);

        snapshot
    }

    fn build_terminal_heap_pointer_scan_snapshot(memory_map: &HashMap<u64, u8>) -> Snapshot {
        let mut snapshot = Snapshot::new();

        snapshot.set_snapshot_regions(vec![
            build_snapshot_region(NormalizedRegion::new(0x1000, 0x40), memory_map),
            build_snapshot_region(NormalizedRegion::new(0x2000, 0x40), memory_map),
            build_snapshot_region(NormalizedRegion::new(0x4000, 0x40), memory_map),
        ]);

        snapshot
    }

    fn build_snapshot_region(
        normalized_region: NormalizedRegion,
        memory_map: &HashMap<u64, u8>,
    ) -> SnapshotRegion {
        let mut snapshot_region = SnapshotRegion::new(normalized_region.clone(), Vec::new());
        snapshot_region.current_values = (0..normalized_region.get_region_size())
            .map(|byte_offset| {
                *memory_map
                    .get(&normalized_region.get_base_address().saturating_add(byte_offset))
                    .unwrap_or(&0)
            })
            .collect();

        snapshot_region
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
}
