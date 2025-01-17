use crate::models::docking::hierarchy::dock_node::DockNode;
use crate::models::docking::hierarchy::dock_split_child::DockSplitChild;
use crate::models::docking::hierarchy::dock_split_direction::DockSplitDirection;
use crate::models::docking::hierarchy::dock_tree::DockTree;
use crate::models::docking::hierarchy::operations::dock_reparent_direction::DockReparentDirection;

impl DockTree {
    /// High-level entry point to re-parent (move) a leaf node `source_id`
    /// relative to a leaf node `target_id` in `direction`.
    ///
    /// Returns `true` if successful, or `false` if something went wrong
    /// (e.g. source/target not found, re-parent onto itself, etc.).
    pub fn reparent_leaf(
        &mut self,
        source_id: &str,
        target_id: &str,
        direction: DockReparentDirection,
    ) -> bool {
        // Find both source + target.
        let source_path = match self.find_leaf_path(source_id) {
            Some(path) => path,
            None => return false,
        };
        let target_path = match self.find_leaf_path(target_id) {
            Some(path) => path,
            None => return false,
        };
        if source_path == target_path {
            // Disallow re-parenting onto itself.
            return false;
        }

        // Remove the source node from the tree.
        let source_node = match self.remove_leaf_by_path(&source_path) {
            Some(node) => node,
            None => return false,
        };

        // Re-fetch the target path in case it changed.
        let new_target_path = match self.find_leaf_path(target_id) {
            Some(path) => path,
            None => {
                // If target somehow disappeared, bail (Optionally re-insert source in old place, or just discard).
                return false;
            }
        };

        // 3) Delegate to subroutines based on direction.
        match direction {
            DockReparentDirection::Tab => self.reparent_as_tab(source_node, &new_target_path),
            DockReparentDirection::Left | DockReparentDirection::Right | DockReparentDirection::Top | DockReparentDirection::Bottom => {
                self.reparent_as_split_sibling(source_node, &new_target_path, direction)
            }
        }
    }

    // ------------------------------------------------------------------------
    //  REMOVING A LEAF
    // ------------------------------------------------------------------------

    /// Removes a leaf by path and returns the removed `DockNode`.
    fn remove_leaf_by_path(
        &mut self,
        leaf_path: &[usize],
    ) -> Option<DockNode> {
        // We expect leaf_path not to be empty
        let (child_index, parent_slice) = leaf_path.split_last()?;
        let child_index = *child_index;

        let parent_node = self.get_node_mut(parent_slice)?;
        match parent_node {
            DockNode::Split { children, .. } => {
                if child_index < children.len() {
                    let removed = children.remove(child_index);
                    Some(removed.node)
                } else {
                    None
                }
            }
            DockNode::Tab { tabs, .. } => {
                if child_index < tabs.len() {
                    Some(tabs.remove(child_index))
                } else {
                    None
                }
            }
            DockNode::Leaf { .. } => {
                // This is a weird edge: The parent itself is a Leaf?
                // Potentially means leaf_path pointed to the root.
                // We “replace” the entire root with a default.
                let old_root = std::mem::replace(parent_node, DockNode::default());
                Some(old_root)
            }
        }
    }

    // ------------------------------------------------------------------------
    //  RE-PARENT AS TAB
    // ------------------------------------------------------------------------

    /// Reparents `source_node` into the same tab group as `target_path`.
    /// If the target’s parent is a Tab, we just insert into that Tab’s list.
    /// Otherwise, if the target itself is a Leaf, we convert that Leaf into a Tab.
    /// If the target is a Split, we wrap it in a Tab (but remember, no splits in tabs).
    /// Returns `true` on success.
    fn reparent_as_tab(
        &mut self,
        source_node: DockNode,
        target_path: &[usize],
    ) -> bool {
        // If the target is a Leaf with a parent Tab, just insert.
        if self.is_leaf_with_tab_parent(target_path) {
            return self.insert_into_tab_parent(source_node, target_path);
        }

        // Otherwise, we get the actual target node.
        let Some(target_node) = self.get_node_mut(target_path) else {
            return false;
        };

        match target_node {
            DockNode::Tab { tabs, active_tab_id } => {
                // Just push the new node into the existing tab.
                tabs.push(source_node);
                if let Some(last_leaf_id) = tabs.last().and_then(|n| n.leaf_id()) {
                    *active_tab_id = last_leaf_id;
                }
                true
            }
            DockNode::Leaf { .. } => {
                // Convert Leaf -> Tab with 2 children
                let new_tab = Self::convert_leaf_to_tab(std::mem::take(target_node), source_node);
                *target_node = new_tab;
                true
            }
            DockNode::Split { .. } => {
                // Tabs cannot contain splits as children.
                false
            }
        }
    }

