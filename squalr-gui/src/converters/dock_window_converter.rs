use crate::DockedWindowViewData;
use crate::models::docking::docking_manager::DockingManager;
use crate::models::docking::hierarchy::dock_node::DockNode;
use slint::{ModelRc, VecModel};
use slint_mvvm::convert_to_view_data::ConvertToViewData;
use slint_mvvm::converters::shared_string_converter::SharedStringConverter;
use std::sync::Arc;
use std::sync::RwLock;

pub struct DockWindowConverter {
    docking_manager: Arc<RwLock<DockingManager>>,
}

impl DockWindowConverter {
    pub fn new(docking_manager: Arc<RwLock<DockingManager>>) -> Self {
        Self { docking_manager }
    }
}

impl ConvertToViewData<DockNode, DockedWindowViewData> for DockWindowConverter {
    fn convert_collection(
        &self,
        docked_window_nodes: &Vec<DockNode>,
    ) -> Vec<DockedWindowViewData> {
        docked_window_nodes
            .into_iter()
            .map(|item| self.convert_to_view_data(item))
            .collect()
    }

    fn convert_to_view_data(
        &self,
        docked_window_node: &DockNode,
    ) -> DockedWindowViewData {
        match docked_window_node {
            DockNode::Window { window_identifier, is_visible } => {
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
                // If there is only 1 sibling (ie 1 active tab), then just treat this as no siblings to prevent tab creation.
                let siblings = {
                    let visible_siblings = manager.get_sibling_tab_ids(window_identifier, true);
                    if visible_siblings.len() == 1 { vec![] } else { visible_siblings }
                };

                let found_active_tab_id = manager.get_active_tab(window_identifier);

                DockedWindowViewData {
                    identifier: window_identifier.clone().into(),
                    is_docked: true,
                    is_visible: *is_visible,
                    position_x: x,
                    position_y: y,
                    width: w,
                    height: h,
                    tab_ids: ModelRc::new(VecModel::from(SharedStringConverter::new().convert_collection(&siblings))),
                    active_tab_id: found_active_tab_id.into(),
                }
            }

            // If it's not a Window, just return a default.
            _ => DockedWindowViewData::default(),
        }
    }
}
