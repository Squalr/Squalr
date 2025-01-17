use crate::models::docking::hierarchy::dock_node::DockNode;
use crate::models::docking::hierarchy::dock_split_child::DockSplitChild;

impl DockNode {
    /// Normalize the ratios in the children so they sum to 1.
    pub fn recalculate_split_ratios(children: &mut [DockSplitChild]) {
        let raw: Vec<f32> = children
            .iter()
            .map(|c| if c.ratio <= 0.0 { 0.1 } else { c.ratio })
            .collect();

        let sum: f32 = raw.iter().sum();
        if sum < f32::EPSILON {
            let each = 1.0 / (children.len() as f32);
            for c in children.iter_mut() {
                c.ratio = each;
            }
        } else {
            for (child, &r) in children.iter_mut().zip(&raw) {
                child.ratio = r / sum;
            }
        }
    }
}
