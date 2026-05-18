use crate::app_context::AppContext;
use crate::views::project_explorer::project_hierarchy::{
    project_hierarchy_clipboard_controller::ProjectHierarchyClipboardController,
    project_hierarchy_drop_operation_planner::{ProjectHierarchyDropOperation, ProjectHierarchyDropOperationPlanner},
    project_item_create_request_builder::ProjectItemCreateRequestBuilder,
    project_item_rename_request_builder::ProjectItemRenameRequestBuilder,
    view_data::{
        project_hierarchy_clipboard::ProjectHierarchyClipboardMode, project_hierarchy_create_item_kind::ProjectHierarchyCreateItemKind,
        project_hierarchy_drop_target::ProjectHierarchyDropTarget, project_hierarchy_pending_operation::ProjectHierarchyPendingOperation,
        project_hierarchy_take_over_state::ProjectHierarchyTakeOverState, project_hierarchy_tree_model::ProjectHierarchyTreeModel,
        project_hierarchy_view_data::ProjectHierarchyViewData,
    },
};
use squalr_engine_api::commands::project_items::activate::project_items_activate_request::ProjectItemsActivateRequest;
use squalr_engine_api::commands::project_items::delete::project_items_delete_request::ProjectItemsDeleteRequest;
use squalr_engine_api::commands::project_items::duplicate::project_items_duplicate_request::ProjectItemsDuplicateRequest;
use squalr_engine_api::commands::project_items::move_item::project_items_move_request::ProjectItemsMoveRequest;
use squalr_engine_api::commands::project_items::promote_symbol::project_items_promote_symbol_request::ProjectItemsPromoteSymbolRequest;
use squalr_engine_api::commands::project_items::promote_symbol::project_items_promote_symbol_response::ProjectItemsPromoteSymbolResponse;
use squalr_engine_api::commands::project_items::reorder::project_items_reorder_request::ProjectItemsReorderRequest;
use squalr_engine_api::commands::project_items::strip_symbol::project_items_strip_symbol_request::ProjectItemsStripSymbolRequest;
use squalr_engine_api::commands::project_items::update_details::project_items_update_details_request::ProjectItemsUpdateDetailsRequest;
use squalr_engine_api::commands::project_items::write_value::project_items_write_value_request::ProjectItemsWriteValueRequest;
use squalr_engine_api::commands::unprivileged_command_request::UnprivilegedCommandRequest;
use squalr_engine_api::dependency_injection::dependency::Dependency;
use squalr_engine_api::engine::engine_execution_context::EngineExecutionContext;
use squalr_engine_api::structures::data_types::data_type_ref::DataTypeRef;
use squalr_engine_api::structures::data_values::anonymous_value_string::AnonymousValueString;
use squalr_engine_api::structures::details::{DetailsFieldSource, DetailsValue};
use squalr_engine_api::structures::projects::project_items::project_item_ref::ProjectItemRef;
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Clone)]
pub struct ProjectHierarchyCommandDispatcher {
    app_context: Arc<AppContext>,
    project_hierarchy_view_data: Dependency<ProjectHierarchyViewData>,
}

impl ProjectHierarchyCommandDispatcher {
    pub fn new(
        app_context: Arc<AppContext>,
        project_hierarchy_view_data: Dependency<ProjectHierarchyViewData>,
    ) -> Self {
        Self {
            app_context,
            project_hierarchy_view_data,
        }
    }

    pub fn strip_symbol_information(
        &self,
        project_item_paths: Vec<PathBuf>,
        details_refresh_callback: Option<Arc<dyn Fn() + Send + Sync>>,
    ) {
        ProjectItemsStripSymbolRequest { project_item_paths }.send(&self.app_context.engine_unprivileged_state, {
            let app_context = self.app_context.clone();
            let project_hierarchy_view_data = self.project_hierarchy_view_data.clone();

            move |project_items_strip_symbol_response| {
                if !project_items_strip_symbol_response.success {
                    log::warn!(
                        "Project item strip-symbol command failed: {}.",
                        project_items_strip_symbol_response
                            .error
                            .as_deref()
                            .unwrap_or("unknown error")
                    );
                    return;
                }

                if project_items_strip_symbol_response.stripped_project_item_count == 0 {
                    return;
                }

                ProjectHierarchyViewData::refresh_project_items(project_hierarchy_view_data, app_context);

                if let Some(details_refresh_callback) = details_refresh_callback {
                    details_refresh_callback();
                }
            }
        });
    }

