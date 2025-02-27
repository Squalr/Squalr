use crate::filters::snapshot_region_filter::SnapshotRegionFilter;

/// A custom type that defines a set of filters (scan results) discovered by scanners.
/// While this looks silly, it is better to have a vector of vectors for parallelization.
/// This is because when we scan a filter, it produces a list of filters. Combining these back into
/// one giant list would cost too much scan time, so it's better to keep it as a list of lists.
pub type SnapshotFilterCollection = Vec<Vec<SnapshotRegionFilter>>;
