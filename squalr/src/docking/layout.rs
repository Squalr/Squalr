use serde::{Deserialize, Serialize};

/// Represents a single docked panel.
#[derive(Serialize, Deserialize)]
struct DockedPanel {
    /// The unique identifier for this panel.
    identifier: String,
}

/// Represents a single standalone panel that has been undocked into its own window.
#[derive(Serialize, Deserialize)]
struct StandalonePanel {
    /// The unique identifier for this panel.
    identifier: String,
    width: u32,
    height: u32,
}

/// Represents a node in the N-ary tree for docked panels.
#[derive(Serialize, Deserialize)]
struct DockedPanelNode {
    /// If None, this node is a container for child panels.
    panel: Option<DockedPanel>,
    /// The child panels (can be nested).
    children: Vec<DockedPanelNode>,
    /// Size ratio between sibling panels (optional for root).
    size_ratio: Option<f32>,
}

/// Represents the layout of all side panels (left/right).
#[derive(Serialize, Deserialize)]
struct SidePanel {
    /// The unique identifier for this panel.
    identifier: String,
}

/// Represents a group of side panels (left or right).
#[derive(Serialize, Deserialize)]
struct SidePanelGroup {
    /// The list of docked side panels.
    panels: Vec<SidePanel>,
    /// Size of the expanded side panel.
    expansion_size: u32,
    /// A value indicating whether the side panel collection has been expanded.
    is_expanded: bool,
}

/// Represents the entire layout structure.
#[derive(Serialize, Deserialize)]
struct Layout {
    /// The root of the N-ary tree representing docked panels.
    docked_panels: DockedPanelNode,
    /// The list of standalone windows that can be docked.
    standalone_windows: Vec<StandalonePanel>,
    /// The panels that are collapsed on the left-hand side.
    left_panels: SidePanelGroup,
    /// The panels that are collapsed on the right-hand side.
    right_panels: SidePanelGroup,
}