    pub fn rename_project_item(
        &self,
        project_item_path: PathBuf,
        project_item_type_id: String,
        edited_name: String,
    ) {
        let Some(project_item_rename_request) = ProjectItemRenameRequestBuilder::build(&project_item_path, &project_item_type_id, edited_name.trim()) else {
            return;
        };
        let app_context = self.app_context.clone();
        let project_hierarchy_view_data = self.project_hierarchy_view_data.clone();
        let previous_project_item_path = project_item_path.clone();

        project_item_rename_request.send(&self.app_context.engine_unprivileged_state, move |project_items_rename_response| {
            if !project_items_rename_response.success {
                log::warn!("Project item rename command failed in hierarchy F2 rename flow.");
                return;
            }

            ProjectHierarchyViewData::finish_project_item_rename(
                project_hierarchy_view_data.clone(),
                &previous_project_item_path,
                &project_items_rename_response.renamed_project_item_path,
            );
            ProjectHierarchyViewData::refresh_project_items(project_hierarchy_view_data, app_context);
        });
    }

    pub fn delete_project_items(
        &self,
        project_item_paths: Vec<PathBuf>,
    ) {
        let filtered_project_item_paths = match self
            .project_hierarchy_view_data
            .write("Project hierarchy filter delete project items")
        {
            Some(mut project_hierarchy_view_data) => {
                let filtered_project_item_paths = project_hierarchy_view_data.filter_deletable_project_item_paths(project_item_paths);

                if filtered_project_item_paths.is_empty() {
                    project_hierarchy_view_data.take_over_state = ProjectHierarchyTakeOverState::None;
                    return;
                }

                project_hierarchy_view_data.pending_operation = ProjectHierarchyPendingOperation::Deleting;
                project_hierarchy_view_data.take_over_state = ProjectHierarchyTakeOverState::None;

                filtered_project_item_paths
            }
            None => return,
        };
        let app_context = self.app_context.clone();
        let project_hierarchy_view_data = self.project_hierarchy_view_data.clone();

        ProjectItemsDeleteRequest {
            project_item_paths: filtered_project_item_paths,
        }
        .send(&self.app_context.engine_unprivileged_state, move |project_items_delete_response| {
            if !project_items_delete_response.success {
                log::error!(
                    "Failed to delete one or more project items. Deleted count: {}.",
                    project_items_delete_response.deleted_project_item_count
                );
            }

            if let Some(mut project_hierarchy_view_data) = project_hierarchy_view_data.write("Project hierarchy delete project items response") {
                project_hierarchy_view_data.pending_operation = ProjectHierarchyPendingOperation::None;
            }

            ProjectHierarchyViewData::refresh_project_items(project_hierarchy_view_data, app_context);
        });
    }

    pub fn promote_project_items_to_symbols(
        &self,
        project_item_paths: Vec<PathBuf>,
        overwrite_conflicting_symbols: bool,
        after_successful_refresh_callback: Option<Arc<dyn Fn() + Send + Sync>>,
    ) {
        let filtered_project_item_paths = match self
            .project_hierarchy_view_data
            .write("Project hierarchy filter promote project items")
        {
            Some(mut project_hierarchy_view_data) => {
                let filtered_project_item_paths = project_hierarchy_view_data.filter_promotable_project_item_paths(project_item_paths);

                if filtered_project_item_paths.is_empty() {
                    return;
                }

                project_hierarchy_view_data.pending_operation = ProjectHierarchyPendingOperation::Promoting;

                filtered_project_item_paths
            }
            None => return,
        };
        let promote_conflict_project_item_paths = filtered_project_item_paths.clone();
        let app_context = self.app_context.clone();
        let project_hierarchy_view_data = self.project_hierarchy_view_data.clone();

        ProjectItemsPromoteSymbolRequest {
            project_item_paths: filtered_project_item_paths,
            overwrite_conflicting_symbols,
        }
        .send(&self.app_context.engine_unprivileged_state, move |project_items_promote_symbol_response| {
            if !project_items_promote_symbol_response.success {
                if project_items_promote_symbol_response.status_message.is_empty() {
                    log::error!(
                        "Failed to promote one or more project items to symbols. Promoted count before failure: {}.",
                        project_items_promote_symbol_response.promoted_symbol_count
                    );
                } else {
                    log::warn!("{}", project_items_promote_symbol_response.status_message);
                }
            }

            if let Some(mut project_hierarchy_view_data) = project_hierarchy_view_data.write("Project hierarchy promote project items response") {
                project_hierarchy_view_data.pending_operation = ProjectHierarchyPendingOperation::None;

                if !project_items_promote_symbol_response.conflicts.is_empty() {
                    project_hierarchy_view_data.take_over_state = ProjectHierarchyTakeOverState::PromoteSymbolConflict {
                        project_item_paths: promote_conflict_project_item_paths.clone(),
                        conflicts: project_items_promote_symbol_response.conflicts.clone(),
                    };
                } else {
                    project_hierarchy_view_data.take_over_state = ProjectHierarchyTakeOverState::None;
                }
            }

            let after_refresh_callback = if Self::should_refocus_details_after_promote_response(&project_items_promote_symbol_response) {
                after_successful_refresh_callback.clone()
            } else {
                None
            };

            ProjectHierarchyViewData::refresh_project_items_with_after_refresh(project_hierarchy_view_data, app_context, after_refresh_callback);
        });
    }

