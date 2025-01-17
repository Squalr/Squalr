use crate::models::docking::hierarchy::dock_node::DockNode;
use crate::models::docking::hierarchy::types::dock_reparent_direction::DockReparentDirection;
use crate::models::docking::hierarchy::types::dock_split_child::DockSplitChild;
use crate::models::docking::hierarchy::types::dock_split_direction::DockSplitDirection;

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

    /// Reparents `source_node` as a sibling of the node at `target_path` in a new or existing
    /// split. (The direction is Left/Right/Top/Bottom.)
    /// - If the existing parent is a matching Split orientation, insert as another child.
    /// - Else, replace the target node with a new Split containing [target, source].
    /// - If the target is inside a Tab, we go “one level up,” because we can’t put splits inside a Tab.
    /// - If the target is the root, we might wrap the root in a new split.
    pub fn reparent_as_split_sibling(
        &mut self,
        source_node: DockNode,
        target_path: &[usize],
        direction: DockReparentDirection,
    ) -> bool {
        // 1) Identify the required orientation
        let split_dir = match direction {
            DockReparentDirection::Left | DockReparentDirection::Right => DockSplitDirection::VerticalDivider,
            DockReparentDirection::Top | DockReparentDirection::Bottom => DockSplitDirection::HorizontalDivider,
            DockReparentDirection::Tab => unreachable!("handled above"),
        };

        // 2) Possibly jump up if the target is inside a Tab
        //    (so we can place a new split *outside* the tab).
        let real_target_path = match self.promote_target_out_of_tab(target_path) {
            Some(path) => path,
            None => return false,
        };

        // Now we do the standard “insert or wrap” logic with that real_target_path
        if real_target_path.is_empty() {
            // Means the target is the root. We wrap root in a new split.
            return self.wrap_root_in_new_split(source_node, direction, split_dir);
        }

        // Otherwise, we have a parent, child_index situation
        let (child_index, parent_slice) = match real_target_path.split_last() {
            Some((ci, ps)) => (*ci, ps),
            None => return false,
        };
        let Some(parent_node) = self.get_node_from_path_mut(parent_slice) else {
            // If we can’t find the parent for some reason, try wrapping root
            return self.wrap_root_in_new_split(source_node, direction, split_dir);
        };

        // If parent is already a matching Split, we do an insertion
        if let DockNode::Split {
            direction: existing_dir,
            children,
        } = parent_node
        {
            if *existing_dir == split_dir {
                return Self::insert_into_existing_split(children, source_node, child_index, direction);
            }
        }

        // Otherwise, we replace the child node with a new Split node that has the old child + the new source
        if let Some(target_child) = self.get_node_from_path_mut(&target_path) {
            let old_target = std::mem::replace(target_child, DockNode::default());
            let (first, second) = match direction {
                DockReparentDirection::Left | DockReparentDirection::Top => (source_node, old_target),
                DockReparentDirection::Right | DockReparentDirection::Bottom => (old_target, source_node),
                _ => unreachable!(),
            };
            *target_child = DockNode::Split {
                direction: split_dir,
                children: vec![DockSplitChild { node: first, ratio: 0.5 }, DockSplitChild {
                    node: second,
                    ratio: 0.5,
                }],
            };
            true
        } else {
            false
        }
    }

    /// If `target_path` is inside a Tab, we move up one level so that we can do a split
    /// *outside* the tab. Example:
    ///   - If `target_path` is `[3, 2]`, and node[3] is a Tab, we want just `[3]`.
    /// If it’s not inside a Tab, we return as-is.
    fn promote_target_out_of_tab(
        &self,
        target_path: &[usize],
    ) -> Option<Vec<usize>> {
        if target_path.is_empty() {
            return Some(vec![]);
        }
        let parent_slice = &target_path[..target_path.len() - 1];

        // Check if the parent is a Tab
        if let Some(DockNode::Tab { .. }) = self.get_node_from_path(parent_slice) {
            // Then we want to “treat” the entire tab node as the target
            // So we just return the parent_slice as the new path
            Some(parent_slice.to_vec())
        } else {
            // Not in a tab; return as-is
            Some(target_path.to_vec())
        }
    }

    /// If the “target” is actually the root, we replace the entire root with a new split.
    fn wrap_root_in_new_split(
        &mut self,
        source_node: DockNode,
        direction: DockReparentDirection,
        split_dir: DockSplitDirection,
    ) -> bool {
        // Save old root
        let (first, second) = match direction {
            DockReparentDirection::Left | DockReparentDirection::Top => (source_node, self.clone()),
            DockReparentDirection::Right | DockReparentDirection::Bottom => (self.clone(), source_node),
            DockReparentDirection::Tab => unreachable!(),
        };
        *self = DockNode::Split {
            direction: split_dir,
            children: vec![DockSplitChild { node: first, ratio: 0.5 }, DockSplitChild {
                node: second,
                ratio: 0.5,
            }],
        };
        true
    }
}
