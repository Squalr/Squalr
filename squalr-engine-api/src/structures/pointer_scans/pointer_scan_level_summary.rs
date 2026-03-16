use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct PointerScanLevelSummary {
    depth: u64,
    node_count: u64,
    static_node_count: u64,
    heap_node_count: u64,
}

impl PointerScanLevelSummary {
    pub fn new(
        depth: u64,
        node_count: u64,
        static_node_count: u64,
        heap_node_count: u64,
    ) -> Self {
        Self {
            depth,
            node_count,
            static_node_count,
            heap_node_count,
        }
    }

    pub fn get_depth(&self) -> u64 {
        self.depth
    }

    pub fn get_node_count(&self) -> u64 {
        self.node_count
    }

    pub fn get_static_node_count(&self) -> u64 {
        self.static_node_count
    }

    pub fn get_heap_node_count(&self) -> u64 {
        self.heap_node_count
    }
}