    pub fn create_project_item(
        &self,
        target_project_item_path: PathBuf,
        create_item_kind: ProjectHierarchyCreateItemKind,
    ) {
        let project_items_create_request = match self
            .project_hierarchy_view_data
            .write("Project hierarchy resolve create project item target")
        {
            Some(project_hierarchy_view_data) => {
                ProjectItemCreateRequestBuilder::build(&project_hierarchy_view_data.project_items, &target_project_item_path, create_item_kind)
            }
            None => return,
        };
        let app_context = self.app_context.clone();
        let project_hierarchy_view_data = self.project_hierarchy_view_data.clone();

        project_items_create_request.send(&self.app_context.engine_unprivileged_state, move |project_items_create_response| {
            if !project_items_create_response.success {
                log::error!("Failed to create project item.");
                return;
            }

            if let Some(mut project_hierarchy_view_data) = project_hierarchy_view_data.write("Project hierarchy select created project item") {
                project_hierarchy_view_data.select_created_project_item(&project_items_create_response.created_project_item_path);
            }

            ProjectHierarchyViewData::refresh_project_items(project_hierarchy_view_data, app_context);
        });
    }

    pub fn set_project_item_activation(
        &self,
        project_item_paths: Vec<PathBuf>,
        is_activated: bool,
    ) {
        if project_item_paths.is_empty() {
            return;
        }

        let app_context = self.app_context.clone();
        let project_hierarchy_view_data = self.project_hierarchy_view_data.clone();

        ProjectItemsActivateRequest {
            project_item_paths: project_item_paths
                .into_iter()
                .map(|project_item_path| project_item_path.to_string_lossy().into_owned())
                .collect(),
            is_activated,
        }
        .send(&self.app_context.engine_unprivileged_state, move |_project_items_activate_response| {
            ProjectHierarchyViewData::refresh_project_items(project_hierarchy_view_data, app_context);
        });
    }

    pub fn commit_reorder_drop(
        &self,
        drop_target: ProjectHierarchyDropTarget,
    ) {
        let drop_operation = {
            let mut project_hierarchy_view_data = match self
                .project_hierarchy_view_data
                .write("Project hierarchy commit reorder drop")
            {
                Some(project_hierarchy_view_data) => project_hierarchy_view_data,
                None => return,
            };
            let dragged_project_item_paths = match project_hierarchy_view_data.dragged_project_item_paths.clone() {
                Some(dragged_project_item_paths) if !dragged_project_item_paths.is_empty() => dragged_project_item_paths,
                _ => return,
            };

            if project_hierarchy_view_data.pending_operation != ProjectHierarchyPendingOperation::None {
                project_hierarchy_view_data.dragged_project_item_paths = None;
                return;
            }

            let drop_operation = ProjectHierarchyDropOperationPlanner::build(
                project_hierarchy_view_data.opened_project_info.as_ref(),
                &project_hierarchy_view_data.project_items,
                &dragged_project_item_paths,
                &drop_target,
            );

            match drop_operation {
                Some(drop_operation) => {
                    project_hierarchy_view_data.pending_operation = ProjectHierarchyPendingOperation::Reordering;
                    project_hierarchy_view_data.dragged_project_item_paths = None;
                    drop_operation
                }
                None => {
                    project_hierarchy_view_data.dragged_project_item_paths = None;
                    return;
                }
            }
        };

        self.dispatch_drop_operation(drop_operation);
    }

