use super::app_shell::AppShell;
use crate::views::project_explorer::pane_state::ProjectSelectorInputMode;
use anyhow::Result;
use squalr_engine::squalr_engine::SqualrEngine;
use squalr_engine_api::commands::memory::write::memory_write_request::MemoryWriteRequest;
use squalr_engine_api::commands::project::close::project_close_request::ProjectCloseRequest;
use squalr_engine_api::commands::project::create::project_create_request::ProjectCreateRequest;
use squalr_engine_api::commands::project::delete::project_delete_request::ProjectDeleteRequest;
use squalr_engine_api::commands::project::list::project_list_request::ProjectListRequest;
use squalr_engine_api::commands::project::open::project_open_request::ProjectOpenRequest;
use squalr_engine_api::commands::project::rename::project_rename_request::ProjectRenameRequest;
use squalr_engine_api::commands::project_items::activate::project_items_activate_request::ProjectItemsActivateRequest;
use squalr_engine_api::commands::project_items::create::project_items_create_request::ProjectItemsCreateRequest;
use squalr_engine_api::commands::project_items::delete::project_items_delete_request::ProjectItemsDeleteRequest;
use squalr_engine_api::commands::project_items::list::project_items_list_request::ProjectItemsListRequest;
use squalr_engine_api::commands::project_items::move_item::project_items_move_request::ProjectItemsMoveRequest;
use squalr_engine_api::commands::project_items::promote_symbol::project_items_promote_symbol_request::ProjectItemsPromoteSymbolRequest;
use squalr_engine_api::commands::project_items::rename::project_items_rename_request::ProjectItemsRenameRequest;
use squalr_engine_api::commands::project_items::reorder::project_items_reorder_request::ProjectItemsReorderRequest;
use squalr_engine_api::commands::project_symbols::delete::project_symbols_delete_request::ProjectSymbolsDeleteRequest;
use squalr_engine_api::commands::project_symbols::list::project_symbols_list_request::ProjectSymbolsListRequest;
use squalr_engine_api::commands::scan_results::set_property::scan_results_set_property_request::ScanResultsSetPropertyRequest;
use squalr_engine_api::commands::unprivileged_command_request::UnprivilegedCommandRequest;
use squalr_engine_api::engine::engine_execution_context::EngineExecutionContext;
use squalr_engine_api::structures::data_values::{anonymous_value_string::AnonymousValueString, container_type::ContainerType};
use squalr_engine_api::structures::projects::project::Project;
use squalr_engine_api::structures::projects::project_items::built_in_types::{
    project_item_type_address::ProjectItemTypeAddress, project_item_type_directory::ProjectItemTypeDirectory, project_item_type_pointer::ProjectItemTypePointer,
};
use squalr_engine_api::structures::projects::project_items::project_item::ProjectItem;
use squalr_engine_api::structures::projects::project_items::project_item_target::ProjectItemTarget;
use squalr_engine_api::structures::scan_results::scan_result_ref::ScanResultRef;
use squalr_engine_api::structures::structs::{
    symbolic_field_definition::SymbolicFieldDefinition, symbolic_struct_definition::SymbolicStructDefinition, valued_struct_field::ValuedStructField,
};
use squalr_engine_session::{
    engine_unprivileged_state::EngineUnprivilegedState,
    virtual_snapshots::{virtual_snapshot_query::VirtualSnapshotQuery, virtual_snapshot_query_result::VirtualSnapshotQueryResult},
};
use std::path::Path;
use std::str::FromStr;
use std::sync::{Arc, mpsc};
use std::time::Duration;

impl AppShell {
    const PROJECT_ITEM_PREVIEW_VIRTUAL_SNAPSHOT_ID: &'static str = "tui_project_item_previews";
    const MAX_PROJECT_ITEM_PREVIEW_ELEMENT_COUNT: u64 = 4;
    const MAX_PROJECT_ITEM_PREVIEW_ARRAY_CHARACTER_COUNT: usize = 96;
    const MAX_PROJECT_ITEM_PREVIEW_DISPLAY_ELEMENT_COUNT: usize = 4;

    pub(super) fn commit_project_selector_input(
        &mut self,
        squalr_engine: &mut SqualrEngine,
    ) {
        match self.app_state.project_explorer_pane_state.input_mode {
            ProjectSelectorInputMode::Search => self.app_state.project_explorer_pane_state.commit_search_input(),
            ProjectSelectorInputMode::CreatingProject => self.create_project_from_pending_name(squalr_engine),
            ProjectSelectorInputMode::RenamingProject => self.rename_selected_project_from_pending_name(squalr_engine),
            ProjectSelectorInputMode::CreatingProjectDirectory => self.create_project_directory_from_pending_name(squalr_engine),
            ProjectSelectorInputMode::None => {}
        }
    }

    pub(super) fn refresh_project_list(
        &mut self,
        squalr_engine: &mut SqualrEngine,
    ) {
        self.refresh_project_list_with_feedback(squalr_engine, true);
    }

    pub(super) fn refresh_project_list_with_feedback(
        &mut self,
        squalr_engine: &mut SqualrEngine,
        should_update_status_message: bool,
    ) {
        if self
            .app_state
            .project_explorer_pane_state
            .is_awaiting_project_list_response
        {
            if should_update_status_message {
                self.app_state.project_explorer_pane_state.status_message = "Project list request already in progress.".to_string();
            }
            return;
        }

        let engine_unprivileged_state = match squalr_engine.get_engine_unprivileged_state().as_ref() {
            Some(engine_unprivileged_state) => engine_unprivileged_state,
            None => {
                if should_update_status_message {
                    self.app_state.project_explorer_pane_state.status_message = "No unprivileged engine state is available for project queries.".to_string();
                }
                return;
            }
        };

        self.app_state
            .project_explorer_pane_state
            .is_awaiting_project_list_response = true;
        if should_update_status_message {
            self.app_state.project_explorer_pane_state.status_message = "Refreshing project list.".to_string();
        }

        let project_list_request = ProjectListRequest {};
        let (response_sender, response_receiver) = mpsc::sync_channel(1);
        project_list_request.send(engine_unprivileged_state, move |project_list_response| {
            let _ = response_sender.send(project_list_response);
        });

        match response_receiver.recv_timeout(Duration::from_secs(3)) {
            Ok(project_list_response) => {
                let project_count = project_list_response.projects_info.len();
                self.app_state
                    .project_explorer_pane_state
                    .apply_project_list(project_list_response.projects_info);
                self.app_state
                    .project_explorer_pane_state
                    .has_loaded_project_list_once = true;
                if should_update_status_message {
                    self.app_state.project_explorer_pane_state.status_message = format!("Loaded {} projects.", project_count);
                }
            }
            Err(receive_error) => {
                if should_update_status_message {
                    self.app_state.project_explorer_pane_state.status_message = format!("Timed out waiting for project list response: {}", receive_error);
                }
            }
        }

        self.app_state
            .project_explorer_pane_state
            .is_awaiting_project_list_response = false;
    }

    pub(super) fn refresh_project_items_list(
        &mut self,
        squalr_engine: &mut SqualrEngine,
    ) {
        self.refresh_project_items_list_with_feedback(squalr_engine, true);
    }

