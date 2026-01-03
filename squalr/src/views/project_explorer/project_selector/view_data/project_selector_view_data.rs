use crate::app_context::AppContext;
use squalr_engine_api::{
    commands::{
        project::{
            create::project_create_request::ProjectCreateRequest, list::project_list_request::ProjectListRequest,
            open::project_open_request::ProjectOpenRequest,
        },
        unprivileged_command_request::UnprivilegedCommandRequest,
    },
    dependency_injection::dependency::Dependency,
    structures::projects::project_info::ProjectInfo,
};
use std::{path::PathBuf, sync::Arc};

#[derive(Clone)]
pub struct ProjectSelectorViewData {
    pub project_list: Vec<ProjectInfo>,
    pub selected_project_path: Option<PathBuf>,
}

impl ProjectSelectorViewData {
    pub fn new() -> Self {
        Self {
            project_list: Vec::new(),
            selected_project_path: None,
        }
    }

    pub fn refresh_project_list(
        project_selector_view_data: Dependency<ProjectSelectorViewData>,
        app_context: Arc<AppContext>,
    ) {
        let project_list_request = ProjectListRequest {};

        project_list_request.send(&app_context.engine_unprivileged_state, move |project_list_response| {
            let mut project_selector_view_data = match project_selector_view_data.write("Project selector view data refresh process list response") {
                Some(project_selector_view_data) => project_selector_view_data,
                None => return,
            };

            project_selector_view_data.project_list = project_list_response.projects_info;
        });
    }

    pub fn browse_for_project(
        project_selector_view_data: Dependency<ProjectSelectorViewData>,
        app_context: Arc<AppContext>,
    ) {
        let app_context_clone = app_context.clone();
        let project_create_request = ProjectOpenRequest {
            open_file_browser: true,
            project_directory_path: None,
            project_name: None,
        };

        project_create_request.send(&app_context.engine_unprivileged_state, move |project_create_response| {
            if !project_create_response.success {
                log::error!("Failed to create new project!")
            }

            Self::refresh_project_list(project_selector_view_data, app_context_clone);
        });
    }

    pub fn create_new_project(
        project_selector_view_data: Dependency<ProjectSelectorViewData>,
        app_context: Arc<AppContext>,
    ) {
        let app_context_clone = app_context.clone();
        let project_create_request = ProjectCreateRequest {
            project_path: None,
            project_name: None,
        };

        project_create_request.send(&app_context.engine_unprivileged_state, move |project_create_response| {
            if !project_create_response.success {
                log::error!("Failed to create new project!")
            }

            Self::refresh_project_list(project_selector_view_data, app_context_clone);
        });
    }

    pub fn select_project(
        project_selector_view_data: Dependency<ProjectSelectorViewData>,
        project_path: PathBuf,
    ) {
        let mut project_selector_view_data = match project_selector_view_data.write("Project selector view data select project") {
            Some(project_selector_view_data) => project_selector_view_data,
            None => return,
        };

        project_selector_view_data.selected_project_path = Some(project_path);
    }

    pub fn open_project(
        app_context: Arc<AppContext>,
        project_path: PathBuf,
        project_name: String,
    ) {
        let project_open_request = ProjectOpenRequest {
            open_file_browser: false,
            project_directory_path: Some(project_path),
            project_name: None,
        };

        project_open_request.send(&app_context.engine_unprivileged_state, move |project_list_response| {
            if !project_list_response.success {
                log::error!("Failed to open project!");
            } else {
                log::info!("Opened project: {}", project_name)
            }
        });
    }
}
