use squalr_engine_api::structures::projects::project::Project;
use squalr_engine_api::structures::projects::project_items::built_in_types::project_item_type_directory::ProjectItemTypeDirectory;
use squalr_engine_api::structures::projects::project_items::{project_item::ProjectItem, project_item_ref::ProjectItemRef};
use squalr_engine_api::utils::file_system::file_system_utils::FileSystemUtils;
use std::collections::HashMap;
use std::fs::{self, File};
use std::path::{Path, PathBuf};

pub fn resolve_project_item_path(
    project_directory_path: &Path,
    project_item_path: &Path,
) -> PathBuf {
    if FileSystemUtils::is_cross_platform_absolute_path(project_item_path) {
        project_item_path.to_path_buf()
    } else {
        project_directory_path.join(project_item_path)
    }
}

pub fn resolve_project_file_parent_directory_path(
    project_directory_path: &Path,
    requested_parent_directory_path: &Path,
) -> PathBuf {
    let project_root_directory_path = project_directory_path.join(Project::PROJECT_DIR);
    let resolved_parent_directory_path = resolve_project_item_path(project_directory_path, requested_parent_directory_path);

    if resolved_parent_directory_path.starts_with(&project_root_directory_path) {
        resolved_parent_directory_path
    } else {
        project_root_directory_path
    }
}

pub fn resolve_selected_directory_path(
    project_directory_path: &Path,
    project_root_directory_path: &Path,
    project_items: &HashMap<ProjectItemRef, ProjectItem>,
    target_directory_path: &Option<PathBuf>,
) -> PathBuf {
    let Some(target_directory_path) = target_directory_path else {
        return project_root_directory_path.to_path_buf();
    };
    let resolved_target_path = resolve_project_item_path(project_directory_path, target_directory_path);
    let resolved_directory_path = if is_directory_path(&resolved_target_path, project_items) {
        resolved_target_path
    } else {
        match resolved_target_path.parent() {
            Some(parent_path) => parent_path.to_path_buf(),
            None => project_root_directory_path.to_path_buf(),
        }
    };

    if resolved_directory_path.starts_with(project_root_directory_path) {
        resolved_directory_path
    } else {
        project_root_directory_path.to_path_buf()
    }
}

pub fn generate_unique_project_item_file_path(
    parent_directory_path: &Path,
    project_items: &HashMap<ProjectItemRef, ProjectItem>,
    project_item_file_stem: &str,
) -> PathBuf {
    let mut duplicate_sequence_number = 0_u64;

    loop {
        let project_item_file_name = if duplicate_sequence_number == 0 {
            format!("{}.{}", project_item_file_stem, Project::PROJECT_ITEM_EXTENSION.trim_start_matches('.'))
        } else {
            format!(
                "{}_{}.{}",
                project_item_file_stem,
                duplicate_sequence_number,
                Project::PROJECT_ITEM_EXTENSION.trim_start_matches('.')
            )
        };
        let project_item_absolute_path = parent_directory_path.join(project_item_file_name);
        let project_item_ref = ProjectItemRef::new(project_item_absolute_path.clone());

        if project_items.contains_key(&project_item_ref) {
            duplicate_sequence_number = duplicate_sequence_number.saturating_add(1);
            continue;
        }

        return project_item_absolute_path;
    }
}

pub fn create_placeholder_file(file_path: &Path) -> Result<(), String> {
    if let Some(parent_path) = file_path.parent() {
        fs::create_dir_all(parent_path).map_err(|error| format!("Failed creating project item parent directory {:?}: {}", parent_path, error))?;
    }

    if !file_path.exists() {
        File::create(file_path).map_err(|error| format!("Failed creating project item file {:?}: {}", file_path, error))?;
    }

    Ok(())
}

pub fn create_placeholder_files(file_paths: &[PathBuf]) -> Result<(), String> {
    for file_path in file_paths {
        create_placeholder_file(file_path)?;
    }

    Ok(())
}

pub fn sanitize_file_name_component(
    file_name_component: &str,
    fallback_name: &str,
) -> String {
    let mut sanitized_component = String::with_capacity(file_name_component.len());
    let mut previous_character_was_underscore = false;

    for name_character in file_name_component.chars() {
        let mapped_character = if name_character.is_ascii_alphanumeric() { name_character } else { '_' };

        if mapped_character == '_' {
            if previous_character_was_underscore {
                continue;
            }

            previous_character_was_underscore = true;
        } else {
            previous_character_was_underscore = false;
        }

        sanitized_component.push(mapped_character);
    }

    let trimmed_component = sanitized_component.trim_matches('_');

    if trimmed_component.is_empty() {
        fallback_name.to_string()
    } else {
        trimmed_component.to_string()
    }
}

fn is_directory_path(
    project_item_path: &Path,
    project_items: &HashMap<ProjectItemRef, ProjectItem>,
) -> bool {
    let project_item_ref = ProjectItemRef::new(project_item_path.to_path_buf());
    project_items
        .get(&project_item_ref)
        .map(|project_item| project_item.get_item_type().get_project_item_type_id() == ProjectItemTypeDirectory::PROJECT_ITEM_TYPE_ID)
        .unwrap_or(project_item_path.extension().is_none())
}

#[cfg(test)]
mod tests {
    use super::{generate_unique_project_item_file_path, sanitize_file_name_component};
    use squalr_engine_api::structures::projects::project_items::{
        built_in_types::project_item_type_directory::ProjectItemTypeDirectory, project_item_ref::ProjectItemRef,
    };
    use std::collections::HashMap;
    use std::path::PathBuf;

    #[test]
    fn sanitize_file_name_component_collapses_invalid_runs_and_uses_fallback() {
        assert_eq!(sanitize_file_name_component("game.exe+0x1234", "project_item"), "game_exe_0x1234");
        assert_eq!(sanitize_file_name_component("!@#$", "project_item"), "project_item");
    }

    #[test]
    fn generate_unique_project_item_file_path_uses_json_suffixes() {
        let parent_directory_path = PathBuf::from(r"C:\Project\.squalr");
        let existing_path = parent_directory_path.join("health.json");
        let existing_project_item_ref = ProjectItemRef::new(existing_path);
        let existing_project_item = ProjectItemTypeDirectory::new_project_item(&existing_project_item_ref);
        let project_items = HashMap::from([(existing_project_item_ref, existing_project_item)]);

        let generated_path = generate_unique_project_item_file_path(&parent_directory_path, &project_items, "health");

        assert_eq!(generated_path, parent_directory_path.join("health_1.json"));
    }
}
