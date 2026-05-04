use crate::app_context::AppContext;
use crate::views::plugins::view_data::plugin_list_view_data::PluginListViewData;
use squalr_engine_api::{
    commands::{
        project::{
            close::project_close_request::ProjectCloseRequest, create::project_create_request::ProjectCreateRequest,
            delete::project_delete_request::ProjectDeleteRequest, list::project_list_request::ProjectListRequest,
            open::project_open_request::ProjectOpenRequest, rename::project_rename_request::ProjectRenameRequest,
        },
        unprivileged_command_request::UnprivilegedCommandRequest,
    },
    dependency_injection::dependency::Dependency,
    structures::projects::{project::Project, project_info::ProjectInfo},
};
use std::{
    path::{Path, PathBuf},
    sync::{Arc, RwLock},
};

#[derive(Clone)]
pub struct ProjectSelectorViewData {
    pub project_list: Vec<ProjectInfo>,
    pub selected_project_file_path: Option<PathBuf>,
    pub editing_project_file_path: Option<PathBuf>,
    pub renaming_project_file_path: Option<PathBuf>,
    pub rename_project_text: Arc<RwLock<(String, bool)>>,
}

impl ProjectSelectorViewData {
    pub fn new() -> Self {
        Self {
            project_list: Vec::new(),
            selected_project_file_path: None,
            editing_project_file_path: None,
            renaming_project_file_path: None,
            rename_project_text: Arc::new(RwLock::new((String::new(), false))),
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
        let project_open_request = ProjectOpenRequest {
            open_file_browser: true,
            project_directory_path: None,
            project_name: None,
        };

        project_open_request.send(&app_context.engine_unprivileged_state, move |project_open_response| {
            if !project_open_response.success {
                log::error!("Failed to create new project!")
            }

            let plugin_list_view_data = app_context_clone
                .dependency_container
                .get_dependency::<PluginListViewData>();
            PluginListViewData::refresh(plugin_list_view_data, app_context_clone.clone());

            Self::cancel_renaming_project(project_selector_view_data.clone());
            Self::cancel_editing_project(project_selector_view_data.clone());
            Self::refresh_project_list(project_selector_view_data, app_context_clone);
        });
    }

    pub fn create_new_project(
        project_selector_view_data: Dependency<ProjectSelectorViewData>,
        app_context: Arc<AppContext>,
    ) {
        let app_context_clone = app_context.clone();
        let project_create_request = ProjectCreateRequest {
            project_directory_path: None,
            project_name: None,
        };

        project_create_request.send(&app_context.engine_unprivileged_state, move |project_create_response| {
            if !project_create_response.success {
                log::error!("Failed to create new project!")
            }

            let project_file_path = project_create_response
                .new_project_path
                .join(Project::PROJECT_FILE);
            let project_name = project_create_response
                .new_project_path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();

            Self::cancel_editing_project(project_selector_view_data.clone());
            Self::refresh_project_list(project_selector_view_data.clone(), app_context_clone);
            Self::select_project(project_selector_view_data.clone(), project_file_path.clone());
            Self::start_renaming_project(project_selector_view_data, project_file_path, project_name);
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

        project_selector_view_data.renaming_project_file_path = None;
        project_selector_view_data.editing_project_file_path = None;
        Self::clear_rename_project_text(&project_selector_view_data.rename_project_text);
        project_selector_view_data.selected_project_file_path = Some(project_file_path);
    }

    pub fn start_editing_project(
        project_selector_view_data: Dependency<ProjectSelectorViewData>,
        project_file_path: PathBuf,
        project_name: String,
    ) {
        let mut project_selector_view_data = match project_selector_view_data.write("Project selector view data start editing project") {
            Some(project_selector_view_data) => project_selector_view_data,
            None => return,
        };

        Self::set_rename_project_text(&project_selector_view_data.rename_project_text, project_name, true);

        project_selector_view_data.selected_project_file_path = Some(project_file_path.clone());
        project_selector_view_data.renaming_project_file_path = None;
        project_selector_view_data.editing_project_file_path = Some(project_file_path);
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

        Self::set_rename_project_text(&project_selector_view_data.rename_project_text, project_name, true);

        project_selector_view_data.selected_project_file_path = Some(project_file_path.clone());
        project_selector_view_data.editing_project_file_path = None;
        project_selector_view_data.renaming_project_file_path = Some(project_file_path);
    }

    pub fn cancel_editing_project(project_selector_view_data: Dependency<ProjectSelectorViewData>) {
        let mut project_selector_view_data = match project_selector_view_data.write("Project selector view data cancel editing project") {
            Some(project_selector_view_data) => project_selector_view_data,
            None => return,
        };

        project_selector_view_data.editing_project_file_path = None;
        Self::clear_rename_project_text(&project_selector_view_data.rename_project_text);
    }

    pub fn cancel_renaming_project(project_selector_view_data: Dependency<ProjectSelectorViewData>) {
        let mut project_selector_view_data = match project_selector_view_data.write("Project selector view data start renaming project") {
            Some(project_selector_view_data) => project_selector_view_data,
            None => return,
        };

        project_selector_view_data.renaming_project_file_path = None;
        Self::clear_rename_project_text(&project_selector_view_data.rename_project_text);
    }

    pub fn rename_project(
        project_selector_view_data: Dependency<ProjectSelectorViewData>,
        app_context: Arc<AppContext>,
        project_file_path: PathBuf,
        new_project_name: String,
    ) {
        let new_project_name = new_project_name.trim().to_string();

        if let Some(mut project_selector_view_data) = project_selector_view_data.write("Project selector view data start renaming project") {
            let current_project_name = project_selector_view_data
                .project_list
                .iter()
                .find(|project_info| project_info.get_project_file_path() == &project_file_path)
                .map(|project_info| project_info.get_name().to_string())
                .unwrap_or_else(|| {
                    project_file_path
                        .parent()
                        .and_then(|project_directory_path| project_directory_path.file_name())
                        .map(|project_directory_name| project_directory_name.to_string_lossy().to_string())
                        .unwrap_or_default()
                });

            if let Err(validation_error) = Self::validate_project_name(
                &project_selector_view_data.project_list,
                &project_file_path,
                &current_project_name,
                &new_project_name,
            ) {
                log::warn!("Ignoring project rename: {}", validation_error);

                return;
            }

            project_selector_view_data.renaming_project_file_path = None;
            project_selector_view_data.editing_project_file_path = None;
            Self::clear_rename_project_text(&project_selector_view_data.rename_project_text);
        }

        let project_directory_path = match project_file_path.parent() {
            Some(parent) => parent.to_path_buf(),
            None => {
                log::error!("Rename failed. Unable to get parent path for project path: {:?}", project_file_path);

                return;
            }
        };
        if project_directory_path
            .file_name()
            .map(|project_directory_name| {
                project_directory_name
                    .to_string_lossy()
                    .eq_ignore_ascii_case(&new_project_name)
            })
            .unwrap_or(false)
        {
            return;
        }

        let app_context_clone = app_context.clone();
        let project_rename_request = ProjectRenameRequest {
            project_directory_path,
            new_project_name: new_project_name.clone(),
        };

        project_rename_request.send(&app_context.engine_unprivileged_state, move |project_rename_response| {
            if !project_rename_response.success {
                log::error!("Failed to rename project!");
            } else {
                log::info!("Renamed project to: {}", new_project_name);

                Self::refresh_project_list(project_selector_view_data.clone(), app_context_clone);
                Self::select_project(
                    project_selector_view_data,
                    project_rename_response
                        .new_project_path
                        .join(Project::PROJECT_FILE),
                );
            }
        });
    }

    pub fn validate_project_name(
        project_list: &[ProjectInfo],
        current_project_file_path: &Path,
        current_project_name: &str,
        project_name: &str,
    ) -> Result<(), String> {
        let project_name = project_name.trim();

        if project_name.is_empty() {
            return Err(String::from("Project name is required."));
        }

        if project_name.chars().any(|project_name_character| {
            project_name_character.is_control() || matches!(project_name_character, '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*')
        }) {
            return Err(String::from("Project name contains invalid path characters."));
        }

        if project_name == "." || project_name == ".." {
            return Err(String::from("Project name cannot be a relative path segment."));
        }

        if project_name.eq_ignore_ascii_case(current_project_name) {
            return Ok(());
        }

        if project_list
            .iter()
            .any(|project_info| project_info.get_project_file_path() != current_project_file_path && project_info.get_name().eq_ignore_ascii_case(project_name))
        {
            return Err(String::from("A project with this name already exists."));
        }

        Ok(())
    }

    fn set_rename_project_text(
        rename_project_text: &Arc<RwLock<(String, bool)>>,
        project_name: String,
        should_highlight_text: bool,
    ) {
        match rename_project_text.write() {
            Ok(mut rename_project_text) => {
                *rename_project_text = (project_name, should_highlight_text);
            }
            Err(error) => {
                log::error!("Failed to acquire project name text to initialize rename text: {}", error);
            }
        }
    }

    fn clear_rename_project_text(rename_project_text: &Arc<RwLock<(String, bool)>>) {
        match rename_project_text.write() {
            Ok(mut rename_project_text) => {
                *rename_project_text = (String::new(), false);
            }
            Err(error) => {
                log::error!("Failed to clear project name text after rename state change: {}", error);
            }
        }
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

        let app_context_clone = app_context.clone();

        project_open_request.send(&app_context.engine_unprivileged_state, move |project_open_response| {
            if !project_open_response.success {
                log::error!("Failed to open project!");
            } else {
                log::info!("Opened project: {}", project_name);

                Self::cancel_renaming_project(project_selector_view_data.clone());
                Self::cancel_editing_project(project_selector_view_data);

                let plugin_list_view_data = app_context_clone
                    .dependency_container
                    .get_dependency::<PluginListViewData>();
                PluginListViewData::refresh(plugin_list_view_data, app_context_clone.clone());
            }
        });
    }

    pub fn close_current_project(app_context: Arc<AppContext>) {
        let project_close_request = ProjectCloseRequest {};

        project_close_request.send(&app_context.engine_unprivileged_state, move |project_list_response| {
            if !project_list_response.success {
                log::error!("Failed to close project!");
            } else {
                log::info!("Closed project.");
            }
        });
    }

    pub fn delete_project(
        project_selector_view_data: Dependency<ProjectSelectorViewData>,
        app_context: Arc<AppContext>,
        project_directory_path: PathBuf,
        project_name: String,
    ) {
        let app_context_clone = app_context.clone();
        let project_delete_request = ProjectDeleteRequest {
            project_directory_path: Some(project_directory_path),
            project_name: None,
        };

        project_delete_request.send(&app_context.engine_unprivileged_state, move |project_delete_response| {
            if !project_delete_response.success {
                log::error!("Failed to delete project!");
            } else {
                log::info!("Deleted project: {}", project_name);

                Self::cancel_renaming_project(project_selector_view_data.clone());
                Self::cancel_editing_project(project_selector_view_data.clone());
                Self::refresh_project_list(project_selector_view_data, app_context_clone);
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::ProjectSelectorViewData;
    use squalr_engine_api::structures::projects::{project::Project, project_info::ProjectInfo, project_manifest::ProjectManifest};
    use std::path::{Path, PathBuf};

    fn test_project_info(project_name: &str) -> ProjectInfo {
        ProjectInfo::new(
            PathBuf::from(format!("C:/Projects/{}/{}", project_name, Project::PROJECT_FILE)),
            None,
            ProjectManifest::new(Vec::new()),
        )
    }

    #[test]
    fn validate_project_name_rejects_empty_name() {
        let project_list = vec![test_project_info("Health")];

        assert!(ProjectSelectorViewData::validate_project_name(&project_list, Path::new("C:/Projects/Health/project.json"), "Health", "   ").is_err());
    }

    #[test]
    fn validate_project_name_rejects_colliding_project_name() {
        let project_list = vec![test_project_info("Health"), test_project_info("Ammo")];

        assert!(ProjectSelectorViewData::validate_project_name(&project_list, Path::new("C:/Projects/Health/project.json"), "Health", "ammo").is_err());
    }

    #[test]
    fn validate_project_name_allows_unchanged_project_name() {
        let project_list = vec![test_project_info("Health")];

        assert!(ProjectSelectorViewData::validate_project_name(&project_list, Path::new("C:/Projects/Health/project.json"), "Health", "health").is_ok());
    }

    #[test]
    fn validate_project_name_rejects_path_separator() {
        let project_list = vec![test_project_info("Health")];

        assert!(ProjectSelectorViewData::validate_project_name(&project_list, Path::new("C:/Projects/Health/project.json"), "Health", "Bad/Name").is_err());
    }
}
