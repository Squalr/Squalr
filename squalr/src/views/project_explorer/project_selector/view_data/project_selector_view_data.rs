use crate::app_context::AppContext;
use squalr_engine_api::{
    commands::{
        project::{
            create::project_create_request::ProjectCreateRequest, list::project_list_request::ProjectListRequest,
            open::project_open_request::ProjectOpenRequest, rename::project_rename_request::ProjectRenameRequest,
        },
        unprivileged_command_request::UnprivilegedCommandRequest,
    },
    dependency_injection::dependency::Dependency,
    structures::projects::project_info::ProjectInfo,
};
use std::{
    path::PathBuf,
    sync::{Arc, RwLock},
};

#[derive(Clone)]
pub struct ProjectSelectorViewData {
    pub project_list: Vec<ProjectInfo>,
    pub selected_project_file_path: Option<PathBuf>,
    pub renaming_project_file_path: Option<PathBuf>,
    pub rename_project_text: Arc<RwLock<String>>,
}

impl ProjectSelectorViewData {
    pub fn new() -> Self {
        Self {
            project_list: Vec::new(),
            selected_project_file_path: None,
            renaming_project_file_path: None,
            rename_project_text: Arc::new(RwLock::new(String::new())),
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

            Self::cancel_renaming_project(project_selector_view_data.clone());
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

            Self::cancel_renaming_project(project_selector_view_data.clone());
            Self::refresh_project_list(project_selector_view_data, app_context_clone);
        });
    }

    pub fn select_project(
        project_selector_view_data: Dependency<ProjectSelectorViewData>,
        project_file_path: PathBuf,
    ) {
        let mut project_selector_view_data = match project_selector_view_data.write("Project selector view data select project") {
            Some(project_selector_view_data) => project_selector_view_data,
            None => return,
        };

        project_selector_view_data.selected_project_file_path = Some(project_file_path);
    }

    pub fn start_renaming_project(
        project_selector_view_data: Dependency<ProjectSelectorViewData>,
        project_file_path: PathBuf,
        project_name: String,
    ) {
        let mut project_selector_view_data = match project_selector_view_data.write("Project selector view data start renaming project") {
            Some(project_selector_view_data) => project_selector_view_data,
            None => return,
        };

        match project_selector_view_data.rename_project_text.write() {
            Ok(mut rename_project_text) => {
                *rename_project_text = project_name;
            }
            Err(error) => {
                log::error!("Failed to acquire project name text to initialize rename text: {}", error);
            }
        };

        project_selector_view_data.renaming_project_file_path = Some(project_file_path);
    }

    pub fn cancel_renaming_project(project_selector_view_data: Dependency<ProjectSelectorViewData>) {
        let mut project_selector_view_data = match project_selector_view_data.write("Project selector view data start renaming project") {
            Some(project_selector_view_data) => project_selector_view_data,
            None => return,
        };

        project_selector_view_data.renaming_project_file_path = None;
    }

    pub fn rename_project(
        project_selector_view_data: Dependency<ProjectSelectorViewData>,
        app_context: Arc<AppContext>,
        project_file_path: PathBuf,
        new_project_name: String,
    ) {
        if let Some(mut project_selector_view_data) = project_selector_view_data.write("Project selector view data start renaming project") {
            project_selector_view_data.renaming_project_file_path = None;
        }

        let project_open_request = ProjectRenameRequest {
            project_file_path,
            new_project_name: new_project_name.clone(),
        };

        project_open_request.send(&app_context.engine_unprivileged_state, move |project_rename_response| {
            if !project_rename_response.success {
                log::error!("Failed to rename project!");
            } else {
                log::info!("Renamed project to: {}", new_project_name)
            }
        });

        Self::refresh_project_list(project_selector_view_data, app_context);
    }

    pub fn open_project(
        project_selector_view_data: Dependency<ProjectSelectorViewData>,
        app_context: Arc<AppContext>,
        project_directory_path: PathBuf,
        project_name: String,
    ) {
        let project_open_request = ProjectOpenRequest {
            open_file_browser: false,
            project_directory_path: Some(project_directory_path),
            project_name: None,
        };

        project_open_request.send(&app_context.engine_unprivileged_state, move |project_list_response| {
            if !project_list_response.success {
                log::error!("Failed to open project!");
            } else {
                Self::cancel_renaming_project(project_selector_view_data);

                log::info!("Opened project: {}", project_name)
            }
        });
    }
}
