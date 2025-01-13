use crate::models::docking::docked_window_node::DockedWindowNode;
use crate::{DockedWindowViewData, models::docking::docking_layout::DockingLayout};
use slint::{ModelRc, VecModel};
use slint_mvvm::view_data_converter::ViewDataConverter;
use std::sync::Arc;
use std::sync::Mutex;

pub struct DockPanelConverter {
    docking_layout: Arc<Mutex<DockingLayout>>,
}

impl DockPanelConverter {
    pub fn new(docking_layout: Arc<Mutex<DockingLayout>>) -> Self {
        Self { docking_layout }
    }
}

impl ViewDataConverter<DockedWindowNode, DockedWindowViewData> for DockPanelConverter {
    fn convert_collection(
        &self,
        docked_window_nodes: &Vec<DockedWindowNode>,
    ) -> Vec<DockedWindowViewData> {
        return docked_window_nodes
            .into_iter()
            .map(|item| self.convert_to_view_data(item))
            .collect();
    }

    fn convert_to_view_data(
        &self,
        docked_window_node: &DockedWindowNode,
    ) -> DockedWindowViewData {
        let dock_bounds = self
            .docking_layout
            .lock()
            .unwrap()
            .calculate_window_rect(&docked_window_node.window_identifier)
            .unwrap_or((0.0, 0.0, 0.0, 0.0));

        DockedWindowViewData {
            identifier: docked_window_node.window_identifier.clone().into(),
            is_docked: true,
            is_visible: true,
            position_x: dock_bounds.0,
            position_y: dock_bounds.1,
            width: dock_bounds.2,
            height: dock_bounds.3,
            tabs: ModelRc::new(VecModel::from(vec![])),
        }
    }

    fn convert_from_view_data(
        &self,
        _: &DockedWindowViewData,
    ) -> DockedWindowNode {
        panic!("Not implemented!");
    }
}
