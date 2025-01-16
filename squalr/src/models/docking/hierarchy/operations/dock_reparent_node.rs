use crate::models::docking::hierarchy::dock_node::DockNode;
use crate::models::docking::hierarchy::dock_reparent_direction::DockReparentDirection;
use crate::models::docking::hierarchy::dock_split_child::DockSplitChild;
use crate::models::docking::hierarchy::dock_split_direction::DockSplitDirection;
use crate::models::docking::hierarchy::dock_tree::DockTree;

/// Validates and corrects any mistakes in tab logic.
impl DockTree {
    /// Attempts to re-parent (move) a leaf node identified by `source_id` so that it becomes
    /// a sibling or tab of the leaf node identified by `target_id`, following the given
    /// `direction`.
    ///
    /// - If `direction == DockReparentDirection::Tab`, we embed `source` as a new tab
    ///   alongside `target`.
    /// - If `direction == Left/Right/Top/Bottom`, we insert into (or create) a suitable Split node.
    /// - Returns `true` on success, `false` on failure or if `source_id`/`target_id` is not found.
    ///
    /// Caveats / behaviors:
    /// - If `source_id` == `target_id`, we do nothing (return false).
    /// - This function removes the source leaf from its current parent before inserting
    ///   it under the target.
    /// - If the target is already in a Tab node, we just add the source leaf to that same tab group.
    /// - If the target is a Leaf node and we want to merge into a tab, we convert that Leaf
    ///   into a Tab node containing `[target, source]`.
    /// - If the target is in a Split of matching orientation (e.g. left/right => vertical),
    ///   we insert `source` as a new sibling child. Otherwise, we wrap `target` with a new Split node.
    pub fn reparent_leaf(
        &mut self,
        source_id: &str,
        target_id: &str,
        direction: DockReparentDirection,
    ) -> bool {
        // 1) Find both source + target paths
        let source_path = match self.find_leaf_path(source_id) {
            Some(path) => path,
            None => return false,
        };
        let target_path = match self.find_leaf_path(target_id) {
            Some(path) => path,
            None => return false,
        };
        // Disallow reparenting a leaf onto itself
        if source_path == target_path {
            return false;
        }

        // 2) Remove the source leaf from the tree
        let source_node = match Self::remove_leaf_from_tree(self, &source_path) {
            Some(node) => node,
            None => return false,
        };

        // After removal, the target path may have changed if the source was “above” or “before” it.
        // But for simplicity, we re-fetch it. If it’s gone or changed drastically, we bail.
        let updated_target_path = match self.find_leaf_path(target_id) {
            Some(path) => path,
            None => {
                // The target got removed or invalidated somehow
                // (e.g. if removing the source caused the target’s parent to vanish).
                // Insert the source node back or discard as you see fit.
                return false;
            }
        };

        // 3) Insert the `source_node` relative to the `updated_target_path` based on `direction`.
        match direction {
            DockReparentDirection::Tab => Self::insert_as_tab(self, source_node, &updated_target_path),
            DockReparentDirection::Left | DockReparentDirection::Right => {
                Self::insert_in_split(self, source_node, &updated_target_path, DockSplitDirection::VerticalDivider, direction)
            }
            DockReparentDirection::Top | DockReparentDirection::Bottom => {
                Self::insert_in_split(self, source_node, &updated_target_path, DockSplitDirection::HorizontalDivider, direction)
            }
        }
    }

