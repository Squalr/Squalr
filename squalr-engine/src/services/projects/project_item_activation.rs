use squalr_engine_api::commands::memory::freeze::memory_freeze_request::MemoryFreezeRequest;
use squalr_engine_api::commands::memory::freeze::memory_freeze_response::MemoryFreezeResponse;
use squalr_engine_api::commands::memory::freeze::memory_freeze_target::MemoryFreezeTarget;
use squalr_engine_api::commands::privileged_command_request::PrivilegedCommandRequest;
use squalr_engine_api::commands::privileged_command_response::TypedPrivilegedCommandResponse;
use squalr_engine_api::engine::engine_execution_context::EngineExecutionContext;
use squalr_engine_api::structures::projects::project_items::built_in_types::{
    project_item_type_address::ProjectItemTypeAddress, project_item_type_address_target::ProjectItemAddressTarget,
    project_item_type_pointer::ProjectItemTypePointer,
};
use squalr_engine_api::structures::projects::project_items::{project_item::ProjectItem, project_item_ref::ProjectItemRef};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::mpsc;
use std::time::Duration;

#[derive(Clone, Debug, Default)]
pub struct ProjectItemActivationChangeSet {
    pub has_activation_changes: bool,
    pub freeze_targets: Vec<MemoryFreezeTarget>,
}

pub fn apply_project_item_activation(
    project_items: &mut HashMap<ProjectItemRef, ProjectItem>,
    requested_project_item_paths: &[String],
    is_activated: bool,
) -> ProjectItemActivationChangeSet {
    let project_item_paths_for_activation = collect_project_item_paths_for_activation(
        project_items
            .keys()
            .map(|project_item_ref| project_item_ref.get_project_item_path())
            .collect::<Vec<_>>()
            .as_slice(),
        requested_project_item_paths,
    );
    let mut activation_change_set = ProjectItemActivationChangeSet::default();

    for (project_item_ref, project_item) in project_items.iter_mut() {
        if !project_item_paths_for_activation.contains(project_item_ref.get_project_item_path()) {
            continue;
        }

        if project_item.get_is_activated() != is_activated {
            project_item.toggle_activated();
            activation_change_set.has_activation_changes = true;
            if let Some(freeze_target) = create_memory_freeze_target(project_item) {
                activation_change_set.freeze_targets.push(freeze_target);
            }
        }
    }

    activation_change_set
}

