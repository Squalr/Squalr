use super::app_shell::AppShell;
use crate::views::settings::pane_state::SettingsCategory;
use crate::views::struct_viewer::pane_state::StructViewerSource;
use squalr_engine::squalr_engine::SqualrEngine;
use squalr_engine_api::commands::privileged_command_request::PrivilegedCommandRequest;
use squalr_engine_api::commands::project::save::project_save_request::ProjectSaveRequest;
use squalr_engine_api::commands::scan_results::freeze::scan_results_freeze_request::ScanResultsFreezeRequest;
use squalr_engine_api::commands::settings::general::list::general_settings_list_request::GeneralSettingsListRequest;
use squalr_engine_api::commands::settings::general::set::general_settings_set_request::GeneralSettingsSetRequest;
use squalr_engine_api::commands::settings::memory::list::memory_settings_list_request::MemorySettingsListRequest;
use squalr_engine_api::commands::settings::memory::set::memory_settings_set_request::MemorySettingsSetRequest;
use squalr_engine_api::commands::settings::scan::list::scan_settings_list_request::ScanSettingsListRequest;
use squalr_engine_api::commands::settings::scan::set::scan_settings_set_request::ScanSettingsSetRequest;
use squalr_engine_api::commands::unprivileged_command_request::UnprivilegedCommandRequest;
use squalr_engine_api::engine::engine_execution_context::EngineExecutionContext;
use squalr_engine_api::structures::projects::project_items::project_item::ProjectItem;
use squalr_engine_api::structures::projects::project_items::project_item_ref::ProjectItemRef;
use squalr_engine_api::structures::scan_results::scan_result::ScanResult;
use squalr_engine_api::structures::structs::valued_struct_field::ValuedStructField;
use std::sync::mpsc;
use std::time::Duration;

impl AppShell {
    pub(super) fn reset_selected_settings_category_to_defaults(
        &mut self,
        squalr_engine: &mut SqualrEngine,
    ) {
        if self
            .app_state
            .settings_pane_state
            .reset_selected_category_to_defaults()
        {
            self.apply_selected_settings_category(squalr_engine);
            self.app_state.settings_pane_state.status_message =
                format!("Reset {} settings to defaults.", self.app_state.settings_pane_state.selected_category.title());
        } else {
            self.app_state.settings_pane_state.status_message = format!(
                "{} settings are already at defaults.",
                self.app_state.settings_pane_state.selected_category.title()
            );
        }
    }

    pub(super) fn refresh_output_log_history(
        &mut self,
        squalr_engine: &mut SqualrEngine,
    ) {
        self.refresh_output_log_history_with_feedback(squalr_engine, false);
    }

    pub(super) fn refresh_output_log_history_with_feedback(
        &mut self,
        squalr_engine: &mut SqualrEngine,
        should_update_status_message: bool,
    ) {
        let engine_unprivileged_state = match squalr_engine.get_engine_unprivileged_state().as_ref() {
            Some(engine_unprivileged_state) => engine_unprivileged_state,
            None => {
                self.app_state.output_pane_state.status_message = "No unprivileged engine state is available for output logs.".to_string();
                return;
            }
        };

        let log_history_guard = match engine_unprivileged_state.get_logger().get_log_history().read() {
            Ok(log_history_guard) => log_history_guard,
            Err(lock_error) => {
                self.app_state.output_pane_state.status_message = format!("Failed to lock output log history: {}", lock_error);
                return;
            }
        };
        let log_history_snapshot = log_history_guard.iter().cloned().collect();
        self.app_state
            .output_pane_state
            .apply_log_history_with_feedback(log_history_snapshot, should_update_status_message);
    }

