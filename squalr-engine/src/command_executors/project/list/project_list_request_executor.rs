use crate::command_executors::unprivileged_request_executor::UnprivilegedCommandRequestExecutor;
use squalr_engine_api::commands::project::list::project_list_request::ProjectListRequest;
use squalr_engine_api::commands::project::list::project_list_response::ProjectListResponse;
use squalr_engine_api::engine::engine_execution_context::EngineExecutionContext;
use squalr_engine_api::structures::projects::project::Project;
use squalr_engine_api::structures::projects::project_info::ProjectInfo;
use squalr_engine_projects::project::serialization::serializable_project_file::SerializableProjectFile;
use squalr_engine_projects::settings::project_settings_config::ProjectSettingsConfig;
use std::sync::Arc;

impl UnprivilegedCommandRequestExecutor for ProjectListRequest {
    type ResponseType = ProjectListResponse;

    fn execute(
        &self,
        _engine_unprivileged_state: &Arc<dyn EngineExecutionContext>,
    ) -> <Self as UnprivilegedCommandRequestExecutor>::ResponseType {
        let projects_root = ProjectSettingsConfig::get_projects_root();
        let mut projects_info = vec![];

        if projects_root.exists() {
            match std::fs::read_dir(projects_root) {
                Ok(read_dir) => {
                    for entry in read_dir {
                        if let Ok(entry) = entry {
                            let entry_path = entry.path();

                            if entry_path.is_dir() {
                                let project_file = entry_path.join(Project::PROJECT_FILE);

                                if project_file.exists() {
                                    if let Ok(project_info) = ProjectInfo::load_from_path(&entry_path.join(Project::PROJECT_FILE)) {
                                        projects_info.push(project_info);
                                    }
                                }
                            }
                        }
                    }
                }
                Err(error) => {
                    log::error!("Failed to list projects: {}", error);
                }
            }
        }

        ProjectListResponse { projects_info }
    }
}