    pub(super) fn refresh_project_items_list_with_feedback(
        &mut self,
        squalr_engine: &mut SqualrEngine,
        should_update_status_message: bool,
    ) {
        if self
            .app_state
            .project_explorer_pane_state
            .is_awaiting_project_item_list_response
        {
            if should_update_status_message {
                self.app_state.project_explorer_pane_state.status_message = "Project item list request already in progress.".to_string();
            }
            return;
        }

        let engine_unprivileged_state = match squalr_engine.get_engine_unprivileged_state().as_ref() {
            Some(engine_unprivileged_state) => engine_unprivileged_state,
            None => {
                if should_update_status_message {
                    self.app_state.project_explorer_pane_state.status_message =
                        "No unprivileged engine state is available for project item listing.".to_string();
                }
                return;
            }
        };

        self.app_state
            .project_explorer_pane_state
            .is_awaiting_project_item_list_response = true;
        if should_update_status_message {
            self.app_state.project_explorer_pane_state.status_message = "Refreshing project item hierarchy.".to_string();
        }

        let project_items_list_request = ProjectItemsListRequest {
            preview_project_item_paths: Some(Vec::new()),
        };
        let (response_sender, response_receiver) = mpsc::sync_channel(1);
        project_items_list_request.send(engine_unprivileged_state, move |project_items_list_response| {
            let _ = response_sender.send(project_items_list_response);
        });

        match response_receiver.recv_timeout(Duration::from_secs(3)) {
            Ok(project_items_list_response) => {
                let opened_project_info = project_items_list_response.opened_project_info;
                let project_item_count = project_items_list_response.opened_project_items.len();
                self.app_state
                    .project_explorer_pane_state
                    .apply_project_items_list(opened_project_info, project_items_list_response.opened_project_items);
                self.sync_project_item_virtual_snapshot(engine_unprivileged_state);
                if should_update_status_message {
                    self.app_state.project_explorer_pane_state.status_message = format!("Loaded {} project items.", project_item_count);
                }
            }
            Err(receive_error) => {
                if should_update_status_message {
                    self.app_state.project_explorer_pane_state.status_message = format!("Timed out waiting for project item list response: {}", receive_error);
                }
            }
        }

        self.app_state
            .project_explorer_pane_state
            .is_awaiting_project_item_list_response = false;
    }

    pub(super) fn refresh_project_symbols_list(
        &mut self,
        squalr_engine: &mut SqualrEngine,
    ) {
        self.refresh_project_symbols_list_with_feedback(squalr_engine, true);
    }

    pub(super) fn refresh_project_symbols_list_with_feedback(
        &mut self,
        squalr_engine: &mut SqualrEngine,
        should_update_status_message: bool,
    ) {
        if self
            .app_state
            .project_explorer_pane_state
            .is_awaiting_project_symbol_list_response
        {
            if should_update_status_message {
                self.app_state.project_explorer_pane_state.status_message = "Project symbol list request already in progress.".to_string();
            }
            return;
        }

        let engine_unprivileged_state = match squalr_engine.get_engine_unprivileged_state().as_ref() {
            Some(engine_unprivileged_state) => engine_unprivileged_state,
            None => {
                if should_update_status_message {
                    self.app_state.project_explorer_pane_state.status_message =
                        "No unprivileged engine state is available for project symbol listing.".to_string();
                }
                return;
            }
        };

        self.app_state
            .project_explorer_pane_state
            .is_awaiting_project_symbol_list_response = true;
        if should_update_status_message {
            self.app_state.project_explorer_pane_state.status_message = "Refreshing symbol claims.".to_string();
        }

        let project_symbols_list_request = ProjectSymbolsListRequest::default();
        let (response_sender, response_receiver) = mpsc::sync_channel(1);
        project_symbols_list_request.send(engine_unprivileged_state, move |project_symbols_list_response| {
            let _ = response_sender.send(project_symbols_list_response);
        });

        match response_receiver.recv_timeout(Duration::from_secs(3)) {
            Ok(project_symbols_list_response) => {
                let symbol_claim_count = project_symbols_list_response
                    .project_symbol_catalog
                    .as_ref()
                    .map(|project_symbol_catalog| project_symbol_catalog.get_symbol_claims().len())
                    .unwrap_or(0);
                self.app_state
                    .project_explorer_pane_state
                    .apply_project_symbols_list(project_symbols_list_response.project_symbol_catalog);
                if should_update_status_message {
                    self.app_state.project_explorer_pane_state.status_message = format!("Loaded {} symbol claims.", symbol_claim_count);
                }
            }
            Err(receive_error) => {
                if should_update_status_message {
                    self.app_state.project_explorer_pane_state.status_message =
                        format!("Timed out waiting for project symbol list response: {}", receive_error);
                }
            }
        }

        self.app_state
            .project_explorer_pane_state
            .is_awaiting_project_symbol_list_response = false;
    }

    pub(super) fn sync_project_item_virtual_snapshot(
        &mut self,
        engine_unprivileged_state: &Arc<EngineUnprivilegedState>,
    ) {
        let requested_preview_project_item_paths = self
            .app_state
            .project_explorer_pane_state
            .collect_requested_preview_project_item_paths();
        let virtual_snapshot_queries = requested_preview_project_item_paths
            .iter()
            .filter_map(|project_item_path| {
                let project_item = self
                    .app_state
                    .project_explorer_pane_state
                    .opened_project_item(project_item_path)?;

                Self::build_project_item_virtual_snapshot_query(project_item_path, project_item, engine_unprivileged_state)
            })
            .collect::<Vec<_>>();

        engine_unprivileged_state.set_virtual_snapshot_queries(
            Self::PROJECT_ITEM_PREVIEW_VIRTUAL_SNAPSHOT_ID,
            self.project_items_periodic_refresh_interval(),
            virtual_snapshot_queries,
        );
        engine_unprivileged_state.request_virtual_snapshot_refresh(Self::PROJECT_ITEM_PREVIEW_VIRTUAL_SNAPSHOT_ID);
        self.apply_project_item_virtual_snapshot_results(engine_unprivileged_state, &requested_preview_project_item_paths);
    }

    fn apply_project_item_virtual_snapshot_results(
        &mut self,
        engine_unprivileged_state: &Arc<EngineUnprivilegedState>,
        requested_preview_project_item_paths: &[std::path::PathBuf],
    ) {
        let Some(virtual_snapshot) = engine_unprivileged_state.get_virtual_snapshot(Self::PROJECT_ITEM_PREVIEW_VIRTUAL_SNAPSHOT_ID) else {
            return;
        };

        for project_item_path in requested_preview_project_item_paths {
            let query_id = project_item_path.to_string_lossy().to_string();
            let Some(project_item) = self
                .app_state
                .project_explorer_pane_state
                .opened_project_item(project_item_path)
                .cloned()
            else {
                continue;
            };
            let Some(virtual_snapshot_query_result) = virtual_snapshot.get_query_results().get(query_id.as_str()) else {
                continue;
            };
            let preview_value =
                Self::build_project_item_preview_value_from_virtual_snapshot_result(engine_unprivileged_state, &project_item, virtual_snapshot_query_result);

            self.app_state
                .project_explorer_pane_state
                .apply_virtual_snapshot_preview(project_item_path, &preview_value, &virtual_snapshot_query_result.evaluated_pointer_path);
        }
    }

