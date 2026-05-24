#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PointerScanRegionMatch {
    pointer_address: u64,
    pointer_value: u64,
}

impl PointerScanRegionMatch {
    pub fn new(
        pointer_address: u64,
        pointer_value: u64,
    ) -> Self {
        Self {
            pointer_address,
            pointer_value,
        }
    }

    pub fn get_pointer_address(&self) -> u64 {
        self.pointer_address
    }

    pub fn get_pointer_value(&self) -> u64 {
        self.pointer_value
    }
}
