use super::app_shell::AppShell;
use crate::state::pane::TuiPane;
use crate::state::workspace_page::TuiWorkspacePage;
use squalr_engine::squalr_engine::SqualrEngine;
use squalr_engine_api::commands::privileged_command_request::PrivilegedCommandRequest;
use squalr_engine_api::commands::process::list::process_list_request::ProcessListRequest;
use squalr_engine_api::commands::process::open::process_open_request::ProcessOpenRequest;
use squalr_engine_api::commands::project_items::add::project_items_add_request::ProjectItemsAddRequest;
use squalr_engine_api::commands::scan::collect_values::scan_collect_values_request::ScanCollectValuesRequest;
use squalr_engine_api::commands::scan::element_scan::element_scan_request::ElementScanRequest;
use squalr_engine_api::commands::scan::new::scan_new_request::ScanNewRequest;
use squalr_engine_api::commands::scan::reset::scan_reset_request::ScanResetRequest;
use squalr_engine_api::commands::scan_results::delete::scan_results_delete_request::ScanResultsDeleteRequest;
use squalr_engine_api::commands::scan_results::freeze::scan_results_freeze_request::ScanResultsFreezeRequest;
use squalr_engine_api::commands::scan_results::query::scan_results_query_request::ScanResultsQueryRequest;
use squalr_engine_api::commands::scan_results::refresh::scan_results_refresh_request::ScanResultsRefreshRequest;
use squalr_engine_api::commands::scan_results::set_property::scan_results_set_property_request::ScanResultsSetPropertyRequest;
use squalr_engine_api::commands::unprivileged_command_request::UnprivilegedCommandRequest;
use squalr_engine_api::structures::data_values::anonymous_value_string::AnonymousValueString;
use squalr_engine_api::structures::data_values::anonymous_value_string_format::AnonymousValueStringFormat;
use squalr_engine_api::structures::data_values::container_type::ContainerType;
use squalr_engine_api::structures::scan_results::scan_result::ScanResult;
use std::sync::mpsc;
use std::time::{Duration, Instant};

impl AppShell {
    pub(super) fn reset_scan_state(
        &mut self,
        squalr_engine: &mut SqualrEngine,
    ) {
        if self
            .app_state
            .element_scanner_pane_state
            .has_pending_scan_request
        {
            self.app_state.element_scanner_pane_state.status_message = "Scan request already in progress.".to_string();
            return;
        }
        if self
            .app_state
            .element_scanner_pane_state
            .selected_data_type_ids()
            .is_empty()
        {
            self.app_state.element_scanner_pane_state.status_message = "Select at least one data type before scanning.".to_string();
            return;
        }

        let engine_unprivileged_state = match squalr_engine.get_engine_unprivileged_state().as_ref() {
            Some(engine_unprivileged_state) => engine_unprivileged_state,
            None => {
                self.app_state.element_scanner_pane_state.status_message = "No unprivileged engine state is available for scan reset.".to_string();
                return;
            }
        };

        self.app_state
            .element_scanner_pane_state
            .has_pending_scan_request = true;
        self.app_state.element_scanner_pane_state.status_message = "Resetting active scan.".to_string();

        let scan_reset_request = ScanResetRequest {};
        let (response_sender, response_receiver) = mpsc::sync_channel(1);
        let request_dispatched = scan_reset_request.send(engine_unprivileged_state, move |scan_reset_response| {
            let _ = response_sender.send(scan_reset_response);
        });

        if !request_dispatched {
            self.app_state
                .element_scanner_pane_state
                .has_pending_scan_request = false;
            self.app_state.element_scanner_pane_state.status_message = "Failed to dispatch scan reset request.".to_string();
            return;
        }

        match response_receiver.recv_timeout(Duration::from_secs(3)) {
            Ok(scan_reset_response) => {
                if scan_reset_response.success {
                    self.app_state.element_scanner_pane_state.has_scan_results = false;
                    self.app_state.element_scanner_pane_state.last_result_count = 0;
                    self.app_state
                        .element_scanner_pane_state
                        .last_total_size_in_bytes = 0;
                    self.app_state.scan_results_pane_state.clear_results();
                    self.app_state.element_scanner_pane_state.status_message = "Scan state reset.".to_string();
                } else {
                    self.app_state.element_scanner_pane_state.status_message = "Scan reset request failed.".to_string();
                }
            }
            Err(receive_error) => {
                self.app_state.element_scanner_pane_state.status_message = format!("Timed out waiting for scan reset response: {}", receive_error);
            }
        }

        self.app_state
            .element_scanner_pane_state
            .has_pending_scan_request = false;
    }