    pub(super) fn create_project_from_pending_name(
        &mut self,
        squalr_engine: &mut SqualrEngine,
    ) {
        if self.app_state.project_explorer_pane_state.is_creating_project {
            self.app_state.project_explorer_pane_state.status_message = "Project create request already in progress.".to_string();
            return;
        }

        let new_project_name = match self
            .app_state
            .project_explorer_pane_state
            .pending_project_name_trimmed()
        {
            Some(new_project_name) => new_project_name,
            None => {
                self.app_state.project_explorer_pane_state.status_message = "Project name is empty.".to_string();
                return;
            }
        };

        let engine_unprivileged_state = match squalr_engine.get_engine_unprivileged_state().as_ref() {
            Some(engine_unprivileged_state) => engine_unprivileged_state,
            None => {
                self.app_state.project_explorer_pane_state.status_message = "No unprivileged engine state is available for project creation.".to_string();
                return;
            }
        };

        self.app_state.project_explorer_pane_state.is_creating_project = true;
        self.app_state.project_explorer_pane_state.status_message = format!("Creating project '{}'.", new_project_name);

        let project_create_request = ProjectCreateRequest {
            project_directory_path: None,
            project_name: Some(new_project_name.clone()),
        };
        let (response_sender, response_receiver) = mpsc::sync_channel(1);
        project_create_request.send(engine_unprivileged_state, move |project_create_response| {
            let _ = response_sender.send(project_create_response);
        });

        match response_receiver.recv_timeout(Duration::from_secs(3)) {
            Ok(project_create_response) => {
                if project_create_response.success {
                    self.app_state
                        .project_explorer_pane_state
                        .cancel_project_name_input();
                    self.app_state.project_explorer_pane_state.status_message = format!(
                        "Created project '{}' at {}.",
                        new_project_name,
                        project_create_response.new_project_path.display()
                    );
                    self.refresh_project_list(squalr_engine);
                    let _ = self
                        .app_state
                        .project_explorer_pane_state
                        .select_project_by_directory_path(&project_create_response.new_project_path);
                } else {
                    self.app_state.project_explorer_pane_state.status_message = "Project create request failed.".to_string();
                }
            }
            Err(receive_error) => {
                self.app_state.project_explorer_pane_state.status_message = format!("Timed out waiting for project create response: {}", receive_error);
            }
        }

        self.app_state.project_explorer_pane_state.is_creating_project = false;
    }

    pub(super) fn create_project_directory_from_pending_name(
        &mut self,
        squalr_engine: &mut SqualrEngine,
    ) {
        if self
            .app_state
            .project_explorer_pane_state
            .is_creating_project_item
        {
            self.app_state.project_explorer_pane_state.status_message = "Project item create request already in progress.".to_string();
            return;
        }

        let parent_directory_path = match self
            .app_state
            .project_explorer_pane_state
            .selected_project_item_directory_target_path()
        {
            Some(parent_directory_path) => parent_directory_path,
            None => {
                self.app_state.project_explorer_pane_state.status_message = "No directory target is selected for project item create.".to_string();
                return;
            }
        };

        let project_item_name = match self
            .app_state
            .project_explorer_pane_state
            .pending_project_name_trimmed()
        {
            Some(project_item_name) => project_item_name,
            None => {
                self.app_state.project_explorer_pane_state.status_message = "Project item name is empty.".to_string();
                return;
            }
        };

        let engine_unprivileged_state = match squalr_engine.get_engine_unprivileged_state().as_ref() {
            Some(engine_unprivileged_state) => engine_unprivileged_state,
            None => {
                self.app_state.project_explorer_pane_state.status_message = "No unprivileged engine state is available for project item create.".to_string();
                return;
            }
        };

        self.app_state
            .project_explorer_pane_state
            .is_creating_project_item = true;
        self.app_state.project_explorer_pane_state.status_message =
            format!("Creating directory '{}' under {}.", project_item_name, parent_directory_path.display());

        let project_items_create_request = ProjectItemsCreateRequest {
            parent_directory_path,
            project_item_name: project_item_name.clone(),
            target: ProjectItemTarget::None,
            data_type_id: None,
        };
        let (response_sender, response_receiver) = mpsc::sync_channel(1);
        project_items_create_request.send(engine_unprivileged_state, move |project_items_create_response| {
            let _ = response_sender.send(project_items_create_response);
        });

        match response_receiver.recv_timeout(Duration::from_secs(3)) {
            Ok(project_items_create_response) => {
                if project_items_create_response.success {
                    self.app_state
                        .project_explorer_pane_state
                        .cancel_project_name_input();
                    self.app_state.project_explorer_pane_state.status_message = format!("Created project directory '{}'.", project_item_name);
                    self.refresh_project_items_list(squalr_engine);
                } else {
                    self.app_state.project_explorer_pane_state.status_message = "Project item create request failed.".to_string();
                }
            }
            Err(receive_error) => {
                self.app_state.project_explorer_pane_state.status_message = format!("Timed out waiting for project item create response: {}", receive_error);
            }
        }

        self.app_state
            .project_explorer_pane_state
            .is_creating_project_item = false;
    }

    pub(super) fn toggle_selected_project_item_activation(
        &mut self,
        squalr_engine: &mut SqualrEngine,
    ) {
        if self
            .app_state
            .project_explorer_pane_state
            .is_toggling_project_item_activation
        {
            self.app_state.project_explorer_pane_state.status_message = "Project item activation request already in progress.".to_string();
            return;
        }

        let selected_project_item_path = match self
            .app_state
            .project_explorer_pane_state
            .selected_project_item_path()
        {
            Some(selected_project_item_path) => selected_project_item_path,
            None => {
                self.app_state.project_explorer_pane_state.status_message = "No project item is selected for activation.".to_string();
                return;
            }
        };
        let is_target_activated = !self
            .app_state
            .project_explorer_pane_state
            .selected_project_item_is_activated();

        let engine_unprivileged_state = match squalr_engine.get_engine_unprivileged_state().as_ref() {
            Some(engine_unprivileged_state) => engine_unprivileged_state,
            None => {
                self.app_state.project_explorer_pane_state.status_message =
                    "No unprivileged engine state is available for project item activation.".to_string();
                return;
            }
        };

        self.app_state
            .project_explorer_pane_state
            .is_toggling_project_item_activation = true;
        self.app_state.project_explorer_pane_state.status_message =
            format!("Setting activation={} for {}.", is_target_activated, selected_project_item_path.display());

        let project_items_activate_request = ProjectItemsActivateRequest {
            project_item_paths: vec![selected_project_item_path.to_string_lossy().into_owned()],
            is_activated: is_target_activated,
        };
        let (response_sender, response_receiver) = mpsc::sync_channel(1);
        project_items_activate_request.send(engine_unprivileged_state, move |project_items_activate_response| {
            let _ = response_sender.send(project_items_activate_response);
        });

        match response_receiver.recv_timeout(Duration::from_secs(3)) {
            Ok(_) => {
                self.app_state.project_explorer_pane_state.status_message = "Updated selected project item activation.".to_string();
                self.refresh_project_items_list(squalr_engine);
            }
            Err(receive_error) => {
                self.app_state.project_explorer_pane_state.status_message =
                    format!("Timed out waiting for project item activation response: {}", receive_error);
            }
        }

        self.app_state
            .project_explorer_pane_state
            .is_toggling_project_item_activation = false;
    }

