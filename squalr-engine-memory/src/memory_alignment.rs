#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MemoryAlignment {
    Auto = 0,
    Alignment1 = 1,
    Alignment2 = 2,
    Alignment4 = 4,
    Alignment8 = 8,
}

impl Default for MemoryAlignment {
    fn default() -> Self {
        MemoryAlignment::Auto
    }
}

impl From<i32> for MemoryAlignment {
    fn from(size: i32) -> Self {
        match size {
            1 => MemoryAlignment::Alignment1,
            2 => MemoryAlignment::Alignment2,
            4 => MemoryAlignment::Alignment4,
            8 => MemoryAlignment::Alignment8,
            _ => MemoryAlignment::Auto,
        }
    }
}
