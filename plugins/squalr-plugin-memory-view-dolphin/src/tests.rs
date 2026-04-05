use crate::{
    DolphinMemoryViewPlugin,
    address_space::{
        DolphinMemoryRegionDescriptor, DolphinMemoryRegionKind, find_dolphin_region_by_module_name, find_dolphin_region_by_virtual_address, gc_wii_module_name,
        resolve_virtual_address_from_module,
    },
    discovery::{is_probable_gamecube_game_id, is_probable_gba_header, select_dolphin_memory_regions},
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

    assert_eq!(region_descriptor.get_module_name(), gc_wii_module_name());
    assert_eq!(region_descriptor.virtual_to_host_address(0x8000_0010), Some(0x0000_7FF6_0000_0010));
    assert_eq!(region_descriptor.host_to_virtual_address(0x0000_7FF6_0000_0010), Some(0x8000_0010));
    assert_eq!(region_descriptor.module_relative_address(0x8000_0010), Some(("gc_wii".to_string(), 0x10)));
}

#[test]
fn wii_extended_memory_maps_guest_addresses_to_host_addresses() {
    let region_descriptor = DolphinMemoryRegionDescriptor::new(DolphinMemoryRegionKind::WiiExtendedMemory, 0x0000_7FF6_2000_0000);

    assert_eq!(region_descriptor.get_module_name(), gc_wii_module_name());
    assert_eq!(region_descriptor.virtual_to_host_address(0x9000_1234), Some(0x0000_7FF6_2000_1234));
    assert_eq!(region_descriptor.host_to_virtual_address(0x0000_7FF6_2000_1234), Some(0x9000_1234));
    assert_eq!(
        region_descriptor.module_relative_address(0x9000_1234),
        Some(("gc_wii".to_string(), 0x0200_1234))
    );
}

#[test]
fn gba_memory_maps_virtual_addresses_to_host_addresses() {
    let region_descriptor = DolphinMemoryRegionDescriptor::new(DolphinMemoryRegionKind::GameBoyAdvanceWorkRam(2), 0x0000_7FF6_5000_0000);

    assert_eq!(region_descriptor.get_module_name(), "gba_wm_2");
    assert_eq!(region_descriptor.virtual_to_host_address(0xA010_0010), Some(0x0000_7FF6_5000_0010));
    assert_eq!(region_descriptor.host_to_virtual_address(0x0000_7FF6_5000_0010), Some(0xA010_0010));
    assert_eq!(region_descriptor.module_relative_address(0xA010_0010), Some(("gba_wm_2".to_string(), 0x10)));
}

#[test]
fn region_lookup_finds_guest_address_and_module_name_matches() {
    let region_descriptors = vec![
        DolphinMemoryRegionDescriptor::new(DolphinMemoryRegionKind::GameCubeMainMemory, 0x1000),
        DolphinMemoryRegionDescriptor::new(DolphinMemoryRegionKind::WiiExtendedMemory, 0x4000),
    ];

    let region_from_guest_address =
        find_dolphin_region_by_virtual_address(&region_descriptors, 0x9000_0010).expect("Expected a Wii extended-memory region for a Wii guest address.");
    let region_from_module_name =
        find_dolphin_region_by_module_name(&region_descriptors, "gc_wii").expect("Expected a GameCube/Wii memory region for the gc_wii module name.");

    assert_eq!(region_from_guest_address.get_region_kind(), DolphinMemoryRegionKind::WiiExtendedMemory);
    assert_eq!(region_from_module_name.get_region_kind(), DolphinMemoryRegionKind::GameCubeMainMemory);
}

#[test]
fn gc_wii_and_gba_module_offsets_resolve_to_virtual_addresses() {
    assert_eq!(resolve_virtual_address_from_module("gc_wii", 0x1234), Some(0x8000_1234));
    assert_eq!(resolve_virtual_address_from_module("gc_wii", 0x0200_1234), Some(0x9000_1234));
    assert_eq!(resolve_virtual_address_from_module("gba_im_3", 0x20), Some(0xA024_0020));
    assert_eq!(resolve_virtual_address_from_module("mem1", 0x10), Some(0x8000_0010));
    assert_eq!(resolve_virtual_address_from_module("wii", 0x10), Some(0x9000_0010));
}