    pub(super) fn move_staged_project_items_to_selected_directory(
        &mut self,
        squalr_engine: &mut SqualrEngine,
    ) {
        if self
            .app_state
            .project_explorer_pane_state
            .is_moving_project_item
        {
            self.app_state.project_explorer_pane_state.status_message = "Project item move request already in progress.".to_string();
            return;
        }

        if !self
            .app_state
            .project_explorer_pane_state
            .has_pending_move_source_paths()
        {
            self.app_state.project_explorer_pane_state.status_message = "No staged project items to move.".to_string();
            return;
        }

        let target_directory_path = match self
            .app_state
            .project_explorer_pane_state
            .selected_project_item_directory_target_path()
        {
            Some(target_directory_path) => target_directory_path,
            None => {
                self.app_state.project_explorer_pane_state.status_message = "No target directory is selected for move.".to_string();
                return;
            }
        };

        let project_item_paths = self
            .app_state
            .project_explorer_pane_state
            .pending_move_source_paths();
        let engine_unprivileged_state = match squalr_engine.get_engine_unprivileged_state().as_ref() {
            Some(engine_unprivileged_state) => engine_unprivileged_state,
            None => {
                self.app_state.project_explorer_pane_state.status_message = "No unprivileged engine state is available for move.".to_string();
                return;
            }
        };

        self.app_state
            .project_explorer_pane_state
            .is_moving_project_item = true;
        self.app_state.project_explorer_pane_state.status_message =
            format!("Moving {} project items to {}.", project_item_paths.len(), target_directory_path.display());

        let project_items_move_request = ProjectItemsMoveRequest {
            project_item_paths,
            target_directory_path,
        };
        let (response_sender, response_receiver) = mpsc::sync_channel(1);
        project_items_move_request.send(engine_unprivileged_state, move |project_items_move_response| {
            let _ = response_sender.send(project_items_move_response);
        });

        match response_receiver.recv_timeout(Duration::from_secs(3)) {
            Ok(project_items_move_response) => {
                if project_items_move_response.success {
                    self.app_state
                        .project_explorer_pane_state
                        .clear_pending_move_source_paths();
                    self.app_state.project_explorer_pane_state.status_message =
                        format!("Moved {} project items.", project_items_move_response.moved_project_item_count);
                    self.refresh_project_items_list(squalr_engine);
                } else {
                    self.app_state.project_explorer_pane_state.status_message = "Project item move request failed.".to_string();
                }
            }
            Err(receive_error) => {
                self.app_state.project_explorer_pane_state.status_message = format!("Timed out waiting for project item move response: {}", receive_error);
            }
        }

        self.app_state
            .project_explorer_pane_state
            .is_moving_project_item = false;
    }

    pub(super) fn reorder_selected_project_item(
        &mut self,
        squalr_engine: &mut SqualrEngine,
        move_toward_previous_position: bool,
    ) {
        if self
            .app_state
            .project_explorer_pane_state
            .is_reordering_project_item
        {
            self.app_state.project_explorer_pane_state.status_message = "Project item reorder request already in progress.".to_string();
            return;
        }

        let project_item_paths = match self
            .app_state
            .project_explorer_pane_state
            .build_reorder_request_paths_for_selected_project_item(move_toward_previous_position)
        {
            Some(project_item_paths) => project_item_paths,
            None => {
                self.app_state.project_explorer_pane_state.status_message = "Selected project item cannot be reordered in that direction.".to_string();
                return;
            }
        };

        let engine_unprivileged_state = match squalr_engine.get_engine_unprivileged_state().as_ref() {
            Some(engine_unprivileged_state) => engine_unprivileged_state,
            None => {
                self.app_state.project_explorer_pane_state.status_message = "No unprivileged engine state is available for reorder.".to_string();
                return;
            }
        };

        self.app_state
            .project_explorer_pane_state
            .is_reordering_project_item = true;
        self.app_state.project_explorer_pane_state.status_message = "Reordering project items.".to_string();

        let project_items_reorder_request = ProjectItemsReorderRequest { project_item_paths };
        let (response_sender, response_receiver) = mpsc::sync_channel(1);
        project_items_reorder_request.send(engine_unprivileged_state, move |project_items_reorder_response| {
            let _ = response_sender.send(project_items_reorder_response);
        });

        match response_receiver.recv_timeout(Duration::from_secs(3)) {
            Ok(project_items_reorder_response) => {
                if project_items_reorder_response.success {
                    self.app_state.project_explorer_pane_state.status_message =
                        format!("Reordered {} project items.", project_items_reorder_response.reordered_project_item_count);
                    self.refresh_project_items_list(squalr_engine);
                } else {
                    self.app_state.project_explorer_pane_state.status_message = "Project item reorder request failed.".to_string();
                }
            }
            Err(receive_error) => {
                self.app_state.project_explorer_pane_state.status_message = format!("Timed out waiting for project item reorder response: {}", receive_error);
            }
        }

        self.app_state
            .project_explorer_pane_state
            .is_reordering_project_item = false;
    }

    pub(super) fn delete_selected_project_item_with_confirmation(
        &mut self,
        squalr_engine: &mut SqualrEngine,
    ) {
        if self
            .app_state
            .project_explorer_pane_state
            .is_deleting_project_item
        {
            self.app_state.project_explorer_pane_state.status_message = "Project item delete request already in progress.".to_string();
            return;
        }

        if !self
            .app_state
            .project_explorer_pane_state
            .has_pending_delete_confirmation_for_selected_project_item()
        {
            if self
                .app_state
                .project_explorer_pane_state
                .arm_delete_confirmation_for_selected_project_item()
            {
                self.app_state.project_explorer_pane_state.status_message = "Press x again to confirm deleting selected project item.".to_string();
            } else {
                self.app_state.project_explorer_pane_state.status_message = "No project item is selected for delete.".to_string();
            }
            return;
        }

        let project_item_paths = self
            .app_state
            .project_explorer_pane_state
            .take_pending_delete_confirmation_paths();
        let engine_unprivileged_state = match squalr_engine.get_engine_unprivileged_state().as_ref() {
            Some(engine_unprivileged_state) => engine_unprivileged_state,
            None => {
                self.app_state.project_explorer_pane_state.status_message = "No unprivileged engine state is available for delete.".to_string();
                return;
            }
        };

        self.app_state
            .project_explorer_pane_state
            .is_deleting_project_item = true;
        self.app_state.project_explorer_pane_state.status_message = format!("Deleting {} project items.", project_item_paths.len());

        let project_items_delete_request = ProjectItemsDeleteRequest { project_item_paths };
        let (response_sender, response_receiver) = mpsc::sync_channel(1);
        project_items_delete_request.send(engine_unprivileged_state, move |project_items_delete_response| {
            let _ = response_sender.send(project_items_delete_response);
        });

        match response_receiver.recv_timeout(Duration::from_secs(3)) {
            Ok(project_items_delete_response) => {
                if project_items_delete_response.success {
                    self.app_state.project_explorer_pane_state.status_message =
                        format!("Deleted {} project items.", project_items_delete_response.deleted_project_item_count);
                    self.refresh_project_items_list(squalr_engine);
                } else {
                    self.app_state.project_explorer_pane_state.status_message = "Project item delete request failed.".to_string();
                }
            }
            Err(receive_error) => {
                self.app_state.project_explorer_pane_state.status_message = format!("Timed out waiting for project item delete response: {}", receive_error);
            }
        }

        self.app_state
            .project_explorer_pane_state
            .is_deleting_project_item = false;
    }

