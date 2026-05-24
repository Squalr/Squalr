use crate::models::docking::hierarchy::dock_node::DockNode;
use crate::models::docking::hierarchy::types::dock_tab_insertion_direction::DockTabInsertionDirection;

impl DockNode {
    /// Reparents `source_node` into the same tab group as `target_path`.
    /// If the target’s parent is a Tab, we just insert into that Tab’s list.
    /// Otherwise, if the target itself is a Window, we convert that Window into a Tab.
    /// If the target is a Split, we wrap it in a Tab (but remember, no splits in tabs).
    /// Returns `true` on success.
    pub fn reparent_as_tab(
        &mut self,
        source_node: DockNode,
        target_path: &[usize],
    ) -> bool {
        // If the target's parent is a Tab, we insert into the parent of `target_path`
        if let Some(parent) = self.get_node_from_path_mut(&target_path[..target_path.len().saturating_sub(1)]) {
            if let DockNode::Tab { tabs, active_tab_id } = parent {
                Self::insert_tab_child_at(tabs, active_tab_id, source_node, 0);
                return true;
            }
        }

        // Otherwise, get or convert the target node to a tab.
        let Some(target_node) = self.get_node_from_path_mut(target_path) else {
            return false;
        };

        match target_node {
            DockNode::Tab { tabs, active_tab_id } => {
                Self::insert_tab_child_at(tabs, active_tab_id, source_node, 0);
                true
            }
            DockNode::Window { .. } => {
                let new_tab = Self::convert_window_to_tab(std::mem::take(target_node), source_node);
                *target_node = new_tab;
                true
            }
            DockNode::Split { .. } => false,
        }
    }

    /// Reparents `source_window_identifier` into an existing tab group relative to `target_window_identifier`.
    pub fn reparent_window_relative_to_tab(
        &mut self,
        source_window_identifier: &str,
        target_window_identifier: &str,
        tab_insertion_direction: DockTabInsertionDirection,
    ) -> bool {
        if source_window_identifier == target_window_identifier {
            return true;
        }

        let source_window_path = match self.find_path_to_window_id(source_window_identifier) {
            Some(path) => path,
            None => return false,
        };
        let source_node = match self.remove_window_by_path(&source_window_path) {
            Some(node) => node,
            None => return false,
        };
        let target_window_path = match self.find_path_to_window_id(target_window_identifier) {
            Some(path) => path,
            None => return false,
        };
        let Some((target_tab_index, target_tab_group_path)) = target_window_path.split_last() else {
            return false;
        };
        let Some(target_tab_group_node) = self.get_node_from_path_mut(target_tab_group_path) else {
            return false;
        };

        match target_tab_group_node {
            DockNode::Tab { tabs, active_tab_id } => {
                let insertion_index = match tab_insertion_direction {
                    DockTabInsertionDirection::BeforeTarget => *target_tab_index,
                    DockTabInsertionDirection::AfterTarget => *target_tab_index + 1,
                };

                Self::insert_tab_child_at(tabs, active_tab_id, source_node, insertion_index)
            }
            DockNode::Split { .. } | DockNode::Window { .. } => false,
        }
    }

    /// Inserts a new child window into an existing tab group.
    fn insert_tab_child_at(
        tabs: &mut Vec<DockNode>,
        active_tab_id: &mut String,
        child: DockNode,
        insertion_index: usize,
    ) -> bool {
        if insertion_index > tabs.len() {
            return false;
        }

        let inserted_window_identifier = child.get_window_id();
        tabs.insert(insertion_index, child);

        if let Some(inserted_window_identifier) = inserted_window_identifier {
            *active_tab_id = inserted_window_identifier;
        }

        true
    }

    /// Convert a single Window node + an extra node into a Tab node with both children.
    fn convert_window_to_tab(
        window: DockNode,
        other: DockNode,
    ) -> DockNode {
        // We assume `window` is actually a Window. If not, be defensive.
        let window_id = window.get_window_id().unwrap_or_default();
        let other_id = other.get_window_id().unwrap_or_default();
        DockNode::Tab {
            tabs: vec![window, other],
            active_tab_id: other_id.is_empty().then(|| window_id).unwrap_or(other_id),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::DockNode;
    use crate::models::docking::hierarchy::types::{
        dock_reparent_direction::DockReparentDirection, dock_split_child::DockSplitChild, dock_split_direction::DockSplitDirection,
        dock_tab_insertion_direction::DockTabInsertionDirection,
    };

    fn build_tabbed_layout() -> DockNode {
        DockNode::Split {
            direction: DockSplitDirection::VerticalDivider,
            children: vec![
                DockSplitChild {
                    node: DockNode::Tab {
                        tabs: vec![
                            DockNode::Window {
                                window_identifier: "tab_a".to_string(),
                                is_visible: true,
                            },
                            DockNode::Window {
                                window_identifier: "tab_b".to_string(),
                                is_visible: true,
                            },
                            DockNode::Window {
                                window_identifier: "tab_c".to_string(),
                                is_visible: true,
                            },
                        ],
                        active_tab_id: "tab_a".to_string(),
                    },
                    ratio: 0.5,
                },
                DockSplitChild {
                    node: DockNode::Window {
                        window_identifier: "solo".to_string(),
                        is_visible: true,
                    },
                    ratio: 0.5,
                },
            ],
        }
    }

    #[test]
    fn dropping_onto_existing_group_inserts_window_as_first_tab() {
        let mut dock_root = build_tabbed_layout();

        assert!(dock_root.reparent_window("solo", "tab_a", DockReparentDirection::Tab));

        assert_eq!(
            dock_root.get_sibling_tab_ids("tab_a", true),
            vec![
                "solo".to_string(),
                "tab_a".to_string(),
                "tab_b".to_string(),
                "tab_c".to_string(),
            ]
        );
        assert_eq!(dock_root.get_active_tab("solo"), "solo".to_string());
    }

    #[test]
    fn reordering_within_same_group_inserts_before_target_tab() {
        let mut dock_root = build_tabbed_layout();

        assert!(dock_root.reparent_window_relative_to_tab("tab_c", "tab_a", DockTabInsertionDirection::BeforeTarget,));

        assert_eq!(
            dock_root.get_sibling_tab_ids("tab_a", true),
            vec!["tab_c".to_string(), "tab_a".to_string(), "tab_b".to_string(),]
        );
        assert_eq!(dock_root.get_active_tab("tab_c"), "tab_c".to_string());
    }

    #[test]
    fn inserting_after_target_tab_places_window_on_right_side() {
        let mut dock_root = build_tabbed_layout();

        assert!(dock_root.reparent_window_relative_to_tab("solo", "tab_a", DockTabInsertionDirection::AfterTarget,));

        assert_eq!(
            dock_root.get_sibling_tab_ids("tab_a", true),
            vec![
                "tab_a".to_string(),
                "solo".to_string(),
                "tab_b".to_string(),
                "tab_c".to_string(),
            ]
        );
        assert_eq!(dock_root.get_active_tab("solo"), "solo".to_string());
    }
}