    pub fn paste_project_item_clipboard(
        &self,
        target_project_item_path: PathBuf,
    ) {
        enum PasteOperation {
            Copy {
                duplicate_request: ProjectItemsDuplicateRequest,
                insert_after_project_item_path: Option<PathBuf>,
            },
            CutMove {
                move_request: ProjectItemsMoveRequest,
                pasted_project_item_paths: Vec<PathBuf>,
            },
            CutMoveAndReorder {
                move_request: ProjectItemsMoveRequest,
                pasted_project_item_paths: Vec<PathBuf>,
                reordered_project_item_paths: Vec<PathBuf>,
            },
            CutReorder {
                pasted_project_item_paths: Vec<PathBuf>,
                reordered_project_item_paths: Vec<PathBuf>,
            },
        }

        let paste_operation = match self
            .project_hierarchy_view_data
            .write("Project hierarchy paste project item clipboard")
        {
            Some(mut project_hierarchy_view_data) => {
                if project_hierarchy_view_data.pending_operation != ProjectHierarchyPendingOperation::None {
                    return;
                }

                let paste_target =
                    ProjectHierarchyClipboardController::resolve_paste_target(&project_hierarchy_view_data.project_items, &target_project_item_path);
                let Some(current_project_file_path) = project_hierarchy_view_data
                    .opened_project_info
                    .as_ref()
                    .map(|opened_project_info| opened_project_info.get_project_file_path().clone())
                else {
                    return;
                };

                if project_hierarchy_view_data
                    .project_item_clipboard
                    .get_project_file_path()
                    != Some(&current_project_file_path)
                {
                    project_hierarchy_view_data.project_item_clipboard.clear();
                    return;
                }

                let clipboard_project_item_paths = project_hierarchy_view_data
                    .project_item_clipboard
                    .get_project_item_paths()
                    .to_vec();

                if clipboard_project_item_paths.is_empty() {
                    project_hierarchy_view_data.project_item_clipboard.clear();
                    return;
                }

                let clipboard_mode = project_hierarchy_view_data
                    .project_item_clipboard
                    .get_mode()
                    .cloned();
                let filtered_project_item_paths = ProjectHierarchyClipboardController::filter_pasteable_paths(
                    project_hierarchy_view_data.opened_project_info.as_ref(),
                    &project_hierarchy_view_data.project_items,
                    &clipboard_project_item_paths,
                    &paste_target,
                    clipboard_mode.as_ref(),
                );

                if filtered_project_item_paths.is_empty() {
                    if clipboard_mode == Some(ProjectHierarchyClipboardMode::Cut) {
                        project_hierarchy_view_data.project_item_clipboard.clear();
                    }
                    return;
                }

                project_hierarchy_view_data.pending_operation = ProjectHierarchyPendingOperation::Pasting;
                project_hierarchy_view_data.menu_target = None;
                project_hierarchy_view_data.menu_position = None;

                match clipboard_mode {
                    Some(ProjectHierarchyClipboardMode::Copy) => PasteOperation::Copy {
                        duplicate_request: ProjectItemsDuplicateRequest {
                            project_item_paths: filtered_project_item_paths,
                            target_directory_path: paste_target.target_directory_path,
                        },
                        insert_after_project_item_path: paste_target.insert_after_project_item_path,
                    },
                    Some(ProjectHierarchyClipboardMode::Cut) => {
                        let pasted_project_item_paths = filtered_project_item_paths
                            .iter()
                            .map(|project_item_path| {
                                if project_item_path.parent() == Some(paste_target.target_directory_path.as_path()) {
                                    project_item_path.clone()
                                } else {
                                    paste_target
                                        .target_directory_path
                                        .join(project_item_path.file_name().unwrap_or_default())
                                }
                            })
                            .collect::<Vec<_>>();
                        let project_item_paths_to_move = filtered_project_item_paths
                            .iter()
                            .filter(|project_item_path| project_item_path.parent() != Some(paste_target.target_directory_path.as_path()))
                            .cloned()
                            .collect::<Vec<_>>();

                        if let Some(insert_after_project_item_path) = paste_target.insert_after_project_item_path.clone() {
                            let reordered_project_item_paths = ProjectHierarchyTreeModel::build_reorder_paths_after_target(
                                project_hierarchy_view_data.opened_project_info.as_ref(),
                                &project_hierarchy_view_data.project_items,
                                &insert_after_project_item_path,
                                &pasted_project_item_paths,
                                &filtered_project_item_paths,
                            );

                            match (project_item_paths_to_move.is_empty(), reordered_project_item_paths) {
                                (true, Some(reordered_project_item_paths)) => PasteOperation::CutReorder {
                                    pasted_project_item_paths,
                                    reordered_project_item_paths,
                                },
                                (false, Some(reordered_project_item_paths)) => PasteOperation::CutMoveAndReorder {
                                    move_request: ProjectItemsMoveRequest {
                                        project_item_paths: project_item_paths_to_move,
                                        target_directory_path: paste_target.target_directory_path,
                                    },
                                    pasted_project_item_paths,
                                    reordered_project_item_paths,
                                },
                                (false, None) => PasteOperation::CutMove {
                                    move_request: ProjectItemsMoveRequest {
                                        project_item_paths: project_item_paths_to_move,
                                        target_directory_path: paste_target.target_directory_path,
                                    },
                                    pasted_project_item_paths,
                                },
                                (true, None) => {
                                    project_hierarchy_view_data.pending_operation = ProjectHierarchyPendingOperation::None;
                                    return;
                                }
                            }
                        } else {
                            PasteOperation::CutMove {
                                move_request: ProjectItemsMoveRequest {
                                    project_item_paths: project_item_paths_to_move,
                                    target_directory_path: paste_target.target_directory_path,
                                },
                                pasted_project_item_paths,
                            }
                        }
                    }
                    None => {
                        project_hierarchy_view_data.pending_operation = ProjectHierarchyPendingOperation::None;
                        return;
                    }
                }
            }
            None => return,
        };

        match paste_operation {
            PasteOperation::Copy {
                duplicate_request,
                insert_after_project_item_path,
            } => self.dispatch_copy_paste(duplicate_request, insert_after_project_item_path),
            PasteOperation::CutMove {
                move_request,
                pasted_project_item_paths,
            } => self.dispatch_cut_move_paste(move_request, pasted_project_item_paths),
            PasteOperation::CutMoveAndReorder {
                move_request,
                pasted_project_item_paths,
                reordered_project_item_paths,
            } => self.dispatch_cut_move_and_reorder_paste(move_request, pasted_project_item_paths, reordered_project_item_paths),
            PasteOperation::CutReorder {
                pasted_project_item_paths,
                reordered_project_item_paths,
            } => self.dispatch_cut_reorder_paste(pasted_project_item_paths, reordered_project_item_paths),
        }
    }

