pub fn parse_arm32_register_name(register_name: &str) -> Option<u8> {
    let normalized_register_name = register_name.trim().to_ascii_lowercase();

    match normalized_register_name.as_str() {
        "sp" => Some(13),
        "lr" => Some(14),
        "pc" => Some(15),
        _ => normalized_register_name
            .strip_prefix('r')
            .and_then(|register_index_text| register_index_text.parse::<u8>().ok())
            .filter(|register_index| *register_index <= 15),
    }
}

pub fn format_arm32_register_name(register_index: u8) -> String {
    match register_index {
        13 => String::from("sp"),
        14 => String::from("lr"),
        15 => String::from("pc"),
        _ => format!("r{}", register_index),
    }
}