    pub(super) fn refresh_all_settings_categories_with_feedback(
        &mut self,
        squalr_engine: &mut SqualrEngine,
        should_update_status_message: bool,
    ) {
        if self.app_state.settings_pane_state.is_refreshing_settings {
            self.app_state.settings_pane_state.status_message = "Settings refresh is already in progress.".to_string();
            return;
        }

        let engine_unprivileged_state = match squalr_engine.get_engine_unprivileged_state().as_ref() {
            Some(engine_unprivileged_state) => engine_unprivileged_state,
            None => {
                self.app_state.settings_pane_state.status_message = "No unprivileged engine state is available for settings refresh.".to_string();
                return;
            }
        };

        self.app_state.settings_pane_state.is_refreshing_settings = true;
        let mut did_read_general_settings = false;
        let mut did_read_memory_settings = false;
        let mut did_read_scan_settings = false;

        let general_settings_list_request = GeneralSettingsListRequest {};
        let (general_response_sender, general_response_receiver) = mpsc::sync_channel(1);
        general_settings_list_request.send(engine_unprivileged_state, move |general_settings_list_response| {
            let _ = general_response_sender.send(general_settings_list_response);
        });

        match general_response_receiver.recv_timeout(Duration::from_secs(3)) {
            Ok(general_settings_list_response) => {
                if let Ok(general_settings) = general_settings_list_response.general_settings {
                    self.app_state
                        .settings_pane_state
                        .apply_general_settings(general_settings);
                    did_read_general_settings = true;
                } else {
                    self.app_state.settings_pane_state.status_message = "Failed to read general settings.".to_string();
                }
            }
            Err(receive_error) => {
                self.app_state.settings_pane_state.status_message = format!("Timed out waiting for general settings response: {}", receive_error);
            }
        }

        let memory_settings_list_request = MemorySettingsListRequest {};
        let (memory_response_sender, memory_response_receiver) = mpsc::sync_channel(1);
        memory_settings_list_request.send(engine_unprivileged_state, move |memory_settings_list_response| {
            let _ = memory_response_sender.send(memory_settings_list_response);
        });

        match memory_response_receiver.recv_timeout(Duration::from_secs(3)) {
            Ok(memory_settings_list_response) => {
                if let Ok(memory_settings) = memory_settings_list_response.memory_settings {
                    self.app_state
                        .settings_pane_state
                        .apply_memory_settings(memory_settings);
                    did_read_memory_settings = true;
                } else {
                    self.app_state.settings_pane_state.status_message = "Failed to read memory settings.".to_string();
                }
            }
            Err(receive_error) => {
                self.app_state.settings_pane_state.status_message = format!("Timed out waiting for memory settings response: {}", receive_error);
            }
        }

        let scan_settings_list_request = ScanSettingsListRequest {};
        let (scan_response_sender, scan_response_receiver) = mpsc::sync_channel(1);
        scan_settings_list_request.send(engine_unprivileged_state, move |scan_settings_list_response| {
            let _ = scan_response_sender.send(scan_settings_list_response);
        });

        match scan_response_receiver.recv_timeout(Duration::from_secs(3)) {
            Ok(scan_settings_list_response) => {
                if let Ok(scan_settings) = scan_settings_list_response.scan_settings {
                    self.app_state
                        .settings_pane_state
                        .apply_scan_settings(scan_settings);
                    did_read_scan_settings = true;
                    if should_update_status_message {
                        self.app_state.settings_pane_state.status_message = "Settings refreshed.".to_string();
                    }
                } else {
                    self.app_state.settings_pane_state.status_message = "Failed to read scan settings.".to_string();
                }
            }
            Err(receive_error) => {
                self.app_state.settings_pane_state.status_message = format!("Timed out waiting for scan settings response: {}", receive_error);
            }
        }

        if did_read_general_settings && did_read_memory_settings && did_read_scan_settings {
            self.app_state.settings_pane_state.has_loaded_settings_once = true;
        }

        self.app_state.settings_pane_state.is_refreshing_settings = false;
    }

