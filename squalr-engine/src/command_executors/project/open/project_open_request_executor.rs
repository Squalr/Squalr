use crate::command_executors::engine_request_executor::EngineCommandRequestExecutor;
use crate::engine_privileged_state::EnginePrivilegedState;
use squalr_engine_api::commands::project::open::project_open_request::ProjectOpenRequest;
use squalr_engine_api::commands::project::open::project_open_response::ProjectOpenResponse;
use std::sync::Arc;
use sysinfo::Pid;

impl EngineCommandRequestExecutor for ProjectOpenRequest {
    type ResponseType = ProjectOpenResponse;

    fn execute(
        &self,
        engine_privileged_state: &Arc<EnginePrivilegedState>,
    ) -> <Self as EngineCommandRequestExecutor>::ResponseType {
        /*
        if self.project_id.is_none() && self.search_name.is_none() {
            log::error!("Error: Neither PID nor search name provided. Cannot open project.");
            return ProjectOpenResponse { opened_project_info: None };
        }

        log::info!("Opening project...");

        let options = ProjectQueryOptions {
            search_name: self.search_name.clone(),
            required_project_id: self.project_id.map(Pid::from_u32),
            require_windowed: false,
            match_case: self.match_case,
            fetch_icons: false,
            limit: Some(1),
        };

        let projectes = ProjectQuery::get_projectes(options);

        if let Some(project_info) = projectes.first() {
            match ProjectQuery::open_project(&project_info) {
                Ok(opened_project_info) => {
                    engine_privileged_state.set_opened_project(opened_project_info.clone());

                    return ProjectOpenResponse {
                        opened_project_info: Some(opened_project_info),
                    };
                }
                Err(err) => {
                    log::info!("Failed to open project {}: {}", project_info.get_project_id_raw(), err);
                }
            }
        } else {
            log::error!("No matching project found.");
        }*/

        ProjectOpenResponse { opened_project_info: None }
    }
}
