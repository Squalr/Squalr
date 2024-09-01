pub struct ScanResult {
    address: u64,
}
impl ScanResult {
    pub fn new(address: u64) -> Self {
        Self { address: address }
    }

    pub fn get_address(&self) -> u64 {
        return self.address;
    }
}
