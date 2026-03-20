use squalr_engine_api::structures::pointer_scans::pointer_scan_node_type::PointerScanNodeType;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct DiscoveredPointerCandidate {
    pub(crate) pointer_scan_node_type: PointerScanNodeType,
    pub(crate) pointer_address: u64,
    pub(crate) pointer_value: u64,
    pub(crate) module_name: String,
    pub(crate) module_offset: u64,
}