    pub fn commit_project_item_value_edit(
        &self,
        project_item_path: PathBuf,
        value_field_name: String,
        validation_data_type_ref: DataTypeRef,
        value_edit: AnonymousValueString,
    ) -> bool {
        if let Err(error) = self
            .app_context
            .engine_unprivileged_state
            .deanonymize_value_string(&validation_data_type_ref, &value_edit)
        {
            log::warn!("Failed to commit project hierarchy runtime value edit: {}", error);
            return false;
        }

        ProjectItemsWriteValueRequest {
            project_item_path,
            field_name: value_field_name,
            anonymous_value_string: value_edit,
        }
        .send(&self.app_context.engine_unprivileged_state, |project_items_write_value_response| {
            if !project_items_write_value_response.success {
                log::warn!("Project item write-value command failed while committing value edit takeover.");
            }
        });

        true
    }

    pub fn write_project_item_details_runtime_value(
        &self,
        project_item_paths: &[PathBuf],
        details_field_source: &DetailsFieldSource,
        anonymous_value_string: AnonymousValueString,
    ) -> bool {
        let Some(project_item_path) = project_item_paths.first().cloned() else {
            return false;
        };
        let DetailsFieldSource::ProjectItemRuntimeValue { field_path } = details_field_source else {
            return false;
        };
        let field_name = field_path
            .last()
            .cloned()
            .unwrap_or_else(|| String::from("value"));

        ProjectItemsWriteValueRequest {
            project_item_path,
            field_name,
            anonymous_value_string,
        }
        .send(&self.app_context.engine_unprivileged_state, |project_items_write_value_response| {
            if !project_items_write_value_response.success {
                log::warn!("Project item write-value command failed while committing details edit.");
            }
        });

        true
    }

    pub fn update_project_item_details_stored_field(
        &self,
        project_item_paths: &[PathBuf],
        details_field_source: &DetailsFieldSource,
        details_value: &DetailsValue,
    ) {
        ProjectItemsUpdateDetailsRequest::from_details_update(project_item_paths.to_vec(), details_field_source.clone(), details_value.clone()).send(
            &self.app_context.engine_unprivileged_state,
            |project_items_update_details_response| {
                if !project_items_update_details_response.success {
                    log::warn!("Project item update-details command failed while committing details edit.");
                }
            },
        );
    }

