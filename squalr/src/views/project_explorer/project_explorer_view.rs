use crate::{
    app_context::AppContext,
    views::project_explorer::project_selector::{project_selector_view::ProjectSelectorView, view_data::project_selector_view_data::ProjectSelectorViewData},
};
use eframe::egui::{Align, Layout, Response, Ui, Widget};
use squalr_engine_api::dependency_injection::dependency::Dependency;
use std::sync::Arc;

#[derive(Clone)]
pub struct ProjectExplorerView {
    app_context: Arc<AppContext>,
    project_selector_view: ProjectSelectorView,
    _project_selector_view_data: Dependency<ProjectSelectorViewData>,
}

impl ProjectExplorerView {
    pub fn new(app_context: Arc<AppContext>) -> Self {
        let project_selector_view = ProjectSelectorView::new(app_context.clone());
        let project_selector_view_data = app_context
            .dependency_container
            .register(ProjectSelectorViewData::new());

        Self {
            app_context,
            project_selector_view,
            _project_selector_view_data: project_selector_view_data,
        }
    }
}

impl Widget for ProjectExplorerView {
    fn ui(
        self,
        user_interface: &mut Ui,
    ) -> Response {
        let response = user_interface
            .allocate_ui_with_layout(user_interface.available_size(), Layout::top_down(Align::Min), |user_interface| {
                let project_manager = self.app_context.engine_unprivileged_state.get_project_manager();
                let opened_project = project_manager.get_opened_project();
                let has_opened_project = match opened_project.read() {
                    Ok(opened_project) => opened_project.is_some(),
                    Err(error) => {
                        log::error!("Failed to acquire opened project lock: {}", error);
                        return;
                    }
                };

                if has_opened_project {
                    //
                } else {
                    user_interface.add(self.project_selector_view.clone());
                }
            })
            .response;

        response
    }
}