#[test]
fn gamecube_game_id_validation_accepts_expected_prefixes_and_ascii() {
    assert!(is_probable_gamecube_game_id(b"GZLE01"));
    assert!(is_probable_gamecube_game_id(b"R5WEA4"));
    assert!(!is_probable_gamecube_game_id(b"XZLE01"));
    assert!(!is_probable_gamecube_game_id(b"GZL*01"));
}

#[test]
fn gba_header_validation_accepts_strict_cartridge_header() {
    let mut header_bytes = [0u8; 0xC0];

    header_bytes[0x03] = 0xEA;
    header_bytes[0x04..0xA0].copy_from_slice(&[
        0x24, 0xFF, 0xAE, 0x51, 0x69, 0x9A, 0xA2, 0x21, 0x3D, 0x84, 0x82, 0x0A, 0x84, 0xE4, 0x09, 0xAD, 0x11, 0x24, 0x8B, 0x98, 0xC0, 0x81, 0x7F, 0x21, 0xA3,
        0x52, 0xBE, 0x19, 0x93, 0x09, 0xCE, 0x20, 0x10, 0x46, 0x4A, 0x4A, 0xF8, 0x27, 0x31, 0xEC, 0x58, 0xC7, 0xE8, 0x33, 0x82, 0xE3, 0xCE, 0xBF, 0x85, 0xF4,
        0xDF, 0x94, 0xCE, 0x4B, 0x09, 0xC1, 0x94, 0x56, 0x8A, 0xC0, 0x13, 0x72, 0xA7, 0xFC, 0x9F, 0x84, 0x4D, 0x73, 0xA3, 0xCA, 0x9A, 0x61, 0x58, 0x97, 0xA3,
        0x27, 0xFC, 0x03, 0x98, 0x76, 0x23, 0x1D, 0xC7, 0x61, 0x03, 0x04, 0xAE, 0x56, 0xBF, 0x38, 0x84, 0x00, 0x40, 0xA7, 0x0E, 0xFD, 0xFF, 0x52, 0xFE, 0x03,
        0x6F, 0x95, 0x30, 0xF1, 0x97, 0xFB, 0xC0, 0x85, 0x60, 0xD6, 0x80, 0x25, 0xA9, 0x63, 0xBE, 0x03, 0x01, 0x4E, 0x38, 0xE2, 0xF9, 0xA2, 0x34, 0xFF, 0xBB,
        0x3E, 0x03, 0x44, 0x78, 0x00, 0x90, 0xCB, 0x88, 0x11, 0x3A, 0x94, 0x65, 0xC0, 0x7C, 0x63, 0x87, 0xF0, 0x3C, 0xAF, 0xD6, 0x25, 0xE4, 0x8B, 0x38, 0x0A,
        0xAC, 0x72, 0x21, 0xD4, 0xF8, 0x07,
    ]);
    header_bytes[0xAC..0xB0].copy_from_slice(b"GCCJ");
    header_bytes[0xB0..0xB2].copy_from_slice(b"GC");
    header_bytes[0xB2] = 0x96;
    header_bytes[0xBD] = 0xB0;

    assert!(is_probable_gba_header(&header_bytes));

    header_bytes[0xBD] = 0x00;

    assert!(!is_probable_gba_header(&header_bytes));
}

#[test]
fn discovery_selects_gc_and_wii_regions_by_size() {
    let raw_memory_regions = vec![
        NormalizedRegion::new(0x1000, 0x1000),
        NormalizedRegion::new(0x2000_0000, DolphinMemoryRegionKind::GameCubeMainMemory.host_region_size()),
        NormalizedRegion::new(0x3000_0000, DolphinMemoryRegionKind::WiiExtendedMemory.host_region_size()),
    ];
    let discovered_region_descriptors = select_dolphin_memory_regions(
        &raw_memory_regions,
        |candidate_region| (candidate_region.get_base_address() == 0x2000_0000).then_some(crate::discovery::DolphinConsoleKind::Wii),
        |_candidate_region| false,
    );

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
    let discovered_region_descriptors = select_dolphin_memory_regions(&raw_memory_regions, |_candidate_region| None, |_candidate_region| false);

    assert!(discovered_region_descriptors.is_empty());
}