    pub fn rename_project_items_from_details(
        &self,
        project_item_paths: &[PathBuf],
        edited_name: &str,
    ) {
        let project_manager = self.app_context.engine_unprivileged_state.get_project_manager();
        let opened_project_lock = project_manager.get_opened_project();
        let opened_project_guard = match opened_project_lock.read() {
            Ok(opened_project_guard) => opened_project_guard,
            Err(error) => {
                log::error!("Failed to acquire opened project lock for project item details rename: {}", error);
                return;
            }
        };
        let Some(opened_project) = opened_project_guard.as_ref() else {
            log::warn!("Cannot rename project item from details without an opened project.");
            return;
        };
        let project_items_rename_requests = project_item_paths
            .iter()
            .filter_map(|project_item_path| {
                let project_item_ref = ProjectItemRef::new(project_item_path.clone());
                let Some(project_item) = opened_project.get_project_items().get(&project_item_ref) else {
                    log::warn!("Cannot rename project item from details, project item was not found: {:?}", project_item_path);
                    return None;
                };
                let project_item_type_id = project_item
                    .get_item_type()
                    .get_project_item_type_id()
                    .to_string();

                ProjectItemRenameRequestBuilder::build(project_item_path, &project_item_type_id, edited_name)
            })
            .collect::<Vec<_>>();
        drop(opened_project_guard);

        for project_items_rename_request in project_items_rename_requests {
            project_items_rename_request.send(&self.app_context.engine_unprivileged_state, |project_items_rename_response| {
                if !project_items_rename_response.success {
                    log::warn!("Project item rename command failed while committing details edit.");
                }
            });
        }
    }

    fn dispatch_drop_operation(
        &self,
        drop_operation: ProjectHierarchyDropOperation,
    ) {
        let app_context = self.app_context.clone();
        let project_hierarchy_view_data = self.project_hierarchy_view_data.clone();

        match drop_operation {
            ProjectHierarchyDropOperation::Reorder { project_item_paths } => {
                ProjectItemsReorderRequest { project_item_paths }.send(&self.app_context.engine_unprivileged_state, move |project_items_reorder_response| {
                    if !project_items_reorder_response.success {
                        log::error!(
                            "Failed to reorder project items. Reordered count: {}.",
                            project_items_reorder_response.reordered_project_item_count
                        );
                    }

                    if let Some(mut project_hierarchy_view_data) = project_hierarchy_view_data.write("Project hierarchy reorder project items response") {
                        project_hierarchy_view_data.pending_operation = ProjectHierarchyPendingOperation::None;
                    }

                    ProjectHierarchyViewData::refresh_project_items(project_hierarchy_view_data, app_context);
                });
            }
            ProjectHierarchyDropOperation::Move {
                project_item_paths,
                target_directory_path,
            } => {
                ProjectItemsMoveRequest {
                    project_item_paths,
                    target_directory_path,
                }
                .send(&self.app_context.engine_unprivileged_state, move |project_items_move_response| {
                    if !project_items_move_response.success {
                        log::error!(
                            "Failed to move project items. Moved count: {}.",
                            project_items_move_response.moved_project_item_count
                        );
                    }

                    if let Some(mut project_hierarchy_view_data) = project_hierarchy_view_data.write("Project hierarchy move project items response") {
                        project_hierarchy_view_data.pending_operation = ProjectHierarchyPendingOperation::None;
                    }

                    ProjectHierarchyViewData::refresh_project_items(project_hierarchy_view_data, app_context);
                });
            }
            ProjectHierarchyDropOperation::MoveAndReorder {
                project_item_paths,
                target_directory_path,
                reordered_project_item_paths,
            } => {
                ProjectItemsMoveRequest {
                    project_item_paths,
                    target_directory_path,
                }
                .send(&self.app_context.engine_unprivileged_state, move |project_items_move_response| {
                    if !project_items_move_response.success {
                        log::error!(
                            "Failed to move project items before reorder. Moved count: {}.",
                            project_items_move_response.moved_project_item_count
                        );

                        if let Some(mut project_hierarchy_view_data) = project_hierarchy_view_data.write("Project hierarchy move and reorder move response") {
                            project_hierarchy_view_data.pending_operation = ProjectHierarchyPendingOperation::None;
                        }

                        ProjectHierarchyViewData::refresh_project_items(project_hierarchy_view_data, app_context);
                        return;
                    }

                    let app_context_after_reorder = app_context.clone();
                    let project_hierarchy_view_data_after_reorder = project_hierarchy_view_data.clone();

                    ProjectItemsReorderRequest {
                        project_item_paths: reordered_project_item_paths.clone(),
                    }
                    .send(&app_context.engine_unprivileged_state, move |project_items_reorder_response| {
                        if !project_items_reorder_response.success {
                            log::error!(
                                "Failed to reorder project items after move. Reordered count: {}.",
                                project_items_reorder_response.reordered_project_item_count
                            );
                        }

                        if let Some(mut project_hierarchy_view_data) =
                            project_hierarchy_view_data_after_reorder.write("Project hierarchy move and reorder reorder response")
                        {
                            project_hierarchy_view_data.pending_operation = ProjectHierarchyPendingOperation::None;
                        }

                        ProjectHierarchyViewData::refresh_project_items(project_hierarchy_view_data_after_reorder, app_context_after_reorder);
                    });
                });
            }
        }
    }

