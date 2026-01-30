pub struct CommandLineConversions {}

impl CommandLineConversions {
    pub fn parse_hex_or_int(src: &str) -> Result<u64, std::num::ParseIntError> {
        if src.starts_with("0x") || src.starts_with("0X") {
            u64::from_str_radix(&src[2..], 16)
        } else {
            src.parse::<u64>()
        }
    }
}