    pub(super) fn collect_scan_values(
        &mut self,
        squalr_engine: &mut SqualrEngine,
    ) {
        if self
            .app_state
            .element_scanner_pane_state
            .has_pending_scan_request
        {
            self.app_state.element_scanner_pane_state.status_message = "Scan request already in progress.".to_string();
            return;
        }

        let engine_unprivileged_state = match squalr_engine.get_engine_unprivileged_state().as_ref() {
            Some(engine_unprivileged_state) => engine_unprivileged_state,
            None => {
                self.app_state.element_scanner_pane_state.status_message = "No unprivileged engine state is available for value collection.".to_string();
                return;
            }
        };

        self.app_state
            .element_scanner_pane_state
            .has_pending_scan_request = true;
        self.app_state.element_scanner_pane_state.status_message = "Collecting scan values.".to_string();

        let scan_collect_values_request = ScanCollectValuesRequest {};
        let (response_sender, response_receiver) = mpsc::sync_channel(1);
        let request_dispatched = scan_collect_values_request.send(engine_unprivileged_state, move |scan_collect_values_response| {
            let _ = response_sender.send(scan_collect_values_response);
        });

        if !request_dispatched {
            self.app_state
                .element_scanner_pane_state
                .has_pending_scan_request = false;
            self.app_state.element_scanner_pane_state.status_message = "Failed to dispatch scan collect values request.".to_string();
            return;
        }

        match response_receiver.recv_timeout(Duration::from_secs(3)) {
            Ok(scan_collect_values_response) => {
                self.app_state.element_scanner_pane_state.last_result_count = scan_collect_values_response.scan_results_metadata.result_count;
                self.app_state
                    .element_scanner_pane_state
                    .last_total_size_in_bytes = scan_collect_values_response
                    .scan_results_metadata
                    .total_size_in_bytes;
                self.app_state.element_scanner_pane_state.status_message = format!(
                    "Collected values for {} results.",
                    scan_collect_values_response.scan_results_metadata.result_count
                );
            }
            Err(receive_error) => {
                self.app_state.element_scanner_pane_state.status_message = format!("Timed out waiting for collect values response: {}", receive_error);
            }
        }

        self.app_state
            .element_scanner_pane_state
            .has_pending_scan_request = false;
    }

    pub(super) fn start_element_scan(
        &mut self,
        squalr_engine: &mut SqualrEngine,
    ) {
        if self
            .app_state
            .element_scanner_pane_state
            .has_pending_scan_request
        {
            self.app_state.element_scanner_pane_state.status_message = "Scan request already in progress.".to_string();
            return;
        }

        let engine_unprivileged_state = match squalr_engine.get_engine_unprivileged_state().as_ref() {
            Some(engine_unprivileged_state) => engine_unprivileged_state,
            None => {
                self.app_state.element_scanner_pane_state.status_message = "No unprivileged engine state is available for element scanning.".to_string();
                return;
            }
        };

        self.app_state
            .element_scanner_pane_state
            .has_pending_scan_request = true;
        self.app_state.element_scanner_pane_state.status_message = "Starting scan.".to_string();

        if !self.app_state.element_scanner_pane_state.has_scan_results {
            let scan_new_request = ScanNewRequest {};
            let (response_sender, response_receiver) = mpsc::sync_channel(1);
            let request_dispatched = scan_new_request.send(engine_unprivileged_state, move |scan_new_response| {
                let _ = response_sender.send(scan_new_response);
            });

            if !request_dispatched {
                self.app_state
                    .element_scanner_pane_state
                    .has_pending_scan_request = false;
                self.app_state.element_scanner_pane_state.status_message = "Failed to dispatch new scan request.".to_string();
                return;
            }

            if let Err(receive_error) = response_receiver.recv_timeout(Duration::from_secs(3)) {
                self.app_state
                    .element_scanner_pane_state
                    .has_pending_scan_request = false;
                self.app_state.element_scanner_pane_state.status_message = format!("Timed out waiting for new scan response: {}", receive_error);
                return;
            }
        }

        let element_scan_request = ElementScanRequest {
            scan_constraints: self
                .app_state
                .element_scanner_pane_state
                .build_anonymous_scan_constraints(),
            data_type_refs: self
                .app_state
                .element_scanner_pane_state
                .selected_data_type_refs(),
        };

        let (response_sender, response_receiver) = mpsc::sync_channel(1);
        let request_dispatched = element_scan_request.send(engine_unprivileged_state, move |element_scan_response| {
            let _ = response_sender.send(element_scan_response);
        });

        if !request_dispatched {
            self.app_state
                .element_scanner_pane_state
                .has_pending_scan_request = false;
            self.app_state.element_scanner_pane_state.status_message = "Failed to dispatch element scan request.".to_string();
            return;
        }

        match response_receiver.recv_timeout(Duration::from_secs(3)) {
            Ok(element_scan_response) => {
                self.app_state.element_scanner_pane_state.has_scan_results = true;
                self.app_state.element_scanner_pane_state.last_result_count = element_scan_response.scan_results_metadata.result_count;
                self.app_state
                    .element_scanner_pane_state
                    .last_total_size_in_bytes = element_scan_response.scan_results_metadata.total_size_in_bytes;
                self.app_state.element_scanner_pane_state.status_message =
                    format!("Scan complete with {} results.", element_scan_response.scan_results_metadata.result_count);
                self.query_scan_results_current_page(squalr_engine);
            }
            Err(receive_error) => {
                self.app_state.element_scanner_pane_state.status_message = format!("Timed out waiting for element scan response: {}", receive_error);
            }
        }

        self.app_state
            .element_scanner_pane_state
            .has_pending_scan_request = false;
    }

