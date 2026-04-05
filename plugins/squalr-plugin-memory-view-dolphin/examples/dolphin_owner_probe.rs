use squalr_engine_operating_system::{
    memory_queryer::{memory_queryer::MemoryQueryer, memory_queryer_trait::MemoryQueryerTrait},
    memory_reader::{MemoryReader, memory_reader_trait::MemoryReaderTrait},
    process_query::{process_query_options::ProcessQueryOptions, process_queryer::ProcessQuery},
};

const GBA_COMBINED_RAM_SIZE: u64 = 0x48000;
const GBA_EWRAM_SIZE: u64 = 0x40000;
const GBA_HEADER_READ_SIZE: usize = 0xC0;
const POINTER_SIZE_BYTES: usize = 8;
const SEARCH_CHUNK_SIZE: usize = 0x100000;
const SEARCH_WINDOW_BYTES: usize = 0x80;
const MAX_REGION_SCAN_SIZE: u64 = 0x0200_0000;
const MAX_MATCHES_PER_BLOCK: usize = 1;

const NINTENDO_LOGO: [u8; 156] = [
    0x24, 0xFF, 0xAE, 0x51, 0x69, 0x9A, 0xA2, 0x21, 0x3D, 0x84, 0x82, 0x0A, 0x84, 0xE4, 0x09, 0xAD, 0x11, 0x24, 0x8B, 0x98, 0xC0, 0x81, 0x7F, 0x21, 0xA3, 0x52,
    0xBE, 0x19, 0x93, 0x09, 0xCE, 0x20, 0x10, 0x46, 0x4A, 0x4A, 0xF8, 0x27, 0x31, 0xEC, 0x58, 0xC7, 0xE8, 0x33, 0x82, 0xE3, 0xCE, 0xBF, 0x85, 0xF4, 0xDF, 0x94,
    0xCE, 0x4B, 0x09, 0xC1, 0x94, 0x56, 0x8A, 0xC0, 0x13, 0x72, 0xA7, 0xFC, 0x9F, 0x84, 0x4D, 0x73, 0xA3, 0xCA, 0x9A, 0x61, 0x58, 0x97, 0xA3, 0x27, 0xFC, 0x03,
    0x98, 0x76, 0x23, 0x1D, 0xC7, 0x61, 0x03, 0x04, 0xAE, 0x56, 0xBF, 0x38, 0x84, 0x00, 0x40, 0xA7, 0x0E, 0xFD, 0xFF, 0x52, 0xFE, 0x03, 0x6F, 0x95, 0x30, 0xF1,
    0x97, 0xFB, 0xC0, 0x85, 0x60, 0xD6, 0x80, 0x25, 0xA9, 0x63, 0xBE, 0x03, 0x01, 0x4E, 0x38, 0xE2, 0xF9, 0xA2, 0x34, 0xFF, 0xBB, 0x3E, 0x03, 0x44, 0x78, 0x00,
    0x90, 0xCB, 0x88, 0x11, 0x3A, 0x94, 0x65, 0xC0, 0x7C, 0x63, 0x87, 0xF0, 0x3C, 0xAF, 0xD6, 0x25, 0xE4, 0x8B, 0x38, 0x0A, 0xAC, 0x72, 0x21, 0xD4, 0xF8, 0x07,
];

#[derive(Clone, Debug)]
struct GbaBlockCandidate {
    base_address: u64,
    title: String,
    game_code: String,
    maker_code: String,
    header_checksum: u8,
}

#[derive(Clone, Debug)]
struct PointerPairMatch {
    address_of_first_pointer: u64,
    second_pointer_address: u64,
    pointer_distance: u64,
    nearby_non_zero_small_dwords: Vec<(i64, u32)>,
    nearby_qwords: Vec<(i64, u64)>,
}

fn main() {
    if let Err(error) = run_probe() {
        eprintln!("owner probe failed: {error}");
        std::process::exit(1);
    }
}

