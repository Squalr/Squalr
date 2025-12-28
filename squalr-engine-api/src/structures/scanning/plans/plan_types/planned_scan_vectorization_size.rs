#[derive(Debug, Clone)]
pub enum PlannedScanVectorizationSize {
    Vector16,
    Vector32,
    Vector64,
}

impl Default for PlannedScanVectorizationSize {
    fn default() -> Self {
        Self::Vector32
    }
}