    pub(super) fn query_scan_results_current_page(
        &mut self,
        squalr_engine: &mut SqualrEngine,
    ) {
        let _ = self.query_scan_results_current_page_with_feedback(squalr_engine, true);
    }

    pub(super) fn query_scan_results_current_page_with_feedback(
        &mut self,
        squalr_engine: &mut SqualrEngine,
        should_update_status_message: bool,
    ) -> bool {
        if self.app_state.scan_results_pane_state.is_querying_scan_results {
            if should_update_status_message {
                self.app_state.scan_results_pane_state.status_message = "Scan results query already in progress.".to_string();
            }
            return false;
        }

        let engine_unprivileged_state = match squalr_engine.get_engine_unprivileged_state().as_ref() {
            Some(engine_unprivileged_state) => engine_unprivileged_state,
            None => {
                if should_update_status_message {
                    self.app_state.scan_results_pane_state.status_message = "No unprivileged engine state is available for scan results query.".to_string();
                }
                return false;
            }
        };

        self.app_state.scan_results_pane_state.is_querying_scan_results = true;
        if should_update_status_message {
            self.app_state.scan_results_pane_state.status_message = format!(
                "Querying scan results page {}.",
                self.app_state
                    .scan_results_pane_state
                    .current_page_index
                    .saturating_add(1)
            );
        }

        let page_index = self.app_state.scan_results_pane_state.current_page_index;
        self.sync_scan_results_type_filters_from_element_scanner();
        let scan_results_query_request = ScanResultsQueryRequest { page_index };
        let (response_sender, response_receiver) = mpsc::sync_channel(1);
        let request_dispatched = scan_results_query_request.send(engine_unprivileged_state, move |scan_results_query_response| {
            let _ = response_sender.send(scan_results_query_response);
        });

        if !request_dispatched {
            self.app_state.scan_results_pane_state.is_querying_scan_results = false;
            if should_update_status_message {
                self.app_state.scan_results_pane_state.status_message = "Failed to dispatch scan results query request.".to_string();
            }
            return false;
        }

        match response_receiver.recv_timeout(Duration::from_secs(3)) {
            Ok(scan_results_query_response) => {
                self.apply_scan_results_query_response(scan_results_query_response, should_update_status_message);
            }
            Err(receive_error) => {
                if should_update_status_message {
                    self.app_state.scan_results_pane_state.status_message = format!("Timed out waiting for scan results query response: {}", receive_error);
                }
            }
        }

        self.app_state.scan_results_pane_state.is_querying_scan_results = false;
        true
    }

    pub(super) fn query_next_scan_results_page(
        &mut self,
        squalr_engine: &mut SqualrEngine,
    ) {
        let current_page_index = self.app_state.scan_results_pane_state.current_page_index;
        let target_page_index = current_page_index.saturating_add(1);

        if self
            .app_state
            .scan_results_pane_state
            .set_current_page_index(target_page_index)
        {
            self.query_scan_results_current_page(squalr_engine);
        }
    }

