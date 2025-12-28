use crate::structures::scanning::plans::plan_types::{
    planned_scan_type_byte_array::PlannedScanTypeByteArray, planned_scan_type_scalar::PlannedScanTypeScalar, planned_scan_type_vector::PlannedScanTypeVector,
    planned_scan_vectorization_size::PlannedScanVectorizationSize,
};

/// Contains processed parameters that define a scan over a region of memory.
/// These transform user input to ensure that the scan is performed as efficiently as possible.
#[derive(Debug, Clone)]
pub enum PlannedScanType {
    Invalid(),
    Scalar(PlannedScanTypeScalar),
    Vector(PlannedScanTypeVector, PlannedScanVectorizationSize),
    ByteArray(PlannedScanTypeByteArray),
}
