use squalr_engine_api::structures::projects::project::Project;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

pub fn normalize_project_item_sort_order(
    project: &mut Project,
    project_directory_path: &Path,
) -> bool {
    let existing_project_item_relative_paths = collect_existing_project_item_relative_paths(project, project_directory_path);
    let existing_project_item_relative_path_set: HashSet<PathBuf> = existing_project_item_relative_paths.iter().cloned().collect();
    let mut seen_project_item_relative_paths: HashSet<PathBuf> = HashSet::new();
    let mut normalized_project_item_sort_order: Vec<PathBuf> = Vec::new();

    for configured_project_item_relative_path in project
        .get_project_manifest()
        .get_project_item_sort_order()
        .iter()
        .cloned()
    {
        if !existing_project_item_relative_path_set.contains(&configured_project_item_relative_path) {
            continue;
        }

        if seen_project_item_relative_paths.insert(configured_project_item_relative_path.clone()) {
            normalized_project_item_sort_order.push(configured_project_item_relative_path);
        }
    }

    for existing_project_item_relative_path in existing_project_item_relative_paths {
        if seen_project_item_relative_paths.insert(existing_project_item_relative_path.clone()) {
            normalized_project_item_sort_order.push(existing_project_item_relative_path);
        }
    }

    set_project_item_sort_order_if_changed(project, normalized_project_item_sort_order)
}

pub fn append_project_items_to_sort_order(
    project: &mut Project,
    project_directory_path: &Path,
    added_project_item_paths: &[PathBuf],
) -> bool {
    let mut did_mutate_sort_order = normalize_project_item_sort_order(project, project_directory_path);
    let mut project_item_sort_order = project
        .get_project_manifest()
        .get_project_item_sort_order()
        .clone();
    let mut known_project_item_relative_paths: HashSet<PathBuf> = project_item_sort_order.iter().cloned().collect();

    for added_project_item_path in added_project_item_paths {
        let Some(added_project_item_relative_path) = to_manifest_relative_path(project_directory_path, added_project_item_path) else {
            continue;
        };

        if is_hidden_project_root_relative_path(&added_project_item_relative_path) {
            continue;
        }

        if known_project_item_relative_paths.insert(added_project_item_relative_path.clone()) {
            project_item_sort_order.push(added_project_item_relative_path);
            did_mutate_sort_order = true;
        }
    }

    if did_mutate_sort_order {
        did_mutate_sort_order = set_project_item_sort_order_if_changed(project, project_item_sort_order) || did_mutate_sort_order;
    }

    did_mutate_sort_order
}

pub fn remove_project_items_from_sort_order(
    project: &mut Project,
    project_directory_path: &Path,
    removed_project_item_paths: &[PathBuf],
) -> bool {
    let removed_project_item_relative_roots: Vec<PathBuf> = removed_project_item_paths
        .iter()
        .filter_map(|removed_project_item_path| to_manifest_relative_path(project_directory_path, removed_project_item_path))
        .filter(|removed_project_item_relative_path| !is_hidden_project_root_relative_path(removed_project_item_relative_path))
        .collect();

    if removed_project_item_relative_roots.is_empty() {
        return normalize_project_item_sort_order(project, project_directory_path);
    }

    let mut project_item_sort_order = project
        .get_project_manifest()
        .get_project_item_sort_order()
        .clone();

    project_item_sort_order.retain(|configured_project_item_relative_path| {
        !removed_project_item_relative_roots
            .iter()
            .any(|removed_project_item_relative_root| {
                configured_project_item_relative_path == removed_project_item_relative_root
                    || configured_project_item_relative_path.starts_with(removed_project_item_relative_root)
            })
    });

    let did_remove_entries = set_project_item_sort_order_if_changed(project, project_item_sort_order);
    normalize_project_item_sort_order(project, project_directory_path) || did_remove_entries
}

pub fn rename_project_item_in_sort_order(
    project: &mut Project,
    project_directory_path: &Path,
    source_project_item_path: &Path,
    target_project_item_path: &Path,
) -> bool {
    let Some(source_project_item_relative_path) = to_manifest_relative_path(project_directory_path, source_project_item_path) else {
        return normalize_project_item_sort_order(project, project_directory_path);
    };
    let Some(target_project_item_relative_path) = to_manifest_relative_path(project_directory_path, target_project_item_path) else {
        return normalize_project_item_sort_order(project, project_directory_path);
    };

    let mut project_item_sort_order = project
        .get_project_manifest()
        .get_project_item_sort_order()
        .clone();

    for configured_project_item_relative_path in &mut project_item_sort_order {
        if configured_project_item_relative_path == &source_project_item_relative_path
            || configured_project_item_relative_path.starts_with(&source_project_item_relative_path)
        {
            let source_relative_suffix = configured_project_item_relative_path
                .strip_prefix(&source_project_item_relative_path)
                .unwrap_or(Path::new(""));
            *configured_project_item_relative_path = target_project_item_relative_path.join(source_relative_suffix);
        }
    }

    let mut seen_project_item_relative_paths: HashSet<PathBuf> = HashSet::new();
    project_item_sort_order
        .retain(|configured_project_item_relative_path| seen_project_item_relative_paths.insert(configured_project_item_relative_path.clone()));

    let did_rename_entries = set_project_item_sort_order_if_changed(project, project_item_sort_order);
    normalize_project_item_sort_order(project, project_directory_path) || did_rename_entries
}

