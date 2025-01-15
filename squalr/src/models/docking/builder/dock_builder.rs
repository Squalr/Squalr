use crate::models::docking::builder::dock_builder_kind::DockBuilderKind;
use crate::models::docking::hierarchy::dock_node::DockNode;
use crate::models::docking::hierarchy::dock_split_child::DockSplitChild;
use crate::models::docking::hierarchy::dock_split_direction::DockSplitDirection;

#[derive(Debug)]
pub struct DockBuilder {
    /// Internal variant describing what this node will become.
    kind: DockBuilderKind,

    /// Whether this node (if leaf) is visible or not.
    /// For splits/tabs, this usually doesn't directly control the node's visibility.
    is_visible: bool,
}

impl DockBuilder {
    /// Create a new builder for a Leaf node.
    pub fn leaf(id: impl Into<String>) -> Self {
        Self {
            kind: DockBuilderKind::Leaf { window_identifier: id.into() },
            is_visible: true,
        }
    }

    /// Create a new builder for a Split node.
    pub fn split_node(direction: DockSplitDirection) -> Self {
        Self {
            kind: DockBuilderKind::Split {
                direction,
                children: Vec::new(),
            },
            is_visible: true,
        }
    }

    /// Create a new builder for a Tab container node.
    pub fn tab_node(active_tab_id: impl Into<String>) -> Self {
        Self {
            kind: DockBuilderKind::Tab {
                tabs: Vec::new(),
                active_tab_id: active_tab_id.into(),
            },
            is_visible: true,
        }
    }

    /// Sets visibility (relevant for leaf nodes).
    pub fn visible(
        mut self,
        visible: bool,
    ) -> Self {
        self.is_visible = visible;
        self
    }

    /// Push a child into a Split node. Each child has its own ratio here.
    pub fn push_child(
        mut self,
        ratio: f32,
        child: DockBuilder,
    ) -> Self {
        match &mut self.kind {
            DockBuilderKind::Split { children, .. } => {
                children.push((child, ratio));
            }
            _ => {
                panic!("push_child() called on a non-Split builder.");
            }
        }
        self
    }

    /// Push a tab into a Tab container node.
    pub fn push_tab(
        mut self,
        child: DockBuilder,
    ) -> Self {
        match &mut self.kind {
            DockBuilderKind::Tab { tabs, .. } => {
                tabs.push(child);
            }
            _ => {
                panic!("push_tab() called on a non-Tab builder.");
            }
        }
        self
    }

    /// Sets (or overrides) the split direction if this is a Split node.
    pub fn direction(
        mut self,
        direction: DockSplitDirection,
    ) -> Self {
        match &mut self.kind {
            DockBuilderKind::Split { direction: dir, .. } => {
                *dir = direction;
            }
            _ => {
                panic!("direction() called on a non-Split builder.");
            }
        }
        self
    }

    /// Consume this builder and produce the final `DockNode`.
    pub fn build(self) -> DockNode {
        match self.kind {
            DockBuilderKind::Split { direction, children } => {
                // For a split node, build each child and gather ratios.
                let mut child_nodes = Vec::with_capacity(children.len());

                for (child_builder, child_ratio) in children {
                    child_nodes.push(DockSplitChild {
                        node: child_builder.build(),
                        ratio: child_ratio,
                    });
                }

                DockNode::Split {
                    direction,
                    children: child_nodes,
                }
            }

            DockBuilderKind::Tab { tabs, active_tab_id } => {
                // For a tab node, each child has a separate builder,
                // but the node itself has a single ratio.
                let child_nodes = tabs.into_iter().map(|b| b.build()).collect();

                DockNode::Tab {
                    tabs: child_nodes,
                    active_tab_id,
                }
            }

            DockBuilderKind::Leaf { window_identifier } => DockNode::Leaf {
                window_identifier,
                is_visible: self.is_visible,
            },
        }
    }
}