    pub(super) fn apply_selected_settings_category(
        &mut self,
        squalr_engine: &mut SqualrEngine,
    ) {
        if self.app_state.settings_pane_state.is_applying_settings {
            self.app_state.settings_pane_state.status_message = "Settings update is already in progress.".to_string();
            return;
        }

        let engine_unprivileged_state = match squalr_engine.get_engine_unprivileged_state().as_ref() {
            Some(engine_unprivileged_state) => engine_unprivileged_state,
            None => {
                self.app_state.settings_pane_state.status_message = "No unprivileged engine state is available for settings update.".to_string();
                return;
            }
        };

        self.app_state.settings_pane_state.is_applying_settings = true;
        let selected_settings_category = self.app_state.settings_pane_state.selected_category;

        match selected_settings_category {
            SettingsCategory::General => {
                let general_settings_set_request = GeneralSettingsSetRequest {
                    engine_request_delay: Some(
                        self.app_state
                            .settings_pane_state
                            .general_settings
                            .debug_engine_request_delay_ms,
                    ),
                };
                let (response_sender, response_receiver) = mpsc::sync_channel(1);
                general_settings_set_request.send(engine_unprivileged_state, move |general_settings_set_response| {
                    let _ = response_sender.send(general_settings_set_response);
                });

                match response_receiver.recv_timeout(Duration::from_secs(3)) {
                    Ok(_general_settings_set_response) => {
                        self.app_state.settings_pane_state.has_pending_changes = false;
                        self.app_state.settings_pane_state.status_message = "Applied general settings.".to_string();
                    }
                    Err(receive_error) => {
                        self.app_state.settings_pane_state.status_message = format!("Timed out waiting for general settings set response: {}", receive_error);
                    }
                }
            }
            SettingsCategory::Memory => {
                let memory_settings = self.app_state.settings_pane_state.memory_settings;
                let memory_settings_set_request = MemorySettingsSetRequest {
                    memory_type_none: Some(memory_settings.memory_type_none),
                    memory_type_private: Some(memory_settings.memory_type_private),
                    memory_type_image: Some(memory_settings.memory_type_image),
                    memory_type_mapped: Some(memory_settings.memory_type_mapped),
                    required_write: Some(memory_settings.required_write),
                    required_execute: Some(memory_settings.required_execute),
                    required_copy_on_write: Some(memory_settings.required_copy_on_write),
                    excluded_write: Some(memory_settings.excluded_write),
                    excluded_execute: Some(memory_settings.excluded_execute),
                    excluded_copy_on_write: Some(memory_settings.excluded_copy_on_write),
                    start_address: Some(memory_settings.start_address),
                    end_address: Some(memory_settings.end_address),
                    only_query_usermode: Some(memory_settings.only_query_usermode),
                };
                let (response_sender, response_receiver) = mpsc::sync_channel(1);
                memory_settings_set_request.send(engine_unprivileged_state, move |memory_settings_set_response| {
                    let _ = response_sender.send(memory_settings_set_response);
                });

                match response_receiver.recv_timeout(Duration::from_secs(3)) {
                    Ok(_memory_settings_set_response) => {
                        self.app_state.settings_pane_state.has_pending_changes = false;
                        self.app_state.settings_pane_state.status_message = "Applied memory settings.".to_string();
                    }
                    Err(receive_error) => {
                        self.app_state.settings_pane_state.status_message = format!("Timed out waiting for memory settings set response: {}", receive_error);
                    }
                }
            }
            SettingsCategory::Scan => {
                let scan_settings = self.app_state.settings_pane_state.scan_settings;
                let scan_settings_set_request = ScanSettingsSetRequest {
                    results_page_size: Some(scan_settings.results_page_size),
                    results_read_interval_ms: Some(scan_settings.results_read_interval_ms),
                    project_read_interval_ms: Some(scan_settings.project_read_interval_ms),
                    freeze_interval_ms: Some(scan_settings.freeze_interval_ms),
                    memory_alignment: scan_settings.memory_alignment,
                    memory_read_mode: Some(scan_settings.memory_read_mode),
                    floating_point_tolerance: Some(scan_settings.floating_point_tolerance),
                    is_single_threaded_scan: Some(scan_settings.is_single_threaded_scan),
                    debug_perform_validation_scan: Some(scan_settings.debug_perform_validation_scan),
                };
                let (response_sender, response_receiver) = mpsc::sync_channel(1);
                scan_settings_set_request.send(engine_unprivileged_state, move |scan_settings_set_response| {
                    let _ = response_sender.send(scan_settings_set_response);
                });

                match response_receiver.recv_timeout(Duration::from_secs(3)) {
                    Ok(_scan_settings_set_response) => {
                        self.app_state.settings_pane_state.has_pending_changes = false;
                        self.app_state.settings_pane_state.status_message = "Applied scan settings.".to_string();
                    }
                    Err(receive_error) => {
                        self.app_state.settings_pane_state.status_message = format!("Timed out waiting for scan settings set response: {}", receive_error);
                    }
                }
            }
        }

        self.app_state.settings_pane_state.is_applying_settings = false;
    }

    pub(super) fn refresh_struct_viewer_focus_from_source(&mut self) {
        match self.app_state.struct_viewer_pane_state.source {
            StructViewerSource::None => {
                self.app_state.struct_viewer_pane_state.status_message = "No struct viewer source is selected.".to_string();
            }
            StructViewerSource::ScanResults => self.sync_struct_viewer_focus_from_scan_results(),
            StructViewerSource::ProjectItems => self.sync_struct_viewer_focus_from_project_items(),
        }
    }