    pub(super) fn query_previous_scan_results_page(
        &mut self,
        squalr_engine: &mut SqualrEngine,
    ) {
        let current_page_index = self.app_state.scan_results_pane_state.current_page_index;
        let target_page_index = current_page_index.saturating_sub(1);

        if self
            .app_state
            .scan_results_pane_state
            .set_current_page_index(target_page_index)
        {
            self.query_scan_results_current_page(squalr_engine);
        }
    }

    pub(super) fn refresh_scan_results_page(
        &mut self,
        squalr_engine: &mut SqualrEngine,
    ) {
        let _ = self.refresh_scan_results_page_with_feedback(squalr_engine, true);
    }

    pub(super) fn refresh_scan_results_page_with_feedback(
        &mut self,
        squalr_engine: &mut SqualrEngine,
        should_update_status_message: bool,
    ) -> bool {
        if self
            .app_state
            .scan_results_pane_state
            .is_refreshing_scan_results
        {
            if should_update_status_message {
                self.app_state.scan_results_pane_state.status_message = "Scan results refresh already in progress.".to_string();
            }
            return false;
        }

        let scan_result_refs_for_current_page = self
            .app_state
            .scan_results_pane_state
            .all_scan_results
            .iter()
            .map(|scan_result| scan_result.get_base_result().get_scan_result_ref().clone())
            .collect::<Vec<_>>();
        if scan_result_refs_for_current_page.is_empty() {
            if should_update_status_message {
                self.app_state.scan_results_pane_state.status_message = "No scan results are available to refresh.".to_string();
            }
            return false;
        }

        let engine_unprivileged_state = match squalr_engine.get_engine_unprivileged_state().as_ref() {
            Some(engine_unprivileged_state) => engine_unprivileged_state,
            None => {
                if should_update_status_message {
                    self.app_state.scan_results_pane_state.status_message = "No unprivileged engine state is available for scan results refresh.".to_string();
                }
                return false;
            }
        };

        self.app_state
            .scan_results_pane_state
            .is_refreshing_scan_results = true;
        if should_update_status_message {
            self.app_state.scan_results_pane_state.status_message =
                format!("Refreshing {} scan results on the current page.", scan_result_refs_for_current_page.len());
        }

        let scan_results_refresh_request = ScanResultsRefreshRequest {
            scan_result_refs: scan_result_refs_for_current_page,
        };
        let (response_sender, response_receiver) = mpsc::sync_channel(1);
        let request_dispatched = scan_results_refresh_request.send(engine_unprivileged_state, move |scan_results_refresh_response| {
            let _ = response_sender.send(scan_results_refresh_response);
        });

        if !request_dispatched {
            self.app_state
                .scan_results_pane_state
                .is_refreshing_scan_results = false;
            if should_update_status_message {
                self.app_state.scan_results_pane_state.status_message = "Failed to dispatch scan results refresh request.".to_string();
            }
            return false;
        }

        match response_receiver.recv_timeout(Duration::from_secs(3)) {
            Ok(scan_results_refresh_response) => {
                let refreshed_result_count = scan_results_refresh_response.scan_results.len();
                self.app_state
                    .scan_results_pane_state
                    .apply_refreshed_results(scan_results_refresh_response.scan_results);
                if should_update_status_message {
                    self.app_state.scan_results_pane_state.status_message = format!("Refreshed {} scan results.", refreshed_result_count);
                }
                self.last_scan_results_periodic_refresh_time = Some(Instant::now());
            }
            Err(receive_error) => {
                if should_update_status_message {
                    self.app_state.scan_results_pane_state.status_message = format!("Timed out waiting for scan results refresh response: {}", receive_error);
                }
            }
        }

        self.app_state
            .scan_results_pane_state
            .is_refreshing_scan_results = false;
        true
    }

