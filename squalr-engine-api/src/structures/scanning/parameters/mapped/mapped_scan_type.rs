#[derive(Debug, Clone)]
pub enum ScanParametersScalar {
    SingleElement,
    ScalarIterative,
}

#[derive(Debug, Clone)]
pub enum ScanParametersVector {
    Aligned,
    Sparse,
    Overlapping,
    OverlappingBytewiseStaggered,
    OverlappingBytewisePeriodic,
}

#[derive(Debug, Clone)]
pub enum ScanParametersByteArray {
    ByteArrayBooyerMoore,
}

/// Contains processed parameters that define a scan over a region of memory.
/// These transform user input to ensure that the scan is performed as efficiently as possible.
#[derive(Debug, Clone)]
pub enum MappedScanType {
    Scalar(ScanParametersScalar),
    Vector(ScanParametersVector),
    ByteArray(ScanParametersByteArray),
}
