use crate::command_executors::unprivileged_request_executor::UnprivilegedCommandRequestExecutor;
use squalr_engine_api::commands::project_items::activate::project_items_activate_request::ProjectItemsActivateRequest;
use squalr_engine_api::commands::project_items::activate::project_items_activate_response::ProjectItemsActivateResponse;
use squalr_engine_api::engine::engine_execution_context::EngineExecutionContext;
use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::Arc;

impl UnprivilegedCommandRequestExecutor for ProjectItemsActivateRequest {
    type ResponseType = ProjectItemsActivateResponse;

    fn execute(
        &self,
        engine_unprivileged_state: &Arc<dyn EngineExecutionContext>,
    ) -> <Self as UnprivilegedCommandRequestExecutor>::ResponseType {
        let project_manager = engine_unprivileged_state.get_project_manager();
        let opened_project_lock = project_manager.get_opened_project();
        let mut opened_project_guard = match opened_project_lock.write() {
            Ok(opened_project_guard) => opened_project_guard,
            Err(error) => {
                log::error!("Failed to acquire opened project lock for project-item activation: {}", error);

                return ProjectItemsActivateResponse {};
            }
        };
        let opened_project = match opened_project_guard.as_mut() {
            Some(opened_project) => opened_project,
            None => {
                log::warn!("Unable to activate project items because no project is opened.");

                return ProjectItemsActivateResponse {};
            }
        };
        let project_item_paths_for_activation = collect_project_item_paths_for_activation(
            opened_project
                .get_project_items()
                .keys()
                .map(|project_item_ref| project_item_ref.get_project_item_path())
                .collect::<Vec<_>>()
                .as_slice(),
            &self.project_item_paths,
        );
        let mut has_activation_changes = false;

        for (project_item_ref, project_item) in opened_project.get_project_items_mut().iter_mut() {
            if !project_item_paths_for_activation.contains(project_item_ref.get_project_item_path()) {
                continue;
            }

            if project_item.get_is_activated() != self.is_activated {
                project_item.toggle_activated();
                has_activation_changes = true;
            }
        }

        drop(opened_project_guard);

        if has_activation_changes {
            project_manager.notify_project_items_changed();
        }

        ProjectItemsActivateResponse {}
    }
}

fn collect_project_item_paths_for_activation(
    all_project_item_paths: &[&PathBuf],
    requested_project_item_paths: &[String],
) -> HashSet<PathBuf> {
    let requested_project_item_paths_set = requested_project_item_paths
        .iter()
        .map(PathBuf::from)
        .collect::<HashSet<PathBuf>>();
    let mut project_item_paths_for_activation = HashSet::new();

    for project_item_path in all_project_item_paths {
        if requested_project_item_paths_set
            .iter()
            .any(|requested_project_item_path| project_item_path.starts_with(requested_project_item_path))
        {
            project_item_paths_for_activation.insert((*project_item_path).clone());
        }
    }

    project_item_paths_for_activation
}

#[cfg(test)]
mod tests {
    use super::collect_project_item_paths_for_activation;
    use std::path::PathBuf;

    #[test]
    fn collect_project_item_paths_for_activation_includes_requested_path_and_descendants() {
        let project_item_root_path = PathBuf::from(r"C:\Project\Items");
        let project_item_folder_path = project_item_root_path.join("Players");
        let project_item_child_path = project_item_folder_path.join("Health.json");
        let project_item_other_path = project_item_root_path.join("Enemies").join("Health.json");
        let all_project_item_paths = vec![
            &project_item_root_path,
            &project_item_folder_path,
            &project_item_child_path,
            &project_item_other_path,
        ];
        let requested_project_item_paths = vec![project_item_folder_path.to_string_lossy().into_owned()];
        let collected_paths = collect_project_item_paths_for_activation(&all_project_item_paths, &requested_project_item_paths);

        assert!(collected_paths.contains(&project_item_folder_path));
        assert!(collected_paths.contains(&project_item_child_path));
        assert!(!collected_paths.contains(&project_item_other_path));
    }

    #[test]
    fn collect_project_item_paths_for_activation_is_empty_for_unknown_path() {
        let project_item_root_path = PathBuf::from(r"C:\Project\Items");
        let project_item_child_path = project_item_root_path.join("Health.json");
        let all_project_item_paths = vec![&project_item_root_path, &project_item_child_path];
        let requested_project_item_paths = vec![
            project_item_root_path
                .join("Unknown")
                .to_string_lossy()
                .into_owned(),
        ];
        let collected_paths = collect_project_item_paths_for_activation(&all_project_item_paths, &requested_project_item_paths);

        assert!(collected_paths.is_empty());
    }
}
