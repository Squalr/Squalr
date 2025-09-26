use squalr_engine_api::structures::scanning::filters::snapshot_region_filter::SnapshotRegionFilter;

pub struct PointerScanLevel {
    static_regions: Vec<SnapshotRegionFilter>,
    heap_regions: Vec<SnapshotRegionFilter>,
}

impl PointerScanLevel {
    pub fn new(
        static_regions: Vec<SnapshotRegionFilter>,
        heap_regions: Vec<SnapshotRegionFilter>,
    ) -> Self {
        Self { static_regions, heap_regions }
    }

    /// Takes ownership of the resulting static region filters from this pointer scan level. Note that once this is called, the regions are emptied from this struct.
    pub fn take_static_regions(&mut self) -> Vec<SnapshotRegionFilter> {
        std::mem::take(&mut self.static_regions)
    }

    /// Gets the resulting static region filters from this pointer scan level.
    pub fn get_static_regions(&mut self) -> Vec<SnapshotRegionFilter> {
        std::mem::take(&mut self.static_regions)
    }

    /// Takes ownership of the resulting static region filters from this pointer scan level. Note that once this is called, the regions are emptied from this struct.
    pub fn take_heap_regions(&mut self) -> Vec<SnapshotRegionFilter> {
        std::mem::take(&mut self.heap_regions)
    }

    /// Gets the resulting static region filters from this pointer scan level.
    pub fn get_heap_regions(&mut self) -> Vec<SnapshotRegionFilter> {
        std::mem::take(&mut self.heap_regions)
    }
}