fn run_probe() -> Result<(), String> {
    let dolphin_process = find_dolphin_process()?;
    let opened_process = ProcessQuery::open_process(&dolphin_process).map_err(|error| error.to_string())?;

    println!("attached to {} (pid={})", opened_process.get_name(), opened_process.get_process_id_raw());

    let all_regions = MemoryQueryer::query_pages_by_address_range(&opened_process, 0, MemoryQueryer::get_instance().get_max_usermode_address(&opened_process));
    println!("queried {} virtual regions", all_regions.len());
    println!("found {} raw 0x48000 regions", count_raw_gba_sized_regions(&all_regions));

    let gba_block_candidates = find_gba_block_candidates(&opened_process, &all_regions);
    println!("validated {} probable linked GBA RAM blocks", gba_block_candidates.len());

    for gba_block_candidate in &gba_block_candidates {
        println!(
            "  block {:#016x} title='{}' code={} maker={} checksum={:#04x}",
            gba_block_candidate.base_address,
            gba_block_candidate.title,
            gba_block_candidate.game_code,
            gba_block_candidate.maker_code,
            gba_block_candidate.header_checksum,
        );
    }

    for gba_block_candidate in &gba_block_candidates {
        println!();
        println!("owner search for block {:#016x}", gba_block_candidate.base_address);
        let pointer_pair_matches = find_pointer_pair_matches(&opened_process, &all_regions, gba_block_candidate);

        if pointer_pair_matches.is_empty() {
            println!("  no nearby base/iwram pointer-pair structures found");
            continue;
        }

        for (match_index, pointer_pair_match) in pointer_pair_matches.iter().enumerate() {
            println!(
                "  match {:02}: base_ptr@{:#016x} iwram_ptr@{:#016x} delta={:#x}",
                match_index, pointer_pair_match.address_of_first_pointer, pointer_pair_match.second_pointer_address, pointer_pair_match.pointer_distance,
            );

            let formatted_qwords = pointer_pair_match
                .nearby_qwords
                .iter()
                .map(|(relative_offset, value)| format!("{relative_offset:+#x}={value:#018x}"))
                .collect::<Vec<_>>()
                .join(", ");
            println!("    nearby qwords: {formatted_qwords}");

            if pointer_pair_match.nearby_non_zero_small_dwords.is_empty() {
                println!("    nearby non-zero small dwords: none");
            } else {
                let formatted_small_values = pointer_pair_match
                    .nearby_non_zero_small_dwords
                    .iter()
                    .map(|(relative_offset, value)| format!("{relative_offset:+#x}={value}"))
                    .collect::<Vec<_>>()
                    .join(", ");
                println!("    nearby non-zero small dwords: {formatted_small_values}");
            }
        }
    }

    let _ = ProcessQuery::close_process(opened_process.get_handle());
    Ok(())
}

fn find_dolphin_process() -> Result<squalr_engine_api::structures::processes::process_info::ProcessInfo, String> {
    let current_process_id = std::process::id();
    let search_terms = ["Dolphin.exe", "Slippi", "DolphinQt2.exe"];

    let mut matched_processes = Vec::new();
    for search_term in search_terms {
        let mut processes = ProcessQuery::get_processes(ProcessQueryOptions {
            required_process_id: None,
            search_name: Some(search_term.to_string()),
            require_windowed: false,
            match_case: false,
            fetch_icons: false,
            limit: Some(16),
        });
        matched_processes.append(&mut processes);
    }

    matched_processes.sort_by_key(|process_info| process_info.get_process_id_raw());
    matched_processes.dedup_by_key(|process_info| process_info.get_process_id_raw());

    matched_processes
        .into_iter()
        .find(|process_info| {
            let process_name = process_info.get_name().to_ascii_lowercase();
            process_info.get_process_id_raw() != current_process_id
                && !process_name.contains("owner_probe")
                && (process_name == "dolphin.exe"
                    || process_name == "slippi launcher.exe"
                    || process_name == "slippi dolphin.exe"
                    || process_name == "dolphinqt2.exe"
                    || process_name.contains("slippi"))
        })
        .ok_or_else(|| "could not find a running Dolphin/Slippi process".to_string())
}