#[test]
fn discovery_does_not_expose_mem2_for_gamecube_titles() {
    let raw_memory_regions = vec![
        NormalizedRegion::new(0x2000_0000, DolphinMemoryRegionKind::GameCubeMainMemory.host_region_size()),
        NormalizedRegion::new(0x3000_0000, DolphinMemoryRegionKind::WiiExtendedMemory.host_region_size()),
    ];
    let discovered_region_descriptors = select_dolphin_memory_regions(
        &raw_memory_regions,
        |candidate_region| (candidate_region.get_base_address() == 0x2000_0000).then_some(crate::discovery::DolphinConsoleKind::GameCube),
        |_candidate_region| false,
    );

    assert_eq!(discovered_region_descriptors.len(), 1);
    assert_eq!(discovered_region_descriptors[0].get_region_kind(), DolphinMemoryRegionKind::GameCubeMainMemory);
}

#[test]
fn discovery_exposes_mem2_for_wii_titles() {
    let raw_memory_regions = vec![
        NormalizedRegion::new(0x2000_0000, DolphinMemoryRegionKind::GameCubeMainMemory.host_region_size()),
        NormalizedRegion::new(0x3000_0000, DolphinMemoryRegionKind::WiiExtendedMemory.host_region_size()),
    ];
    let discovered_region_descriptors = select_dolphin_memory_regions(
        &raw_memory_regions,
        |candidate_region| (candidate_region.get_base_address() == 0x2000_0000).then_some(crate::discovery::DolphinConsoleKind::Wii),
        |_candidate_region| false,
    );

    assert_eq!(discovered_region_descriptors.len(), 2);
    assert_eq!(discovered_region_descriptors[0].get_region_kind(), DolphinMemoryRegionKind::GameCubeMainMemory);
    assert_eq!(discovered_region_descriptors[1].get_region_kind(), DolphinMemoryRegionKind::WiiExtendedMemory);
}

#[test]
fn discovery_assigns_gba_regions_by_encounter_order() {
    let raw_memory_regions = vec![
        NormalizedRegion::new(0x5000_0000, 0x0004_8000),
        NormalizedRegion::new(0x3000_0000, 0x0004_8000),
    ];
    let discovered_region_descriptors = select_dolphin_memory_regions(
        &raw_memory_regions,
        |_candidate_region| None,
        |candidate_region| candidate_region.get_base_address() == 0x5000_0000 || candidate_region.get_base_address() == 0x3000_0000,
    );

    assert_eq!(discovered_region_descriptors.len(), 4);
    assert_eq!(
        discovered_region_descriptors[0].get_region_kind(),
        DolphinMemoryRegionKind::GameBoyAdvanceWorkRam(1)
    );
    assert_eq!(discovered_region_descriptors[0].get_host_base_address(), 0x5000_0000);
    assert_eq!(
        discovered_region_descriptors[1].get_region_kind(),
        DolphinMemoryRegionKind::GameBoyAdvanceInternalRam(1)
    );
    assert_eq!(discovered_region_descriptors[1].get_host_base_address(), 0x5004_0000);
    assert_eq!(
        discovered_region_descriptors[2].get_region_kind(),
        DolphinMemoryRegionKind::GameBoyAdvanceWorkRam(2)
    );
    assert_eq!(discovered_region_descriptors[2].get_host_base_address(), 0x3000_0000);
    assert_eq!(
        discovered_region_descriptors[3].get_region_kind(),
        DolphinMemoryRegionKind::GameBoyAdvanceInternalRam(2)
    );
    assert_eq!(discovered_region_descriptors[3].get_host_base_address(), 0x3004_0000);
}