pub fn collect_project_item_paths_for_activation(
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

pub fn create_memory_freeze_target(
    project_item: &mut squalr_engine_api::structures::projects::project_items::project_item::ProjectItem
) -> Option<MemoryFreezeTarget> {
    let project_item_type_id = project_item
        .get_item_type()
        .get_project_item_type_id()
        .to_string();

    if project_item_type_id == ProjectItemTypeAddress::PROJECT_ITEM_TYPE_ID {
        let address_target = ProjectItemTypeAddress::get_address_target(project_item);
        let data_type_id = ProjectItemTypeAddress::get_field_symbolic_struct_definition_reference(project_item)?
            .get_symbolic_struct_namespace()
            .to_string();

        if data_type_id.trim().is_empty() {
            return None;
        }

        return build_memory_freeze_target_from_address_target(&address_target, data_type_id);
    }

    if project_item_type_id == ProjectItemTypePointer::PROJECT_ITEM_TYPE_ID {
        let pointer = ProjectItemTypePointer::get_field_pointer(project_item);
        let data_type_id = ProjectItemTypePointer::get_field_symbolic_struct_definition_reference(project_item)?
            .get_symbolic_struct_namespace()
            .to_string();

        if data_type_id.trim().is_empty() {
            return None;
        }

        if pointer.has_symbolic_offsets() {
            return None;
        }

        return Some(MemoryFreezeTarget {
            address: pointer.get_address(),
            module_name: pointer.get_module_name().to_string(),
            data_type_id,
            pointer_offsets: pointer.get_offsets(),
            pointer_size: pointer.get_pointer_size(),
        });
    }

    None
}

fn build_memory_freeze_target_from_address_target(
    address_target: &ProjectItemAddressTarget,
    data_type_id: String,
) -> Option<MemoryFreezeTarget> {
    let runtime_pointer = address_target.to_runtime_pointer()?;

    if runtime_pointer.has_symbolic_offsets() {
        return None;
    }

    Some(MemoryFreezeTarget {
        address: runtime_pointer.get_address(),
        module_name: runtime_pointer.get_module_name().to_string(),
        data_type_id,
        pointer_offsets: runtime_pointer.get_offsets(),
        pointer_size: runtime_pointer.get_pointer_size(),
    })
}

pub fn dispatch_memory_freeze_request(
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
    use super::{apply_project_item_activation, collect_project_item_paths_for_activation, create_memory_freeze_target};
    use squalr_engine_api::structures::data_types::built_in_types::u8::data_type_u8::DataTypeU8;
    use squalr_engine_api::structures::memory::pointer::Pointer;
    use squalr_engine_api::structures::pointer_scans::pointer_scan_pointer_size::PointerScanPointerSize;
    use squalr_engine_api::structures::projects::project_items::built_in_types::{
        project_item_type_address::ProjectItemTypeAddress, project_item_type_directory::ProjectItemTypeDirectory,
        project_item_type_pointer::ProjectItemTypePointer,
    };
    use squalr_engine_api::structures::projects::project_items::project_item_ref::ProjectItemRef;
    use std::collections::HashMap;
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
    fn apply_project_item_activation_toggles_requested_descendants_and_collects_freeze_targets() {
        let project_item_root_path = PathBuf::from(r"C:\Project\Items");
        let requested_folder_path = project_item_root_path.join("Players");
        let child_item_path = requested_folder_path.join("Health.json");
        let sibling_item_path = project_item_root_path.join("Enemies").join("Health.json");
        let child_item_ref = ProjectItemRef::new(child_item_path.clone());
        let sibling_item_ref = ProjectItemRef::new(sibling_item_path.clone());
        let mut project_items = HashMap::from([
            (
                child_item_ref.clone(),
                ProjectItemTypeAddress::new_project_item("Health", 0x579C, "winmine.exe", "", DataTypeU8::get_value_from_primitive(0)),
            ),
            (
                sibling_item_ref.clone(),
                ProjectItemTypeAddress::new_project_item("Enemy Health", 0x6020, "winmine.exe", "", DataTypeU8::get_value_from_primitive(0)),
            ),
        ]);
        let requested_project_item_paths = vec![requested_folder_path.to_string_lossy().into_owned()];

        let activation_change_set = apply_project_item_activation(&mut project_items, &requested_project_item_paths, true);

        assert!(activation_change_set.has_activation_changes);
        assert_eq!(activation_change_set.freeze_targets.len(), 1);
        assert_eq!(activation_change_set.freeze_targets[0].address, 0x579C);
        assert!(
            project_items
                .get(&child_item_ref)
                .expect("Expected child item to remain in project item map.")
                .get_is_activated()
        );
        assert!(
            !project_items
                .get(&sibling_item_ref)
                .expect("Expected sibling item to remain in project item map.")
                .get_is_activated()
        );
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

    #[test]
    fn create_memory_freeze_target_uses_pointer_project_item_values() {
        let pointer = Pointer::new_with_size(0x44, vec![0x10, -0x8], "game.exe".to_string(), PointerScanPointerSize::Pointer64);
        let mut pointer_project_item = ProjectItemTypePointer::new_project_item("Ammo Pointer", &pointer, "", "u8");

        let freeze_target = create_memory_freeze_target(&mut pointer_project_item).expect("Expected pointer project item to produce a freeze target.");

        assert_eq!(freeze_target.address, 0x44);
        assert_eq!(freeze_target.module_name, "game.exe");
        assert_eq!(freeze_target.data_type_id, "u8");
        assert_eq!(freeze_target.pointer_offsets, vec![0x10, -0x8]);
        assert_eq!(freeze_target.pointer_size, PointerScanPointerSize::Pointer64);
    }
}
