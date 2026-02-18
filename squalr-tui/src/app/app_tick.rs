use super::app_shell::AppShell;
use squalr_engine::squalr_engine::SqualrEngine;
use squalr_engine_api::engine::engine_execution_context::EngineExecutionContext;
use squalr_engine_api::events::process::changed::process_changed_event::ProcessChangedEvent;
use squalr_engine_api::events::scan_results::updated::scan_results_updated_event::ScanResultsUpdatedEvent;
use squalr_engine_api::structures::processes::opened_process_info::OpenedProcessInfo;
use std::path::PathBuf;
use std::sync::atomic::Ordering;
use std::time::{Duration, Instant};

impl AppShell {
    pub(super) fn on_tick(
        &mut self,
        squalr_engine: &mut SqualrEngine,
    ) {
        let current_tick_time = Instant::now();
        self.synchronize_active_project_from_engine_state(squalr_engine);
        self.register_scan_results_updated_listener_if_needed(squalr_engine);
        self.register_process_changed_listener_if_needed(squalr_engine);
        let _did_synchronize_opened_process = self.synchronize_opened_process_from_engine_event_if_pending();
        let did_requery_after_scan_results_update = self.query_scan_results_page_if_engine_event_pending(squalr_engine);
        if !did_requery_after_scan_results_update {
            self.refresh_scan_results_on_interval_if_eligible(squalr_engine);
        }

        self.refresh_output_log_history(squalr_engine);

        if self.should_refresh_process_list_on_tick(current_tick_time) {
            self.last_process_list_auto_refresh_attempt_time = Some(current_tick_time);
            self.refresh_process_list_with_feedback(squalr_engine, false);
        }

        if self.should_refresh_project_list_on_tick(current_tick_time) {
            self.last_project_list_auto_refresh_attempt_time = Some(current_tick_time);
            self.refresh_project_list_with_feedback(squalr_engine, false);
        }

        if self.should_refresh_project_items_list_on_tick(current_tick_time) {
            self.last_project_items_auto_refresh_attempt_time = Some(current_tick_time);
            self.refresh_project_items_list_with_feedback(squalr_engine, false);
        }

        self.refresh_settings_on_tick_if_eligible(squalr_engine);
    }

    pub(super) fn synchronize_active_project_from_engine_state(
        &mut self,
        squalr_engine: &mut SqualrEngine,
    ) {
        let Some(engine_unprivileged_state) = squalr_engine.get_engine_unprivileged_state().as_ref() else {
            return;
        };

        let opened_project_lock = engine_unprivileged_state
            .get_project_manager()
            .get_opened_project();
        let (engine_active_project_name, engine_active_project_directory_path) = match opened_project_lock.read() {
            Ok(opened_project_read_guard) => match opened_project_read_guard.as_ref() {
                Some(opened_project) => (
                    Some(opened_project.get_project_info().get_name().to_string()),
                    opened_project.get_project_info().get_project_directory(),
                ),
                None => (None, None),
            },
            Err(lock_error) => {
                log::error!("Failed to acquire opened project lock for TUI project-state synchronization: {}", lock_error);
                return;
            }
        };

        self.apply_engine_active_project_state(engine_active_project_name, engine_active_project_directory_path);
    }

    pub(super) fn apply_engine_active_project_state(
        &mut self,
        engine_active_project_name: Option<String>,
        engine_active_project_directory_path: Option<PathBuf>,
    ) {
        let did_active_project_change = self.app_state.project_explorer_pane_state.active_project_name != engine_active_project_name
            || self
                .app_state
                .project_explorer_pane_state
                .active_project_directory_path
                != engine_active_project_directory_path;
        if !did_active_project_change {
            return;
        }

        let did_active_project_directory_change = self
            .app_state
            .project_explorer_pane_state
            .active_project_directory_path
            != engine_active_project_directory_path;

        self.app_state
            .project_explorer_pane_state
            .set_active_project(engine_active_project_name, engine_active_project_directory_path);

        if did_active_project_directory_change {
            self.app_state.project_explorer_pane_state.clear_project_items();
            self.last_project_items_auto_refresh_attempt_time = None;
        }
    }

    pub(super) fn register_scan_results_updated_listener_if_needed(
        &mut self,
        squalr_engine: &mut SqualrEngine,
    ) {
        if self.has_registered_scan_results_updated_listener {
            return;
        }

        let Some(engine_unprivileged_state) = squalr_engine.get_engine_unprivileged_state().as_ref() else {
            return;
        };
        let scan_results_update_counter = self.scan_results_update_counter.clone();
        engine_unprivileged_state.listen_for_engine_event::<ScanResultsUpdatedEvent>(move |_scan_results_updated_event| {
            scan_results_update_counter.fetch_add(1, Ordering::Relaxed);
        });

        self.has_registered_scan_results_updated_listener = true;
    }

