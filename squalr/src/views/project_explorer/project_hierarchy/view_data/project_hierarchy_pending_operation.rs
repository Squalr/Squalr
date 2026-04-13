#[derive(Clone, PartialEq)]
pub enum ProjectHierarchyPendingOperation {
    None,
    ConvertingSymbolRefs,
    Refreshing,
    Deleting,
    Promoting,
    Reordering,
}