    fn dispatch_copy_paste(
        &self,
        duplicate_request: ProjectItemsDuplicateRequest,
        insert_after_project_item_path: Option<PathBuf>,
    ) {
        let app_context = self.app_context.clone();
        let project_hierarchy_view_data = self.project_hierarchy_view_data.clone();
        let engine_unprivileged_state = self.app_context.engine_unprivileged_state.clone();

        duplicate_request.send(&engine_unprivileged_state, move |project_items_duplicate_response| {
            if !project_items_duplicate_response.success {
                log::error!(
                    "Failed to duplicate one or more project items. Duplicated count: {}.",
                    project_items_duplicate_response.duplicated_project_item_count
                );
            }

            let duplicated_project_item_paths = project_items_duplicate_response
                .duplicated_project_item_paths
                .clone();
            let reordered_project_item_paths = insert_after_project_item_path
                .as_ref()
                .and_then(|insert_after_project_item_path| {
                    project_hierarchy_view_data
                        .read("Project hierarchy duplicate project items reorder plan")
                        .and_then(|project_hierarchy_view_data| {
                            ProjectHierarchyTreeModel::build_reorder_paths_after_target(
                                project_hierarchy_view_data.opened_project_info.as_ref(),
                                &project_hierarchy_view_data.project_items,
                                insert_after_project_item_path,
                                &duplicated_project_item_paths,
                                &[],
                            )
                        })
                });

            if let Some(reordered_project_item_paths) = reordered_project_item_paths {
                let app_context_after_reorder = app_context.clone();
                let project_hierarchy_view_data_after_reorder = project_hierarchy_view_data.clone();
                let duplicated_project_item_paths_after_reorder = duplicated_project_item_paths.clone();
                let engine_unprivileged_state_after_reorder = app_context.engine_unprivileged_state.clone();

                ProjectItemsReorderRequest {
                    project_item_paths: reordered_project_item_paths,
                }
                .send(&engine_unprivileged_state_after_reorder, move |project_items_reorder_response| {
                    if !project_items_reorder_response.success {
                        log::error!(
                            "Failed to reorder duplicated project items. Reordered count: {}.",
                            project_items_reorder_response.reordered_project_item_count
                        );
                    }

                    if let Some(mut project_hierarchy_view_data) =
                        project_hierarchy_view_data_after_reorder.write("Project hierarchy duplicate project items reorder response")
                    {
                        project_hierarchy_view_data.pending_operation = ProjectHierarchyPendingOperation::None;
                        ProjectHierarchyViewData::apply_pasted_project_item_selection(
                            &mut project_hierarchy_view_data,
                            &duplicated_project_item_paths_after_reorder,
                        );
                    }

                    ProjectHierarchyViewData::refresh_project_items(project_hierarchy_view_data_after_reorder, app_context_after_reorder);
                });

                return;
            }

            if let Some(mut project_hierarchy_view_data) = project_hierarchy_view_data.write("Project hierarchy duplicate project items response") {
                project_hierarchy_view_data.pending_operation = ProjectHierarchyPendingOperation::None;
                ProjectHierarchyViewData::apply_pasted_project_item_selection(&mut project_hierarchy_view_data, &duplicated_project_item_paths);
            }

            ProjectHierarchyViewData::refresh_project_items(project_hierarchy_view_data, app_context);
        });
    }

    fn dispatch_cut_move_paste(
        &self,
        move_request: ProjectItemsMoveRequest,
        pasted_project_item_paths: Vec<PathBuf>,
    ) {
        let app_context = self.app_context.clone();
        let project_hierarchy_view_data = self.project_hierarchy_view_data.clone();
        let engine_unprivileged_state = self.app_context.engine_unprivileged_state.clone();

        move_request.send(&engine_unprivileged_state, move |project_items_move_response| {
            if !project_items_move_response.success {
                log::error!(
                    "Failed to paste cut project items. Moved count: {}.",
                    project_items_move_response.moved_project_item_count
                );
            }

            if let Some(mut project_hierarchy_view_data) = project_hierarchy_view_data.write("Project hierarchy move cut project items response") {
                project_hierarchy_view_data.pending_operation = ProjectHierarchyPendingOperation::None;

                if project_items_move_response.success {
                    project_hierarchy_view_data.project_item_clipboard.clear();
                    ProjectHierarchyViewData::apply_pasted_project_item_selection(&mut project_hierarchy_view_data, &pasted_project_item_paths);
                }
            }

            ProjectHierarchyViewData::refresh_project_items(project_hierarchy_view_data, app_context);
        });
    }

