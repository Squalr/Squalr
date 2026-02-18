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
use squalr_engine_api::commands::project_items::rename::project_items_rename_request::ProjectItemsRenameRequest;
use squalr_engine_api::commands::project_items::reorder::project_items_reorder_request::ProjectItemsReorderRequest;
use squalr_engine_api::commands::scan_results::set_property::scan_results_set_property_request::ScanResultsSetPropertyRequest;
use squalr_engine_api::commands::unprivileged_command_request::UnprivilegedCommandRequest;
use squalr_engine_api::registries::symbols::symbol_registry::SymbolRegistry;
use squalr_engine_api::structures::projects::project::Project;
use squalr_engine_api::structures::projects::project_items::built_in_types::project_item_type_address::ProjectItemTypeAddress;
use squalr_engine_api::structures::projects::project_items::built_in_types::project_item_type_directory::ProjectItemTypeDirectory;
use squalr_engine_api::structures::projects::project_items::project_item::ProjectItem;
use squalr_engine_api::structures::scan_results::scan_result_ref::ScanResultRef;
use squalr_engine_api::structures::structs::valued_struct_field::ValuedStructField;
use std::path::Path;
use std::sync::mpsc;
use std::time::Duration;

impl AppShell {
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

        let project_items_list_request = ProjectItemsListRequest {};
        let (response_sender, response_receiver) = mpsc::sync_channel(1);
        project_items_list_request.send(engine_unprivileged_state, move |project_items_list_response| {
            let _ = response_sender.send(project_items_list_response);
        });

        match response_receiver.recv_timeout(Duration::from_secs(3)) {
            Ok(project_items_list_response) => {
                let project_item_count = project_items_list_response.opened_project_items.len();
                self.app_state
                    .project_explorer_pane_state
                    .apply_project_items_list(project_items_list_response.opened_project_items);
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
            project_item_type: "directory".to_string(),
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
                    self.app_state.project_explorer_pane_state.status_message = format!("Opened project '{}'.", selected_project_name);
                    self.refresh_project_items_list(squalr_engine);
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
    ) -> Result<ScanResultsSetPropertyRequest, String> {
        let edited_data_value = edited_field
            .get_data_value()
            .ok_or_else(|| "Nested struct scan result edits are not supported in the TUI yet.".to_string())?;
        let symbol_registry = SymbolRegistry::get_instance();
        let default_edit_format = symbol_registry.get_default_anonymous_value_string_format(edited_data_value.get_data_type_ref());
        let edited_anonymous_value = symbol_registry
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