    pub(super) fn promote_selected_project_item_to_symbol(
        &mut self,
        squalr_engine: &mut SqualrEngine,
    ) {
        let Some(selected_project_item_path) = self
            .app_state
            .project_explorer_pane_state
            .selected_project_item_path()
        else {
            self.app_state.project_explorer_pane_state.status_message = "No project item is selected for symbol promotion.".to_string();
            return;
        };

        let engine_unprivileged_state = match squalr_engine.get_engine_unprivileged_state().as_ref() {
            Some(engine_unprivileged_state) => engine_unprivileged_state,
            None => {
                self.app_state.project_explorer_pane_state.status_message = "No unprivileged engine state is available for symbol promotion.".to_string();
                return;
            }
        };

        self.app_state.project_explorer_pane_state.status_message = format!("Promoting {} to symbol claim.", selected_project_item_path.display());

        let project_items_promote_symbol_request = ProjectItemsPromoteSymbolRequest {
            project_item_paths: vec![selected_project_item_path.clone()],
            overwrite_conflicting_symbols: false,
        };
        let (response_sender, response_receiver) = mpsc::sync_channel(1);
        project_items_promote_symbol_request.send(engine_unprivileged_state, move |project_items_promote_symbol_response| {
            let _ = response_sender.send(project_items_promote_symbol_response);
        });

        match response_receiver.recv_timeout(Duration::from_secs(3)) {
            Ok(project_items_promote_symbol_response) => {
                if project_items_promote_symbol_response.success {
                    self.app_state.project_explorer_pane_state.status_message = format!(
                        "Promoted {} symbol claim(s), reused {}, conflicts {}.",
                        project_items_promote_symbol_response.promoted_symbol_count,
                        project_items_promote_symbol_response.reused_symbol_count,
                        project_items_promote_symbol_response.conflicts.len()
                    );
                    self.refresh_project_symbols_list_with_feedback(squalr_engine, false);
                } else {
                    self.app_state.project_explorer_pane_state.status_message = "Project item symbol promotion failed.".to_string();
                }
            }
            Err(receive_error) => {
                self.app_state.project_explorer_pane_state.status_message =
                    format!("Timed out waiting for project item symbol promotion response: {}", receive_error);
            }
        }
    }

    pub(super) fn delete_selected_symbol_claim(
        &mut self,
        squalr_engine: &mut SqualrEngine,
    ) {
        let Some(selected_symbol_claim) = self
            .app_state
            .project_explorer_pane_state
            .selected_symbol_claim()
            .cloned()
        else {
            self.app_state.project_explorer_pane_state.status_message = "No symbol claim is selected for delete.".to_string();
            return;
        };

        let engine_unprivileged_state = match squalr_engine.get_engine_unprivileged_state().as_ref() {
            Some(engine_unprivileged_state) => engine_unprivileged_state,
            None => {
                self.app_state.project_explorer_pane_state.status_message = "No unprivileged engine state is available for symbol claim delete.".to_string();
                return;
            }
        };

        self.app_state.project_explorer_pane_state.status_message = format!("Deleting symbol claim '{}'.", selected_symbol_claim.get_display_name());

        let project_symbols_delete_request = ProjectSymbolsDeleteRequest {
            symbol_locator_keys: vec![selected_symbol_claim.get_symbol_locator_key().to_string()],
            module_names: Vec::new(),
            module_ranges: Vec::new(),
            convert_symbol_refs: false,
        };
        let (response_sender, response_receiver) = mpsc::sync_channel(1);
        project_symbols_delete_request.send(engine_unprivileged_state, move |project_symbols_delete_response| {
            let _ = response_sender.send(project_symbols_delete_response);
        });

        match response_receiver.recv_timeout(Duration::from_secs(3)) {
            Ok(project_symbols_delete_response) => {
                if project_symbols_delete_response.success {
                    self.app_state.project_explorer_pane_state.status_message =
                        format!("Deleted {} symbol claim(s).", project_symbols_delete_response.deleted_symbol_count);
                    self.refresh_project_symbols_list_with_feedback(squalr_engine, false);
                } else {
                    self.app_state.project_explorer_pane_state.status_message = if project_symbols_delete_response.blocked_symbol_ref_count > 0 {
                        format!(
                            "Symbol claim delete blocked by {} symbol-ref project item(s).",
                            project_symbols_delete_response.blocked_symbol_ref_count
                        )
                    } else {
                        "Symbol claim delete request failed.".to_string()
                    };
                }
            }
            Err(receive_error) => {
                self.app_state.project_explorer_pane_state.status_message = format!("Timed out waiting for symbol claim delete response: {}", receive_error);
            }
        }
    }

    pub(super) fn open_selected_project(
        &mut self,
        squalr_engine: &mut SqualrEngine,
    ) {
        if self.app_state.project_explorer_pane_state.is_opening_project {
            self.app_state.project_explorer_pane_state.status_message = "Project open request already in progress.".to_string();
            return;
        }

        let selected_project_directory_path = match self
            .app_state
            .project_explorer_pane_state
            .selected_project_directory_path()
        {
            Some(selected_project_directory_path) => selected_project_directory_path,
            None => {
                self.app_state.project_explorer_pane_state.status_message = "No project is selected.".to_string();
                return;
            }
        };

        let selected_project_name = self
            .app_state
            .project_explorer_pane_state
            .selected_project_name()
            .unwrap_or_else(|| "<unknown>".to_string());

        let engine_unprivileged_state = match squalr_engine.get_engine_unprivileged_state().as_ref() {
            Some(engine_unprivileged_state) => engine_unprivileged_state,
            None => {
                self.app_state.project_explorer_pane_state.status_message = "No unprivileged engine state is available for project opening.".to_string();
                return;
            }
        };

        self.app_state.project_explorer_pane_state.is_opening_project = true;
        self.app_state.project_explorer_pane_state.status_message = format!("Opening project '{}'.", selected_project_name);

        let project_open_request = ProjectOpenRequest {
            open_file_browser: false,
            project_directory_path: Some(selected_project_directory_path.clone()),
            project_name: None,
        };
        let (response_sender, response_receiver) = mpsc::sync_channel(1);
        project_open_request.send(engine_unprivileged_state, move |project_open_response| {
            let _ = response_sender.send(project_open_response);
        });

        match response_receiver.recv_timeout(Duration::from_secs(3)) {
            Ok(project_open_response) => {
                if project_open_response.success {
                    self.app_state
                        .project_explorer_pane_state
                        .set_active_project(Some(selected_project_name.clone()), Some(selected_project_directory_path.clone()));
                    self.app_state.project_explorer_pane_state.clear_project_items();
                    self.app_state
                        .project_explorer_pane_state
                        .clear_project_symbols();
                    self.app_state.project_explorer_pane_state.status_message = format!("Opened project '{}'.", selected_project_name);
                    self.refresh_project_items_list(squalr_engine);
                    self.refresh_project_symbols_list_with_feedback(squalr_engine, false);
                    self.refresh_plugins_with_feedback(squalr_engine, false);
                } else {
                    self.app_state.project_explorer_pane_state.status_message = "Project open request failed.".to_string();
                }
            }
            Err(receive_error) => {
                self.app_state.project_explorer_pane_state.status_message = format!("Timed out waiting for project open response: {}", receive_error);
            }
        }

        self.app_state.project_explorer_pane_state.is_opening_project = false;
    }

