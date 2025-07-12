use crate::command_executors::engine_request_executor::EngineCommandRequestExecutor;
use crate::engine_privileged_state::EnginePrivilegedState;
use olorin_engine_api::commands::project::list::project_list_request::ProjectListRequest;
use olorin_engine_api::commands::project::list::project_list_response::ProjectListResponse;
use olorin_engine_api::structures::projects::project_info::ProjectInfo;
use olorin_engine_projects::project::project::Project;
use olorin_engine_projects::project::serialization::serializable_project_file::SerializableProjectFile;
use olorin_engine_projects::settings::project_settings_config::ProjectSettingsConfig;
use std::sync::Arc;

impl EngineCommandRequestExecutor for ProjectListRequest {
    type ResponseType = ProjectListResponse;

    fn execute(
        &self,
        _engine_privileged_state: &Arc<EnginePrivilegedState>,
    ) -> <Self as EngineCommandRequestExecutor>::ResponseType {
        let projects_root = ProjectSettingsConfig::get_projects_root();
        let mut projects_info = vec![];

        match std::fs::read_dir(projects_root) {
            Ok(read_dir) => {
                for entry in read_dir {
                    if let Ok(entry) = entry {
                        let entry_path = entry.path();

                        if entry_path.is_dir() {
                            if let Ok(project_info) = ProjectInfo::load_from_path(&entry_path.join(Project::PROJECT_FILE)) {
                                projects_info.push(project_info);
                            }
                        }
                    }
                }
            }
            Err(error) => {
                log::error!("Failed to list projects: {}", error);
            }
        }

        ProjectListResponse { projects_info }
    }
}
