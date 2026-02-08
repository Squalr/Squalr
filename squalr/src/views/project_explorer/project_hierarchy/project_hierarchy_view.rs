use crate::{
    app_context::AppContext,
    views::project_explorer::project_hierarchy::{
        project_hierarchy_toolbar_view::ProjectHierarchyToolbarView,
        view_data::{project_hierarchy_frame_action::ProjectHierarchyFrameAction, project_hierarchy_view_data::ProjectHierarchyViewData},
    },
};
use eframe::egui::{Align, Layout, Response, Ui, Widget};
use squalr_engine_api::dependency_injection::dependency::Dependency;
use std::sync::Arc;

#[derive(Clone)]
pub struct ProjectHierarchyView {
    app_context: Arc<AppContext>,
    project_hierarchy_toolbar_view: ProjectHierarchyToolbarView,
    project_hierarchy_view_data: Dependency<ProjectHierarchyViewData>,
}

impl ProjectHierarchyView {
    pub fn new(app_context: Arc<AppContext>) -> Self {
        let project_hierarchy_view_data = app_context
            .dependency_container
            .get_dependency::<ProjectHierarchyViewData>();
        let project_hierarchy_toolbar_view = ProjectHierarchyToolbarView::new(app_context.clone());

        Self {
            app_context,
            project_hierarchy_toolbar_view,
            project_hierarchy_view_data,
        }
    }
}

impl Widget for ProjectHierarchyView {
    fn ui(
        self,
        user_interface: &mut Ui,
    ) -> Response {
        let mut project_hierarchy_frame_action = ProjectHierarchyFrameAction::None;
        let response = user_interface
            .allocate_ui_with_layout(user_interface.available_size(), Layout::top_down(Align::Min), |user_interface| {
                let project_hierarchy_view_data = match self.project_hierarchy_view_data.read("Project hierarchy view") {
                    Some(project_hierarchy_view_data) => project_hierarchy_view_data,
                    None => return,
                };

                user_interface.add(self.project_hierarchy_toolbar_view);
            })
            .response;

        match project_hierarchy_frame_action {
            ProjectHierarchyFrameAction::None => {}
        }

        response
    }
}
