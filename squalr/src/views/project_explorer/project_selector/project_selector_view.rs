use crate::{
    app_context::AppContext,
    views::project_explorer::project_selector::{
        project_entry_view::ProjectEntryView,
        project_selector_toolbar_view::ProjectSelectorToolbarView,
        view_data::{project_selector_frame_action::ProjectSelectorFrameAction, project_selector_view_data::ProjectSelectorViewData},
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

        // Perform an initial refresh on boot to load the project list.
        ProjectSelectorViewData::refresh_project_list(project_selector_view_data.clone(), app_context.clone());

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
        let mut project_selector_frame_action = ProjectSelectorFrameAction::None;
        let response = user_interface
            .allocate_ui_with_layout(user_interface.available_size(), Layout::top_down(Align::Min), |user_interface| {
                let project_selector_view_data = match self.project_selector_view_data.read("Project selector view") {
                    Some(project_selector_view_data) => project_selector_view_data,
                    None => return,
                };

                user_interface.add(self.project_selector_toolbar_view);

                for project_entry in &project_selector_view_data.project_list {
                    user_interface.add(ProjectEntryView::new(
                        self.app_context.clone(),
                        project_entry,
                        None,
                        project_selector_view_data
                            .selected_project_path
                            .as_ref()
                            .map(|selected_project_path| selected_project_path == project_entry.get_project_file_path())
                            .unwrap_or(false),
                        &mut project_selector_frame_action,
                    ));
                }
            })
            .response;

        match project_selector_frame_action {
            ProjectSelectorFrameAction::None => {}
            ProjectSelectorFrameAction::SelectProject(project_path) => {
                ProjectSelectorViewData::select_project(self.project_selector_view_data.clone(), project_path);
            }
            ProjectSelectorFrameAction::OpenProject(project_path, project_name) => {
                ProjectSelectorViewData::open_project(self.app_context.clone(), project_path, project_name);
            }
        }

        response
    }
}
