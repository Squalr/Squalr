mod populate_pe_symbols_action;

pub(crate) use populate_pe_symbols_action::PopulatePeSymbolsAction;

pub(crate) const DISPLAY_NAME: &str = "PE";

pub(crate) fn matches_header(header_bytes: &[u8]) -> bool {
    header_bytes.starts_with(b"MZ")
}