    /// Removes a leaf (by path) from its parent's child list or tab list in `tree` and
    /// returns the corresponding `DockNode`.  
    fn remove_leaf_from_tree(
        &mut self,
        leaf_path: &[usize],
    ) -> Option<DockNode> {
        // Safely split off the last element
        let (child_index, parent_slice) = leaf_path.split_last()?;
        let child_index = *child_index; // child_index is &usize, so deref it

        let parent_node = self.get_node_mut(parent_slice)?;
        match parent_node {
            DockNode::Split { children, .. } => {
                if child_index < children.len() {
                    let DockSplitChild { node: removed_node, .. } = children.remove(child_index);
                    Some(removed_node)
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
                let old_root = std::mem::replace(parent_node, DockNode::default());
                Some(old_root)
            }
        }
    }

    /// insert_as_tab attempts to insert `source_node` into the same tab as `target_path`.
    /// - If `target_path` is a Leaf with a *Tab parent*, we insert into that parent Tab.
    /// - Else, we match on {Tab, Leaf, Split} as before.
    fn insert_as_tab(
        &mut self,
        source_node: DockNode,
        target_path: &[usize],
    ) -> bool {
        // 1) We do a read-only check to see if target_path is a Leaf with a Tab parent
        let leaf_parent_is_tab = self.is_leaf_with_tab_parent(target_path);

        if leaf_parent_is_tab {
            // 2a) If yes, we do a single mutable borrow for the parent Tab
            //     and push `source_node` into that parent's `tabs`.
            return self.insert_into_parent_tab(source_node, target_path);
        }

        // 2b) Otherwise, we do a single mutable borrow for the target node
        //     and handle it as (Tab -> push, Leaf -> convert, Split -> wrap).
        let Some(target_node) = self.get_node_mut(target_path) else {
            return false;
        };

        match target_node {
            DockNode::Tab { tabs, active_tab_id } => {
                tabs.push(source_node);
                if let DockNode::Leaf { window_identifier, .. } = &tabs[tabs.len() - 1] {
                    *active_tab_id = window_identifier.clone();
                }
                true
            }

            DockNode::Leaf { window_identifier, is_visible } => {
                // Convert Leaf -> Tab containing [old_leaf, source_node]
                let old_id = window_identifier.clone();
                let old_vis = *is_visible;
                let tabs = vec![
                    DockNode::Leaf {
                        window_identifier: old_id,
                        is_visible: old_vis,
                    },
                    source_node,
                ];
                // Choose the newly added leaf as active
                let active_tab_id = if let DockNode::Leaf { window_identifier, .. } = &tabs[1] {
                    window_identifier.clone()
                } else {
                    String::new()
                };
                *target_node = DockNode::Tab { tabs, active_tab_id };
                true
            }

            DockNode::Split { .. } => {
                // Wrap Split in a Tab (or whatever your existing code does).
                let old_split = std::mem::replace(target_node, DockNode::default());
                *target_node = DockNode::Tab {
                    tabs: vec![old_split, source_node],
                    active_tab_id: String::new(),
                };
                true
            }
        }
    }

    /// Returns true if `target_path` points to a Leaf whose parent is a Tab.
    fn is_leaf_with_tab_parent(
        &self,
        target_path: &[usize],
    ) -> bool {
        if target_path.is_empty() {
            return false;
        }
        let Some(target_node) = self.get_node(target_path) else {
            return false;
        };
        // Check that the target is a Leaf
        if !matches!(target_node, DockNode::Leaf { .. }) {
            return false;
        }

        // Check that the parent node exists and is a Tab
        let parent_slice = &target_path[..target_path.len() - 1];
        let Some(parent_node) = self.get_node(parent_slice) else {
            return false;
        };
        matches!(parent_node, DockNode::Tab { .. })
    }

    /// If the target node is a Leaf with a Tab parent, this inserts `source_node`
    /// into that parent's `tabs`. Returns `true` on success, `false` otherwise.
    fn insert_into_parent_tab(
        &mut self,
        source_node: DockNode,
        target_path: &[usize],
    ) -> bool {
        // parent_slice = everything except the last index
        let parent_slice = &target_path[..target_path.len() - 1];

        // We do one mutable borrow for the parent node
        let Some(parent_node) = self.get_node_mut(parent_slice) else {
            return false;
        };

        if let DockNode::Tab { tabs, active_tab_id } = parent_node {
            // Just push source_node into the parent's tabs
            tabs.push(source_node);

            // Optionally set the newly inserted leaf as active
            if let DockNode::Leaf { window_identifier, .. } = &tabs[tabs.len() - 1] {
                *active_tab_id = window_identifier.clone();
            }
            true
        } else {
            false
        }
    }

    /// Inserts `source_node` to the left/right (vertical split) or top/bottom (horizontal split)
    /// of the `target_path` node. If the parent of `target_path` is already a matching Split,
    /// we simply insert a new child. Otherwise, we replace the `target_path` node with a newly
    /// created Split node that contains `[target, source]`.
    fn insert_in_split(
        &mut self,
        source_node: DockNode,
        target_path: &[usize],
        required_direction: DockSplitDirection,
        reparent_direction: DockReparentDirection,
    ) -> bool {
        // If the path is empty, handle root
        if target_path.is_empty() {
            return Self::wrap_root_in_split(self, source_node, required_direction, reparent_direction);
        }

        // Manually slice:
        let (parent_slice, child_slice) = target_path.split_at(target_path.len() - 1);
        // child_slice now has exactly 1 element, so:
        let child_index = child_slice[0];

        let parent_node = match self.get_node_mut(parent_slice) {
            Some(n) => n,
            None => {
                return Self::wrap_root_in_split(self, source_node, required_direction, reparent_direction);
            }
        };

        // 2) Check if `parent_node` is already a matching split
        if let DockNode::Split { direction, children } = parent_node {
            if *direction == required_direction {
                // Insert the new child to the left/right/top/bottom of the target child
                let new_child = DockSplitChild {
                    node: source_node,
                    ratio: 0.0, // We'll fix the ratio below
                };

                // If we want to insert `source_node` *after* the target child (Right/Bottom),
                // we do child_index+1, otherwise we do child_index.
                let insert_at = match reparent_direction {
                    DockReparentDirection::Right | DockReparentDirection::Bottom => child_index + 1,
                    DockReparentDirection::Left | DockReparentDirection::Top => child_index,
                    DockReparentDirection::Tab => unreachable!("Handled elsewhere"),
                };

                if insert_at <= children.len() {
                    children.insert(insert_at, new_child);
                    Self::recalculate_split_ratios(children);
                    return true;
                } else {
                    return false;
                }
            }
        }

        // 3) If we get here, parent_node is not a matching Split => we need to replace
        // the *target child* itself with a new Split node of the correct orientation.
        if let Some(target_child) = Self::get_mut_child_in_container(parent_node, child_index) {
            let old_target_node = std::mem::replace(target_child, DockNode::default());
            let (first_child, second_child) = match reparent_direction {
                DockReparentDirection::Right | DockReparentDirection::Bottom => (old_target_node, source_node),
                DockReparentDirection::Left | DockReparentDirection::Top => (source_node, old_target_node),
                DockReparentDirection::Tab => unreachable!("Handled in insert_as_tab"),
            };

            // Create the new Split node
            let new_split = DockNode::Split {
                direction: required_direction,
                children: vec![
                    DockSplitChild { node: first_child, ratio: 0.5 },
                    DockSplitChild {
                        node: second_child,
                        ratio: 0.5,
                    },
                ],
            };
            *target_child = new_split;
            return true;
        }

        false
    }

    /// Helper to replace the `tree.root` with a new Split node if the target node was the root.
    fn wrap_root_in_split(
        tree: &mut DockTree,
        source_node: DockNode,
        split_direction: DockSplitDirection,
        reparent_direction: DockReparentDirection,
    ) -> bool {
        let old_root = std::mem::replace(&mut tree.root, DockNode::default());
        let (first_child, second_child) = match reparent_direction {
            DockReparentDirection::Right | DockReparentDirection::Bottom => (old_root, source_node),
            DockReparentDirection::Left | DockReparentDirection::Top => (source_node, old_root),
            DockReparentDirection::Tab => unreachable!("Handled in insert_as_tab"),
        };
        tree.root = DockNode::Split {
            direction: split_direction,
            children: vec![
                DockSplitChild { node: first_child, ratio: 0.5 },
                DockSplitChild {
                    node: second_child,
                    ratio: 0.5,
                },
            ],
        };
        true
    }

    /// Helper to retrieve a mutable child node from either a Split or Tab parent by index.
    /// Returns None if invalid.
    fn get_mut_child_in_container<'a>(
        parent_node: &'a mut DockNode,
        child_index: usize,
    ) -> Option<&'a mut DockNode> {
        match parent_node {
            DockNode::Split { children, .. } => {
                if child_index < children.len() {
                    Some(&mut children[child_index].node)
                } else {
                    None
                }
            }
            DockNode::Tab { tabs, .. } => {
                if child_index < tabs.len() {
                    Some(&mut tabs[child_index])
                } else {
                    None
                }
            }
            DockNode::Leaf { .. } => None,
        }
    }

    /// Recalculate the ratios in a split’s children so that all children sum to 1.
    /// A simple approach is to take the previous ratio for each child (or 1/n if 0) and normalize.
    fn recalculate_split_ratios(children: &mut [DockSplitChild]) {
        // Start by collecting the children’s raw ratios. If any ratio is 0, treat as some default (e.g. 0.1).
        let raw: Vec<f32> = children
            .iter()
            .map(|child| if child.ratio <= 0.0 { 0.1 } else { child.ratio })
            .collect();

        // Sum them up
        let sum: f32 = raw.iter().sum();
        if sum < f32::EPSILON {
            // If everything was zero, just set everything to 1/n
            let each = 1.0 / (children.len() as f32);
            for c in children.iter_mut() {
                c.ratio = each;
            }
            return;
        }

        // Otherwise, normalize to sum=1
        for (child, &r) in children.iter_mut().zip(&raw) {
            child.ratio = r / sum;
        }
    }
}
