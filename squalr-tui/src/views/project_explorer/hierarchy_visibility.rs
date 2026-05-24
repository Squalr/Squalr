use crate::views::project_explorer::hierarchy_graph::is_directory_project_item;
use crate::views::project_explorer::pane_state::ProjectHierarchyEntry;
use squalr_engine_api::structures::projects::project::Project;
use squalr_engine_api::structures::projects::project_items::built_in_types::{
    project_item_type_address::ProjectItemTypeAddress, project_item_type_pointer::ProjectItemTypePointer,
};
use squalr_engine_api::structures::projects::project_items::project_item::ProjectItem;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

/// Builds currently visible hierarchy entries from the expanded-directory state.
pub fn build_visible_hierarchy_entries(
    is_hierarchy_expanded: bool,
    opened_project_item_map: &HashMap<PathBuf, ProjectItem>,
    child_paths_by_parent_path: &HashMap<PathBuf, Vec<PathBuf>>,
    root_project_item_paths: &[PathBuf],
    expanded_directory_paths: &HashSet<PathBuf>,
) -> Vec<ProjectHierarchyEntry> {
    if !is_hierarchy_expanded {
        return Vec::new();
    }

    let mut visible_entries = Vec::new();
    for root_project_item_path in root_project_item_paths {
        if should_hide_synthetic_project_root(
            root_project_item_paths,
            root_project_item_path,
            opened_project_item_map,
            child_paths_by_parent_path,
        ) {
            append_child_hierarchy_entries(
                root_project_item_path,
                opened_project_item_map,
                child_paths_by_parent_path,
                expanded_directory_paths,
                &mut visible_entries,
            );
            continue;
        }

        append_visible_hierarchy_entries(
            root_project_item_path,
            0,
            opened_project_item_map,
            child_paths_by_parent_path,
            expanded_directory_paths,
            &mut visible_entries,
        );
    }

    visible_entries
}

fn append_visible_hierarchy_entries(
    project_item_path: &Path,
    project_item_depth: usize,
    opened_project_item_map: &HashMap<PathBuf, ProjectItem>,
    child_paths_by_parent_path: &HashMap<PathBuf, Vec<PathBuf>>,
    expanded_directory_paths: &HashSet<PathBuf>,
    visible_entries: &mut Vec<ProjectHierarchyEntry>,
) {
    let Some(project_item) = opened_project_item_map.get(project_item_path) else {
        return;
    };

    let is_directory = is_directory_project_item(project_item);
    let is_expanded = is_directory && expanded_directory_paths.contains(project_item_path);
    let child_paths = child_paths_by_parent_path
        .get(project_item_path)
        .cloned()
        .unwrap_or_default();

    let mut display_name = project_item.get_field_name();
    if display_name.is_empty() {
        display_name = project_item_path
            .file_name()
            .and_then(|file_name| file_name.to_str())
            .unwrap_or_default()
            .to_string();
    }

    visible_entries.push(ProjectHierarchyEntry {
        project_item_path: project_item_path.to_path_buf(),
        display_name,
        preview_value: build_preview_value(project_item),
        depth: project_item_depth,
        is_directory,
        is_expanded,
        is_activated: project_item.get_is_activated(),
    });

    if !is_expanded {
        return;
    }

    for child_path in &child_paths {
        append_visible_hierarchy_entries(
            child_path,
            project_item_depth + 1,
            opened_project_item_map,
            child_paths_by_parent_path,
            expanded_directory_paths,
            visible_entries,
        );
    }
}

fn build_preview_value(project_item: &ProjectItem) -> String {
    let project_item_type_id = project_item.get_item_type().get_project_item_type_id();

    if project_item_type_id == ProjectItemTypeAddress::PROJECT_ITEM_TYPE_ID {
        let mut project_item = project_item.clone();
        let preview_value = ProjectItemTypeAddress::get_field_freeze_data_value_interpreter(&mut project_item);

        if preview_value.is_empty() { "??".to_string() } else { preview_value }
    } else if project_item_type_id == ProjectItemTypePointer::PROJECT_ITEM_TYPE_ID {
        let preview_value = ProjectItemTypePointer::get_field_freeze_data_value_interpreter(project_item);

        if preview_value.is_empty() { "??".to_string() } else { preview_value }
    } else {
        String::new()
    }
}

fn append_child_hierarchy_entries(
    parent_project_item_path: &Path,
    opened_project_item_map: &HashMap<PathBuf, ProjectItem>,
    child_paths_by_parent_path: &HashMap<PathBuf, Vec<PathBuf>>,
    expanded_directory_paths: &HashSet<PathBuf>,
    visible_entries: &mut Vec<ProjectHierarchyEntry>,
) {
    let child_paths = child_paths_by_parent_path
        .get(parent_project_item_path)
        .cloned()
        .unwrap_or_default();

    for child_path in &child_paths {
        append_visible_hierarchy_entries(
            child_path,
            0,
            opened_project_item_map,
            child_paths_by_parent_path,
            expanded_directory_paths,
            visible_entries,
        );
    }
}

