use crate::address_space::{DolphinMemoryRegionDescriptor, DolphinMemoryRegionKind};
use squalr_engine_api::{
    plugins::memory_view::MemoryViewPluginError,
    structures::{memory::normalized_region::NormalizedRegion, processes::opened_process_info::OpenedProcessInfo},
};
use squalr_engine_operating_system::{
    memory_queryer::{memory_queryer::MemoryQueryer, memory_queryer_trait::MemoryQueryerTrait},
    memory_reader::{MemoryReader, memory_reader_trait::MemoryReaderTrait},
};

const GBA_COMBINED_RAM_SIZE: u64 = 0x0004_8000;
const GBA_WORK_RAM_SIZE: u64 = 0x0004_0000;
const GBA_HEADER_READ_SIZE: usize = 0xC0;
const MAX_DISCOVERED_GBA_SLOT_NUMBER: u8 = 4;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum DolphinConsoleKind {
    GameCube,
    Wii,
}

const NINTENDO_LOGO: [u8; 156] = [
    0x24, 0xFF, 0xAE, 0x51, 0x69, 0x9A, 0xA2, 0x21, 0x3D, 0x84, 0x82, 0x0A, 0x84, 0xE4, 0x09, 0xAD, 0x11, 0x24, 0x8B, 0x98, 0xC0, 0x81, 0x7F, 0x21, 0xA3, 0x52,
    0xBE, 0x19, 0x93, 0x09, 0xCE, 0x20, 0x10, 0x46, 0x4A, 0x4A, 0xF8, 0x27, 0x31, 0xEC, 0x58, 0xC7, 0xE8, 0x33, 0x82, 0xE3, 0xCE, 0xBF, 0x85, 0xF4, 0xDF, 0x94,
    0xCE, 0x4B, 0x09, 0xC1, 0x94, 0x56, 0x8A, 0xC0, 0x13, 0x72, 0xA7, 0xFC, 0x9F, 0x84, 0x4D, 0x73, 0xA3, 0xCA, 0x9A, 0x61, 0x58, 0x97, 0xA3, 0x27, 0xFC, 0x03,
    0x98, 0x76, 0x23, 0x1D, 0xC7, 0x61, 0x03, 0x04, 0xAE, 0x56, 0xBF, 0x38, 0x84, 0x00, 0x40, 0xA7, 0x0E, 0xFD, 0xFF, 0x52, 0xFE, 0x03, 0x6F, 0x95, 0x30, 0xF1,
    0x97, 0xFB, 0xC0, 0x85, 0x60, 0xD6, 0x80, 0x25, 0xA9, 0x63, 0xBE, 0x03, 0x01, 0x4E, 0x38, 0xE2, 0xF9, 0xA2, 0x34, 0xFF, 0xBB, 0x3E, 0x03, 0x44, 0x78, 0x00,
    0x90, 0xCB, 0x88, 0x11, 0x3A, 0x94, 0x65, 0xC0, 0x7C, 0x63, 0x87, 0xF0, 0x3C, 0xAF, 0xD6, 0x25, 0xE4, 0x8B, 0x38, 0x0A, 0xAC, 0x72, 0x21, 0xD4, 0xF8, 0x07,
];

pub(crate) fn discover_dolphin_memory_regions(opened_process_info: &OpenedProcessInfo) -> Result<Vec<DolphinMemoryRegionDescriptor>, MemoryViewPluginError> {
    let raw_memory_regions = MemoryQueryer::query_pages_by_address_range(
        opened_process_info,
        0,
        MemoryQueryer::get_instance().get_max_usermode_address(opened_process_info),
    );
    let discovered_region_descriptors = select_dolphin_memory_regions(
        &raw_memory_regions,
        |candidate_region| classify_gamecube_memory_region(opened_process_info, candidate_region),
        |candidate_region| validate_gba_memory_region(opened_process_info, candidate_region),
    );

    if discovered_region_descriptors.is_empty() {
        return Err(MemoryViewPluginError::unavailable(
            crate::constants::DOLPHIN_PLUGIN_ID,
            format!(
                "no Dolphin memory regions are currently exposed for process `{}`",
                opened_process_info.get_name()
            ),
        ));
    }

    Ok(discovered_region_descriptors)
}

