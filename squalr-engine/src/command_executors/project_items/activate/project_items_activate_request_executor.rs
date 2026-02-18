use crate::command_executors::unprivileged_request_executor::UnprivilegedCommandRequestExecutor;
use squalr_engine_api::commands::memory::freeze::memory_freeze_request::MemoryFreezeRequest;
use squalr_engine_api::commands::memory::freeze::memory_freeze_response::MemoryFreezeResponse;
use squalr_engine_api::commands::memory::freeze::memory_freeze_target::MemoryFreezeTarget;
use squalr_engine_api::commands::privileged_command_request::PrivilegedCommandRequest;
use squalr_engine_api::commands::privileged_command_response::TypedPrivilegedCommandResponse;
use squalr_engine_api::commands::project_items::activate::project_items_activate_request::ProjectItemsActivateRequest;
use squalr_engine_api::commands::project_items::activate::project_items_activate_response::ProjectItemsActivateResponse;
use squalr_engine_api::engine::engine_execution_context::EngineExecutionContext;
use squalr_engine_api::structures::projects::project_items::built_in_types::project_item_type_address::ProjectItemTypeAddress;
use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::mpsc;
use std::time::Duration;

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
        let mut freeze_targets = Vec::new();

        for (project_item_ref, project_item) in opened_project.get_project_items_mut().iter_mut() {
            if !project_item_paths_for_activation.contains(project_item_ref.get_project_item_path()) {
                continue;
            }

            if project_item.get_is_activated() != self.is_activated {
                project_item.toggle_activated();
                has_activation_changes = true;
                if let Some(freeze_target) = create_memory_freeze_target(project_item) {
                    freeze_targets.push(freeze_target);
                }
            }
        }

        drop(opened_project_guard);

        if has_activation_changes {
            dispatch_memory_freeze_request(engine_unprivileged_state, &freeze_targets, self.is_activated);
        }

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

fn create_memory_freeze_target(
    project_item: &mut squalr_engine_api::structures::projects::project_items::project_item::ProjectItem
) -> Option<MemoryFreezeTarget> {
    if project_item.get_item_type().get_project_item_type_id() != ProjectItemTypeAddress::PROJECT_ITEM_TYPE_ID {
        return None;
    }

    let address = ProjectItemTypeAddress::get_field_address(project_item);
    let module_name = ProjectItemTypeAddress::get_field_module(project_item);
    let data_type_id = ProjectItemTypeAddress::get_field_symbolic_struct_definition_reference(project_item)?
        .get_symbolic_struct_namespace()
        .to_string();

    if data_type_id.trim().is_empty() {
        return None;
    }

    Some(MemoryFreezeTarget {
        address,
        module_name,
        data_type_id,
    })
}

fn dispatch_memory_freeze_request(
    engine_unprivileged_state: &Arc<dyn EngineExecutionContext>,
    freeze_targets: &[MemoryFreezeTarget],
    is_frozen: bool,
) {
    if freeze_targets.is_empty() {
        return;
    }

    let memory_freeze_request = MemoryFreezeRequest {
        freeze_targets: freeze_targets.to_vec(),
        is_frozen,
    };
    let memory_freeze_command = memory_freeze_request.to_engine_command();
    let (freeze_response_sender, freeze_response_receiver) = mpsc::channel();

    let dispatch_result = match engine_unprivileged_state.get_bindings().read() {
        Ok(engine_bindings) => engine_bindings.dispatch_privileged_command(
            memory_freeze_command,
            Box::new(move |engine_response| {
                let conversion_result = match MemoryFreezeResponse::from_engine_response(engine_response) {
                    Ok(memory_freeze_response) => Ok(memory_freeze_response),
                    Err(unexpected_response) => Err(format!(
                        "Unexpected response variant for project-items activation freeze request: {:?}",
                        unexpected_response
                    )),
                };
                let _ = freeze_response_sender.send(conversion_result);
            }),
        ),
        Err(error) => {
            log::error!("Failed to acquire engine bindings lock for project-item activation freeze dispatch: {}", error);
            return;
        }
    };

    if let Err(error) = dispatch_result {
        log::error!("Failed to dispatch project-item activation freeze request: {}", error);
        return;
    }

    match freeze_response_receiver.recv_timeout(Duration::from_secs(5)) {
        Ok(Ok(memory_freeze_response)) => {
            if memory_freeze_response.failed_freeze_target_count > 0 {
                log::warn!(
                    "Project-item activation freeze request failed for {} targets.",
                    memory_freeze_response.failed_freeze_target_count
                );
            }
        }
        Ok(Err(error)) => {
            log::error!("Failed to convert project-item activation freeze response: {}", error);
        }
        Err(error) => {
            log::error!("Timed out waiting for project-item activation freeze response: {}", error);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{collect_project_item_paths_for_activation, create_memory_freeze_target};
    use squalr_engine_api::structures::data_types::built_in_types::u8::data_type_u8::DataTypeU8;
    use squalr_engine_api::structures::projects::project_items::built_in_types::{
        project_item_type_address::ProjectItemTypeAddress, project_item_type_directory::ProjectItemTypeDirectory,
    };
    use squalr_engine_api::structures::projects::project_items::project_item_ref::ProjectItemRef;
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

    #[test]
    fn create_memory_freeze_target_uses_address_project_item_values() {
        let mut address_project_item = ProjectItemTypeAddress::new_project_item("Health", 0x579C, "winmine.exe", "", DataTypeU8::get_value_from_primitive(0));

        let freeze_target = create_memory_freeze_target(&mut address_project_item).expect("Expected address project item to produce a freeze target.");

        assert_eq!(freeze_target.address, 0x579C);
        assert_eq!(freeze_target.module_name, "winmine.exe");
        assert_eq!(freeze_target.data_type_id, "u8");
    }

    #[test]
    fn create_memory_freeze_target_skips_non_address_project_items() {
        let directory_project_item_ref = ProjectItemRef::new(PathBuf::from(r"C:\Project\Items\Folder"));
        let mut directory_project_item = ProjectItemTypeDirectory::new_project_item(&directory_project_item_ref);

        let freeze_target = create_memory_freeze_target(&mut directory_project_item);

        assert!(freeze_target.is_none());
    }
}
