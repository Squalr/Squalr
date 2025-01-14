use crate::models::docking::layout::dock_node::DockNode;
use crate::{DockedWindowViewData, models::docking::layout::docking_layout::DockingLayout};
use slint::{ModelRc, SharedString, VecModel};
use slint_mvvm::view_data_converter::ViewDataConverter;
use std::sync::Arc;
use std::sync::RwLock;

pub struct DockPanelConverter {
    docking_layout: Arc<RwLock<DockingLayout>>,
}

impl DockPanelConverter {
    pub fn new(docking_layout: Arc<RwLock<DockingLayout>>) -> Self {
        Self { docking_layout }
    }

    /// Returns the rectangle of the given panel identifier.
    fn get_bounds(
        &self,
        window_id: &str,
    ) -> (f32, f32, f32, f32) {
        let docking_layout = match self.docking_layout.read() {
            Ok(dl) => dl,
            Err(_) => {
                log::error!("Failed to lock docking_layout for reading");
                return (0.0, 0.0, 0.0, 0.0);
            }
        };

        docking_layout
            .calculate_window_rect(window_id)
            .unwrap_or((0.0, 0.0, 0.0, 0.0))
    }

    /// Finds sibling leaves in a parent Tab node (if any) plus the parent's active_tab_id for that leaf.
    fn get_siblings_and_active_tab(
        &self,
        window_id: &str,
    ) -> (Vec<SharedString>, String) {
        let docking_layout = match self.docking_layout.read() {
            Ok(dl) => dl,
            Err(_) => {
                log::error!("Failed to lock docking_layout for reading");
                return (Vec::new(), window_id.to_owned());
            }
        };

        // Use the new single-pass method to get node + path
        if let Some((_leaf, path)) = docking_layout.find_node_by_id(window_id) {
            // If there's a parent:
            if !path.is_empty() {
                let parent_path = &path[..path.len() - 1];
                let parent_node = DockingLayout::get_node(&docking_layout.root, parent_path);

                // If that parent is a Tab node, gather siblings
                if let DockNode::Tab { tabs, active_tab_id, .. } = parent_node {
                    let visible_siblings = tabs
                        .iter()
                        .filter_map(|tab_node| match tab_node {
                            DockNode::Leaf {
                                window_identifier, is_visible, ..
                            } if *is_visible => Some(window_identifier.clone().into()),
                            _ => None,
                        })
                        .collect();

                    return (visible_siblings, active_tab_id.clone());
                }
            }
        }

        // Fallback: no parent tab found.
        (Vec::new(), window_id.to_owned())
    }
}

impl ViewDataConverter<DockNode, DockedWindowViewData> for DockPanelConverter {
    fn convert_collection(
        &self,
        docked_window_nodes: &Vec<DockNode>,
    ) -> Vec<DockedWindowViewData> {
        return docked_window_nodes
            .into_iter()
            .map(|item| self.convert_to_view_data(item))
            .collect();
    }

    fn convert_to_view_data(
        &self,
        docked_window_node: &DockNode,
    ) -> DockedWindowViewData {
        match docked_window_node {
            DockNode::Leaf {
                window_identifier,
                is_visible,
                ratio: _,
            } => {
                // Get bounding rectangle.
                let (x, y, w, h) = self.get_bounds(window_identifier);

                // Gather siblings if in a parent tab.
                let (siblings, found_active_tab_id) = self.get_siblings_and_active_tab(window_identifier);

                // If the active_tab_id is NOT this leaf, we treat it as occluded.
                let is_occluded = !siblings.is_empty() && found_active_tab_id != *window_identifier;

                DockedWindowViewData {
                    identifier: window_identifier.clone().into(),
                    is_docked: true,
                    is_visible: *is_visible && !is_occluded,
                    position_x: x,
                    position_y: y,
                    width: w,
                    height: h,
                    tab_ids: ModelRc::new(VecModel::from(siblings)),
                    active_tab_id: found_active_tab_id.into(),
                }
            }

            // If it's not a Leaf, just return a default.
            _ => DockedWindowViewData::default(),
        }
    }

    fn convert_from_view_data(
        &self,
        _: &DockedWindowViewData,
    ) -> DockNode {
        panic!("Not implemented!");
    }
}
