#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Endian {
    Little,
    Big,
}

impl Default for Endian {
    fn default() -> Self {
        Endian::Little
    }
}
