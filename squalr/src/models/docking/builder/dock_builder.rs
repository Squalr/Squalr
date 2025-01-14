use crate::models::docking::layout::dock_node::DockNode;
use crate::models::docking::layout::dock_split_direction::DockSplitDirection;

#[derive(Debug)]
pub enum DockBuilderKind {
    Split {
        direction: DockSplitDirection,
        children: Vec<DockBuilder>,
    },
    Tab {
        tabs: Vec<DockBuilder>,
        active_tab_id: String,
    },
    Leaf {
        window_identifier: String,
    },
}

#[derive(Debug)]
pub struct DockBuilder {
    kind: DockBuilderKind,
    is_visible: bool,
    ratio: f32,
}

impl DockBuilder {
    /// Create a new builder for a Leaf node.
    pub fn leaf(id: impl Into<String>) -> Self {
        Self {
            kind: DockBuilderKind::Leaf { window_identifier: id.into() },
            is_visible: true,
            ratio: 1.0,
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
            ratio: 1.0,
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
            ratio: 1.0,
        }
    }

    /// Sets the ratio for this node (relevant for siblings).
    pub fn ratio(
        mut self,
        ratio: f32,
    ) -> Self {
        self.ratio = ratio;
        self
    }

    /// Sets visibility.
    pub fn visible(
        mut self,
        visible: bool,
    ) -> Self {
        self.is_visible = visible;
        self
    }

    /// Push a child into a Split node.
    pub fn push_child(
        mut self,
        ratio: f32,
        child: DockBuilder,
    ) -> Self {
        match &mut self.kind {
            DockBuilderKind::Split { children, .. } => {
                children.push(child.ratio(ratio));
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

    /// Consume the builder and produce a `DockNode`.
    pub fn build(self) -> DockNode {
        match self.kind {
            DockBuilderKind::Split { direction, children } => DockNode::Split {
                ratio: self.ratio,
                direction,
                children: children.into_iter().map(|b| b.build()).collect(),
            },
            DockBuilderKind::Tab { tabs, active_tab_id } => DockNode::Tab {
                ratio: self.ratio,
                tabs: tabs.into_iter().map(|b| b.build()).collect(),
                active_tab_id: active_tab_id,
            },
            DockBuilderKind::Leaf { window_identifier } => DockNode::Leaf {
                ratio: self.ratio,
                window_identifier,
                is_visible: self.is_visible,
            },
        }
    }
}