    pub(super) fn sync_struct_viewer_focus_from_scan_results(&mut self) {
        let selected_scan_results = self.app_state.scan_results_pane_state.selected_scan_results();
        let selected_scan_result_refs = self
            .app_state
            .scan_results_pane_state
            .selected_scan_result_refs();
        self.app_state
            .struct_viewer_pane_state
            .focus_scan_results(&selected_scan_results, selected_scan_result_refs);
    }

    pub(super) fn sync_struct_viewer_focus_from_project_items(&mut self) {
        let selected_project_items = self
            .app_state
            .project_explorer_pane_state
            .selected_project_items_for_struct_viewer();
        self.app_state
            .struct_viewer_pane_state
            .focus_project_items(selected_project_items);
    }

    pub(super) fn commit_struct_viewer_field_edit(
        &mut self,
        squalr_engine: &mut SqualrEngine,
    ) {
        if self.app_state.struct_viewer_pane_state.is_committing_edit {
            self.app_state.struct_viewer_pane_state.status_message = "Struct field edit is already in progress.".to_string();
            return;
        }

        let edited_field = match self
            .app_state
            .struct_viewer_pane_state
            .build_edited_field_from_pending_text()
        {
            Ok(edited_field) => edited_field,
            Err(error) => {
                self.app_state.struct_viewer_pane_state.status_message = error;
                return;
            }
        };

        self.app_state.struct_viewer_pane_state.is_committing_edit = true;
        self.app_state.struct_viewer_pane_state.status_message = format!("Committing field '{}'.", edited_field.get_name());

        match self.app_state.struct_viewer_pane_state.source {
            StructViewerSource::None => {
                self.app_state.struct_viewer_pane_state.status_message = "No struct viewer source is selected for commit.".to_string();
            }
            StructViewerSource::ScanResults => self.commit_scan_result_struct_field_edit(squalr_engine, edited_field),
            StructViewerSource::ProjectItems => self.commit_project_item_struct_field_edit(squalr_engine, edited_field),
        }

        self.app_state.struct_viewer_pane_state.is_committing_edit = false;
    }

    pub(super) fn commit_scan_result_struct_field_edit(
        &mut self,
        squalr_engine: &mut SqualrEngine,
        edited_field: ValuedStructField,
    ) {
        let selected_scan_result_refs = self
            .app_state
            .struct_viewer_pane_state
            .selected_scan_result_refs
            .clone();
        if selected_scan_result_refs.is_empty() {
            self.app_state.struct_viewer_pane_state.status_message = "No scan results are selected for struct edit commit.".to_string();
            return;
        }

        let engine_unprivileged_state = match squalr_engine.get_engine_unprivileged_state().as_ref() {
            Some(engine_unprivileged_state) => engine_unprivileged_state,
            None => {
                self.app_state.struct_viewer_pane_state.status_message = "No unprivileged engine state is available for scan result struct edits.".to_string();
                return;
            }
        };

        if edited_field.get_name() == ScanResult::PROPERTY_NAME_IS_FROZEN {
            let target_frozen_state = edited_field
                .get_data_value()
                .map(|edited_data_value| {
                    edited_data_value
                        .get_value_bytes()
                        .iter()
                        .any(|edited_value_byte| *edited_value_byte != 0)
                })
                .unwrap_or(false);

            let scan_results_freeze_request = ScanResultsFreezeRequest {
                scan_result_refs: selected_scan_result_refs,
                is_frozen: target_frozen_state,
            };
            let (response_sender, response_receiver) = mpsc::sync_channel(1);
            let request_dispatched = scan_results_freeze_request.send(engine_unprivileged_state, move |scan_results_freeze_response| {
                let _ = response_sender.send(scan_results_freeze_response);
            });

            if !request_dispatched {
                self.app_state.struct_viewer_pane_state.status_message = "Failed to dispatch scan result freeze request from struct viewer.".to_string();
                return;
            }

            match response_receiver.recv_timeout(Duration::from_secs(3)) {
                Ok(scan_results_freeze_response) => {
                    if scan_results_freeze_response
                        .failed_freeze_toggle_scan_result_refs
                        .is_empty()
                    {
                        self.app_state.struct_viewer_pane_state.status_message = if target_frozen_state {
                            "Committed frozen state from struct viewer.".to_string()
                        } else {
                            "Committed unfrozen state from struct viewer.".to_string()
                        };
                    } else {
                        self.app_state.struct_viewer_pane_state.status_message = format!(
                            "Freeze commit partially failed for {} scan results.",
                            scan_results_freeze_response
                                .failed_freeze_toggle_scan_result_refs
                                .len()
                        );
                    }
                    self.refresh_scan_results_page(squalr_engine);
                    self.sync_struct_viewer_focus_from_scan_results();
                }
                Err(receive_error) => {
                    self.app_state.struct_viewer_pane_state.status_message = format!("Timed out waiting for scan result freeze response: {}", receive_error);
                }
            }
            return;
        }

        let scan_results_set_property_request = match Self::build_scan_results_set_property_request_for_struct_edit(selected_scan_result_refs, &edited_field) {
            Ok(scan_results_set_property_request) => scan_results_set_property_request,
            Err(error) => {
                self.app_state.struct_viewer_pane_state.status_message = error;
                return;
            }
        };
        let (response_sender, response_receiver) = mpsc::sync_channel(1);
        let request_dispatched = scan_results_set_property_request.send(engine_unprivileged_state, move |scan_results_set_property_response| {
            let _ = response_sender.send(scan_results_set_property_response);
        });

        if !request_dispatched {
            self.app_state.struct_viewer_pane_state.status_message = "Failed to dispatch scan result property request from struct viewer.".to_string();
            return;
        }

        match response_receiver.recv_timeout(Duration::from_secs(3)) {
            Ok(_scan_results_set_property_response) => {
                self.app_state.struct_viewer_pane_state.status_message =
                    format!("Committed scan result field '{}' from struct viewer.", edited_field.get_name());
                self.refresh_scan_results_page(squalr_engine);
                self.sync_struct_viewer_focus_from_scan_results();
            }
            Err(receive_error) => {
                self.app_state.struct_viewer_pane_state.status_message = format!("Timed out waiting for scan result property response: {}", receive_error);
            }
        }
    }