    pub(super) fn register_process_changed_listener_if_needed(
        &mut self,
        squalr_engine: &mut SqualrEngine,
    ) {
        if self.has_registered_process_changed_listener {
            return;
        }

        let Some(engine_unprivileged_state) = squalr_engine.get_engine_unprivileged_state().as_ref() else {
            return;
        };
        let process_changed_update_counter = self.process_changed_update_counter.clone();
        let pending_opened_process_from_event = self.pending_opened_process_from_event.clone();
        engine_unprivileged_state.listen_for_engine_event::<ProcessChangedEvent>(move |process_changed_event| {
            if let Ok(mut pending_opened_process_guard) = pending_opened_process_from_event.write() {
                *pending_opened_process_guard = process_changed_event.process_info.clone();
                process_changed_update_counter.fetch_add(1, Ordering::Relaxed);
            } else {
                log::error!("Failed to acquire process event lock in TUI process-change listener.");
            }
        });

        self.has_registered_process_changed_listener = true;
    }

    pub(super) fn synchronize_opened_process_from_engine_event_if_pending(&mut self) -> bool {
        let latest_process_changed_update_counter = self.process_changed_update_counter.load(Ordering::Relaxed);
        if latest_process_changed_update_counter == self.consumed_process_changed_update_counter {
            return false;
        }

        let pending_opened_process_from_event = match self.pending_opened_process_from_event.read() {
            Ok(pending_opened_process_guard) => pending_opened_process_guard.clone(),
            Err(lock_error) => {
                log::error!("Failed to acquire process event lock for TUI process-state synchronization: {}", lock_error);
                return false;
            }
        };

        self.apply_engine_opened_process_state(pending_opened_process_from_event);
        self.consumed_process_changed_update_counter = latest_process_changed_update_counter;
        true
    }

    pub(super) fn apply_engine_opened_process_state(
        &mut self,
        opened_process: Option<OpenedProcessInfo>,
    ) {
        let has_opened_process = opened_process.is_some();
        self.app_state
            .process_selector_pane_state
            .set_opened_process(opened_process);
        if !has_opened_process {
            self.app_state
                .process_selector_pane_state
                .activate_process_selector_view();
        }
    }

    pub(super) fn query_scan_results_page_if_engine_event_pending(
        &mut self,
        squalr_engine: &mut SqualrEngine,
    ) -> bool {
        let latest_scan_results_update_counter = self.scan_results_update_counter.load(Ordering::Relaxed);
        if latest_scan_results_update_counter == self.consumed_scan_results_update_counter {
            return false;
        }
        if self.app_state.scan_results_pane_state.is_querying_scan_results {
            return false;
        }

        self.consumed_scan_results_update_counter = latest_scan_results_update_counter;
        let _ = self.query_scan_results_current_page_with_feedback(squalr_engine, false);
        true
    }

    pub(super) fn refresh_scan_results_on_interval_if_eligible(
        &mut self,
        squalr_engine: &mut SqualrEngine,
    ) {
        let current_tick_time = Instant::now();
        if !self.should_refresh_scan_results_page_on_tick(current_tick_time) {
            return;
        }

        if self.refresh_scan_results_page_with_feedback(squalr_engine, false) {
            self.last_scan_results_periodic_refresh_time = Some(Instant::now());
        }
    }

    pub(super) fn refresh_settings_on_tick_if_eligible(
        &mut self,
        squalr_engine: &mut SqualrEngine,
    ) {
        let current_tick_time = Instant::now();
        if !self.should_refresh_settings_on_tick(current_tick_time) {
            return;
        }

        self.last_settings_auto_refresh_attempt_time = Some(current_tick_time);
        self.refresh_all_settings_categories_with_feedback(squalr_engine, false);
    }

    pub(super) fn should_refresh_settings_on_tick(
        &self,
        current_tick_time: Instant,
    ) -> bool {
        if self.app_state.settings_pane_state.has_loaded_settings_once || self.app_state.settings_pane_state.is_refreshing_settings {
            return false;
        }

        match self.last_settings_auto_refresh_attempt_time {
            Some(last_settings_auto_refresh_attempt_time) => {
                current_tick_time.duration_since(last_settings_auto_refresh_attempt_time) >= Duration::from_millis(Self::MIN_SETTINGS_AUTO_REFRESH_INTERVAL_MS)
            }
            None => true,
        }
    }

    pub(super) fn should_refresh_process_list_on_tick(
        &self,
        current_tick_time: Instant,
    ) -> bool {
        if self
            .app_state
            .process_selector_pane_state
            .has_loaded_process_list_once
            || self
                .app_state
                .process_selector_pane_state
                .is_awaiting_process_list_response
        {
            return false;
        }

        match self.last_process_list_auto_refresh_attempt_time {
            Some(last_process_list_auto_refresh_attempt_time) => {
                current_tick_time.duration_since(last_process_list_auto_refresh_attempt_time)
                    >= Duration::from_millis(Self::MIN_PROCESS_AND_PROJECT_AUTO_REFRESH_INTERVAL_MS)
            }
            None => true,
        }
    }