    pub(super) fn rename_selected_project_from_pending_name(
        &mut self,
        squalr_engine: &mut SqualrEngine,
    ) {
        if self.app_state.project_explorer_pane_state.is_renaming_project {
            self.app_state.project_explorer_pane_state.status_message = "Project rename request already in progress.".to_string();
            return;
        }

        let selected_project_directory_path = match self
            .app_state
            .project_explorer_pane_state
            .selected_project_directory_path()
        {
            Some(selected_project_directory_path) => selected_project_directory_path,
            None => {
                self.app_state.project_explorer_pane_state.status_message = "No project is selected for rename.".to_string();
                return;
            }
        };
        let selected_project_directory_path_for_active_check = selected_project_directory_path.clone();

        let new_project_name = match self
            .app_state
            .project_explorer_pane_state
            .pending_project_name_trimmed()
        {
            Some(new_project_name) => new_project_name,
            None => {
                self.app_state.project_explorer_pane_state.status_message = "Project name is empty.".to_string();
                return;
            }
        };

        let engine_unprivileged_state = match squalr_engine.get_engine_unprivileged_state().as_ref() {
            Some(engine_unprivileged_state) => engine_unprivileged_state,
            None => {
                self.app_state.project_explorer_pane_state.status_message = "No unprivileged engine state is available for project renaming.".to_string();
                return;
            }
        };

        self.app_state.project_explorer_pane_state.is_renaming_project = true;
        self.app_state.project_explorer_pane_state.status_message = format!("Renaming project to '{}'.", new_project_name);

        let project_rename_request = ProjectRenameRequest {
            project_directory_path: selected_project_directory_path,
            new_project_name: new_project_name.clone(),
        };
        let (response_sender, response_receiver) = mpsc::sync_channel(1);
        project_rename_request.send(engine_unprivileged_state, move |project_rename_response| {
            let _ = response_sender.send(project_rename_response);
        });

        match response_receiver.recv_timeout(Duration::from_secs(3)) {
            Ok(project_rename_response) => {
                if project_rename_response.success {
                    self.app_state
                        .project_explorer_pane_state
                        .cancel_project_name_input();
                    self.app_state.project_explorer_pane_state.status_message = format!("Renamed project to '{}'.", new_project_name);
                    self.refresh_project_list(squalr_engine);
                    let _ = self
                        .app_state
                        .project_explorer_pane_state
                        .select_project_by_directory_path(&project_rename_response.new_project_path);
                    if self
                        .app_state
                        .project_explorer_pane_state
                        .active_project_directory_path
                        .as_ref()
                        .is_some_and(|active_project_directory_path| *active_project_directory_path == selected_project_directory_path_for_active_check)
                    {
                        self.app_state
                            .project_explorer_pane_state
                            .set_active_project(Some(new_project_name), Some(project_rename_response.new_project_path));
                    }
                } else {
                    self.app_state.project_explorer_pane_state.status_message = "Project rename request failed.".to_string();
                }
            }
            Err(receive_error) => {
                self.app_state.project_explorer_pane_state.status_message = format!("Timed out waiting for project rename response: {}", receive_error);
            }
        }

        self.app_state.project_explorer_pane_state.is_renaming_project = false;
    }

    pub(super) fn delete_selected_project(
        &mut self,
        squalr_engine: &mut SqualrEngine,
    ) {
        if self.app_state.project_explorer_pane_state.is_deleting_project {
            self.app_state.project_explorer_pane_state.status_message = "Project delete request already in progress.".to_string();
            return;
        }

        let selected_project_directory_path = match self
            .app_state
            .project_explorer_pane_state
            .selected_project_directory_path()
        {
            Some(selected_project_directory_path) => selected_project_directory_path,
            None => {
                self.app_state.project_explorer_pane_state.status_message = "No project is selected for delete.".to_string();
                return;
            }
        };

        let selected_project_name = self
            .app_state
            .project_explorer_pane_state
            .selected_project_name()
            .unwrap_or_else(|| "<unknown>".to_string());

        let engine_unprivileged_state = match squalr_engine.get_engine_unprivileged_state().as_ref() {
            Some(engine_unprivileged_state) => engine_unprivileged_state,
            None => {
                self.app_state.project_explorer_pane_state.status_message = "No unprivileged engine state is available for project deletion.".to_string();
                return;
            }
        };

        self.app_state.project_explorer_pane_state.is_deleting_project = true;
        self.app_state.project_explorer_pane_state.status_message = format!("Deleting project '{}'.", selected_project_name);

        let project_delete_request = ProjectDeleteRequest {
            project_directory_path: Some(selected_project_directory_path.clone()),
            project_name: None,
        };
        let (response_sender, response_receiver) = mpsc::sync_channel(1);
        project_delete_request.send(engine_unprivileged_state, move |project_delete_response| {
            let _ = response_sender.send(project_delete_response);
        });

        match response_receiver.recv_timeout(Duration::from_secs(3)) {
            Ok(project_delete_response) => {
                if project_delete_response.success {
                    if self
                        .app_state
                        .project_explorer_pane_state
                        .active_project_directory_path
                        .as_ref()
                        .is_some_and(|active_project_directory_path| *active_project_directory_path == selected_project_directory_path)
                    {
                        self.app_state
                            .project_explorer_pane_state
                            .set_active_project(None, None);
                        self.app_state.project_explorer_pane_state.clear_project_items();
                        self.app_state
                            .project_explorer_pane_state
                            .clear_project_symbols();
                    }
                    self.app_state.project_explorer_pane_state.status_message = format!("Deleted project '{}'.", selected_project_name);
                    self.refresh_project_list(squalr_engine);
                } else {
                    self.app_state.project_explorer_pane_state.status_message = "Project delete request failed.".to_string();
                }
            }
            Err(receive_error) => {
                self.app_state.project_explorer_pane_state.status_message = format!("Timed out waiting for project delete response: {}", receive_error);
            }
        }

        self.app_state.project_explorer_pane_state.is_deleting_project = false;
    }