    pub(super) fn toggle_selected_scan_results_frozen_state(
        &mut self,
        squalr_engine: &mut SqualrEngine,
    ) {
        if self.app_state.scan_results_pane_state.is_freezing_scan_results {
            self.app_state.scan_results_pane_state.status_message = "Scan results freeze request already in progress.".to_string();
            return;
        }

        let selected_scan_result_refs = self
            .app_state
            .scan_results_pane_state
            .selected_scan_result_refs();
        if selected_scan_result_refs.is_empty() {
            self.app_state.scan_results_pane_state.status_message = "No scan results are selected to freeze/unfreeze.".to_string();
            return;
        }

        let target_frozen_state = !self
            .app_state
            .scan_results_pane_state
            .any_selected_result_frozen();
        let engine_unprivileged_state = match squalr_engine.get_engine_unprivileged_state().as_ref() {
            Some(engine_unprivileged_state) => engine_unprivileged_state,
            None => {
                self.app_state.scan_results_pane_state.status_message = "No unprivileged engine state is available for freeze toggles.".to_string();
                return;
            }
        };

        self.app_state.scan_results_pane_state.is_freezing_scan_results = true;
        self.app_state.scan_results_pane_state.status_message = if target_frozen_state {
            "Freezing selected scan results.".to_string()
        } else {
            "Unfreezing selected scan results.".to_string()
        };

        let scan_results_freeze_request = ScanResultsFreezeRequest {
            scan_result_refs: selected_scan_result_refs,
            is_frozen: target_frozen_state,
        };
        let (response_sender, response_receiver) = mpsc::sync_channel(1);
        let request_dispatched = scan_results_freeze_request.send(engine_unprivileged_state, move |scan_results_freeze_response| {
            let _ = response_sender.send(scan_results_freeze_response);
        });

        if !request_dispatched {
            self.app_state.scan_results_pane_state.is_freezing_scan_results = false;
            self.app_state.scan_results_pane_state.status_message = "Failed to dispatch scan results freeze request.".to_string();
            return;
        }

        match response_receiver.recv_timeout(Duration::from_secs(3)) {
            Ok(scan_results_freeze_response) => {
                let failed_toggle_count = scan_results_freeze_response
                    .failed_freeze_toggle_scan_result_refs
                    .len();
                self.app_state.scan_results_pane_state.status_message = if failed_toggle_count == 0 {
                    if target_frozen_state {
                        "Selected scan results frozen.".to_string()
                    } else {
                        "Selected scan results unfrozen.".to_string()
                    }
                } else {
                    format!("Freeze toggle partially failed for {} entries.", failed_toggle_count)
                };
                self.refresh_scan_results_page(squalr_engine);
            }
            Err(receive_error) => {
                self.app_state.scan_results_pane_state.status_message = format!("Timed out waiting for scan results freeze response: {}", receive_error);
            }
        }

        self.app_state.scan_results_pane_state.is_freezing_scan_results = false;
    }

    pub(super) fn add_selected_scan_results_to_project(
        &mut self,
        squalr_engine: &mut SqualrEngine,
    ) {
        if self
            .app_state
            .scan_results_pane_state
            .is_adding_scan_results_to_project
        {
            self.app_state.scan_results_pane_state.status_message = "Add to project request already in progress.".to_string();
            return;
        }

        let selected_scan_result_refs = self
            .app_state
            .scan_results_pane_state
            .selected_scan_result_refs();
        if selected_scan_result_refs.is_empty() {
            self.app_state.scan_results_pane_state.status_message = "No scan results are selected to add to project.".to_string();
            return;
        }

        let engine_unprivileged_state = match squalr_engine.get_engine_unprivileged_state().as_ref() {
            Some(engine_unprivileged_state) => engine_unprivileged_state,
            None => {
                self.app_state.scan_results_pane_state.status_message = "No unprivileged engine state is available for project item creation.".to_string();
                return;
            }
        };

        self.app_state
            .scan_results_pane_state
            .is_adding_scan_results_to_project = true;
        self.app_state.scan_results_pane_state.status_message = format!("Adding {} scan results to project.", selected_scan_result_refs.len());

        let project_items_add_request = ProjectItemsAddRequest {
            scan_result_refs: selected_scan_result_refs,
            target_directory_path: None,
        };
        let (response_sender, response_receiver) = mpsc::sync_channel(1);
        project_items_add_request.send(engine_unprivileged_state, move |project_items_add_response| {
            let _ = response_sender.send(project_items_add_response);
        });

        match response_receiver.recv_timeout(Duration::from_secs(3)) {
            Ok(project_items_add_response) => {
                self.app_state.scan_results_pane_state.status_message = if project_items_add_response.success {
                    format!(
                        "Added {} project items from selected scan results.",
                        project_items_add_response.added_project_item_count
                    )
                } else {
                    "Add-to-project request failed.".to_string()
                };
            }
            Err(receive_error) => {
                self.app_state.scan_results_pane_state.status_message = format!("Timed out waiting for add-to-project response: {}", receive_error);
            }
        }

        self.app_state
            .scan_results_pane_state
            .is_adding_scan_results_to_project = false;
    }

