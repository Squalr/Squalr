use super::app_shell::AppShell;
use anyhow::Result;
use squalr_engine::squalr_engine::SqualrEngine;
use squalr_engine_api::commands::memory::query::memory_query_request::MemoryQueryRequest;
use squalr_engine_api::commands::privileged_command_request::PrivilegedCommandRequest;
use squalr_engine_api::structures::projects::project_items::built_in_types::project_item_type_address::ProjectItemTypeAddress;
use squalr_engine_session::engine_unprivileged_state::EngineUnprivilegedState;
use std::sync::{Arc, mpsc};
use std::time::Duration;

impl AppShell {
    pub(super) fn refresh_memory_viewer_pages(
        &mut self,
        squalr_engine: &mut SqualrEngine,
    ) {
        self.refresh_memory_viewer_pages_with_feedback(squalr_engine, true);
    }

    pub(super) fn refresh_memory_viewer_pages_with_feedback(
        &mut self,
        squalr_engine: &mut SqualrEngine,
        should_update_status_message: bool,
    ) {
        if self.app_state.memory_viewer_pane_state.is_querying_memory_pages {
            if should_update_status_message {
                self.app_state.memory_viewer_pane_state.status_message = String::from("Memory page refresh already in progress.");
            }
            return;
        }

        let Some(engine_unprivileged_state) = squalr_engine.get_engine_unprivileged_state().as_ref() else {
            if should_update_status_message {
                self.app_state.memory_viewer_pane_state.status_message = String::from("No unprivileged engine state is available for memory page queries.");
            }
            return;
        };

        self.app_state.memory_viewer_pane_state.is_querying_memory_pages = true;
        if should_update_status_message {
            self.app_state.memory_viewer_pane_state.status_message = String::from("Refreshing memory pages.");
        }

        let selected_page_base_address = self
            .app_state
            .memory_viewer_pane_state
            .current_page_base_address();
        let memory_query_request = MemoryQueryRequest {
            page_retrieval_mode: self.app_state.memory_viewer_pane_state.page_retrieval_mode,
        };
        let (response_sender, response_receiver) = mpsc::sync_channel(1);
        let request_dispatched = memory_query_request.send(engine_unprivileged_state, move |memory_query_response| {
            let _ = response_sender.send(memory_query_response);
        });
        if !request_dispatched {
            self.app_state.memory_viewer_pane_state.is_querying_memory_pages = false;
            self.app_state.memory_viewer_pane_state.status_message = String::from("Failed to dispatch memory page query.");
            return;
        }

        match response_receiver.recv_timeout(Duration::from_secs(3)) {
            Ok(memory_query_response) => {
                self.app_state.memory_viewer_pane_state.is_querying_memory_pages = false;
                if !memory_query_response.success {
                    self.app_state.memory_viewer_pane_state.status_message = String::from("Memory page query failed.");
                    return;
                }

                let page_count = memory_query_response.virtual_pages.len();
                self.app_state
                    .memory_viewer_pane_state
                    .refresh_pages_from_response(memory_query_response.virtual_pages, memory_query_response.modules, selected_page_base_address);
                self.app_state.memory_viewer_pane_state.status_message = format!("Loaded {} memory pages.", page_count);
                self.sync_memory_viewer_virtual_snapshot(engine_unprivileged_state.clone());
            }
            Err(receive_error) => {
                self.app_state.memory_viewer_pane_state.is_querying_memory_pages = false;
                self.app_state.memory_viewer_pane_state.status_message = format!("Timed out waiting for memory page query response: {}", receive_error);
            }
        }
    }

    pub(super) fn sync_memory_viewer_virtual_snapshot(
        &mut self,
        engine_unprivileged_state: Arc<EngineUnprivilegedState>,
    ) {
        let memory_viewer_queries = self
            .app_state
            .memory_viewer_pane_state
            .build_visible_chunk_queries();
        engine_unprivileged_state.set_virtual_snapshot_queries(
            crate::views::memory_viewer::pane_state::MemoryViewerPaneState::VIRTUAL_SNAPSHOT_ID,
            crate::views::memory_viewer::pane_state::MemoryViewerPaneState::SNAPSHOT_REFRESH_INTERVAL,
            memory_viewer_queries,
        );
        engine_unprivileged_state.request_virtual_snapshot_refresh(crate::views::memory_viewer::pane_state::MemoryViewerPaneState::VIRTUAL_SNAPSHOT_ID);

        if let Some(virtual_snapshot) =
            engine_unprivileged_state.get_virtual_snapshot(crate::views::memory_viewer::pane_state::MemoryViewerPaneState::VIRTUAL_SNAPSHOT_ID)
        {
            self.app_state
                .memory_viewer_pane_state
                .apply_virtual_snapshot_results(&virtual_snapshot);
        }
    }