    pub(super) fn should_refresh_project_list_on_tick(
        &self,
        current_tick_time: Instant,
    ) -> bool {
        if self
            .app_state
            .project_explorer_pane_state
            .has_loaded_project_list_once
            || self
                .app_state
                .project_explorer_pane_state
                .is_awaiting_project_list_response
        {
            return false;
        }

        match self.last_project_list_auto_refresh_attempt_time {
            Some(last_project_list_auto_refresh_attempt_time) => {
                current_tick_time.duration_since(last_project_list_auto_refresh_attempt_time)
                    >= Duration::from_millis(Self::MIN_PROCESS_AND_PROJECT_AUTO_REFRESH_INTERVAL_MS)
            }
            None => true,
        }
    }

    pub(super) fn should_refresh_project_items_list_on_tick(
        &self,
        current_tick_time: Instant,
    ) -> bool {
        if self
            .app_state
            .project_explorer_pane_state
            .active_project_directory_path
            .is_none()
            || self
                .app_state
                .project_explorer_pane_state
                .is_awaiting_project_item_list_response
        {
            return false;
        }

        let project_items_refresh_interval = self.project_items_periodic_refresh_interval();
        match self.last_project_items_auto_refresh_attempt_time {
            Some(last_project_items_auto_refresh_attempt_time) => {
                current_tick_time.duration_since(last_project_items_auto_refresh_attempt_time) >= project_items_refresh_interval
            }
            None => true,
        }
    }

    pub(super) fn should_refresh_scan_results_page_on_tick(
        &self,
        current_tick_time: Instant,
    ) -> bool {
        if self
            .app_state
            .scan_results_pane_state
            .all_scan_results
            .is_empty()
        {
            return false;
        }
        if self.app_state.scan_results_pane_state.is_querying_scan_results
            || self
                .app_state
                .scan_results_pane_state
                .is_refreshing_scan_results
            || self.app_state.scan_results_pane_state.is_freezing_scan_results
            || self.app_state.scan_results_pane_state.is_deleting_scan_results
            || self.app_state.scan_results_pane_state.is_committing_value_edit
        {
            return false;
        }

        let refresh_interval = self.scan_results_periodic_refresh_interval();
        match self.last_scan_results_periodic_refresh_time {
            Some(last_scan_results_periodic_refresh_time) => current_tick_time.duration_since(last_scan_results_periodic_refresh_time) >= refresh_interval,
            None => true,
        }
    }

    pub(super) fn scan_results_periodic_refresh_interval(&self) -> Duration {
        let configured_results_read_interval_ms = self
            .app_state
            .settings_pane_state
            .scan_settings
            .results_read_interval_ms;
        let bounded_results_read_interval_ms =
            configured_results_read_interval_ms.clamp(Self::MIN_SCAN_RESULTS_REFRESH_INTERVAL_MS, Self::MAX_SCAN_RESULTS_REFRESH_INTERVAL_MS);

        Duration::from_millis(bounded_results_read_interval_ms)
    }

    pub(super) fn project_items_periodic_refresh_interval(&self) -> Duration {
        let configured_project_read_interval_ms = self
            .app_state
            .settings_pane_state
            .scan_settings
            .project_read_interval_ms;
        let bounded_project_read_interval_ms =
            configured_project_read_interval_ms.clamp(Self::MIN_PROJECT_ITEMS_REFRESH_INTERVAL_MS, Self::MAX_PROJECT_ITEMS_REFRESH_INTERVAL_MS);

        Duration::from_millis(bounded_project_read_interval_ms)
    }
}

#[cfg(test)]
mod tests {
    use super::AppShell;
    use std::path::PathBuf;
    use std::time::{Duration, Instant};

    #[test]
    fn project_item_refresh_does_not_stop_after_first_load() {
        let mut app_shell = AppShell::new(Duration::from_millis(16));
        app_shell
            .app_state
            .project_explorer_pane_state
            .active_project_directory_path = Some(PathBuf::from("C:/projects/test"));
        app_shell
            .app_state
            .project_explorer_pane_state
            .has_loaded_project_item_list_once = true;
        app_shell
            .app_state
            .settings_pane_state
            .scan_settings
            .project_read_interval_ms = 100;
        app_shell.last_project_items_auto_refresh_attempt_time = Some(Instant::now() - Duration::from_millis(200));

        let should_refresh = app_shell.should_refresh_project_items_list_on_tick(Instant::now());

        assert!(should_refresh);
    }
}