pub(crate) fn select_dolphin_memory_regions(
    raw_memory_regions: &[NormalizedRegion],
    mut classify_gamecube_region: impl FnMut(&NormalizedRegion) -> Option<DolphinConsoleKind>,
    mut validate_gba_region: impl FnMut(&NormalizedRegion) -> bool,
) -> Vec<DolphinMemoryRegionDescriptor> {
    let mut discovered_region_descriptors = Vec::new();
    let mut found_gamecube_main_memory = false;
    let mut found_wii_extended_memory = false;
    let mut next_gba_slot_number = 1u8;
    let mut detected_console_kind = None;

    for candidate_region in raw_memory_regions {
        if !found_gamecube_main_memory && candidate_region.get_region_size() == DolphinMemoryRegionKind::GameCubeMainMemory.host_region_size() {
            if let Some(console_kind) = classify_gamecube_region(candidate_region) {
                discovered_region_descriptors.push(DolphinMemoryRegionDescriptor::new(
                    DolphinMemoryRegionKind::GameCubeMainMemory,
                    candidate_region.get_base_address(),
                ));
                found_gamecube_main_memory = true;
                detected_console_kind = Some(console_kind);
                continue;
            }
        }

        if detected_console_kind == Some(DolphinConsoleKind::Wii)
            && !found_wii_extended_memory
            && candidate_region.get_region_size() == DolphinMemoryRegionKind::WiiExtendedMemory.host_region_size()
        {
            discovered_region_descriptors.push(DolphinMemoryRegionDescriptor::new(
                DolphinMemoryRegionKind::WiiExtendedMemory,
                candidate_region.get_base_address(),
            ));
            found_wii_extended_memory = true;
        }

        if next_gba_slot_number <= MAX_DISCOVERED_GBA_SLOT_NUMBER
            && candidate_region.get_region_size() == GBA_COMBINED_RAM_SIZE
            && validate_gba_region(candidate_region)
        {
            discovered_region_descriptors.push(DolphinMemoryRegionDescriptor::new(
                DolphinMemoryRegionKind::GameBoyAdvanceWorkRam(next_gba_slot_number),
                candidate_region.get_base_address(),
            ));
            discovered_region_descriptors.push(DolphinMemoryRegionDescriptor::new(
                DolphinMemoryRegionKind::GameBoyAdvanceInternalRam(next_gba_slot_number),
                candidate_region
                    .get_base_address()
                    .saturating_add(GBA_WORK_RAM_SIZE),
            ));
            next_gba_slot_number = next_gba_slot_number.saturating_add(1);
        }
    }

    discovered_region_descriptors
}

fn classify_gamecube_memory_region(
    opened_process_info: &OpenedProcessInfo,
    candidate_region: &NormalizedRegion,
) -> Option<DolphinConsoleKind> {
    let mut game_id_bytes = [0u8; 6];
    let read_succeeded = MemoryReader::get_instance().read_bytes(opened_process_info, candidate_region.get_base_address(), &mut game_id_bytes);

    read_succeeded
        .then(|| classify_gamecube_game_id(&game_id_bytes))
        .flatten()
}

#[cfg(test)]
pub(crate) fn is_probable_gamecube_game_id(game_id_bytes: &[u8]) -> bool {
    classify_gamecube_game_id(game_id_bytes).is_some()
}

fn classify_gamecube_game_id(game_id_bytes: &[u8]) -> Option<DolphinConsoleKind> {
    if game_id_bytes.len() < 6 {
        return None;
    }

    let Some((&first_byte, remaining_bytes)) = game_id_bytes.split_first() else {
        return None;
    };

    if !remaining_bytes
        .iter()
        .all(|candidate_byte| candidate_byte.is_ascii_alphanumeric())
    {
        return None;
    }

    match first_byte {
        b'G' => Some(DolphinConsoleKind::GameCube),
        b'R' => Some(DolphinConsoleKind::Wii),
        _ => None,
    }
}

fn validate_gba_memory_region(
    opened_process_info: &OpenedProcessInfo,
    candidate_region: &NormalizedRegion,
) -> bool {
    let mut header_bytes = [0u8; GBA_HEADER_READ_SIZE];
    let read_succeeded = MemoryReader::get_instance().read_bytes(opened_process_info, candidate_region.get_base_address(), &mut header_bytes);

    read_succeeded && is_probable_gba_header(&header_bytes)
}

pub(crate) fn is_probable_gba_header(header_bytes: &[u8]) -> bool {
    if header_bytes.len() < GBA_HEADER_READ_SIZE {
        return false;
    }

    if header_bytes[0x03] != 0xEA {
        return false;
    }

    if header_bytes[0x04..0xA0] != NINTENDO_LOGO {
        return false;
    }

    if header_bytes[0xB2] != 0x96 {
        return false;
    }

    let computed_checksum = compute_gba_header_checksum(header_bytes);
    if computed_checksum != header_bytes[0xBD] {
        return false;
    }

    header_bytes[0xAC..0xB0]
        .iter()
        .all(|candidate_byte| candidate_byte.is_ascii_alphanumeric())
        && header_bytes[0xB0..0xB2]
            .iter()
            .all(|candidate_byte| candidate_byte.is_ascii_alphanumeric())
}

fn compute_gba_header_checksum(header_bytes: &[u8]) -> u8 {
    let mut checksum = 0u8;

    for header_byte in &header_bytes[0xA0..0xBD] {
        checksum = checksum.wrapping_sub(*header_byte);
    }

    checksum.wrapping_sub(0x19)
}
