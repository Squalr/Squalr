use crate::{
    app_context::AppContext,
    views::project_explorer::{
        project_explorer_view::ProjectExplorerView,
        project_selector::{
            project_edit_take_over_view::ProjectEditTakeOverView,
            project_entry_view::ProjectEntryView,
            project_selector_toolbar_view::ProjectSelectorToolbarView,
            view_data::{project_selector_frame_action::ProjectSelectorFrameAction, project_selector_view_data::ProjectSelectorViewData},
        },
    },
};
use eframe::egui::{Align, Key, Layout, Response, ScrollArea, Ui, Widget};
use squalr_engine_api::dependency_injection::dependency::Dependency;
use std::sync::Arc;

#[derive(Clone)]
pub struct ProjectSelectorView {
    app_context: Arc<AppContext>,
    project_selector_toolbar_view: ProjectSelectorToolbarView,
    project_selector_view_data: Dependency<ProjectSelectorViewData>,
}

impl ProjectSelectorView {
    pub const WINDOW_ID: &'static str = "window_project_selector";

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
                let editing_project_file_path = project_selector_view_data.editing_project_file_path.clone();
                let renaming_project_file_path = project_selector_view_data.renaming_project_file_path.clone();
                let selected_project_for_rename = project_selector_view_data
                    .selected_project_file_path
                    .as_ref()
                    .and_then(|selected_project_file_path| {
                        project_selector_view_data
                            .project_list
                            .iter()
                            .find(|project_info| project_info.get_project_file_path() == selected_project_file_path)
                            .map(|project_info| (project_info.get_project_file_path().to_path_buf(), project_info.get_name().to_string()))
                    });

                user_interface.add(self.project_selector_toolbar_view);

                if let Some(editing_project_file_path) = editing_project_file_path.as_ref() {
                    if let Some(project_info) = project_selector_view_data
                        .project_list
                        .iter()
                        .find(|project_info| project_info.get_project_file_path() == editing_project_file_path)
                    {
                        let edit_take_over_response = ProjectEditTakeOverView::new(
                            self.app_context.clone(),
                            project_info,
                            &project_selector_view_data.project_list,
                            &rename_project_text,
                        )
                        .show(&mut user_interface);

                        if edit_take_over_response.should_cancel {
                            project_selector_frame_action = ProjectSelectorFrameAction::CancelEditingProject();
                        } else if edit_take_over_response.should_delete {
                            project_selector_frame_action = ProjectSelectorFrameAction::DeleteProject(
                                project_info.get_project_directory().unwrap_or_default(),
                                project_info.get_name().to_string(),
                            );
                        } else if let Some((project_file_path, project_name)) = edit_take_over_response.rename_submission {
                            project_selector_frame_action = ProjectSelectorFrameAction::CommitRename(project_file_path, project_name);
                        }
                    } else {
                        project_selector_frame_action = ProjectSelectorFrameAction::CancelEditingProject();
                    }
                } else {
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

                                user_interface.add(ProjectEntryView::new(
                                    self.app_context.clone(),
                                    project_entry,
                                    None,
                                    is_selected,
                                    is_renaming,
                                    &rename_project_text,
                                    &mut project_selector_frame_action,
                                ));
                            }
                        });
                }

                let is_window_focused = self
                    .app_context
                    .window_focus_manager
                    .is_window_focused(ProjectExplorerView::WINDOW_ID);
                let can_handle_window_shortcuts = self
                    .app_context
                    .window_focus_manager
                    .can_window_handle_shortcuts(user_interface.ctx(), ProjectExplorerView::WINDOW_ID);

                if editing_project_file_path.is_none()
                    && renaming_project_file_path.is_none()
                    && can_handle_window_shortcuts
                    && user_interface.input(|input_state| input_state.key_pressed(Key::F2))
                {
                    if let Some((project_file_path, project_name)) = selected_project_for_rename {
                        project_selector_frame_action = ProjectSelectorFrameAction::StartRenamingProject(project_file_path, project_name);
                    }
                }

                if renaming_project_file_path.is_some() && is_window_focused && user_interface.input(|input_state| input_state.key_pressed(Key::Escape)) {
                    project_selector_frame_action = ProjectSelectorFrameAction::CancelRenamingProject();
                }
            })
            .response;

        match project_selector_frame_action {
            ProjectSelectorFrameAction::None => {}
            ProjectSelectorFrameAction::SelectProject(project_file_path) => {
                ProjectSelectorViewData::select_project(self.project_selector_view_data.clone(), project_file_path);
            }
            ProjectSelectorFrameAction::StartEditingProject(project_file_path, project_name) => {
                ProjectSelectorViewData::start_editing_project(self.project_selector_view_data.clone(), project_file_path, project_name);
            }
            ProjectSelectorFrameAction::CancelEditingProject() => {
                ProjectSelectorViewData::cancel_editing_project(self.project_selector_view_data.clone());
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