    pub(super) fn clear_memory_viewer_for_process_change(
        &mut self,
        engine_unprivileged_state: &Arc<EngineUnprivilegedState>,
    ) {
        self.app_state
            .memory_viewer_pane_state
            .clear_for_process_change(engine_unprivileged_state);
    }

    pub(super) fn sync_memory_viewer_on_tick(
        &mut self,
        squalr_engine: &mut SqualrEngine,
    ) -> Result<()> {
        let Some(engine_unprivileged_state) = squalr_engine.get_engine_unprivileged_state().clone() else {
            return Ok(());
        };

        if self.app_state.active_workspace_page() != crate::state::workspace_page::TuiWorkspacePage::MemoryWorkspace {
            return Ok(());
        }

        if !self
            .app_state
            .memory_viewer_pane_state
            .has_loaded_memory_pages_once
            && !self.app_state.memory_viewer_pane_state.is_querying_memory_pages
        {
            self.refresh_memory_viewer_pages_with_feedback(squalr_engine, false);
        }

        if self
            .app_state
            .memory_viewer_pane_state
            .has_loaded_memory_pages_once
        {
            self.sync_memory_viewer_virtual_snapshot(engine_unprivileged_state);
        }

        Ok(())
    }

    pub(super) fn open_memory_viewer_for_selected_project_item(
        &mut self,
        squalr_engine: &mut SqualrEngine,
    ) {
        let Some(selected_project_item_path) = self
            .app_state
            .project_explorer_pane_state
            .selected_project_item_path()
        else {
            self.app_state.project_explorer_pane_state.status_message = String::from("No project item is selected for memory viewer focus.");
            return;
        };
        let Some(project_item) = self
            .app_state
            .project_explorer_pane_state
            .opened_project_item(&selected_project_item_path)
            .cloned()
        else {
            self.app_state.project_explorer_pane_state.status_message = String::from("Selected project item is no longer available.");
            return;
        };

        if project_item.get_item_type().get_project_item_type_id() != ProjectItemTypeAddress::PROJECT_ITEM_TYPE_ID {
            self.app_state.project_explorer_pane_state.status_message = String::from("Only address project items can open in the TUI memory viewer right now.");
            return;
        }

        let mut project_item = project_item;
        let address = ProjectItemTypeAddress::get_field_address(&mut project_item);
        let module_name = ProjectItemTypeAddress::get_field_module(&mut project_item);

        self.app_state
            .set_active_workspace_page(crate::state::workspace_page::TuiWorkspacePage::MemoryWorkspace);
        self.app_state
            .set_focused_pane(crate::state::pane::TuiPane::MemoryViewer);
        if !self
            .app_state
            .memory_viewer_pane_state
            .focus_address(address, &module_name)
        {
            self.refresh_memory_viewer_pages_with_feedback(squalr_engine, false);
            let _ = self
                .app_state
                .memory_viewer_pane_state
                .focus_address(address, &module_name);
        }
    }

    pub(super) fn open_memory_viewer_for_selected_rooted_symbol(
        &mut self,
        squalr_engine: &mut SqualrEngine,
    ) {
        let Some(selected_rooted_symbol) = self
            .app_state
            .project_explorer_pane_state
            .selected_rooted_symbol()
            .cloned()
        else {
            self.app_state.project_explorer_pane_state.status_message = String::from("No rooted symbol is selected for memory viewer focus.");
            return;
        };
        let address = selected_rooted_symbol.get_root_locator().get_focus_address();
        let module_name = selected_rooted_symbol
            .get_root_locator()
            .get_focus_module_name()
            .to_string();

        self.app_state
            .set_active_workspace_page(crate::state::workspace_page::TuiWorkspacePage::MemoryWorkspace);
        self.app_state
            .set_focused_pane(crate::state::pane::TuiPane::MemoryViewer);
        if !self
            .app_state
            .memory_viewer_pane_state
            .focus_address(address, &module_name)
        {
            self.refresh_memory_viewer_pages_with_feedback(squalr_engine, false);
            let _ = self
                .app_state
                .memory_viewer_pane_state
                .focus_address(address, &module_name);
        }
    }
}