fn should_hide_synthetic_project_root(
    root_project_item_paths: &[PathBuf],
    root_project_item_path: &Path,
    opened_project_item_map: &HashMap<PathBuf, ProjectItem>,
    child_paths_by_parent_path: &HashMap<PathBuf, Vec<PathBuf>>,
) -> bool {
    if root_project_item_paths.len() != 1 {
        return false;
    }

    let Some(root_project_item) = opened_project_item_map.get(root_project_item_path) else {
        return false;
    };

    if !is_directory_project_item(root_project_item) {
        return false;
    }

    if !root_project_item_path
        .file_name()
        .and_then(|file_name| file_name.to_str())
        .is_some_and(|file_name| file_name == Project::PROJECT_DIR)
    {
        return false;
    }

    child_paths_by_parent_path
        .get(root_project_item_path)
        .is_some_and(|child_paths| !child_paths.is_empty())
}

#[cfg(test)]
mod tests {
    use super::build_visible_hierarchy_entries;
    use squalr_engine_api::structures::data_types::built_in_types::u8::data_type_u8::DataTypeU8;
    use squalr_engine_api::structures::projects::project::Project;
    use squalr_engine_api::structures::projects::project_items::built_in_types::project_item_type_address::ProjectItemTypeAddress;
    use squalr_engine_api::structures::projects::project_items::built_in_types::project_item_type_directory::ProjectItemTypeDirectory;
    use squalr_engine_api::structures::projects::project_items::project_item::ProjectItem;
    use squalr_engine_api::structures::projects::project_items::project_item_ref::ProjectItemRef;
    use std::collections::{HashMap, HashSet};
    use std::path::PathBuf;

    #[test]
    fn synthetic_project_root_is_hidden_when_children_exist() {
        let project_root_path = PathBuf::from(format!("C:/Projects/Test/{}", Project::PROJECT_DIR));
        let child_path = project_root_path.join("Addresses");
        let mut opened_project_item_map = HashMap::new();
        opened_project_item_map.insert(project_root_path.clone(), create_directory_project_item(&project_root_path));
        opened_project_item_map.insert(child_path.clone(), create_directory_project_item(&child_path));

        let mut child_paths_by_parent_path = HashMap::new();
        child_paths_by_parent_path.insert(project_root_path.clone(), vec![child_path.clone()]);

        let visible_entries = build_visible_hierarchy_entries(
            true,
            &opened_project_item_map,
            &child_paths_by_parent_path,
            std::slice::from_ref(&project_root_path),
            &HashSet::new(),
        );

        assert_eq!(visible_entries.len(), 1);
        assert_eq!(visible_entries[0].project_item_path, child_path);
        assert_eq!(visible_entries[0].depth, 0);
    }

    #[test]
    fn multiple_root_entries_remain_visible() {
        let first_root_path = PathBuf::from("C:/Projects/Test/Addresses");
        let second_root_path = PathBuf::from("C:/Projects/Test/Pointers");
        let mut opened_project_item_map = HashMap::new();
        opened_project_item_map.insert(first_root_path.clone(), create_directory_project_item(&first_root_path));
        opened_project_item_map.insert(second_root_path.clone(), create_directory_project_item(&second_root_path));

        let visible_entries = build_visible_hierarchy_entries(
            true,
            &opened_project_item_map,
            &HashMap::new(),
            &[first_root_path.clone(), second_root_path.clone()],
            &HashSet::new(),
        );

        assert_eq!(visible_entries.len(), 2);
        assert_eq!(visible_entries[0].project_item_path, first_root_path);
        assert_eq!(visible_entries[1].project_item_path, second_root_path);
    }

    #[test]
    fn address_preview_value_is_captured_in_visible_entry() {
        let address_path = PathBuf::from("C:/Projects/Test/Addresses/Health.json");
        let mut address_project_item = ProjectItemTypeAddress::new_project_item("Health", 0x1234, "", "", DataTypeU8::get_value_from_primitive(0));
        ProjectItemTypeAddress::set_field_freeze_data_value_interpreter(&mut address_project_item, "255");
        let mut opened_project_item_map = HashMap::new();
        opened_project_item_map.insert(address_path.clone(), address_project_item);

        let visible_entries = build_visible_hierarchy_entries(
            true,
            &opened_project_item_map,
            &HashMap::new(),
            std::slice::from_ref(&address_path),
            &HashSet::new(),
        );

        assert_eq!(visible_entries[0].preview_value, "255");
    }

    fn create_directory_project_item(project_item_path: &std::path::Path) -> ProjectItem {
        let project_item_ref = ProjectItemRef::new(project_item_path.to_path_buf());
        ProjectItemTypeDirectory::new_project_item(&project_item_ref)
    }
}
