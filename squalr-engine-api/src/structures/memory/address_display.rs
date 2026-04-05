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
    let module_base_address: u64 = if module_name.eq_ignore_ascii_case("GC") {
        0x8000_0000
    } else if module_name.eq_ignore_ascii_case("Wii") {
        0x9000_0000
    } else {
        return None;
    };

    module_base_address.checked_add(module_offset)
}

pub fn format_module_address(
    module_name: &str,
    module_offset: u64,
) -> String {
    if let Some(absolute_address) = try_resolve_virtual_module_address(module_name, module_offset) {
        format_absolute_address(absolute_address)
    } else {
        format!("{}+0x{:X}", module_name, module_offset)
    }
}
