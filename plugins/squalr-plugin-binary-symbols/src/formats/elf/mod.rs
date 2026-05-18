pub(crate) const DISPLAY_NAME: &str = "ELF";

pub(crate) fn matches_header(header_bytes: &[u8]) -> bool {
    header_bytes.starts_with(b"\x7FELF")
}
