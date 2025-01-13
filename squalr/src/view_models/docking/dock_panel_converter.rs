use crate::models::docking::dock_node::DockNode;
use crate::{DockedWindowViewData, models::docking::docking_layout::DockingLayout};
use slint::{ModelRc, VecModel};
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
                if let Ok(docking_layout) = self.docking_layout.read() {
                    let dock_bounds = docking_layout
                        .calculate_window_rect(&window_identifier)
                        .unwrap_or((0.0, 0.0, 0.0, 0.0));

                    DockedWindowViewData {
                        identifier: window_identifier.clone().into(),
                        is_docked: true,
                        is_visible: *is_visible,
                        position_x: dock_bounds.0,
                        position_y: dock_bounds.1,
                        width: dock_bounds.2,
                        height: dock_bounds.3,
                        tabs: ModelRc::new(VecModel::from(vec![])),
                    }
                } else {
                    DockedWindowViewData::default()
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
