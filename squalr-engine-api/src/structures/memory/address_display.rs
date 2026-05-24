const GAMECUBE_MAIN_MEMORY_BASE_ADDRESS: u64 = 0x8000_0000;
const WII_EXTENDED_MEMORY_BASE_ADDRESS: u64 = 0x9000_0000;
const GAMECUBE_MAIN_MEMORY_SIZE: u64 = 0x0200_0000;
const WII_EXTENDED_MEMORY_SIZE: u64 = 0x0400_0000;
const GBA_WORK_RAM_BASE_ADDRESS: u64 = 0xA000_0000;
const GBA_INTERNAL_RAM_BASE_ADDRESS: u64 = 0xA004_0000;
const GBA_SLOT_STRIDE: u64 = 0x0010_0000;
const GBA_WORK_RAM_SIZE: u64 = 0x0004_0000;
const GBA_INTERNAL_RAM_SIZE: u64 = 0x0000_8000;

pub fn format_absolute_address(address: u64) -> String {
    if address <= u32::MAX as u64 {
        format!("{:08X}", address)
    } else {
        format!("{:016X}", address)
    }
}

pub fn try_resolve_virtual_module_address(
    module_name: &str,
    module_offset: u64,
) -> Option<u64> {
    if module_name.eq_ignore_ascii_case("gc_wii") {
        if module_offset < GAMECUBE_MAIN_MEMORY_SIZE {
            return Some(GAMECUBE_MAIN_MEMORY_BASE_ADDRESS.saturating_add(module_offset));
        }

        let wii_offset = module_offset.saturating_sub(GAMECUBE_MAIN_MEMORY_SIZE);

        if wii_offset < WII_EXTENDED_MEMORY_SIZE {
            return Some(WII_EXTENDED_MEMORY_BASE_ADDRESS.saturating_add(wii_offset));
        }

        return None;
    }

    if module_name.eq_ignore_ascii_case("MEM1") || module_name.eq_ignore_ascii_case("GC") {
        return (module_offset < GAMECUBE_MAIN_MEMORY_SIZE).then(|| GAMECUBE_MAIN_MEMORY_BASE_ADDRESS.saturating_add(module_offset));
    }

    if module_name.eq_ignore_ascii_case("MEM2") || module_name.eq_ignore_ascii_case("Wii") {
        return (module_offset < WII_EXTENDED_MEMORY_SIZE).then(|| WII_EXTENDED_MEMORY_BASE_ADDRESS.saturating_add(module_offset));
    }

    if let Some(slot_index) = parse_gba_slot(module_name, "gba_wm_") {
        return (module_offset < GBA_WORK_RAM_SIZE).then(|| {
            GBA_WORK_RAM_BASE_ADDRESS
                .saturating_add(slot_index as u64 * GBA_SLOT_STRIDE)
                .saturating_add(module_offset)
        });
    }

    if let Some(slot_index) = parse_gba_slot(module_name, "gba_im_") {
        return (module_offset < GBA_INTERNAL_RAM_SIZE).then(|| {
            GBA_INTERNAL_RAM_BASE_ADDRESS
                .saturating_add(slot_index as u64 * GBA_SLOT_STRIDE)
                .saturating_add(module_offset)
        });
    }

    None
}

pub fn is_virtual_module_address(address: u64) -> bool {
    (GAMECUBE_MAIN_MEMORY_BASE_ADDRESS..GAMECUBE_MAIN_MEMORY_BASE_ADDRESS + GAMECUBE_MAIN_MEMORY_SIZE).contains(&address)
        || (WII_EXTENDED_MEMORY_BASE_ADDRESS..WII_EXTENDED_MEMORY_BASE_ADDRESS + WII_EXTENDED_MEMORY_SIZE).contains(&address)
        || (GBA_WORK_RAM_BASE_ADDRESS..GBA_WORK_RAM_BASE_ADDRESS + 4 * GBA_SLOT_STRIDE).contains(&address)
        || (GBA_INTERNAL_RAM_BASE_ADDRESS..GBA_INTERNAL_RAM_BASE_ADDRESS + 4 * GBA_SLOT_STRIDE).contains(&address)
}

pub fn format_module_address(
    module_name: &str,
    module_offset: u64,
) -> String {
    format!("{}+0x{:X}", module_name, module_offset)
}

fn parse_gba_slot(
    module_name: &str,
    module_prefix: &str,
) -> Option<u8> {
    let normalized_module_name = module_name.to_ascii_lowercase();
    let slot_number = normalized_module_name
        .strip_prefix(module_prefix)?
        .parse::<u8>()
        .ok()?;

    ((1..=4).contains(&slot_number)).then_some(slot_number.saturating_sub(1))
}

#[cfg(test)]
mod tests {
    use super::{format_module_address, try_resolve_virtual_module_address};

    #[test]
    fn module_addresses_render_as_module_relative_strings() {
        assert_eq!(format_module_address("gc_wii", 0x1234), "gc_wii+0x1234");
        assert_eq!(format_module_address("gba_im_1", 0x20), "gba_im_1+0x20");
        assert_eq!(format_module_address("gba_wm_4", 0x44), "gba_wm_4+0x44");
    }

    #[test]
    fn gba_virtual_module_resolution_uses_one_based_slot_numbers() {
        assert_eq!(try_resolve_virtual_module_address("gba_im_1", 0x20), Some(0xA004_0020));
        assert_eq!(try_resolve_virtual_module_address("gba_im_4", 0x20), Some(0xA034_0020));
        assert_eq!(try_resolve_virtual_module_address("gba_wm_1", 0x20), Some(0xA000_0020));
        assert_eq!(try_resolve_virtual_module_address("gba_wm_4", 0x20), Some(0xA030_0020));
    }
}
