use crate::{
    DolphinMemoryViewPlugin,
    address_space::{DolphinMemoryRegionDescriptor, DolphinMemoryRegionKind, find_dolphin_region_by_guest_address, find_dolphin_region_by_module_name},
    discovery::{is_probable_gamecube_game_id, select_dolphin_memory_regions},
    process_detection::matches_dolphin_process_name,
};
use squalr_engine_api::{
    plugins::memory_view::MemoryViewPlugin,
    structures::{
        memory::{bitness::Bitness, normalized_region::NormalizedRegion},
        processes::opened_process_info::OpenedProcessInfo,
    },
};

#[test]
fn dolphin_plugin_matches_dolphin_process_names() {
    let plugin = DolphinMemoryViewPlugin::new();
    let process_info = OpenedProcessInfo::new(1, "Dolphin.exe".to_string(), 0, Bitness::Bit64, None);

    assert!(plugin.can_attach(&process_info));
}

#[test]
fn dolphin_plugin_rejects_non_dolphin_process_names() {
    let plugin = DolphinMemoryViewPlugin::new();
    let process_info = OpenedProcessInfo::new(1, "notepad.exe".to_string(), 0, Bitness::Bit64, None);

    assert!(!plugin.can_attach(&process_info));
}

#[test]
fn process_detection_matches_slippi_names() {
    assert!(matches_dolphin_process_name("Slippi Launcher.exe"));
}

#[test]
fn gamecube_main_memory_maps_guest_addresses_to_host_addresses() {
    let region_descriptor = DolphinMemoryRegionDescriptor::new(DolphinMemoryRegionKind::GameCubeMainMemory, 0x0000_7FF6_0000_0000);

    assert_eq!(region_descriptor.get_module_name(), "GC");
    assert_eq!(region_descriptor.guest_to_host_address(0x8000_0010), Some(0x0000_7FF6_0000_0010));
    assert_eq!(region_descriptor.host_to_guest_address(0x0000_7FF6_0000_0010), Some(0x8000_0010));
}

#[test]
fn wii_extended_memory_maps_guest_addresses_to_host_addresses() {
    let region_descriptor = DolphinMemoryRegionDescriptor::new(DolphinMemoryRegionKind::WiiExtendedMemory, 0x0000_7FF6_2000_0000);

    assert_eq!(region_descriptor.get_module_name(), "Wii");
    assert_eq!(region_descriptor.guest_to_host_address(0x9000_1234), Some(0x0000_7FF6_2000_1234));
    assert_eq!(region_descriptor.host_to_guest_address(0x0000_7FF6_2000_1234), Some(0x9000_1234));
}

#[test]
fn region_lookup_finds_guest_address_and_module_name_matches() {
    let region_descriptors = vec![
        DolphinMemoryRegionDescriptor::new(DolphinMemoryRegionKind::GameCubeMainMemory, 0x1000),
        DolphinMemoryRegionDescriptor::new(DolphinMemoryRegionKind::WiiExtendedMemory, 0x4000),
    ];

    let region_from_guest_address =
        find_dolphin_region_by_guest_address(&region_descriptors, 0x9000_0010).expect("Expected a Wii extended-memory region for a Wii guest address.");
    let region_from_module_name =
        find_dolphin_region_by_module_name(&region_descriptors, "gc").expect("Expected a GameCube main-memory region for the GC module name.");

    assert_eq!(region_from_guest_address.get_region_kind(), DolphinMemoryRegionKind::WiiExtendedMemory);
    assert_eq!(region_from_module_name.get_region_kind(), DolphinMemoryRegionKind::GameCubeMainMemory);
}

#[test]
fn gamecube_game_id_validation_accepts_expected_prefixes_and_ascii() {
    assert!(is_probable_gamecube_game_id(b"GZLE01"));
    assert!(is_probable_gamecube_game_id(b"R5WEA4"));
    assert!(!is_probable_gamecube_game_id(b"XZLE01"));
    assert!(!is_probable_gamecube_game_id(b"GZL*01"));
}

#[test]
fn discovery_selects_gc_and_wii_regions_by_size() {
    let raw_memory_regions = vec![
        NormalizedRegion::new(0x1000, 0x1000),
        NormalizedRegion::new(0x2000_0000, DolphinMemoryRegionKind::GameCubeMainMemory.host_region_size()),
        NormalizedRegion::new(0x3000_0000, DolphinMemoryRegionKind::WiiExtendedMemory.host_region_size()),
    ];
    let discovered_region_descriptors =
        select_dolphin_memory_regions(&raw_memory_regions, |candidate_region| candidate_region.get_base_address() == 0x2000_0000);

    assert_eq!(discovered_region_descriptors.len(), 2);
    assert_eq!(discovered_region_descriptors[0].get_region_kind(), DolphinMemoryRegionKind::GameCubeMainMemory);
    assert_eq!(discovered_region_descriptors[1].get_region_kind(), DolphinMemoryRegionKind::WiiExtendedMemory);
}

#[test]
fn discovery_rejects_gamecube_sized_region_without_valid_header() {
    let raw_memory_regions = vec![NormalizedRegion::new(
        0x2000_0000,
        DolphinMemoryRegionKind::GameCubeMainMemory.host_region_size(),
    )];
    let discovered_region_descriptors = select_dolphin_memory_regions(&raw_memory_regions, |_candidate_region| false);

    assert!(discovered_region_descriptors.is_empty());
}