    pub(super) fn commit_project_item_struct_field_edit(
        &mut self,
        squalr_engine: &mut SqualrEngine,
        edited_field: ValuedStructField,
    ) {
        let selected_project_item_paths = self
            .app_state
            .struct_viewer_pane_state
            .selected_project_item_paths
            .clone();
        if selected_project_item_paths.is_empty() {
            self.app_state.struct_viewer_pane_state.status_message = "No project items are selected for struct edit commit.".to_string();
            return;
        }

        let engine_unprivileged_state = match squalr_engine.get_engine_unprivileged_state().as_ref() {
            Some(engine_unprivileged_state) => engine_unprivileged_state,
            None => {
                self.app_state.struct_viewer_pane_state.status_message = "No unprivileged engine state is available for project item struct edits.".to_string();
                return;
            }
        };

        let project_manager = engine_unprivileged_state.get_project_manager();
        let opened_project_lock = project_manager.get_opened_project();
        let edited_field_name = edited_field.get_name().to_string();
        let edited_name = if edited_field_name == ProjectItem::PROPERTY_NAME {
            Self::extract_string_value_from_edited_field(&edited_field)
        } else {
            None
        };

        let mut pending_memory_write_requests = Vec::new();
        let mut pending_rename_requests = Vec::new();
        let mut has_persisted_property_edit = false;
        let mut opened_project_write_guard = match opened_project_lock.write() {
            Ok(opened_project_write_guard) => opened_project_write_guard,
            Err(error) => {
                self.app_state.struct_viewer_pane_state.status_message = format!("Failed to acquire opened project lock for struct edit: {}", error);
                return;
            }
        };
        let opened_project = match opened_project_write_guard.as_mut() {
            Some(opened_project) => opened_project,
            None => {
                self.app_state.struct_viewer_pane_state.status_message = "Cannot apply struct edit because no project is currently open.".to_string();
                return;
            }
        };
        let root_project_item_path = opened_project
            .get_project_root_ref()
            .get_project_item_path()
            .clone();

        for selected_project_item_path in &selected_project_item_paths {
            if edited_field_name == ProjectItem::PROPERTY_NAME && selected_project_item_path == &root_project_item_path {
                continue;
            }

            let project_item_ref = ProjectItemRef::new(selected_project_item_path.clone());
            let selected_project_item = match opened_project.get_project_item_mut(&project_item_ref) {
                Some(selected_project_item) => selected_project_item,
                None => continue,
            };
            let project_item_type_id = selected_project_item
                .get_item_type()
                .get_project_item_type_id()
                .to_string();
            let should_apply_edited_field = Self::should_apply_struct_field_edit_to_project_item(&project_item_type_id, &edited_field_name);

            if should_apply_edited_field {
                selected_project_item.get_properties_mut().set_field_data(
                    edited_field.get_name(),
                    edited_field.get_field_data().clone(),
                    edited_field.get_is_read_only(),
                );
                selected_project_item.set_has_unsaved_changes(true);
                has_persisted_property_edit = true;
            }

            if let Some(edited_name) = &edited_name {
                if let Some(project_items_rename_request) =
                    Self::build_project_item_rename_request(selected_project_item_path, &project_item_type_id, edited_name)
                {
                    pending_rename_requests.push(project_items_rename_request);
                }
            }

            if let Some(memory_write_request) = Self::build_memory_write_request_for_project_item_edit(selected_project_item, &edited_field) {
                pending_memory_write_requests.push(memory_write_request);
            }
        }

        if !has_persisted_property_edit && pending_rename_requests.is_empty() && pending_memory_write_requests.is_empty() {
            self.app_state.struct_viewer_pane_state.status_message = "Selected project item field cannot be committed through TUI struct routing.".to_string();
            return;
        }

        drop(opened_project_write_guard);

        if has_persisted_property_edit {
            if let Ok(mut opened_project_write_guard) = opened_project_lock.write() {
                if let Some(opened_project) = opened_project_write_guard.as_mut() {
                    opened_project
                        .get_project_info_mut()
                        .set_has_unsaved_changes(true);
                }
            }

            let project_save_request = ProjectSaveRequest {};
            let (response_sender, response_receiver) = mpsc::sync_channel(1);
            project_save_request.send(engine_unprivileged_state, move |project_save_response| {
                let _ = response_sender.send(project_save_response);
            });

            match response_receiver.recv_timeout(Duration::from_secs(3)) {
                Ok(project_save_response) => {
                    if !project_save_response.success {
                        self.app_state.struct_viewer_pane_state.status_message = "Project save failed while committing project item struct field.".to_string();
                        return;
                    }
                }
                Err(receive_error) => {
                    self.app_state.struct_viewer_pane_state.status_message = format!("Timed out waiting for project save response: {}", receive_error);
                    return;
                }
            }

            project_manager.notify_project_items_changed();
        }

        for pending_rename_request in pending_rename_requests {
            let (response_sender, response_receiver) = mpsc::sync_channel(1);
            pending_rename_request.send(engine_unprivileged_state, move |project_items_rename_response| {
                let _ = response_sender.send(project_items_rename_response);
            });

            match response_receiver.recv_timeout(Duration::from_secs(3)) {
                Ok(project_items_rename_response) => {
                    if !project_items_rename_response.success {
                        self.app_state.struct_viewer_pane_state.status_message = "Project item rename failed during struct edit commit.".to_string();
                        return;
                    }
                }
                Err(receive_error) => {
                    self.app_state.struct_viewer_pane_state.status_message = format!("Timed out waiting for project item rename response: {}", receive_error);
                    return;
                }
            }
        }

        for pending_memory_write_request in pending_memory_write_requests {
            let (response_sender, response_receiver) = mpsc::sync_channel(1);
            let request_dispatched = pending_memory_write_request.send(engine_unprivileged_state, move |memory_write_response| {
                let _ = response_sender.send(memory_write_response);
            });
            if !request_dispatched {
                self.app_state.struct_viewer_pane_state.status_message = "Failed to dispatch memory write request during struct edit commit.".to_string();
                return;
            }

            match response_receiver.recv_timeout(Duration::from_secs(3)) {
                Ok(memory_write_response) => {
                    if !memory_write_response.success {
                        self.app_state.struct_viewer_pane_state.status_message = "Memory write failed during project item struct edit commit.".to_string();
                        return;
                    }
                }
                Err(receive_error) => {
                    self.app_state.struct_viewer_pane_state.status_message = format!("Timed out waiting for memory write response: {}", receive_error);
                    return;
                }
            }
        }

        self.app_state
            .struct_viewer_pane_state
            .apply_committed_field(&edited_field);
        self.app_state.struct_viewer_pane_state.status_message = format!("Committed project item field '{}' from struct viewer.", edited_field.get_name());
        self.refresh_project_items_list(squalr_engine);
        self.sync_struct_viewer_focus_from_project_items();
    }
}
