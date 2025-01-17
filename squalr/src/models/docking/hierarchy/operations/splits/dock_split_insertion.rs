use crate::models::docking::hierarchy::dock_node::DockNode;
use crate::models::docking::hierarchy::types::dock_reparent_direction::DockReparentDirection;
use crate::models::docking::hierarchy::types::dock_split_child::DockSplitChild;

impl DockNode {
    /// Inserts `source_node` into an existing Split's `children` at the correct position,
    /// (either before or after `child_index`).
    pub fn insert_into_existing_split(
        children: &mut Vec<DockSplitChild>,
        source_node: DockNode,
        child_index: usize,
        direction: DockReparentDirection,
    ) -> bool {
        let insert_at = match direction {
            DockReparentDirection::Left | DockReparentDirection::Top => child_index,
            DockReparentDirection::Right | DockReparentDirection::Bottom => child_index + 1,
            DockReparentDirection::Tab => unreachable!(),
        };
        if insert_at > children.len() {
            return false;
        }

        children.insert(insert_at, DockSplitChild {
            node: source_node,
            ratio: 0.0, // Will recalc below
        });
        Self::recalculate_split_ratios(children);
        true
    }
}
