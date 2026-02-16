use crate::command_executors::unprivileged_request_executor::UnprivilegedCommandRequestExecutor;
use squalr_engine_api::commands::project::create::project_create_request::ProjectCreateRequest;
use squalr_engine_api::commands::project::create::project_create_response::ProjectCreateResponse;
use squalr_engine_api::engine::engine_execution_context::EngineExecutionContext;
use squalr_engine_api::structures::projects::project::Project;
use squalr_engine_api::structures::projects::project_info::ProjectInfo;
use squalr_engine_api::structures::projects::project_items::built_in_types::project_item_type_directory::ProjectItemTypeDirectory;
use squalr_engine_api::structures::projects::project_items::project_item_ref::ProjectItemRef;
use squalr_engine_api::structures::projects::project_manifest::ProjectManifest;
use squalr_engine_api::utils::file_system::file_system_utils::FileSystemUtils;
use squalr_engine_projects::project::serialization::serializable_project_file::SerializableProjectFile;
use squalr_engine_projects::settings::project_settings_config::ProjectSettingsConfig;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

impl UnprivilegedCommandRequestExecutor for ProjectCreateRequest {
    type ResponseType = ProjectCreateResponse;

    fn execute(
        &self,
        _engine_unprivileged_state: &Arc<dyn EngineExecutionContext>,
    ) -> <Self as UnprivilegedCommandRequestExecutor>::ResponseType {
        // If a path is provided, use this directly. Otherwise, try to use the project settings relative name to construct the path.
        // If no path nor project name is provided, we will just make an empty project with a default name.
        let project_directory_path = if let Some(project_directory_path) = &self.project_directory_path {
            project_directory_path.into()
        } else if let Some(project_name) = self.project_name.as_deref() {
            ProjectSettingsConfig::get_projects_root().join(project_name)
        } else {
            let project_root = ProjectSettingsConfig::get_projects_root();

            match FileSystemUtils::create_unique_folder(project_root.as_path(), "New Project") {
                Ok(project_path) => project_path,
                Err(error) => {
                    log::error!("Failed to create a unique default project name: {}", error);

                    return ProjectCreateResponse {
                        success: false,
                        new_project_path: PathBuf::default(),
                    };
                }
            }
        };

        if project_directory_path.exists() {
            match project_directory_path.read_dir() {
                Ok(mut read_dir) => {
                    if read_dir.next().is_some() {
                        log::error!("Cannot create project: directory already contains files");

                        return ProjectCreateResponse {
                            success: false,
                            new_project_path: PathBuf::default(),
                        };
                    }
                }
                Err(error) => {
                    log::error!("Failed to create project. Failed to check new project directory for existing files: {}", error);

                    return ProjectCreateResponse {
                        success: false,
                        new_project_path: PathBuf::default(),
                    };
                }
            }
        }

        if let Err(error) = fs::create_dir_all(project_directory_path.to_path_buf()) {
            log::error!("Failed to create new project directory: {}", error);

            return ProjectCreateResponse {
                success: false,
                new_project_path: PathBuf::default(),
            };
        }

        let project_info = ProjectInfo::new(project_directory_path.to_path_buf(), None, ProjectManifest::default());
        let project_root_ref = ProjectItemRef::new(project_directory_path.join(Project::PROJECT_DIR));
        let mut project_items = HashMap::new();

        project_items.insert(project_root_ref.clone(), ProjectItemTypeDirectory::new_project_item(&project_root_ref));

        let mut new_project = Project::new(project_info, project_items, project_root_ref);

        if let Err(error) = new_project.save_to_path(&project_directory_path, true) {
            log::error!("Failed to save initial new project directory: {}", error);

            return ProjectCreateResponse {
                success: false,
                new_project_path: PathBuf::default(),
            };
        }

        ProjectCreateResponse {
            success: true,
            new_project_path: project_directory_path,
        }
    }
}
