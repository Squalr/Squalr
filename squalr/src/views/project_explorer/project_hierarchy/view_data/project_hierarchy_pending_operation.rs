#[derive(Clone, PartialEq)]
pub enum ProjectHierarchyPendingOperation {
    None,
    Refreshing,
    Deleting,
    Promoting,
    Reordering,
}
