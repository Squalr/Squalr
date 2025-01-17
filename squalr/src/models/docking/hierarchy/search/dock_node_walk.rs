use crate::models::docking::hierarchy::dock_node::DockNode;

impl DockNode {
    /// A generic tree walker that visits every node in depth-first order.
    /// `path` is updated with the child indices as we traverse.
    pub fn walk<'a, F>(
        &'a self,
        path: &mut Vec<usize>,
        visitor: &mut F,
    ) where
        F: FnMut(&'a DockNode, &[usize]),
    {
        // Visit the current node
        visitor(self, path);

        // Recurse into children, if any
        match self {
            DockNode::Split { children, .. } => {
                for (index, child) in children.iter().enumerate() {
                    path.push(index);
                    child.node.walk(path, visitor);
                    path.pop();
                }
            }
            DockNode::Tab { tabs, .. } => {
                for (index, tab) in tabs.iter().enumerate() {
                    path.push(index);
                    tab.walk(path, visitor);
                    path.pop();
                }
            }
            // No children to recurse.
            DockNode::Window { .. } => {}
        }
    }
}
