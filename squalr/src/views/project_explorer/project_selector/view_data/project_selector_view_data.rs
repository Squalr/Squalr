use crate::app_context::AppContext;
use squalr_engine_api::{
    commands::{project::list::project_list_request::ProjectListRequest, unprivileged_command_request::UnprivilegedCommandRequest},
    dependency_injection::dependency::Dependency,
    structures::projects::project_info::ProjectInfo,
};
use std::sync::Arc;

#[derive(Clone)]
pub struct ProjectSelectorViewData {
    pub project_list: Vec<ProjectInfo>,
}

impl ProjectSelectorViewData {
    pub fn new() -> Self {
        Self { project_list: Vec::new() }
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
}
