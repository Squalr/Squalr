use crate::DockedWindowViewData;
use crate::models::docking::dock_node::DockNode;
use crate::models::docking::docking_manager::DockingManager;
use slint::SharedString;
use slint::{ModelRc, VecModel};
use slint_mvvm::view_data_converter::ViewDataConverter;
use std::sync::Arc;
use std::sync::RwLock;

pub struct DockPanelConverter {
    docking_manager: Arc<RwLock<DockingManager>>,
}

impl DockPanelConverter {
    pub fn new(docking_manager: Arc<RwLock<DockingManager>>) -> Self {
        Self { docking_manager }
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
            DockNode::Leaf { window_identifier, is_visible } => {
                let manager = match self.docking_manager.read() {
                    Ok(m) => m,
                    Err(_) => {
                        log::error!("Failed to lock DockingManager for reading");
                        return DockedWindowViewData::default();
                    }
                };

                // Find bounding rectangle.
                let (x, y, w, h) = manager
                    .find_window_rect(window_identifier)
                    .unwrap_or((0.0, 0.0, 0.0, 0.0));

                // Gather siblings if in a parent tab, as well as which of those tabs is active.
                let (siblings, found_active_tab_id) = manager.get_siblings_and_active_tab(window_identifier);

                // If the active_tab_id is NOT this leaf, we treat it as occluded.
                let is_occluded = !siblings.is_empty() && found_active_tab_id != *window_identifier;

                let siblings_converted: Vec<SharedString> = siblings.iter().map(|str| SharedString::from(str)).collect();

                DockedWindowViewData {
                    identifier: window_identifier.clone().into(),
                    is_docked: true,
                    is_visible: *is_visible && !is_occluded,
                    position_x: x,
                    position_y: y,
                    width: w,
                    height: h,
                    tab_ids: ModelRc::new(VecModel::from(siblings_converted)),
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
