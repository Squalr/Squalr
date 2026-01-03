use crate::{
    app_context::AppContext,
    views::project_explorer::project_selector::{
        project_entry_view::ProjectEntryView, project_selector_toolbar_view::ProjectSelectorToolbarView,
        view_data::project_selector_view_data::ProjectSelectorViewData,
    },
};
use eframe::egui::{Align, Layout, Response, Ui, Widget};
use squalr_engine_api::dependency_injection::dependency::Dependency;
use std::sync::Arc;

#[derive(Clone)]
pub struct ProjectSelectorView {
    app_context: Arc<AppContext>,
    project_selector_toolbar_view: ProjectSelectorToolbarView,
    project_selector_view_data: Dependency<ProjectSelectorViewData>,
}

impl ProjectSelectorView {
    pub fn new(app_context: Arc<AppContext>) -> Self {
        let project_selector_view_data = app_context
            .dependency_container
            .get_dependency::<ProjectSelectorViewData>();
        let project_selector_toolbar_view = ProjectSelectorToolbarView::new(app_context.clone());

        Self {
            app_context,
            project_selector_toolbar_view,
            project_selector_view_data,
        }
    }
}

impl Widget for ProjectSelectorView {
    fn ui(
        self,
        user_interface: &mut Ui,
    ) -> Response {
        let response = user_interface
            .allocate_ui_with_layout(user_interface.available_size(), Layout::top_down(Align::Min), |user_interface| {
                let project_selector_view_data = match self.project_selector_view_data.read("Project selector view") {
                    Some(project_selector_view_data) => project_selector_view_data,
                    None => return,
                };

                user_interface.add(self.project_selector_toolbar_view);

                for project_entry in &project_selector_view_data.project_list {
                    user_interface.add(ProjectEntryView::new(self.app_context.clone(), project_entry.get_name(), None));
                }
            })
            .response;

        response
    }
}
