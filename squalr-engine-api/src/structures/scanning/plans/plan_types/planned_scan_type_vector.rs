#[derive(Debug, Clone)]
pub enum PlannedScanTypeVector {
    Aligned,
    Sparse,
    Overlapping,
    OverlappingBytewiseStaggered,
    OverlappingBytewisePeriodic,
}