    /// Helper: check if `target_path` is a Leaf with a parent Tab node.
    fn is_leaf_with_tab_parent(
        &self,
        target_path: &[usize],
    ) -> bool {
        if target_path.is_empty() {
            return false;
        }
        let child_node = self.get_node(target_path);
        if !matches!(child_node, Some(DockNode::Leaf { .. })) {
            return false;
        }
        let parent_slice = &target_path[..target_path.len() - 1];
        match self.get_node(parent_slice) {
            Some(DockNode::Tab { .. }) => true,
            _ => false,
        }
    }

    /// Insert `source_node` into the Tab parent of the leaf at `target_path`.
    fn insert_into_tab_parent(
        &mut self,
        source_node: DockNode,
        target_path: &[usize],
    ) -> bool {
        let parent_slice = &target_path[..target_path.len() - 1];
        let Some(parent_node) = self.get_node_mut(parent_slice) else {
            return false;
        };

        if let DockNode::Tab { tabs, active_tab_id } = parent_node {
            tabs.push(source_node);
            if let Some(last_leaf_id) = tabs.last().and_then(|n| n.leaf_id()) {
                *active_tab_id = last_leaf_id;
            }
            true
        } else {
            false
        }
    }

    /// Helper: Convert a single Leaf node + an extra node into a Tab node with both children.
    fn convert_leaf_to_tab(
        leaf: DockNode,
        other: DockNode,
    ) -> DockNode {
        // We assume `leaf` is actually a Leaf. If not, be defensive.
        let leaf_id = leaf.leaf_id().unwrap_or_default();
        let other_id = other.leaf_id().unwrap_or_default();
        DockNode::Tab {
            tabs: vec![leaf, other],
            active_tab_id: other_id.is_empty().then(|| leaf_id).unwrap_or(other_id),
        }
    }

    // ------------------------------------------------------------------------
    //  RE-PARENT AS SPLIT SIBLING
    // ------------------------------------------------------------------------

    /// Reparents `source_node` as a sibling of the node at `target_path` in a new or existing
    /// split. (The direction is Left/Right/Top/Bottom.)
    /// - If the existing parent is a matching Split orientation, insert as another child.
    /// - Else, replace the target node with a new Split containing [target, source].
    /// - If the target is inside a Tab, we go “one level up,” because we can’t put splits inside a Tab.
    /// - If the target is the root, we might wrap the root in a new split.
    fn reparent_as_split_sibling(
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
        let Some(parent_node) = self.get_node_mut(parent_slice) else {
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
        if let Some(target_child) = Self::get_mut_child(parent_node, child_index) {
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
        if let Some(DockNode::Tab { .. }) = self.get_node(parent_slice) {
            // Then we want to “treat” the entire tab node as the target
            // So we just return the parent_slice as the new path
            Some(parent_slice.to_vec())
        } else {
            // Not in a tab; return as-is
            Some(target_path.to_vec())
        }
    }

    /// Inserts `source_node` into an existing Split's `children` at the correct position,
    /// (either before or after `child_index`).
    fn insert_into_existing_split(
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

    /// If the “target” is actually the root, we replace the entire root with a new split.
    fn wrap_root_in_new_split(
        &mut self,
        source_node: DockNode,
        direction: DockReparentDirection,
        split_dir: DockSplitDirection,
    ) -> bool {
        // Save old root
        let old_root = std::mem::replace(&mut self.root, DockNode::default());
        let (first, second) = match direction {
            DockReparentDirection::Left | DockReparentDirection::Top => (source_node, old_root),
            DockReparentDirection::Right | DockReparentDirection::Bottom => (old_root, source_node),
            DockReparentDirection::Tab => unreachable!(),
        };
        self.root = DockNode::Split {
            direction: split_dir,
            children: vec![DockSplitChild { node: first, ratio: 0.5 }, DockSplitChild {
                node: second,
                ratio: 0.5,
            }],
        };
        true
    }
}
