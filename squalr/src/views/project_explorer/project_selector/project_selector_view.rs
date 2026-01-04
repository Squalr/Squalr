use crate::{
    app_context::AppContext,
    views::project_explorer::project_selector::{
        project_entry_view::ProjectEntryView,
        project_selector_toolbar_view::ProjectSelectorToolbarView,
        view_data::{project_selector_frame_action::ProjectSelectorFrameAction, project_selector_view_data::ProjectSelectorViewData},
    },
};
use eframe::egui::{Align, Layout, Response, ScrollArea, Ui, Widget};
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
            .allocate_ui_with_layout(user_interface.available_size(), Layout::top_down(Align::Min), |mut user_interface| {
                let project_selector_view_data = match self.project_selector_view_data.read("Project selector view") {
                    Some(project_selector_view_data) => project_selector_view_data,
                    None => return,
                };
                let rename_project_text = project_selector_view_data.rename_project_text.clone();

                user_interface.add(self.project_selector_toolbar_view);

                ScrollArea::vertical()
                    .id_salt("project_selector")
                    .auto_shrink([false, false])
                    .show(&mut user_interface, |user_interface| {
                        for project_entry in &project_selector_view_data.project_list {
                            let is_renaming = project_selector_view_data
                                .renaming_project_file_path
                                .as_ref()
                                .map(|renaming_project_file_path| renaming_project_file_path == project_entry.get_project_file_path())
                                .unwrap_or(false);
                            let is_selected = project_selector_view_data
                                .selected_project_file_path
                                .as_ref()
                                .map(|selected_project_file_path| selected_project_file_path == project_entry.get_project_file_path())
                                .unwrap_or(false);
                            let is_context_menu_visible = project_selector_view_data
                                .context_menu_focus_file_path
                                .as_ref()
                                .map(|context_menu_focus_file_path| context_menu_focus_file_path == project_entry.get_project_file_path())
                                .unwrap_or(false);

                            user_interface.add(ProjectEntryView::new(
                                self.app_context.clone(),
                                project_entry,
                                None,
                                is_context_menu_visible,
                                is_selected,
                                is_renaming,
                                &rename_project_text,
                                &mut project_selector_frame_action,
                            ));
                        }
                    });
            })
            .response;

        match project_selector_frame_action {
            ProjectSelectorFrameAction::None => {}
            ProjectSelectorFrameAction::ShowContextMenu(project_file_path) => {
                ProjectSelectorViewData::show_context_menu(self.project_selector_view_data.clone(), project_file_path);
            }
            ProjectSelectorFrameAction::HideContextMenu() => {
                ProjectSelectorViewData::hide_context_menu(self.project_selector_view_data.clone());
            }
            ProjectSelectorFrameAction::SelectProject(project_file_path) => {
                ProjectSelectorViewData::select_project(self.project_selector_view_data.clone(), project_file_path);
            }
            ProjectSelectorFrameAction::StartRenamingProject(project_file_path, project_name) => {
                ProjectSelectorViewData::start_renaming_project(self.project_selector_view_data.clone(), project_file_path, project_name);
            }
            ProjectSelectorFrameAction::CancelRenamingProject() => {
                ProjectSelectorViewData::cancel_renaming_project(self.project_selector_view_data.clone());
            }
            ProjectSelectorFrameAction::CommitRename(project_file_path, new_project_name) => {
                ProjectSelectorViewData::rename_project(
                    self.project_selector_view_data.clone(),
                    self.app_context.clone(),
                    project_file_path,
                    new_project_name,
                );
            }
            ProjectSelectorFrameAction::OpenProject(project_directory_path, project_name) => {
                ProjectSelectorViewData::open_project(
                    self.project_selector_view_data.clone(),
                    self.app_context.clone(),
                    project_directory_path,
                    project_name,
                );
            }
            ProjectSelectorFrameAction::DeleteProject(project_directory_path, project_name) => {
                ProjectSelectorViewData::delete_project(
                    self.project_selector_view_data.clone(),
                    self.app_context.clone(),
                    project_directory_path,
                    project_name,
                );
            }
        }

        response
    }
}