fn find_gba_block_candidates(
    opened_process: &squalr_engine_api::structures::processes::opened_process_info::OpenedProcessInfo,
    all_regions: &[squalr_engine_api::structures::memory::normalized_region::NormalizedRegion],
) -> Vec<GbaBlockCandidate> {
    let mut gba_block_candidates = Vec::new();

    for region in all_regions {
        if region.get_region_size() != GBA_COMBINED_RAM_SIZE {
            continue;
        }

        let mut header_bytes = [0u8; GBA_HEADER_READ_SIZE];
        let read_succeeded = MemoryReader::get_instance().read_bytes(opened_process, region.get_base_address(), &mut header_bytes);

        if !read_succeeded {
            continue;
        }

        if !is_probable_gba_header(&header_bytes) {
            continue;
        }

        gba_block_candidates.push(GbaBlockCandidate {
            base_address: region.get_base_address(),
            title: parse_header_string(&header_bytes[0xA0..0xAC]),
            game_code: parse_header_string(&header_bytes[0xAC..0xB0]),
            maker_code: parse_header_string(&header_bytes[0xB0..0xB2]),
            header_checksum: header_bytes[0xBD],
        });
    }

    gba_block_candidates.sort_by_key(|candidate| candidate.base_address);
    gba_block_candidates
}

fn count_raw_gba_sized_regions(all_regions: &[squalr_engine_api::structures::memory::normalized_region::NormalizedRegion]) -> usize {
    all_regions
        .iter()
        .filter(|region| region.get_region_size() == GBA_COMBINED_RAM_SIZE)
        .count()
}

fn is_probable_gba_header(header_bytes: &[u8]) -> bool {
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

    let checksum = compute_gba_header_checksum(header_bytes);
    if checksum != header_bytes[0xBD] {
        return false;
    }

    header_bytes[0xAC..0xB0]
        .iter()
        .all(|byte| byte.is_ascii_alphanumeric())
        && header_bytes[0xB0..0xB2]
            .iter()
            .all(|byte| byte.is_ascii_alphanumeric())
}

fn compute_gba_header_checksum(header_bytes: &[u8]) -> u8 {
    let mut checksum = 0u8;
    for header_byte in &header_bytes[0xA0..0xBD] {
        checksum = checksum.wrapping_sub(*header_byte);
    }
    checksum.wrapping_sub(0x19)
}

fn parse_header_string(header_bytes: &[u8]) -> String {
    let trimmed_bytes = header_bytes
        .iter()
        .copied()
        .take_while(|byte| *byte != 0)
        .collect::<Vec<_>>();

    String::from_utf8_lossy(&trimmed_bytes).trim().to_string()
}

fn find_pointer_pair_matches(
    opened_process: &squalr_engine_api::structures::processes::opened_process_info::OpenedProcessInfo,
    all_regions: &[squalr_engine_api::structures::memory::normalized_region::NormalizedRegion],
    gba_block_candidate: &GbaBlockCandidate,
) -> Vec<PointerPairMatch> {
    let base_pointer_value = gba_block_candidate.base_address;
    let iwram_pointer_value = gba_block_candidate.base_address + GBA_EWRAM_SIZE;

    let mut pointer_pair_matches = Vec::new();

    'region_loop: for region in all_regions {
        if region.get_region_size() == 0 || region.get_region_size() > MAX_REGION_SCAN_SIZE {
            continue;
        }

        let mut region_offset = 0u64;
        while region_offset < region.get_region_size() {
            let remaining_region_bytes = region.get_region_size() - region_offset;
            let chunk_size = remaining_region_bytes.min(SEARCH_CHUNK_SIZE as u64) as usize;
            let chunk_address = region.get_base_address() + region_offset;
            let mut chunk_buffer = vec![0u8; chunk_size];

            if !MemoryReader::get_instance().read_bytes(opened_process, chunk_address, &mut chunk_buffer) {
                region_offset += chunk_size as u64;
                continue;
            }

            scan_chunk_for_pointer_pairs(&chunk_buffer, chunk_address, base_pointer_value, iwram_pointer_value, &mut pointer_pair_matches);

            if pointer_pair_matches.len() >= MAX_MATCHES_PER_BLOCK {
                break 'region_loop;
            }

            region_offset += chunk_size as u64;
        }
    }

    pointer_pair_matches.sort_by_key(|pointer_pair_match| pointer_pair_match.address_of_first_pointer);
    pointer_pair_matches.dedup_by_key(|pointer_pair_match| pointer_pair_match.address_of_first_pointer);
    pointer_pair_matches.truncate(MAX_MATCHES_PER_BLOCK);
    pointer_pair_matches
}