    pub(super) fn delete_selected_scan_results(
        &mut self,
        squalr_engine: &mut SqualrEngine,
    ) {
        if self.app_state.scan_results_pane_state.is_deleting_scan_results {
            self.app_state.scan_results_pane_state.status_message = "Delete request already in progress.".to_string();
            return;
        }

        let selected_scan_result_refs = self
            .app_state
            .scan_results_pane_state
            .selected_scan_result_refs();
        if selected_scan_result_refs.is_empty() {
            self.app_state.scan_results_pane_state.status_message = "No scan results are selected to delete.".to_string();
            return;
        }

        let engine_unprivileged_state = match squalr_engine.get_engine_unprivileged_state().as_ref() {
            Some(engine_unprivileged_state) => engine_unprivileged_state,
            None => {
                self.app_state.scan_results_pane_state.status_message = "No unprivileged engine state is available for deletion.".to_string();
                return;
            }
        };

        self.app_state.scan_results_pane_state.is_deleting_scan_results = true;
        self.app_state.scan_results_pane_state.status_message = format!("Deleting {} selected scan results.", selected_scan_result_refs.len());

        let scan_results_delete_request = ScanResultsDeleteRequest {
            scan_result_refs: selected_scan_result_refs,
        };
        let (response_sender, response_receiver) = mpsc::sync_channel(1);
        let request_dispatched = scan_results_delete_request.send(engine_unprivileged_state, move |scan_results_delete_response| {
            let _ = response_sender.send(scan_results_delete_response);
        });

        if !request_dispatched {
            self.app_state.scan_results_pane_state.is_deleting_scan_results = false;
            self.app_state.scan_results_pane_state.status_message = "Failed to dispatch scan results delete request.".to_string();
            return;
        }

        match response_receiver.recv_timeout(Duration::from_secs(3)) {
            Ok(_scan_results_delete_response) => {
                self.app_state.scan_results_pane_state.status_message = "Deleted selected scan results.".to_string();
                self.query_scan_results_current_page(squalr_engine);
            }
            Err(receive_error) => {
                self.app_state.scan_results_pane_state.status_message = format!("Timed out waiting for scan results delete response: {}", receive_error);
            }
        }

        self.app_state.scan_results_pane_state.is_deleting_scan_results = false;
    }

    pub(super) fn commit_selected_scan_results_value_edit(
        &mut self,
        squalr_engine: &mut SqualrEngine,
    ) {
        if self.app_state.scan_results_pane_state.is_committing_value_edit {
            self.app_state.scan_results_pane_state.status_message = "Value commit request already in progress.".to_string();
            return;
        }

        let selected_scan_result_refs = self
            .app_state
            .scan_results_pane_state
            .selected_scan_result_refs();
        if selected_scan_result_refs.is_empty() {
            self.app_state.scan_results_pane_state.status_message = "No scan results are selected to commit value edits.".to_string();
            return;
        }

        let pending_value_edit_text = self
            .app_state
            .scan_results_pane_state
            .pending_value_edit_text
            .trim()
            .to_string();
        if pending_value_edit_text.is_empty() {
            self.app_state.scan_results_pane_state.status_message = "Edit value is empty.".to_string();
            return;
        }

        let engine_unprivileged_state = match squalr_engine.get_engine_unprivileged_state().as_ref() {
            Some(engine_unprivileged_state) => engine_unprivileged_state,
            None => {
                self.app_state.scan_results_pane_state.status_message = "No unprivileged engine state is available for value commits.".to_string();
                return;
            }
        };

        self.app_state.scan_results_pane_state.is_committing_value_edit = true;
        self.app_state.scan_results_pane_state.status_message = format!(
            "Committing value edit '{}' for {} selected results.",
            pending_value_edit_text,
            selected_scan_result_refs.len()
        );

        let scan_results_set_property_request = ScanResultsSetPropertyRequest {
            scan_result_refs: selected_scan_result_refs,
            anonymous_value_string: AnonymousValueString::new(pending_value_edit_text, AnonymousValueStringFormat::Decimal, ContainerType::None),
            field_namespace: ScanResult::PROPERTY_NAME_VALUE.to_string(),
        };
        let (response_sender, response_receiver) = mpsc::sync_channel(1);
        let request_dispatched = scan_results_set_property_request.send(engine_unprivileged_state, move |scan_results_set_property_response| {
            let _ = response_sender.send(scan_results_set_property_response);
        });

        if !request_dispatched {
            self.app_state.scan_results_pane_state.is_committing_value_edit = false;
            self.app_state.scan_results_pane_state.status_message = "Failed to dispatch scan results set property request.".to_string();
            return;
        }

        match response_receiver.recv_timeout(Duration::from_secs(3)) {
            Ok(_scan_results_set_property_response) => {
                self.app_state.scan_results_pane_state.status_message = "Committed selected scan result values.".to_string();
                self.refresh_scan_results_page(squalr_engine);
            }
            Err(receive_error) => {
                self.app_state.scan_results_pane_state.status_message = format!("Timed out waiting for scan results set property response: {}", receive_error);
            }
        }

        self.app_state.scan_results_pane_state.is_committing_value_edit = false;
    }

