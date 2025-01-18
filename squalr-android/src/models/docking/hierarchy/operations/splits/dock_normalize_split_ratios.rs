use crate::models::docking::hierarchy::dock_node::DockNode;
use crate::models::docking::hierarchy::types::dock_split_child::DockSplitChild;

impl DockNode {
    /// Normalize the ratios in the children so they sum to 1.0.
    pub fn normalize_split_ratios(children: &mut [DockSplitChild]) {
        let raw: Vec<f32> = children
            .iter()
            .map(|child| if child.ratio <= 0.0 { 0.1 } else { child.ratio })
            .collect();

        let sum: f32 = raw.iter().sum();
        if sum < f32::EPSILON {
            let each = 1.0 / (children.len() as f32);
            for child in children.iter_mut() {
                child.ratio = each;
            }
        } else {
            for (child, &ratio) in children.iter_mut().zip(&raw) {
                child.ratio = ratio / sum;
            }
        }
    }
}
