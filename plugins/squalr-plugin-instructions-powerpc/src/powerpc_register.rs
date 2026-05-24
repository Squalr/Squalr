pub fn parse_powerpc_register_name(register_name: &str) -> Option<u8> {
    register_name
        .trim()
        .to_ascii_lowercase()
        .strip_prefix('r')
        .and_then(|register_index_text| register_index_text.parse::<u8>().ok())
        .filter(|register_index| *register_index <= 31)
}

pub fn format_powerpc_register_name(register_index: u8) -> String {
    format!("r{}", register_index)
}