    pub(super) fn apply_scan_results_query_response(
        &mut self,
        scan_results_query_response: squalr_engine_api::commands::scan_results::query::scan_results_query_response::ScanResultsQueryResponse,
        should_update_status_message: bool,
    ) {
        let result_count = scan_results_query_response.result_count;
        let page_index = scan_results_query_response.page_index;
        self.app_state
            .scan_results_pane_state
            .apply_query_response(scan_results_query_response);
        if should_update_status_message {
            self.app_state.scan_results_pane_state.status_message = format!("Loaded page {} ({} total results).", page_index.saturating_add(1), result_count);
        }
    }

    pub(super) fn sync_scan_results_type_filters_from_element_scanner(&mut self) {
        let selected_data_type_ids = self
            .app_state
            .element_scanner_pane_state
            .selected_data_type_ids()
            .iter()
            .map(|selected_data_type_id| (*selected_data_type_id).to_string())
            .collect();
        self.app_state
            .scan_results_pane_state
            .set_filtered_data_type_ids(selected_data_type_ids);
    }

    pub(super) fn refresh_process_list(
        &mut self,
        squalr_engine: &mut SqualrEngine,
    ) {
        self.refresh_process_list_with_feedback(squalr_engine, true);
    }

    pub(super) fn refresh_process_list_with_feedback(
        &mut self,
        squalr_engine: &mut SqualrEngine,
        should_update_status_message: bool,
    ) {
        if self
            .app_state
            .process_selector_pane_state
            .is_awaiting_process_list_response
        {
            if should_update_status_message {
                self.app_state.process_selector_pane_state.status_message = "Process list request already in progress.".to_string();
            }
            return;
        }

        let engine_unprivileged_state = match squalr_engine.get_engine_unprivileged_state().as_ref() {
            Some(engine_unprivileged_state) => engine_unprivileged_state,
            None => {
                if should_update_status_message {
                    self.app_state.process_selector_pane_state.status_message = "No unprivileged engine state is available for process queries.".to_string();
                }
                return;
            }
        };

        self.app_state
            .process_selector_pane_state
            .is_awaiting_process_list_response = true;
        if should_update_status_message {
            self.app_state.process_selector_pane_state.status_message = "Refreshing process list.".to_string();
        }

        let process_list_request = ProcessListRequest {
            require_windowed: self
                .app_state
                .process_selector_pane_state
                .show_windowed_processes_only,
            search_name: None,
            match_case: false,
            limit: None,
            fetch_icons: false,
        };

        let (response_sender, response_receiver) = mpsc::sync_channel(1);
        let request_dispatched = process_list_request.send(engine_unprivileged_state, move |process_list_response| {
            let _ = response_sender.send(process_list_response);
        });

        if !request_dispatched {
            self.app_state
                .process_selector_pane_state
                .is_awaiting_process_list_response = false;
            if should_update_status_message {
                self.app_state.process_selector_pane_state.status_message = "Failed to dispatch process list request.".to_string();
            }
            return;
        }

        match response_receiver.recv_timeout(Duration::from_secs(3)) {
            Ok(process_list_response) => {
                let process_count = process_list_response.processes.len();
                self.app_state
                    .process_selector_pane_state
                    .apply_process_list(process_list_response.processes);
                self.app_state
                    .process_selector_pane_state
                    .has_loaded_process_list_once = true;
                if should_update_status_message {
                    self.app_state.process_selector_pane_state.status_message = format!("Loaded {} processes.", process_count);
                }
            }
            Err(receive_error) => {
                if should_update_status_message {
                    self.app_state.process_selector_pane_state.status_message = format!("Timed out waiting for process list response: {}", receive_error);
                }
            }
        }

        self.app_state
            .process_selector_pane_state
            .is_awaiting_process_list_response = false;
    }