fn scan_chunk_for_pointer_pairs(
    chunk_buffer: &[u8],
    chunk_address: u64,
    base_pointer_value: u64,
    iwram_pointer_value: u64,
    pointer_pair_matches: &mut Vec<PointerPairMatch>,
) {
    if chunk_buffer.len() < POINTER_SIZE_BYTES * 2 {
        return;
    }

    let mut pointer_offset = 0usize;
    while pointer_offset + POINTER_SIZE_BYTES <= chunk_buffer.len() {
        let current_pointer = read_u64_le(&chunk_buffer[pointer_offset..pointer_offset + POINTER_SIZE_BYTES]);
        if current_pointer != base_pointer_value {
            pointer_offset += POINTER_SIZE_BYTES;
            continue;
        }

        let max_second_offset = (pointer_offset + SEARCH_WINDOW_BYTES).min(chunk_buffer.len().saturating_sub(POINTER_SIZE_BYTES));
        let mut second_pointer_offset = pointer_offset + POINTER_SIZE_BYTES;
        while second_pointer_offset <= max_second_offset {
            let second_pointer = read_u64_le(&chunk_buffer[second_pointer_offset..second_pointer_offset + POINTER_SIZE_BYTES]);
            if second_pointer == iwram_pointer_value {
                let address_of_first_pointer = chunk_address + pointer_offset as u64;
                let second_pointer_address = chunk_address + second_pointer_offset as u64;
                let nearby_non_zero_small_dwords = collect_nearby_non_zero_small_dwords(chunk_buffer, pointer_offset);
                let nearby_qwords = collect_nearby_qwords(chunk_buffer, pointer_offset);

                pointer_pair_matches.push(PointerPairMatch {
                    address_of_first_pointer,
                    second_pointer_address,
                    pointer_distance: second_pointer_offset as u64 - pointer_offset as u64,
                    nearby_non_zero_small_dwords,
                    nearby_qwords,
                });
            }

            second_pointer_offset += 4;
        }

        pointer_offset += POINTER_SIZE_BYTES;
    }
}

fn collect_nearby_non_zero_small_dwords(
    chunk_buffer: &[u8],
    pointer_offset: usize,
) -> Vec<(i64, u32)> {
    let mut nearby_values = Vec::new();
    let window_start = pointer_offset.saturating_sub(0x40);
    let window_end = (pointer_offset + 0xA0).min(chunk_buffer.len().saturating_sub(4));

    let mut dword_offset = window_start;
    while dword_offset <= window_end {
        let dword_value = u32::from_le_bytes([
            chunk_buffer[dword_offset],
            chunk_buffer[dword_offset + 1],
            chunk_buffer[dword_offset + 2],
            chunk_buffer[dword_offset + 3],
        ]);

        if (1..=8).contains(&dword_value) {
            nearby_values.push((dword_offset as i64 - pointer_offset as i64, dword_value));
        }

        dword_offset += 4;
    }

    nearby_values
}

fn collect_nearby_qwords(
    chunk_buffer: &[u8],
    pointer_offset: usize,
) -> Vec<(i64, u64)> {
    let mut nearby_values = Vec::new();
    let window_start = pointer_offset.saturating_sub(0x40);
    let window_end = (pointer_offset + 0x80).min(chunk_buffer.len().saturating_sub(8));

    let mut qword_offset = window_start;
    while qword_offset <= window_end {
        nearby_values.push((
            qword_offset as i64 - pointer_offset as i64,
            read_u64_le(&chunk_buffer[qword_offset..qword_offset + 8]),
        ));
        qword_offset += 8;
    }

    nearby_values
}

fn read_u64_le(bytes: &[u8]) -> u64 {
    u64::from_le_bytes(
        bytes[0..8]
            .try_into()
            .expect("expected 8 bytes for little-endian u64 read"),
    )
}
