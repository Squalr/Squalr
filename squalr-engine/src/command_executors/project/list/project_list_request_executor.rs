use crate::command_executors::engine_request_executor::EngineCommandRequestExecutor;
use crate::engine_privileged_state::EnginePrivilegedState;
use squalr_engine_api::commands::project::list::project_list_request::ProjectListRequest;
use squalr_engine_api::commands::project::list::project_list_response::ProjectListResponse;
use squalr_engine_api::structures::processes::process_icon::ProcessIcon;
use squalr_engine_api::structures::projects::project_info::ProjectInfo;
use squalr_engine_projects::project_settings_config::ProjectSettingsConfig;
use std::sync::Arc;

impl EngineCommandRequestExecutor for ProjectListRequest {
    type ResponseType = ProjectListResponse;

    fn execute(
        &self,
        _engine_privileged_state: &Arc<EnginePrivilegedState>,
    ) -> <Self as EngineCommandRequestExecutor>::ResponseType {
        let projects_root = ProjectSettingsConfig::get_projects_root();
        let mut projects_info = vec![];

        if let Ok(read_dir) = std::fs::read_dir(projects_root) {
            for entry in read_dir {
                if let Ok(entry) = entry {
                    let entry_path = entry.path();

                    if entry_path.is_dir() {
                        if let Some(directory_name) = entry_path.file_name() {
                            let project_name = directory_name.to_string_lossy().to_string();
                            let icon = ProcessIcon::new(vec![], 0, 0);

                            projects_info.push(ProjectInfo::new(project_name, icon));
                        }
                    }
                }
            }
        }

        ProjectListResponse { projects_info }
    }
}