    fn dispatch_cut_move_and_reorder_paste(
        &self,
        move_request: ProjectItemsMoveRequest,
        pasted_project_item_paths: Vec<PathBuf>,
        reordered_project_item_paths: Vec<PathBuf>,
    ) {
        let app_context = self.app_context.clone();
        let project_hierarchy_view_data = self.project_hierarchy_view_data.clone();
        let engine_unprivileged_state = self.app_context.engine_unprivileged_state.clone();

        move_request.send(&engine_unprivileged_state, move |project_items_move_response| {
            if !project_items_move_response.success {
                log::error!(
                    "Failed to move cut project items before reorder. Moved count: {}.",
                    project_items_move_response.moved_project_item_count
                );

                if let Some(mut project_hierarchy_view_data) =
                    project_hierarchy_view_data.write("Project hierarchy move and reorder cut project items move response")
                {
                    project_hierarchy_view_data.pending_operation = ProjectHierarchyPendingOperation::None;
                }

                ProjectHierarchyViewData::refresh_project_items(project_hierarchy_view_data, app_context);
                return;
            }

            if let Some(mut project_hierarchy_view_data) =
                project_hierarchy_view_data.write("Project hierarchy move and reorder cut project items move success")
            {
                project_hierarchy_view_data.project_item_clipboard.clear();
            }

            let app_context_after_reorder = app_context.clone();
            let project_hierarchy_view_data_after_reorder = project_hierarchy_view_data.clone();
            let pasted_project_item_paths_after_reorder = pasted_project_item_paths.clone();
            let engine_unprivileged_state_after_reorder = app_context.engine_unprivileged_state.clone();

            ProjectItemsReorderRequest {
                project_item_paths: reordered_project_item_paths.clone(),
            }
            .send(&engine_unprivileged_state_after_reorder, move |project_items_reorder_response| {
                if !project_items_reorder_response.success {
                    log::error!(
                        "Failed to reorder cut project items after move. Reordered count: {}.",
                        project_items_reorder_response.reordered_project_item_count
                    );
                }

                if let Some(mut project_hierarchy_view_data) =
                    project_hierarchy_view_data_after_reorder.write("Project hierarchy move and reorder cut project items reorder response")
                {
                    project_hierarchy_view_data.pending_operation = ProjectHierarchyPendingOperation::None;
                    ProjectHierarchyViewData::apply_pasted_project_item_selection(&mut project_hierarchy_view_data, &pasted_project_item_paths_after_reorder);
                }

                ProjectHierarchyViewData::refresh_project_items(project_hierarchy_view_data_after_reorder, app_context_after_reorder);
            });
        });
    }

    fn dispatch_cut_reorder_paste(
        &self,
        pasted_project_item_paths: Vec<PathBuf>,
        reordered_project_item_paths: Vec<PathBuf>,
    ) {
        let app_context = self.app_context.clone();
        let project_hierarchy_view_data = self.project_hierarchy_view_data.clone();
        let engine_unprivileged_state = self.app_context.engine_unprivileged_state.clone();

        ProjectItemsReorderRequest {
            project_item_paths: reordered_project_item_paths,
        }
        .send(&engine_unprivileged_state, move |project_items_reorder_response| {
            if !project_items_reorder_response.success {
                log::error!(
                    "Failed to reorder cut project items. Reordered count: {}.",
                    project_items_reorder_response.reordered_project_item_count
                );
            }

            if let Some(mut project_hierarchy_view_data) = project_hierarchy_view_data.write("Project hierarchy reorder cut project items response") {
                project_hierarchy_view_data.pending_operation = ProjectHierarchyPendingOperation::None;

                if project_items_reorder_response.success {
                    project_hierarchy_view_data.project_item_clipboard.clear();
                    ProjectHierarchyViewData::apply_pasted_project_item_selection(&mut project_hierarchy_view_data, &pasted_project_item_paths);
                }
            }

            ProjectHierarchyViewData::refresh_project_items(project_hierarchy_view_data, app_context);
        });
    }

    fn should_refocus_details_after_promote_response(project_items_promote_symbol_response: &ProjectItemsPromoteSymbolResponse) -> bool {
        project_items_promote_symbol_response.success
            && project_items_promote_symbol_response.conflicts.is_empty()
            && project_items_promote_symbol_response
                .promoted_symbol_count
                .saturating_add(project_items_promote_symbol_response.reused_symbol_count)
                > 0
    }
}
