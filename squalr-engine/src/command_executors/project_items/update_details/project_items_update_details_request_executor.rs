use crate::command_executors::unprivileged_request_executor::UnprivilegedCommandRequestExecutor;
use crate::services::projects::project_item_file_mutation::resolve_project_item_path;
use squalr_engine_api::commands::project_items::update_details::project_items_update_details_request::ProjectItemsUpdateDetailsRequest;
use squalr_engine_api::commands::project_items::update_details::project_items_update_details_response::ProjectItemsUpdateDetailsResponse;
use squalr_engine_api::engine::engine_execution_context::EngineExecutionContext;
use squalr_engine_api::structures::projects::project_items::details::ProjectItemDetailsEditApplier;
use squalr_engine_api::structures::projects::project_items::project_item_ref::ProjectItemRef;
use squalr_engine_projects::project::serialization::serializable_project_file::SerializableProjectFile;
use std::sync::Arc;

impl UnprivilegedCommandRequestExecutor for ProjectItemsUpdateDetailsRequest {
    type ResponseType = ProjectItemsUpdateDetailsResponse;

    fn execute(
        &self,
        engine_unprivileged_state: &Arc<dyn EngineExecutionContext>,
    ) -> <Self as UnprivilegedCommandRequestExecutor>::ResponseType {
        if self.project_item_paths.is_empty() {
            return ProjectItemsUpdateDetailsResponse {
                success: true,
                updated_project_item_count: 0,
                error: None,
            };
        }

        let (details_field_source, details_value) = match self.resolve_details_update() {
            Ok(details_update) => details_update,
            Err(error) => {
                log::warn!("{}", error);
                return ProjectItemsUpdateDetailsResponse {
                    success: false,
                    updated_project_item_count: 0,
                    error: Some(error),
                };
            }
        };
        let project_manager = engine_unprivileged_state.get_project_manager();
        let opened_project_lock = project_manager.get_opened_project();
        let mut opened_project_guard = match opened_project_lock.write() {
            Ok(opened_project_guard) => opened_project_guard,
            Err(error) => {
                let error = format!("Failed to acquire opened project lock for project item details update: {}.", error);
                log::error!("{}", error);

                return ProjectItemsUpdateDetailsResponse {
                    success: false,
                    updated_project_item_count: 0,
                    error: Some(error),
                };
            }
        };
        let Some(opened_project) = opened_project_guard.as_mut() else {
            let error = String::from("Cannot update project item details without an opened project.");
            log::warn!("{}", error);

            return ProjectItemsUpdateDetailsResponse {
                success: false,
                updated_project_item_count: 0,
                error: Some(error),
            };
        };
        let Some(project_directory_path) = opened_project.get_project_info().get_project_directory() else {
            let error = String::from("Failed to resolve opened project directory for project item details update.");
            log::error!("{}", error);

            return ProjectItemsUpdateDetailsResponse {
                success: false,
                updated_project_item_count: 0,
                error: Some(error),
            };
        };
        let mut updated_project_item_count = 0_u64;

        for project_item_path in &self.project_item_paths {
            let resolved_project_item_path = resolve_project_item_path(&project_directory_path, project_item_path);
            let project_item_ref = ProjectItemRef::new(resolved_project_item_path.clone());
            let Some(project_item) = opened_project.get_project_item_mut(&project_item_ref) else {
                log::warn!(
                    "Cannot update project item details, project item was not found: {:?}.",
                    resolved_project_item_path
                );
                continue;
            };

            match ProjectItemDetailsEditApplier::apply_update(project_item, &details_field_source, &details_value) {
                Ok(true) => {
                    project_item.set_has_unsaved_changes(true);
                    updated_project_item_count = updated_project_item_count.saturating_add(1);
                }
                Ok(false) => {}
                Err(error) => log::warn!("Failed to apply project item details update: {}", error),
            }
        }

        if updated_project_item_count == 0 {
            return ProjectItemsUpdateDetailsResponse {
                success: true,
                updated_project_item_count,
                error: None,
            };
        }

        opened_project
            .get_project_info_mut()
            .set_has_unsaved_changes(true);
        if let Err(error) = opened_project.save_to_path(&project_directory_path, false) {
            let error = format!("Failed to save project after project item details update: {}.", error);
            log::error!("{}", error);

            return ProjectItemsUpdateDetailsResponse {
                success: false,
                updated_project_item_count,
                error: Some(error),
            };
        }
        drop(opened_project_guard);

        project_manager.notify_project_items_changed();

        ProjectItemsUpdateDetailsResponse {
            success: true,
            updated_project_item_count,
            error: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ProjectItemsUpdateDetailsRequest;
    use crate::command_executors::project_symbols::test_support::{
        MockProjectSymbolsBindings, create_engine_unprivileged_state, create_project_with_symbol_catalog,
    };
    use crate::command_executors::unprivileged_request_executor::UnprivilegedCommandRequestExecutor;
    use squalr_engine_api::engine::engine_execution_context::EngineExecutionContext;
    use squalr_engine_api::structures::data_types::built_in_types::u64::data_type_u64::DataTypeU64;
    use squalr_engine_api::structures::details::{DetailsFieldSource, DetailsValue};
    use squalr_engine_api::structures::projects::project::Project;
    use squalr_engine_api::structures::projects::project_items::built_in_types::project_item_type_address::ProjectItemTypeAddress;
    use squalr_engine_api::structures::projects::project_items::project_item::ProjectItem;
    use squalr_engine_api::structures::projects::project_items::project_item_ref::ProjectItemRef;
    use squalr_engine_api::structures::projects::project_symbol_catalog::ProjectSymbolCatalog;
    use std::path::PathBuf;
    use std::sync::Arc;

    #[test]
    fn update_details_request_persists_project_item_property() {
        let temp_directory = tempfile::tempdir().expect("Expected a temporary directory.");
        let mut project = create_project_with_symbol_catalog(temp_directory.path(), ProjectSymbolCatalog::default());
        let project_item_relative_path = PathBuf::from(Project::PROJECT_DIR).join("health.json");
        let project_item_absolute_path = temp_directory.path().join(&project_item_relative_path);
        let project_item_ref = ProjectItemRef::new(project_item_absolute_path.clone());
        let project_item = ProjectItemTypeAddress::new_project_item("Health", 0x1234, "game.exe", "", DataTypeU64::get_value_from_primitive(0));

        project
            .get_project_items_mut()
            .insert(project_item_ref.clone(), project_item);

        let engine_unprivileged_state = create_engine_unprivileged_state(MockProjectSymbolsBindings::new());
        *engine_unprivileged_state
            .get_project_manager()
            .get_opened_project()
            .write()
            .expect("Expected opened project write lock in test.") = Some(project);

        let engine_execution_context: Arc<dyn EngineExecutionContext> = engine_unprivileged_state.clone();
        let project_items_update_details_response = ProjectItemsUpdateDetailsRequest::from_details_update(
            vec![project_item_relative_path],
            DetailsFieldSource::ProjectItemProperty {
                property_name: ProjectItem::PROPERTY_ICON_ID.to_string(),
            },
            DetailsValue::Text(String::from("u64")),
        )
        .execute(&engine_execution_context);

        assert!(project_items_update_details_response.success);
        assert_eq!(project_items_update_details_response.updated_project_item_count, 1);

        let opened_project_lock = engine_unprivileged_state
            .get_project_manager()
            .get_opened_project();
        let opened_project_guard = opened_project_lock
            .read()
            .expect("Expected opened project read lock in test.");
        let opened_project = opened_project_guard
            .as_ref()
            .expect("Expected opened project in test.");
        let updated_project_item = opened_project
            .get_project_items()
            .get(&project_item_ref)
            .expect("Expected updated project item.");

        assert_eq!(updated_project_item.get_field_icon_id(), "u64");
        assert!(updated_project_item.get_has_unsaved_changes());
    }
}