    pub(super) fn close_active_project(
        &mut self,
        squalr_engine: &mut SqualrEngine,
    ) {
        if self.app_state.project_explorer_pane_state.is_closing_project {
            self.app_state.project_explorer_pane_state.status_message = "Project close request already in progress.".to_string();
            return;
        }

        let engine_unprivileged_state = match squalr_engine.get_engine_unprivileged_state().as_ref() {
            Some(engine_unprivileged_state) => engine_unprivileged_state,
            None => {
                self.app_state.project_explorer_pane_state.status_message = "No unprivileged engine state is available for project close.".to_string();
                return;
            }
        };

        self.app_state.project_explorer_pane_state.is_closing_project = true;
        self.app_state.project_explorer_pane_state.status_message = "Closing active project.".to_string();

        let project_close_request = ProjectCloseRequest {};
        let (response_sender, response_receiver) = mpsc::sync_channel(1);
        project_close_request.send(engine_unprivileged_state, move |project_close_response| {
            let _ = response_sender.send(project_close_response);
        });

        match response_receiver.recv_timeout(Duration::from_secs(3)) {
            Ok(project_close_response) => {
                if project_close_response.success {
                    self.app_state
                        .project_explorer_pane_state
                        .set_active_project(None, None);
                    self.app_state.project_explorer_pane_state.clear_project_items();
                    self.app_state
                        .project_explorer_pane_state
                        .clear_project_symbols();
                    self.app_state.project_explorer_pane_state.status_message = "Closed active project.".to_string();
                } else {
                    self.app_state.project_explorer_pane_state.status_message = "Project close request failed.".to_string();
                }
            }
            Err(receive_error) => {
                self.app_state.project_explorer_pane_state.status_message = format!("Timed out waiting for project close response: {}", receive_error);
            }
        }

        self.app_state.project_explorer_pane_state.is_closing_project = false;
    }

    fn build_project_item_virtual_snapshot_query(
        project_item_path: &Path,
        project_item: &ProjectItem,
        engine_unprivileged_state: &Arc<EngineUnprivilegedState>,
    ) -> Option<VirtualSnapshotQuery> {
        let query_id = project_item_path.to_string_lossy().to_string();
        let symbolic_struct_namespace = Self::resolve_project_item_symbolic_struct_namespace(project_item)?;
        let symbolic_struct_definition = Self::build_project_item_preview_symbolic_struct_definition(engine_unprivileged_state, &symbolic_struct_namespace)?;
        let project_item_type_id = project_item.get_item_type().get_project_item_type_id();

        if project_item_type_id == ProjectItemTypeAddress::PROJECT_ITEM_TYPE_ID {
            let mut project_item = project_item.clone();

            return Some(VirtualSnapshotQuery::Address {
                query_id,
                address: ProjectItemTypeAddress::get_field_address(&mut project_item),
                module_name: ProjectItemTypeAddress::get_field_module(&mut project_item),
                symbolic_struct_definition,
            });
        }

        if project_item_type_id == ProjectItemTypePointer::PROJECT_ITEM_TYPE_ID {
            return Some(VirtualSnapshotQuery::Pointer {
                query_id,
                pointer: ProjectItemTypePointer::get_field_pointer(project_item),
                symbolic_struct_definition,
            });
        }

        None
    }

    fn build_project_item_preview_symbolic_struct_definition(
        engine_unprivileged_state: &Arc<EngineUnprivilegedState>,
        symbolic_struct_namespace: &str,
    ) -> Option<SymbolicStructDefinition> {
        let symbolic_struct_definition = engine_unprivileged_state.resolve_struct_layout_definition(symbolic_struct_namespace)?;
        let preview_field_definition = SymbolicFieldDefinition::from_str(symbolic_struct_namespace).ok();

        let Some(preview_field_definition) = preview_field_definition else {
            return Some(symbolic_struct_definition);
        };

        let preview_container_type = match preview_field_definition.get_container_type() {
            ContainerType::ArrayFixed(length) if length > Self::MAX_PROJECT_ITEM_PREVIEW_ELEMENT_COUNT => {
                ContainerType::ArrayFixed(Self::MAX_PROJECT_ITEM_PREVIEW_ELEMENT_COUNT)
            }
            container_type => container_type,
        };

        if preview_container_type == preview_field_definition.get_container_type() {
            Some(symbolic_struct_definition)
        } else {
            Some(SymbolicStructDefinition::new_anonymous(vec![SymbolicFieldDefinition::new(
                preview_field_definition.get_data_type_ref().clone(),
                preview_container_type,
            )]))
        }
    }

    fn build_project_item_preview_value_from_virtual_snapshot_result(
        engine_unprivileged_state: &Arc<EngineUnprivilegedState>,
        project_item: &ProjectItem,
        virtual_snapshot_query_result: &VirtualSnapshotQueryResult,
    ) -> String {
        let Some(memory_read_response) = virtual_snapshot_query_result.memory_read_response.as_ref() else {
            return String::new();
        };

        if !memory_read_response.success {
            return String::new();
        }

        let Some(first_read_field_data_value) = memory_read_response
            .valued_struct
            .get_fields()
            .first()
            .and_then(|valued_struct_field| valued_struct_field.get_data_value())
        else {
            return String::new();
        };

        let default_anonymous_value_string_format =
            engine_unprivileged_state.get_default_anonymous_value_string_format(first_read_field_data_value.get_data_type_ref());
        let symbolic_field_container_type = Self::resolve_project_item_symbolic_container_type(project_item);
        let preview_was_truncated = Self::project_item_preview_was_truncated(project_item);

        engine_unprivileged_state
            .anonymize_value(first_read_field_data_value, default_anonymous_value_string_format)
            .map(|anonymous_value_string| {
                Self::format_project_item_preview_value(&anonymous_value_string, symbolic_field_container_type, preview_was_truncated)
            })
            .unwrap_or_default()
    }

    fn resolve_project_item_symbolic_struct_namespace(project_item: &ProjectItem) -> Option<String> {
        let project_item_type_id = project_item.get_item_type().get_project_item_type_id();

        if project_item_type_id == ProjectItemTypeAddress::PROJECT_ITEM_TYPE_ID {
            let mut project_item = project_item.clone();

            return ProjectItemTypeAddress::get_field_symbolic_struct_definition_reference(&mut project_item).map(|symbolic_struct_reference| {
                symbolic_struct_reference
                    .get_symbolic_struct_namespace()
                    .to_string()
            });
        }

        if project_item_type_id == ProjectItemTypePointer::PROJECT_ITEM_TYPE_ID {
            return ProjectItemTypePointer::get_field_symbolic_struct_definition_reference(project_item).map(|symbolic_struct_reference| {
                symbolic_struct_reference
                    .get_symbolic_struct_namespace()
                    .to_string()
            });
        }

        None
    }

    fn resolve_project_item_symbolic_container_type(project_item: &ProjectItem) -> ContainerType {
        Self::resolve_project_item_symbolic_struct_namespace(project_item)
            .and_then(|symbolic_struct_namespace| SymbolicFieldDefinition::from_str(&symbolic_struct_namespace).ok())
            .map(|symbolic_field_definition| symbolic_field_definition.get_container_type())
            .unwrap_or(ContainerType::None)
    }

    fn project_item_preview_was_truncated(project_item: &ProjectItem) -> bool {
        let Some(symbolic_struct_namespace) = Self::resolve_project_item_symbolic_struct_namespace(project_item) else {
            return false;
        };
        let Some(symbolic_field_definition) = SymbolicFieldDefinition::from_str(&symbolic_struct_namespace).ok() else {
            return false;
        };

        matches!(
            symbolic_field_definition.get_container_type(),
            ContainerType::ArrayFixed(length) if length > Self::MAX_PROJECT_ITEM_PREVIEW_ELEMENT_COUNT
        )
    }

