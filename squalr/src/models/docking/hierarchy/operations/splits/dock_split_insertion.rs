use crate::models::docking::hierarchy::dock_node::DockNode;
use crate::models::docking::hierarchy::types::dock_reparent_direction::DockReparentDirection;
use crate::models::docking::hierarchy::types::dock_split_child::DockSplitChild;
use crate::models::docking::hierarchy::types::dock_split_direction::DockSplitDirection;

impl DockNode {
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
        let split_dir = match direction {
            DockReparentDirection::Left | DockReparentDirection::Right => DockSplitDirection::VerticalDivider,
            DockReparentDirection::Top | DockReparentDirection::Bottom => DockSplitDirection::HorizontalDivider,
            DockReparentDirection::Tab => unreachable!(),
        };

        // Possibly jump up if the target is inside a tab, so the new split is outside the tab.
        let real_target_path = match self.promote_target_out_of_tab(target_path) {
            Some(path) => path,
            None => {
                return false;
            }
        };

        // If the parent is already a matching split, just insert as a child.
        if let Some((child_index, parent_path)) = real_target_path.split_last() {
            let Some(parent_node) = self.get_node_from_path_mut(parent_path) else {
                // Fallback: just replace the entire root.
                return self.replace_with_new_split(&[], source_node, split_dir, direction);
            };

            if let DockNode::Split {
                direction: existing_dir,
                children,
            } = parent_node
            {
                if *existing_dir == split_dir {
                    // Insert at or after child_index.
                    let insert_at = match direction {
                        DockReparentDirection::Left | DockReparentDirection::Top => *child_index,
                        DockReparentDirection::Right | DockReparentDirection::Bottom => *child_index + 1,
                        DockReparentDirection::Tab => unreachable!(),
                    };

                    if insert_at <= children.len() {
                        children.insert(insert_at, DockSplitChild {
                            node: source_node,
                            ratio: 0.0, // recalc below
                        });
                        Self::recalculate_split_ratios(children);
                        return true;
                    }
                    return false;
                }
            }
        }

        // Otherwise, wrap and the node at `real_target_path` with a new split, and insert our new node.
        self.replace_with_new_split(&real_target_path, source_node, split_dir, direction)
    }

    /// Replace the node at `target_path` with a new `Split` containing the original node plus `source_node` (in the correct order).
    fn replace_with_new_split(
        &mut self,
        target_path: &[usize],
        source_node: DockNode,
        split_dir: DockSplitDirection,
        direction: DockReparentDirection,
    ) -> bool {
        // If `target_path` is empty, we are replacing the entire root. Otherwise, we’re replacing a child at `target_path`.
        if target_path.is_empty() {
            // Move out the entire old root.
            let old_root = std::mem::replace(self, DockNode::default());

            // Build the new children:
            let (first, second) = match direction {
                DockReparentDirection::Left | DockReparentDirection::Top => (source_node, old_root),
                DockReparentDirection::Right | DockReparentDirection::Bottom => (old_root, source_node),
                DockReparentDirection::Tab => unreachable!(),
            };

            *self = DockNode::Split {
                direction: split_dir,
                children: vec![DockSplitChild { node: first, ratio: 0.5 }, DockSplitChild {
                    node: second,
                    ratio: 0.5,
                }],
            };
            return true;
        }

        // Otherwise, get mutable reference to the node being replaced.
        let Some(target_node) = self.get_node_from_path_mut(target_path) else {
            return false;
        };

        let old_target = std::mem::replace(target_node, DockNode::default());
        let (first, second) = match direction {
            DockReparentDirection::Left | DockReparentDirection::Top => (source_node, old_target),
            DockReparentDirection::Right | DockReparentDirection::Bottom => (old_target, source_node),
            DockReparentDirection::Tab => unreachable!(),
        };

        *target_node = DockNode::Split {
            direction: split_dir,
            children: vec![DockSplitChild { node: first, ratio: 0.5 }, DockSplitChild {
                node: second,
                ratio: 0.5,
            }],
        };

        true
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

        // Check if the parent is a tab.
        if let Some(DockNode::Tab { .. }) = self.get_node_from_path(parent_slice) {
            // Then we want to “treat” the entire tab node as the target, so we just return the parent_slice as the new path.
            Some(parent_slice.to_vec())
        } else {
            // Not in a tab; return as-is.
            Some(target_path.to_vec())
        }
    }
}
