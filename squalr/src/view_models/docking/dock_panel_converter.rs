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
                // Compute bounds, as you already do
                let (x, y, w, h) = if let Ok(docking_layout) = self.docking_layout.read() {
                    docking_layout
                        .calculate_window_rect(window_identifier)
                        .unwrap_or((0.0, 0.0, 0.0, 0.0))
                } else {
                    (0.0, 0.0, 0.0, 0.0)
                };

                // We gather siblings (including self) if the parent is a Tab node
                let mut siblings: Vec<SharedString> = Vec::new();
                let mut found_active_tab_id = window_identifier.clone();

                if let Ok(docking_layout) = self.docking_layout.read() {
                    // 1) Find the path from root to this leaf
                    if let Some(path) = DockingLayout::find_path_to_leaf(&docking_layout.root, window_identifier) {
                        // If there's a parent
                        if !path.is_empty() {
                            // parent path = path without the last index
                            let parent_path = &path[..path.len() - 1];

                            // 2) Get the parent node
                            let parent_node = DockingLayout::get_node(&docking_layout.root, parent_path);

                            // 3) If parent is a Tab, gather all tab identifiers
                            if let DockNode::Tab { tabs, active_tab_id, .. } = parent_node {
                                found_active_tab_id = active_tab_id.clone();
                                for tab_node in tabs {
                                    if let DockNode::Leaf {
                                        window_identifier: tab_id,
                                        is_visible,
                                        ..
                                    } = tab_node
                                    {
                                        if *is_visible {
                                            siblings.push(tab_id.clone().into());
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                let is_occluded = siblings.len() > 0 && found_active_tab_id != *window_identifier;

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