    fn format_project_item_preview_value(
        anonymous_value_string: &AnonymousValueString,
        symbolic_field_container_type: ContainerType,
        preview_was_truncated: bool,
    ) -> String {
        let effective_container_type = if matches!(anonymous_value_string.get_container_type(), ContainerType::Array | ContainerType::ArrayFixed(_)) {
            anonymous_value_string.get_container_type()
        } else {
            symbolic_field_container_type
        };
        let display_value = anonymous_value_string.get_anonymous_value_string();

        if matches!(effective_container_type, ContainerType::Array | ContainerType::ArrayFixed(_)) && !display_value.is_empty() {
            let preview_value = if preview_was_truncated {
                Self::append_project_item_preview_ellipsis(display_value)
            } else {
                Self::truncate_project_item_preview_value(display_value)
            };

            format!("[{}]", preview_value)
        } else {
            display_value.to_string()
        }
    }

    fn append_project_item_preview_ellipsis(display_value: &str) -> String {
        if let Some(truncated_array_preview) = Self::format_project_item_preview_from_elements(display_value, true) {
            return truncated_array_preview;
        }

        let trimmed_display_value = display_value.trim_end_matches(|character: char| character.is_ascii_whitespace() || matches!(character, ',' | ';'));

        if trimmed_display_value.is_empty() {
            String::from("...")
        } else {
            format!("{}...", trimmed_display_value)
        }
    }

    fn truncate_project_item_preview_value(display_value: &str) -> String {
        if let Some(truncated_array_preview) = Self::format_project_item_preview_from_elements(display_value, false) {
            return truncated_array_preview;
        }

        let display_value_character_count = display_value.chars().count();

        if display_value_character_count <= Self::MAX_PROJECT_ITEM_PREVIEW_ARRAY_CHARACTER_COUNT {
            return display_value.to_string();
        }

        let truncated_prefix = display_value
            .chars()
            .take(Self::MAX_PROJECT_ITEM_PREVIEW_ARRAY_CHARACTER_COUNT)
            .collect::<String>()
            .trim_end_matches(|character: char| character.is_ascii_whitespace() || matches!(character, ',' | ';'))
            .to_string();

        format!("{}...", truncated_prefix)
    }

    fn format_project_item_preview_from_elements(
        display_value: &str,
        force_ellipsis: bool,
    ) -> Option<String> {
        let array_elements = Self::split_project_item_preview_elements(display_value);

        if array_elements.len() <= 1 {
            return None;
        }

        let visible_element_count = array_elements
            .len()
            .min(Self::MAX_PROJECT_ITEM_PREVIEW_DISPLAY_ELEMENT_COUNT);
        let mut preview_elements = array_elements
            .iter()
            .take(visible_element_count)
            .map(|array_element| (*array_element).to_string())
            .collect::<Vec<_>>();
        let has_hidden_elements = force_ellipsis || array_elements.len() > visible_element_count;

        if has_hidden_elements {
            preview_elements.push(String::from("..."));
        }

        Some(preview_elements.join(", "))
    }

    fn split_project_item_preview_elements(display_value: &str) -> Vec<&str> {
        display_value
            .split([',', ';'])
            .map(str::trim)
            .filter(|array_element| !array_element.is_empty())
            .collect::<Vec<_>>()
    }

    pub(super) fn extract_string_value_from_edited_field(edited_field: &ValuedStructField) -> Option<String> {
        let edited_data_value = edited_field.get_data_value()?;
        let edited_name = String::from_utf8(edited_data_value.get_value_bytes().clone()).ok()?;
        let edited_name = edited_name.trim();

        if edited_name.is_empty() { None } else { Some(edited_name.to_string()) }
    }

    pub(super) fn build_project_item_rename_request(
        project_item_path: &Path,
        project_item_type_id: &str,
        edited_name: &str,
    ) -> Option<ProjectItemsRenameRequest> {
        let sanitized_file_name = Path::new(edited_name)
            .file_name()
            .and_then(|file_name| file_name.to_str())
            .map(str::trim)
            .filter(|file_name| !file_name.is_empty())?
            .to_string();
        let is_directory_project_item = project_item_type_id == ProjectItemTypeDirectory::PROJECT_ITEM_TYPE_ID;
        let renamed_project_item_name = if is_directory_project_item {
            sanitized_file_name
        } else {
            let mut file_name_with_extension = sanitized_file_name.clone();
            let expected_extension = Project::PROJECT_ITEM_EXTENSION.trim_start_matches('.');
            let has_expected_extension = Path::new(&sanitized_file_name)
                .extension()
                .and_then(|extension| extension.to_str())
                .map(|extension| extension.eq_ignore_ascii_case(expected_extension))
                .unwrap_or(false);
            if !has_expected_extension {
                file_name_with_extension.push('.');
                file_name_with_extension.push_str(expected_extension);
            }

            file_name_with_extension
        };
        let current_file_name = project_item_path
            .file_name()
            .and_then(|file_name| file_name.to_str())
            .unwrap_or_default();
        if current_file_name == renamed_project_item_name {
            return None;
        }

        Some(ProjectItemsRenameRequest {
            project_item_path: project_item_path.to_path_buf(),
            project_item_name: renamed_project_item_name,
        })
    }

    pub(super) fn build_memory_write_request_for_project_item_edit(
        project_item: &mut ProjectItem,
        edited_field: &ValuedStructField,
    ) -> Option<MemoryWriteRequest> {
        if project_item.get_item_type().get_project_item_type_id() != ProjectItemTypeAddress::PROJECT_ITEM_TYPE_ID {
            return None;
        }
        if edited_field.get_name() != ProjectItemTypeAddress::PROPERTY_ADDRESS {
            return None;
        }

        let edited_data_value = edited_field.get_data_value()?;
        let address = ProjectItemTypeAddress::get_field_address(project_item);
        let module_name = ProjectItemTypeAddress::get_field_module(project_item);

        Some(MemoryWriteRequest {
            address,
            module_name,
            value: edited_data_value.get_value_bytes().clone(),
        })
    }

    pub(super) fn build_scan_results_set_property_request_for_struct_edit(
        scan_result_refs: Vec<ScanResultRef>,
        edited_field: &ValuedStructField,
        engine_unprivileged_state: &EngineUnprivilegedState,
    ) -> Result<ScanResultsSetPropertyRequest, String> {
        let edited_data_value = edited_field
            .get_data_value()
            .ok_or_else(|| "Nested struct scan result edits are not supported in the TUI yet.".to_string())?;
        let default_edit_format = engine_unprivileged_state.get_default_anonymous_value_string_format(edited_data_value.get_data_type_ref());
        let edited_anonymous_value = engine_unprivileged_state
            .anonymize_value(edited_data_value, default_edit_format)
            .map_err(|error| format!("Failed to format edited scan result value: {}", error))?;

        Ok(ScanResultsSetPropertyRequest {
            scan_result_refs,
            field_namespace: edited_field.get_name().to_string(),
            anonymous_value_string: edited_anonymous_value,
        })
    }

    pub(super) fn should_apply_struct_field_edit_to_project_item(
        project_item_type_id: &str,
        edited_field_name: &str,
    ) -> bool {
        !(edited_field_name == ProjectItem::PROPERTY_NAME && project_item_type_id == ProjectItemTypeDirectory::PROJECT_ITEM_TYPE_ID)
    }
}
