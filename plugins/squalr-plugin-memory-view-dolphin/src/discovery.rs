use crate::address_space::{DolphinMemoryRegionDescriptor, DolphinMemoryRegionKind};
use squalr_engine_api::{
    plugins::memory_view::MemoryViewPluginError,
    structures::{memory::normalized_region::NormalizedRegion, processes::opened_process_info::OpenedProcessInfo},
};
use squalr_engine_operating_system::{
    memory_queryer::{memory_queryer::MemoryQueryer, memory_queryer_trait::MemoryQueryerTrait},
    memory_reader::{MemoryReader, memory_reader_trait::MemoryReaderTrait},
};

pub(crate) fn discover_dolphin_memory_regions(
    opened_process_info: &OpenedProcessInfo,
) -> Result<Vec<DolphinMemoryRegionDescriptor>, MemoryViewPluginError> {
    let raw_memory_regions = MemoryQueryer::query_pages_by_address_range(
        opened_process_info,
        0,
        MemoryQueryer::get_instance().get_max_usermode_address(opened_process_info),
    );
    let discovered_region_descriptors = select_dolphin_memory_regions(&raw_memory_regions, |candidate_region| {
        validate_gamecube_memory_region(opened_process_info, candidate_region)
    });

    if discovered_region_descriptors.is_empty() {
        return Err(MemoryViewPluginError::message(
            crate::constants::DOLPHIN_PLUGIN_ID,
            format!("no Dolphin memory regions were discovered for process `{}`", opened_process_info.get_name()),
        ));
    }

    Ok(discovered_region_descriptors)
}

pub(crate) fn select_dolphin_memory_regions(
    raw_memory_regions: &[NormalizedRegion],
    mut validate_gamecube_region: impl FnMut(&NormalizedRegion) -> bool,
) -> Vec<DolphinMemoryRegionDescriptor> {
    let mut discovered_region_descriptors = Vec::new();
    let mut found_gamecube_main_memory = false;
    let mut found_wii_extended_memory = false;

    for candidate_region in raw_memory_regions {
        if !found_gamecube_main_memory
            && candidate_region.get_region_size() == DolphinMemoryRegionKind::GameCubeMainMemory.host_region_size()
            && validate_gamecube_region(candidate_region)
        {
            discovered_region_descriptors.push(DolphinMemoryRegionDescriptor::new(
                DolphinMemoryRegionKind::GameCubeMainMemory,
                candidate_region.get_base_address(),
            ));
            found_gamecube_main_memory = true;
            continue;
        }

        if !found_wii_extended_memory
            && candidate_region.get_region_size() == DolphinMemoryRegionKind::WiiExtendedMemory.host_region_size()
        {
            discovered_region_descriptors.push(DolphinMemoryRegionDescriptor::new(
                DolphinMemoryRegionKind::WiiExtendedMemory,
                candidate_region.get_base_address(),
            ));
            found_wii_extended_memory = true;
        }
    }

    discovered_region_descriptors
}

fn validate_gamecube_memory_region(
    opened_process_info: &OpenedProcessInfo,
    candidate_region: &NormalizedRegion,
) -> bool {
    let mut game_id_bytes = [0u8; 6];
    let read_succeeded = MemoryReader::get_instance().read_bytes(
        opened_process_info,
        candidate_region.get_base_address(),
        &mut game_id_bytes,
    );

    read_succeeded && is_probable_gamecube_game_id(&game_id_bytes)
}

pub(crate) fn is_probable_gamecube_game_id(game_id_bytes: &[u8]) -> bool {
    if game_id_bytes.len() < 6 {
        return false;
    }

    let Some((&first_byte, remaining_bytes)) = game_id_bytes.split_first() else {
        return false;
    };

    (first_byte == b'G' || first_byte == b'R')
        && remaining_bytes
            .iter()
            .all(|candidate_byte| candidate_byte.is_ascii_alphanumeric())
}