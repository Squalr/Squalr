use crate::command_executors::engine_request_executor::EngineCommandRequestExecutor;
use crate::engine_privileged_state::EnginePrivilegedState;
use olorin_engine_api::commands::project_items::activate::project_items_activate_request::ProjectItemsActivateRequest;
use olorin_engine_api::commands::project_items::activate::project_items_activate_response::ProjectItemsActivateResponse;
use olorin_engine_api::structures::data_values::data_value::DataValue;
use std::sync::Arc;

impl EngineCommandRequestExecutor for ProjectItemsActivateRequest {
    type ResponseType = ProjectItemsActivateResponse;

    fn execute(
        &self,
        engine_privileged_state: &Arc<EnginePrivilegedState>,
    ) -> <Self as EngineCommandRequestExecutor>::ResponseType {
        if let Ok(snapshot_scan_result_freeze_list) = engine_privileged_state
            .get_snapshot_scan_result_freeze_list()
            .read()
        {
            for project_item_id in &self.project_item_ids {
                if self.is_activated {
                    snapshot_scan_result_freeze_list.set_address_frozen(0, DataValue::default());
                } else {
                    snapshot_scan_result_freeze_list.set_address_unfrozen(0);
                }
            }
        }
        ProjectItemsActivateResponse {}
    }
}