pub fn apply_reorder_subset_to_sort_order(
    project: &mut Project,
    project_directory_path: &Path,
    reordered_project_item_paths: &[PathBuf],
) -> bool {
    let mut did_mutate_sort_order = normalize_project_item_sort_order(project, project_directory_path);
    let current_project_item_sort_order = project
        .get_project_manifest()
        .get_project_item_sort_order()
        .clone();
    let existing_project_item_relative_path_set: HashSet<PathBuf> = current_project_item_sort_order.iter().cloned().collect();
    let mut seen_reordered_project_item_relative_paths: HashSet<PathBuf> = HashSet::new();
    let reordered_project_item_relative_paths: Vec<PathBuf> = reordered_project_item_paths
        .iter()
        .filter_map(|reordered_project_item_path| to_manifest_relative_path(project_directory_path, reordered_project_item_path))
        .filter(|reordered_project_item_relative_path| {
            !is_hidden_project_root_relative_path(reordered_project_item_relative_path)
                && existing_project_item_relative_path_set.contains(reordered_project_item_relative_path)
                && seen_reordered_project_item_relative_paths.insert(reordered_project_item_relative_path.clone())
        })
        .collect();

    if reordered_project_item_relative_paths.is_empty() {
        return did_mutate_sort_order;
    }

    let existing_project_item_sort_positions: HashMap<PathBuf, usize> = current_project_item_sort_order
        .iter()
        .cloned()
        .enumerate()
        .map(|(project_item_sort_position, project_item_relative_path)| (project_item_relative_path, project_item_sort_position))
        .collect();
    let insertion_project_item_sort_position = reordered_project_item_relative_paths
        .iter()
        .filter_map(|reordered_project_item_relative_path| {
            existing_project_item_sort_positions
                .get(reordered_project_item_relative_path)
                .copied()
        })
        .min()
        .unwrap_or(current_project_item_sort_order.len());
    let reordered_project_item_relative_path_set: HashSet<PathBuf> = reordered_project_item_relative_paths.iter().cloned().collect();
    let mut merged_project_item_sort_order: Vec<PathBuf> = current_project_item_sort_order
        .into_iter()
        .filter(|configured_project_item_relative_path| !reordered_project_item_relative_path_set.contains(configured_project_item_relative_path))
        .collect();
    let bounded_insertion_project_item_sort_position = insertion_project_item_sort_position.min(merged_project_item_sort_order.len());

    for (reordered_path_position, reordered_project_item_relative_path) in reordered_project_item_relative_paths.into_iter().enumerate() {
        merged_project_item_sort_order.insert(
            bounded_insertion_project_item_sort_position + reordered_path_position,
            reordered_project_item_relative_path,
        );
    }

    did_mutate_sort_order = set_project_item_sort_order_if_changed(project, merged_project_item_sort_order) || did_mutate_sort_order;
    normalize_project_item_sort_order(project, project_directory_path) || did_mutate_sort_order
}

fn collect_existing_project_item_relative_paths(
    project: &Project,
    project_directory_path: &Path,
) -> Vec<PathBuf> {
    let mut existing_project_item_relative_paths: Vec<PathBuf> = project
        .get_project_items()
        .keys()
        .filter_map(|project_item_ref| to_manifest_relative_path(project_directory_path, project_item_ref.get_project_item_path()))
        .filter(|project_item_relative_path| !is_hidden_project_root_relative_path(project_item_relative_path))
        .collect();

    existing_project_item_relative_paths.sort();
    existing_project_item_relative_paths
}

fn set_project_item_sort_order_if_changed(
    project: &mut Project,
    next_project_item_sort_order: Vec<PathBuf>,
) -> bool {
    if project.get_project_manifest().get_project_item_sort_order() == &next_project_item_sort_order {
        return false;
    }

    project
        .get_project_manifest_mut()
        .set_project_item_sort_order(next_project_item_sort_order);
    project.get_project_info_mut().set_has_unsaved_changes(true);
    true
}

fn to_manifest_relative_path(
    project_directory_path: &Path,
    project_item_path: &Path,
) -> Option<PathBuf> {
    let resolved_project_item_path = if project_item_path.is_absolute() {
        project_item_path.to_path_buf()
    } else {
        project_directory_path.join(project_item_path)
    };

    resolved_project_item_path
        .strip_prefix(project_directory_path)
        .ok()
        .map(Path::to_path_buf)
}

fn is_hidden_project_root_relative_path(project_item_relative_path: &Path) -> bool {
    project_item_relative_path == Path::new(Project::PROJECT_DIR)
}