    pub(super) fn open_selected_process(
        &mut self,
        squalr_engine: &mut SqualrEngine,
    ) {
        if self.app_state.process_selector_pane_state.is_opening_process {
            self.app_state.process_selector_pane_state.status_message = "Process open request already in progress.".to_string();
            return;
        }

        let selected_process_identifier = match self.app_state.process_selector_pane_state.selected_process_id() {
            Some(selected_process_identifier) => selected_process_identifier,
            None => {
                self.app_state.process_selector_pane_state.status_message = "No process is selected.".to_string();
                return;
            }
        };

        let engine_unprivileged_state = match squalr_engine.get_engine_unprivileged_state().as_ref() {
            Some(engine_unprivileged_state) => engine_unprivileged_state,
            None => {
                self.app_state.process_selector_pane_state.status_message = "No unprivileged engine state is available for process opening.".to_string();
                return;
            }
        };

        self.app_state.process_selector_pane_state.is_opening_process = true;
        self.app_state.process_selector_pane_state.status_message = format!("Opening process {}.", selected_process_identifier);

        let process_open_request = ProcessOpenRequest {
            process_id: Some(selected_process_identifier),
            search_name: None,
            match_case: false,
        };

        let (response_sender, response_receiver) = mpsc::sync_channel(1);
        let request_dispatched = process_open_request.send(engine_unprivileged_state, move |process_open_response| {
            let _ = response_sender.send(process_open_response);
        });

        if !request_dispatched {
            self.app_state.process_selector_pane_state.is_opening_process = false;
            self.app_state.process_selector_pane_state.status_message = "Failed to dispatch process open request.".to_string();
            return;
        }

        match response_receiver.recv_timeout(Duration::from_secs(3)) {
            Ok(process_open_response) => {
                let opened_process = process_open_response.opened_process_info;
                self.app_state
                    .process_selector_pane_state
                    .set_opened_process(opened_process.clone());
                self.app_state.process_selector_pane_state.status_message = if let Some(opened_process_info) = opened_process {
                    self.apply_project_workspace_process_context(squalr_engine, opened_process_info.get_name());
                    format!(
                        "Opened process {} ({}).",
                        opened_process_info.get_name(),
                        opened_process_info.get_process_id_raw()
                    )
                } else {
                    "Open process request completed with no process.".to_string()
                };
            }
            Err(receive_error) => {
                self.app_state.process_selector_pane_state.status_message = format!("Timed out waiting for process open response: {}", receive_error);
            }
        }

        self.app_state.process_selector_pane_state.is_opening_process = false;
    }

    fn apply_project_workspace_process_context(
        &mut self,
        squalr_engine: &mut SqualrEngine,
        opened_process_name: &str,
    ) {
        self.app_state
            .set_active_workspace_page(TuiWorkspacePage::ProjectWorkspace);
        self.app_state.process_selector_pane_state.commit_search_input();
        if !self.has_auto_seeked_project_explorer_once {
            self.app_state
                .process_selector_pane_state
                .activate_project_explorer_view();
            self.app_state.set_focused_pane(TuiPane::ProjectExplorer);
            self.has_auto_seeked_project_explorer_once = true;
        }

        let has_active_project = self
            .app_state
            .project_explorer_pane_state
            .active_project_directory_path
            .is_some();

        if has_active_project {
            self.app_state.project_explorer_pane_state.status_message =
                format!("Process '{}' opened. Project hierarchy is active for item workflows.", opened_process_name);
            self.refresh_project_items_list_with_feedback(squalr_engine, false);
        } else {
            self.app_state.project_explorer_pane_state.status_message =
                format!("Process '{}' opened. Open or create a project to persist addresses.", opened_process_name);
        }
    }
}
