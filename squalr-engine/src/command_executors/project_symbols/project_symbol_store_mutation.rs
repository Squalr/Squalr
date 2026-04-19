use crate::command_executors::project::project_symbol_sync::sync_project_symbol_catalog;
use squalr_engine_api::engine::engine_execution_context::EngineExecutionContext;
use squalr_engine_api::structures::projects::{project::Project, project_root_symbol::ProjectRootSymbol};
use squalr_engine_projects::project::serialization::serializable_project_file::SerializableProjectFile;
use std::path::PathBuf;
use std::sync::Arc;

pub fn build_unique_symbol_key(
    display_name: &str,
    existing_rooted_symbols: &[ProjectRootSymbol],
) -> String {
    let sanitized_component = sanitize_symbol_key_component(display_name);
    let base_symbol_key = format!("sym.{}", sanitized_component);
    let mut duplicate_sequence_number = 1_u64;
    let mut candidate_symbol_key = base_symbol_key.clone();

    while existing_rooted_symbols
        .iter()
        .any(|existing_rooted_symbol| existing_rooted_symbol.get_symbol_key() == candidate_symbol_key)
    {
        duplicate_sequence_number = duplicate_sequence_number.saturating_add(1);
        candidate_symbol_key = format!("{}.{}", base_symbol_key, duplicate_sequence_number);
    }

    candidate_symbol_key
}

pub fn sanitize_symbol_key_component(display_name: &str) -> String {
    let mut sanitized_component = String::with_capacity(display_name.len());
    let mut previous_character_was_separator = false;

    for display_name_character in display_name.chars() {
        let mapped_character = if display_name_character.is_ascii_alphanumeric() {
            display_name_character.to_ascii_lowercase()
        } else {
            '.'
        };

        if mapped_character == '.' {
            if previous_character_was_separator {
                continue;
            }

            previous_character_was_separator = true;
        } else {
            previous_character_was_separator = false;
        }

        sanitized_component.push(mapped_character);
    }

    let trimmed_component = sanitized_component.trim_matches('.');

    if trimmed_component.is_empty() {
        String::from("symbol")
    } else {
        trimmed_component.to_string()
    }
}

pub fn save_and_sync_project_symbol_catalog(
    engine_execution_context: &Arc<dyn EngineExecutionContext>,
    opened_project: &mut Project,
    project_directory_path: &PathBuf,
) -> bool {
    let updated_project_symbol_catalog = opened_project
        .get_project_info()
        .get_project_symbol_catalog()
        .clone();

    opened_project
        .get_project_info_mut()
        .set_has_unsaved_changes(true);

    if let Err(error) = opened_project.save_to_path(project_directory_path, false) {
        log::error!("Failed to save project after project symbol mutation: {}", error);
        return false;
    }

    if !sync_project_symbol_catalog(engine_execution_context, updated_project_symbol_catalog) {
        log::error!("Failed to sync project symbol catalog after project symbol mutation.");
        return false;
    }

    true
}
