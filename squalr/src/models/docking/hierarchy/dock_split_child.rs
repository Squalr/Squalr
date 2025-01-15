use crate::models::docking::hierarchy::dock_node::DockNode;
use serde::{Deserialize, Serialize};

/// Contains a child as well as their ratio based share of the layout space.
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct DockSplitChild {
    pub node: DockNode,
    pub ratio: f32,
}
