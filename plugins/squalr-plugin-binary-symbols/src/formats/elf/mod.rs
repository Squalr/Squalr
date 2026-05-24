mod populate_elf_symbols_action;

pub(crate) use populate_elf_symbols_action::PopulateElfSymbolsAction;

pub(crate) fn matches_header(header_bytes: &[u8]) -> bool {
    header_bytes.starts_with(b"\x7FELF")
}
