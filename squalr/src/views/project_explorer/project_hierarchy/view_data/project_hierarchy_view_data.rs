use crate::app_context::AppContext;
use squalr_engine_api::{
    commands::{
        project::{close::project_close_request::ProjectCloseRequest, list::project_list_request::ProjectListRequest},
        unprivileged_command_request::UnprivilegedCommandRequest,
    },
    dependency_injection::dependency::Dependency,
};
use std::sync::Arc;

#[derive(Clone)]
pub struct ProjectHierarchyViewData {}

impl ProjectHierarchyViewData {
    pub fn new() -> Self {
        Self {}
    }

    pub fn close_current_project(
        project_hierarchy_view_data: Dependency<ProjectHierarchyViewData>,
        app_context: Arc<AppContext>,
    ) {
        let project_close_request = ProjectCloseRequest {};

        project_close_request.send(&app_context.engine_unprivileged_state, move |project_list_response| {
            if !project_list_response.success {
                log::error!("Failed to close project!");
            } else {
                let mut project_hierarchy_view_data = match project_hierarchy_view_data.write("Project hierarchy close project") {
                    Some(project_hierarchy_view_data) => project_hierarchy_view_data,
                    None => return,
                };

                // JIRA: Clear out cached front end data?
            }
        });
    }
}
