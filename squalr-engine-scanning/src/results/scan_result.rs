use std::fmt;

pub struct ScanResult {
    address: u64,
}

impl ScanResult {
    pub fn new(address: u64) -> Self {
        Self { address }
    }

    pub fn get_address(&self) -> u64 {
        self.address
    }
}

impl fmt::Debug for ScanResult {
    fn fmt(
        &self,
        formatter: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        write!(formatter, "ScanResult {{ address: 0x{:X} }}", self.address)
    }
}
