#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PointerScanTargetRange {
    lower_bound: u64,
    upper_bound: u64,
}

impl PointerScanTargetRange {
    pub fn new(
        lower_bound: u64,
        upper_bound: u64,
    ) -> Self {
        Self { lower_bound, upper_bound }
    }

    pub fn get_lower_bound(&self) -> u64 {
        self.lower_bound
    }

    pub fn get_upper_bound(&self) -> u64 {
        self.upper_bound
    }
}
