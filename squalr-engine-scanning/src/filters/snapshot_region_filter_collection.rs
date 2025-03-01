use crate::filters::snapshot_region_filter::SnapshotRegionFilter;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use squalr_engine_common::{structures::memory_alignment::MemoryAlignment, values::data_type::DataType};

/// A custom type that defines a set of filters (scan results) discovered by scanners.
pub struct SnapshotRegionFilterCollection {
    /// The filters contained in this collection. This is kept as a vector of vectors for better parallelization.
    snapshot_region_filters: Vec<Vec<SnapshotRegionFilter>>,
    data_type: DataType,
    memory_alignment: MemoryAlignment,
}

impl SnapshotRegionFilterCollection {
    pub fn new(
        snapshot_region_filters: Vec<Vec<SnapshotRegionFilter>>,
        data_type: DataType,
        memory_alignment: MemoryAlignment,
    ) -> Self {
        Self {
            snapshot_region_filters,
            data_type,
            memory_alignment,
        }
    }

    pub fn get_data_type(&self) -> DataType {
        self.data_type.clone()
    }

    pub fn get_memory_alignment(&self) -> MemoryAlignment {
        self.memory_alignment.clone()
    }

    pub fn iter(&self) -> std::iter::Flatten<std::slice::Iter<'_, Vec<SnapshotRegionFilter>>> {
        self.snapshot_region_filters.iter().flatten()
    }

    pub fn par_iter(&self) -> rayon::iter::Flatten<rayon::slice::Iter<'_, Vec<SnapshotRegionFilter>>> {
        self.snapshot_region_filters.par_iter().flatten()
    }
}
