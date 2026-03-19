use crate::ui::widgets::controls::data_type_selector::data_type_selection::DataTypeSelection;
use squalr_engine_api::commands::pointer_scan::expand::pointer_scan_expand_request::PointerScanExpandRequest;
use squalr_engine_api::commands::pointer_scan::expand::pointer_scan_expand_response::PointerScanExpandResponse;
use squalr_engine_api::commands::pointer_scan::reset::pointer_scan_reset_request::PointerScanResetRequest;
use squalr_engine_api::commands::pointer_scan::start::pointer_scan_start_request::PointerScanStartRequest;
use squalr_engine_api::commands::pointer_scan::summary::pointer_scan_summary_request::PointerScanSummaryRequest;
use squalr_engine_api::commands::pointer_scan::validate::pointer_scan_validate_request::PointerScanValidateRequest;
use squalr_engine_api::commands::privileged_command_request::PrivilegedCommandRequest;
use squalr_engine_api::commands::project_items::create::project_items_create_request::ProjectItemsCreateRequest;
use squalr_engine_api::dependency_injection::dependency::Dependency;
use squalr_engine_api::structures::data_types::data_type_ref::DataTypeRef;
use squalr_engine_api::structures::data_values::anonymous_value_string::AnonymousValueString;
use squalr_engine_api::structures::data_values::anonymous_value_string_format::AnonymousValueStringFormat;
use squalr_engine_api::structures::data_values::container_type::ContainerType;
use squalr_engine_api::structures::memory::bitness::Bitness;
use squalr_engine_api::structures::memory::pointer::Pointer;
use squalr_engine_api::structures::pointer_scans::pointer_scan_node::PointerScanNode;
use squalr_engine_api::structures::pointer_scans::pointer_scan_node_type::PointerScanNodeType;
use squalr_engine_api::structures::pointer_scans::pointer_scan_pointer_size::PointerScanPointerSize;
use squalr_engine_api::structures::pointer_scans::pointer_scan_summary::PointerScanSummary;
use squalr_engine_session::engine_unprivileged_state::EngineUnprivilegedState;
use std::collections::{HashMap, HashSet};
use std::ops::Range;
use std::path::PathBuf;
use std::sync::Arc;
use std::thread;

type RepaintRequestCallback = Arc<dyn Fn() + Send + Sync>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PointerScannerTreeRow {
    pub node_id: u64,
    pub tree_depth: usize,
    pub has_children: bool,
    pub is_expanded: bool,
    pub is_selected: bool,
    pub module_base_text: String,
    pub offset_chain_text: String,
    pub resolved_address_text: String,
    pub depth_text: String,
    pub state_text: String,
}

#[derive(Clone)]
pub struct PointerScannerViewData {
    pub target_address_input: AnonymousValueString,
    pub validation_target_address_input: AnonymousValueString,
    pub pointer_size: PointerScanPointerSize,
    pub pointer_size_data_type_selection: DataTypeSelection,
    pub max_depth_input: AnonymousValueString,
    pub offset_radius_input: AnonymousValueString,
    pub status_message: String,
    pub pointer_scan_summary: Option<PointerScanSummary>,
    pub root_node_ids: Vec<u64>,
    pub nodes_by_id: HashMap<u64, PointerScanNode>,
    pub child_node_ids_by_parent_id: HashMap<u64, Vec<u64>>,
    pub expanded_node_ids: HashSet<u64>,
    pub loaded_parent_node_ids: HashSet<Option<u64>>,
    pub pending_parent_node_ids: HashSet<Option<u64>>,
    queued_parent_node_ids: HashSet<Option<u64>>,
    pub selected_node_id: Option<u64>,
    pub is_querying_summary: bool,
    pub is_starting_scan: bool,
    pub is_validating_scan: bool,
    pub is_resetting_scan: bool,
    latest_session_request_revision: u64,
    next_session_request_revision: u64,
    session_state_revision: u64,
    repaint_request_callback: Option<RepaintRequestCallback>,
}

impl PointerScannerViewData {
    pub fn new() -> Self {
        let pointer_size = PointerScanPointerSize::Pointer64;

        Self {
            target_address_input: Self::create_hex_input(String::new()),
            validation_target_address_input: Self::create_hex_input(String::new()),
            pointer_size,
            pointer_size_data_type_selection: DataTypeSelection::new(Self::pointer_size_data_type_ref(pointer_size)),
            max_depth_input: Self::create_unsigned_input(String::from("5")),
            offset_radius_input: Self::create_hex_input(Self::format_hexadecimal(2048)),
            status_message: String::from("No pointer scan session."),
            pointer_scan_summary: None,
            root_node_ids: Vec::new(),
            nodes_by_id: HashMap::new(),
            child_node_ids_by_parent_id: HashMap::new(),
            expanded_node_ids: HashSet::new(),
            loaded_parent_node_ids: HashSet::new(),
            pending_parent_node_ids: HashSet::new(),
            queued_parent_node_ids: HashSet::new(),
            selected_node_id: None,
            is_querying_summary: false,
            is_starting_scan: false,
            is_validating_scan: false,
            is_resetting_scan: false,
            latest_session_request_revision: 0,
            next_session_request_revision: 1,
            session_state_revision: 0,
            repaint_request_callback: None,
        }
    }

    pub fn set_repaint_request_callback(
        &mut self,
        repaint_request_callback: RepaintRequestCallback,
    ) {
        self.repaint_request_callback = Some(repaint_request_callback);
    }

    pub fn initialize(
        pointer_scanner_view_data: Dependency<Self>,
        engine_unprivileged_state: Arc<EngineUnprivilegedState>,
    ) {
        Self::request_summary(pointer_scanner_view_data, engine_unprivileged_state, None);
    }

    pub fn request_summary(
        pointer_scanner_view_data: Dependency<Self>,
        engine_unprivileged_state: Arc<EngineUnprivilegedState>,
        session_id: Option<u64>,
    ) {
        let session_request_revision = {
            let mut pointer_scanner_view_data_guard = match pointer_scanner_view_data.write("Pointer scanner request summary") {
                Some(pointer_scanner_view_data_guard) => pointer_scanner_view_data_guard,
                None => return,
            };

            if pointer_scanner_view_data_guard.is_querying_summary
                || pointer_scanner_view_data_guard.is_starting_scan
                || pointer_scanner_view_data_guard.is_validating_scan
                || pointer_scanner_view_data_guard.is_resetting_scan
            {
                return;
            }

            pointer_scanner_view_data_guard.is_querying_summary = true;
            let session_request_revision = pointer_scanner_view_data_guard.begin_session_request();
            pointer_scanner_view_data_guard.status_message = Self::format_refreshing_summary_status(session_id);
            pointer_scanner_view_data_guard.request_repaint();

            session_request_revision
        };

        let pointer_scan_summary_request = PointerScanSummaryRequest { session_id };
        let pointer_scanner_view_data_clone = pointer_scanner_view_data.clone();
        let pointer_scanner_view_data_for_dispatch = pointer_scanner_view_data.clone();

        let did_spawn_thread = Self::spawn_request_thread("pointer-scan-summary", move || {
            let did_dispatch = pointer_scan_summary_request.send(&engine_unprivileged_state, move |pointer_scan_summary_response| {
                let pointer_scan_summary = pointer_scan_summary_response.pointer_scan_summary.clone();

                if let Some(mut pointer_scanner_view_data_guard) = pointer_scanner_view_data_clone.write("Pointer scanner request summary response") {
                    pointer_scanner_view_data_guard.is_querying_summary = false;

                    if pointer_scanner_view_data_guard.should_apply_session_request(session_request_revision) {
                        pointer_scanner_view_data_guard.apply_summary(pointer_scan_summary.clone());
                        if pointer_scan_summary.is_some() {
                            pointer_scanner_view_data_guard.queue_expand_request(None);
                        }
                    }

                    pointer_scanner_view_data_guard.request_repaint();
                }
            });

            if !did_dispatch {
                Self::clear_summary_request_state(pointer_scanner_view_data_for_dispatch, "Pointer scanner request summary dispatch failure");
            }
        });

        if !did_spawn_thread {
            Self::clear_summary_request_state(pointer_scanner_view_data, "Pointer scanner request summary thread spawn failure");
        }
    }

    pub fn start_scan(
        pointer_scanner_view_data: Dependency<Self>,
        engine_unprivileged_state: Arc<EngineUnprivilegedState>,
    ) {
        let should_validate_scan = {
            let pointer_scanner_view_data_guard = match pointer_scanner_view_data.read("Pointer scanner start scan mode") {
                Some(pointer_scanner_view_data_guard) => pointer_scanner_view_data_guard,
                None => return,
            };

            if pointer_scanner_view_data_guard.is_resetting_scan {
                return;
            }

            pointer_scanner_view_data_guard.has_active_session()
        };

        if should_validate_scan {
            Self::validate_scan(pointer_scanner_view_data, engine_unprivileged_state);

            return;
        }

        Self::start_new_scan(pointer_scanner_view_data, engine_unprivileged_state);
    }

    fn start_new_scan(
        pointer_scanner_view_data: Dependency<Self>,
        engine_unprivileged_state: Arc<EngineUnprivilegedState>,
    ) {
        let (target_address_input, pointer_size, max_depth, offset_radius, session_request_revision) = {
            let mut pointer_scanner_view_data_guard = match pointer_scanner_view_data.write("Pointer scanner start scan") {
                Some(pointer_scanner_view_data_guard) => pointer_scanner_view_data_guard,
                None => return,
            };

            if pointer_scanner_view_data_guard.is_starting_scan || pointer_scanner_view_data_guard.is_resetting_scan {
                return;
            }

            let Some(max_depth) = Self::parse_unsigned_input(&pointer_scanner_view_data_guard.max_depth_input) else {
                pointer_scanner_view_data_guard.status_message = String::from("Cannot start pointer scan: invalid max depth.");
                pointer_scanner_view_data_guard.request_repaint();
                log::error!(
                    "Invalid pointer scan max depth: {}",
                    pointer_scanner_view_data_guard
                        .max_depth_input
                        .get_anonymous_value_string()
                );
                return;
            };
            let Some(offset_radius) = Self::parse_unsigned_input(&pointer_scanner_view_data_guard.offset_radius_input) else {
                pointer_scanner_view_data_guard.status_message = String::from("Cannot start pointer scan: invalid offset.");
                pointer_scanner_view_data_guard.request_repaint();
                log::error!(
                    "Invalid pointer scan offset radius: {}",
                    pointer_scanner_view_data_guard
                        .offset_radius_input
                        .get_anonymous_value_string()
                );
                return;
            };

            pointer_scanner_view_data_guard.is_starting_scan = true;
            let session_request_revision = pointer_scanner_view_data_guard.begin_session_request();
            pointer_scanner_view_data_guard.status_message = Self::format_starting_scan_status(
                &pointer_scanner_view_data_guard.target_address_input,
                pointer_scanner_view_data_guard.pointer_size,
                max_depth,
                offset_radius,
            );
            pointer_scanner_view_data_guard.request_repaint();

            (
                pointer_scanner_view_data_guard.target_address_input.clone(),
                pointer_scanner_view_data_guard.pointer_size,
                max_depth,
                offset_radius,
                session_request_revision,
            )
        };
        let pointer_scan_start_request = PointerScanStartRequest {
            target_address: target_address_input,
            pointer_size,
            max_depth,
            offset_radius,
        };
        let pointer_scanner_view_data_clone = pointer_scanner_view_data.clone();
        let pointer_scanner_view_data_for_dispatch = pointer_scanner_view_data.clone();

        let did_spawn_thread = Self::spawn_request_thread("pointer-scan-start", move || {
            let did_dispatch = pointer_scan_start_request.send(&engine_unprivileged_state, move |pointer_scan_start_response| {
                let pointer_scan_summary = pointer_scan_start_response.pointer_scan_summary.clone();

                if let Some(mut pointer_scanner_view_data_guard) = pointer_scanner_view_data_clone.write("Pointer scanner start scan response") {
                    pointer_scanner_view_data_guard.is_starting_scan = false;

                    if pointer_scanner_view_data_guard.should_apply_session_request(session_request_revision) {
                        pointer_scanner_view_data_guard.apply_summary(pointer_scan_summary.clone());
                        if pointer_scan_summary.is_some() {
                            pointer_scanner_view_data_guard.queue_expand_request(None);
                        }
                    }

                    pointer_scanner_view_data_guard.request_repaint();
                }
            });

            if !did_dispatch {
                Self::clear_start_scan_request_state(pointer_scanner_view_data_for_dispatch, "Pointer scanner start scan dispatch failure");
            }
        });

        if !did_spawn_thread {
            Self::clear_start_scan_request_state(pointer_scanner_view_data, "Pointer scanner start scan thread spawn failure");
        }
    }

    pub fn reset_scan(
        pointer_scanner_view_data: Dependency<Self>,
        engine_unprivileged_state: Arc<EngineUnprivilegedState>,
    ) {
        let session_request_revision = {
            let mut pointer_scanner_view_data_guard = match pointer_scanner_view_data.write("Pointer scanner reset scan") {
                Some(pointer_scanner_view_data_guard) => pointer_scanner_view_data_guard,
                None => return,
            };

            if pointer_scanner_view_data_guard.is_starting_scan
                || pointer_scanner_view_data_guard.is_validating_scan
                || pointer_scanner_view_data_guard.is_resetting_scan
            {
                return;
            }

            pointer_scanner_view_data_guard.is_querying_summary = false;
            pointer_scanner_view_data_guard.is_resetting_scan = true;

            let session_request_revision = pointer_scanner_view_data_guard.begin_session_request();
            pointer_scanner_view_data_guard.apply_summary(None);
            pointer_scanner_view_data_guard.status_message = Self::format_resetting_scan_status();
            pointer_scanner_view_data_guard.request_repaint();

            session_request_revision
        };

        let pointer_scan_reset_request = PointerScanResetRequest {};
        let pointer_scanner_view_data_clone = pointer_scanner_view_data.clone();
        let pointer_scanner_view_data_for_dispatch = pointer_scanner_view_data.clone();
        let engine_unprivileged_state_clone = engine_unprivileged_state.clone();

        let did_spawn_thread = Self::spawn_request_thread("pointer-scan-reset", move || {
            let did_dispatch = pointer_scan_reset_request.send(&engine_unprivileged_state, move |pointer_scan_reset_response| {
                let mut should_refresh_summary = false;

                if let Some(mut pointer_scanner_view_data_guard) = pointer_scanner_view_data_clone.write("Pointer scanner reset scan response") {
                    pointer_scanner_view_data_guard.is_resetting_scan = false;

                    if !pointer_scanner_view_data_guard.should_apply_session_request(session_request_revision) {
                        return;
                    }

                    if !pointer_scan_reset_response.success {
                        log::error!("Failed to clear the active pointer scan session.");
                        should_refresh_summary = true;
                    } else {
                        pointer_scanner_view_data_guard.apply_summary(None);
                    }

                    pointer_scanner_view_data_guard.request_repaint();
                }

                if should_refresh_summary {
                    Self::request_summary(pointer_scanner_view_data_clone, engine_unprivileged_state_clone, None);
                }
            });

            if !did_dispatch {
                Self::clear_reset_scan_request_state(pointer_scanner_view_data_for_dispatch, "Pointer scanner reset scan dispatch failure");
            }
        });

        if !did_spawn_thread {
            Self::clear_reset_scan_request_state(pointer_scanner_view_data, "Pointer scanner reset scan thread spawn failure");
        }
    }

    pub fn validate_scan(
        pointer_scanner_view_data: Dependency<Self>,
        engine_unprivileged_state: Arc<EngineUnprivilegedState>,
    ) {
        let (session_id, validation_target_address_input, session_request_revision) = {
            let mut pointer_scanner_view_data_guard = match pointer_scanner_view_data.write("Pointer scanner validate scan") {
                Some(pointer_scanner_view_data_guard) => pointer_scanner_view_data_guard,
                None => return,
            };
            let Some(session_id) = pointer_scanner_view_data_guard
                .pointer_scan_summary
                .as_ref()
                .map(PointerScanSummary::get_session_id)
            else {
                pointer_scanner_view_data_guard.status_message = String::from("Cannot validate pointer scan without an active session.");
                pointer_scanner_view_data_guard.request_repaint();
                log::error!("Cannot validate pointer scan without an active pointer scan session.");
                return;
            };

            if pointer_scanner_view_data_guard.is_validating_scan || pointer_scanner_view_data_guard.is_resetting_scan {
                return;
            }

            pointer_scanner_view_data_guard.is_validating_scan = true;
            let session_request_revision = pointer_scanner_view_data_guard.begin_session_request();
            pointer_scanner_view_data_guard.status_message =
                Self::format_validating_scan_status(session_id, &pointer_scanner_view_data_guard.validation_target_address_input);
            pointer_scanner_view_data_guard.request_repaint();

            (
                session_id,
                pointer_scanner_view_data_guard
                    .validation_target_address_input
                    .clone(),
                session_request_revision,
            )
        };
        let pointer_scan_validate_request = PointerScanValidateRequest {
            session_id,
            target_address: validation_target_address_input,
        };
        let pointer_scanner_view_data_clone = pointer_scanner_view_data.clone();
        let pointer_scanner_view_data_for_dispatch = pointer_scanner_view_data.clone();

        let did_spawn_thread = Self::spawn_request_thread("pointer-scan-validate", move || {
            let did_dispatch = pointer_scan_validate_request.send(&engine_unprivileged_state, move |pointer_scan_validate_response| {
                let pointer_scan_summary = pointer_scan_validate_response.pointer_scan_summary.clone();

                if let Some(mut pointer_scanner_view_data_guard) = pointer_scanner_view_data_clone.write("Pointer scanner validate scan response") {
                    pointer_scanner_view_data_guard.is_validating_scan = false;

                    if pointer_scanner_view_data_guard.should_apply_session_request(session_request_revision) {
                        pointer_scanner_view_data_guard.apply_summary(pointer_scan_summary.clone());
                        if pointer_scan_summary.is_some() {
                            pointer_scanner_view_data_guard.queue_expand_request(None);
                        }

                        if !pointer_scan_validate_response.status_message.trim().is_empty() {
                            let summary_status = pointer_scanner_view_data_guard
                                .pointer_scan_summary
                                .as_ref()
                                .map(Self::format_summary_status)
                                .unwrap_or_default();

                            pointer_scanner_view_data_guard.status_message = if summary_status.is_empty() {
                                pointer_scan_validate_response.status_message.clone()
                            } else {
                                format!("{} {}", pointer_scan_validate_response.status_message, summary_status)
                            };
                        }
                    }

                    pointer_scanner_view_data_guard.request_repaint();
                }
            });

            if !did_dispatch {
                Self::clear_validate_scan_request_state(pointer_scanner_view_data_for_dispatch, "Pointer scanner validate scan dispatch failure");
            }
        });

        if !did_spawn_thread {
            Self::clear_validate_scan_request_state(pointer_scanner_view_data, "Pointer scanner validate scan thread spawn failure");
        }
    }

    pub fn dispatch_queued_expand_requests(
        pointer_scanner_view_data: Dependency<Self>,
        engine_unprivileged_state: Arc<EngineUnprivilegedState>,
    ) {
        let queued_parent_node_ids = {
            let mut pointer_scanner_view_data_guard = match pointer_scanner_view_data.write("Pointer scanner dispatch queued expand requests") {
                Some(pointer_scanner_view_data_guard) => pointer_scanner_view_data_guard,
                None => return,
            };

            if pointer_scanner_view_data_guard
                .queued_parent_node_ids
                .is_empty()
            {
                return;
            }

            pointer_scanner_view_data_guard
                .queued_parent_node_ids
                .drain()
                .collect::<Vec<_>>()
        };

        for queued_parent_node_id in queued_parent_node_ids {
            Self::request_expand(pointer_scanner_view_data.clone(), engine_unprivileged_state.clone(), queued_parent_node_id);
        }
    }

    pub fn request_expand(
        pointer_scanner_view_data: Dependency<Self>,
        engine_unprivileged_state: Arc<EngineUnprivilegedState>,
        parent_node_id: Option<u64>,
    ) {
        let (session_id, session_state_revision) = {
            let mut pointer_scanner_view_data_guard = match pointer_scanner_view_data.write("Pointer scanner request expand") {
                Some(pointer_scanner_view_data_guard) => pointer_scanner_view_data_guard,
                None => return,
            };
            let Some(session_id) = pointer_scanner_view_data_guard
                .pointer_scan_summary
                .as_ref()
                .map(PointerScanSummary::get_session_id)
            else {
                return;
            };

            if pointer_scanner_view_data_guard
                .pending_parent_node_ids
                .contains(&parent_node_id)
            {
                return;
            }

            if pointer_scanner_view_data_guard
                .loaded_parent_node_ids
                .contains(&parent_node_id)
            {
                return;
            }

            pointer_scanner_view_data_guard
                .pending_parent_node_ids
                .insert(parent_node_id);
            pointer_scanner_view_data_guard.request_repaint();

            (session_id, pointer_scanner_view_data_guard.session_state_revision)
        };
        let pointer_scan_expand_request = PointerScanExpandRequest { session_id, parent_node_id };
        let pointer_scanner_view_data_clone = pointer_scanner_view_data.clone();
        let pointer_scanner_view_data_for_dispatch = pointer_scanner_view_data.clone();

        let did_spawn_thread = Self::spawn_request_thread("pointer-scan-expand", move || {
            let did_dispatch = pointer_scan_expand_request.send(&engine_unprivileged_state, move |pointer_scan_expand_response| {
                if let Some(mut pointer_scanner_view_data_guard) = pointer_scanner_view_data_clone.write("Pointer scanner request expand response") {
                    pointer_scanner_view_data_guard.apply_expand_response(session_state_revision, pointer_scan_expand_response);
                    pointer_scanner_view_data_guard.request_repaint();
                }
            });

            if !did_dispatch {
                Self::clear_expand_request_state(
                    pointer_scanner_view_data_for_dispatch,
                    parent_node_id,
                    "Pointer scanner request expand dispatch failure",
                );
            }
        });

        if !did_spawn_thread {
            Self::clear_expand_request_state(pointer_scanner_view_data, parent_node_id, "Pointer scanner request expand thread spawn failure");
        }
    }

    pub fn toggle_node_expansion(
        pointer_scanner_view_data: Dependency<Self>,
        engine_unprivileged_state: Arc<EngineUnprivilegedState>,
        node_id: u64,
    ) {
        let should_expand = {
            let mut pointer_scanner_view_data_guard = match pointer_scanner_view_data.write("Pointer scanner toggle node expansion") {
                Some(pointer_scanner_view_data_guard) => pointer_scanner_view_data_guard,
                None => return,
            };

            if pointer_scanner_view_data_guard
                .expanded_node_ids
                .contains(&node_id)
            {
                pointer_scanner_view_data_guard
                    .expanded_node_ids
                    .remove(&node_id);
                pointer_scanner_view_data_guard.request_repaint();
                false
            } else {
                pointer_scanner_view_data_guard
                    .expanded_node_ids
                    .insert(node_id);
                pointer_scanner_view_data_guard.request_repaint();
                true
            }
        };

        if should_expand {
            Self::request_expand(pointer_scanner_view_data, engine_unprivileged_state, Some(node_id));
        }
    }

    pub fn select_node(
        pointer_scanner_view_data: Dependency<Self>,
        node_id: u64,
    ) {
        if let Some(mut pointer_scanner_view_data_guard) = pointer_scanner_view_data.write("Pointer scanner select node") {
            pointer_scanner_view_data_guard.selected_node_id = Some(node_id);
            pointer_scanner_view_data_guard.request_repaint();
        }
    }

    pub fn build_visible_rows(pointer_scanner_view_data: Dependency<Self>) -> Vec<PointerScannerTreeRow> {
        let visible_row_count = Self::get_visible_row_count(pointer_scanner_view_data.clone());

        Self::build_visible_rows_in_range(pointer_scanner_view_data, 0..visible_row_count)
    }

    pub fn get_visible_row_count(pointer_scanner_view_data: Dependency<Self>) -> usize {
        let pointer_scanner_view_data_guard = match pointer_scanner_view_data.read("Pointer scanner build visible rows") {
            Some(pointer_scanner_view_data_guard) => pointer_scanner_view_data_guard,
            None => return 0,
        };

        pointer_scanner_view_data_guard
            .root_node_ids
            .iter()
            .map(|root_node_id| pointer_scanner_view_data_guard.count_visible_rows(*root_node_id))
            .sum()
    }

    pub fn build_visible_rows_in_range(
        pointer_scanner_view_data: Dependency<Self>,
        row_range: Range<usize>,
    ) -> Vec<PointerScannerTreeRow> {
        if row_range.is_empty() {
            return Vec::new();
        }

        let pointer_scanner_view_data_guard = match pointer_scanner_view_data.read("Pointer scanner build visible rows in range") {
            Some(pointer_scanner_view_data_guard) => pointer_scanner_view_data_guard,
            None => return Vec::new(),
        };
        let mut pointer_scanner_tree_rows = Vec::new();
        let mut visible_row_index = 0_usize;

        for root_node_id in &pointer_scanner_view_data_guard.root_node_ids {
            let should_stop = pointer_scanner_view_data_guard.append_visible_rows_in_range(
                *root_node_id,
                0,
                &row_range,
                &mut visible_row_index,
                &mut pointer_scanner_tree_rows,
            );

            if should_stop {
                break;
            }
        }

        pointer_scanner_tree_rows
    }

    pub fn build_copy_text(pointer_scanner_view_data: Dependency<Self>) -> Option<String> {
        let pointer_scanner_view_data_guard = pointer_scanner_view_data.read("Pointer scanner build copy text")?;

        pointer_scanner_view_data_guard.build_selected_chain_text()
    }

    pub fn build_export_text(pointer_scanner_view_data: Dependency<Self>) -> Option<String> {
        let pointer_scanner_view_data_guard = pointer_scanner_view_data.read("Pointer scanner build export text")?;
        let selected_node_id = pointer_scanner_view_data_guard.selected_node_id?;
        let selected_pointer_scan_node = pointer_scanner_view_data_guard
            .nodes_by_id
            .get(&selected_node_id)?;
        let pointer_chain_text = pointer_scanner_view_data_guard.build_pointer_chain_text(selected_node_id)?;
        let summary = pointer_scanner_view_data_guard.pointer_scan_summary.as_ref()?;

        Some(format!(
            "Session: {}\nTarget: {}\nChain: {}\nResolved Address: {}\nDepth: {}\nState: {}",
            summary.get_session_id(),
            Self::format_address(summary.get_target_address()),
            pointer_chain_text,
            Self::format_address(selected_pointer_scan_node.get_resolved_target_address()),
            selected_pointer_scan_node.get_depth(),
            Self::format_pointer_scan_node_type(selected_pointer_scan_node.get_pointer_scan_node_type()),
        ))
    }

    pub fn build_project_item_create_request(
        pointer_scanner_view_data: Dependency<Self>,
        target_directory_path: Option<PathBuf>,
    ) -> Option<ProjectItemsCreateRequest> {
        let pointer_scanner_view_data_guard = pointer_scanner_view_data.read("Pointer scanner build project item create request")?;
        let pointer = pointer_scanner_view_data_guard.build_selected_leaf_pointer()?;
        let project_item_name = pointer_scanner_view_data_guard.build_selected_chain_text()?;

        Some(ProjectItemsCreateRequest {
            parent_directory_path: target_directory_path.unwrap_or_default(),
            project_item_name,
            project_item_type: String::from("pointer"),
            pointer: Some(pointer),
            data_type_id: Some(String::from("u8")),
        })
    }

    pub fn format_address(address: u64) -> String {
        Self::format_hexadecimal(address)
    }

    pub fn format_hexadecimal(value: u64) -> String {
        format!("0x{:X}", value)
    }

    pub fn set_scan_target_from_project_address(
        pointer_scanner_view_data: Dependency<Self>,
        address: u64,
        module_name: &str,
    ) {
        if let Some(mut pointer_scanner_view_data_guard) = pointer_scanner_view_data.write("Pointer scanner set scan target from project address") {
            let formatted_address = Self::format_address(address);
            pointer_scanner_view_data_guard.target_address_input = Self::create_hex_input(formatted_address.clone());
            pointer_scanner_view_data_guard.validation_target_address_input = Self::create_hex_input(formatted_address);
            pointer_scanner_view_data_guard.status_message = if module_name.trim().is_empty() {
                String::from("Pointer scanner target autofilled from the project explorer.")
            } else {
                format!(
                    "Pointer scanner target autofilled from {}+0x{:X}. Stored module-relative addresses are not resolved here, so verify the live target before starting.",
                    module_name, address
                )
            };
        }
    }

    pub fn parse_u64_input(input: &str) -> Option<u64> {
        let trimmed_input = input.trim();

        if trimmed_input.is_empty() {
            return None;
        }

        if let Some(hexadecimal_input) = trimmed_input
            .strip_prefix("0x")
            .or_else(|| trimmed_input.strip_prefix("0X"))
        {
            u64::from_str_radix(hexadecimal_input, 16).ok()
        } else {
            trimmed_input.parse::<u64>().ok()
        }
    }

    pub fn parse_unsigned_input(anonymous_value_string: &AnonymousValueString) -> Option<u64> {
        let trimmed_input = anonymous_value_string.get_anonymous_value_string().trim();

        if trimmed_input.is_empty() {
            return None;
        }

        match anonymous_value_string.get_anonymous_value_string_format() {
            AnonymousValueStringFormat::Address | AnonymousValueStringFormat::Hexadecimal => {
                let hexadecimal_input = trimmed_input
                    .strip_prefix("0x")
                    .or_else(|| trimmed_input.strip_prefix("0X"))
                    .unwrap_or(trimmed_input);

                u64::from_str_radix(hexadecimal_input, 16).ok()
            }
            AnonymousValueStringFormat::Binary => {
                let binary_input = trimmed_input
                    .strip_prefix("0b")
                    .or_else(|| trimmed_input.strip_prefix("0B"))
                    .unwrap_or(trimmed_input);

                u64::from_str_radix(binary_input, 2).ok()
            }
            AnonymousValueStringFormat::Decimal => trimmed_input.parse::<u64>().ok(),
            _ => Self::parse_u64_input(trimmed_input),
        }
    }

    fn apply_summary(
        &mut self,
        pointer_scan_summary: Option<PointerScanSummary>,
    ) {
        self.session_state_revision = self.session_state_revision.saturating_add(1);
        self.pointer_scan_summary = pointer_scan_summary.clone();
        self.root_node_ids.clear();
        self.nodes_by_id.clear();
        self.child_node_ids_by_parent_id.clear();
        self.expanded_node_ids.clear();
        self.loaded_parent_node_ids.clear();
        self.pending_parent_node_ids.clear();
        self.queued_parent_node_ids.clear();
        self.selected_node_id = None;

        if let Some(pointer_scan_summary) = pointer_scan_summary {
            let formatted_target_address = Self::format_address(pointer_scan_summary.get_target_address());
            self.target_address_input = Self::create_hex_input(formatted_target_address.clone());
            self.validation_target_address_input = Self::create_hex_input(formatted_target_address);
            self.pointer_size = pointer_scan_summary.get_pointer_size();
            self.pointer_size_data_type_selection
                .replace_selected_data_types(vec![Self::pointer_size_data_type_ref(self.pointer_size)]);
            self.max_depth_input = Self::create_unsigned_input(pointer_scan_summary.get_max_depth().to_string());
            self.offset_radius_input = Self::create_hex_input(Self::format_hexadecimal(pointer_scan_summary.get_offset_radius()));
            self.status_message = Self::format_summary_status(&pointer_scan_summary);
        } else {
            self.status_message = String::from("No pointer scan session.");
        }
    }

    fn begin_session_request(&mut self) -> u64 {
        let session_request_revision = self.next_session_request_revision;
        self.next_session_request_revision = self.next_session_request_revision.saturating_add(1);
        self.latest_session_request_revision = session_request_revision;

        session_request_revision
    }

    fn should_apply_session_request(
        &self,
        session_request_revision: u64,
    ) -> bool {
        self.latest_session_request_revision == session_request_revision
    }

    fn queue_expand_request(
        &mut self,
        parent_node_id: Option<u64>,
    ) {
        self.queued_parent_node_ids.insert(parent_node_id);
    }

    fn apply_expand_response(
        &mut self,
        session_state_revision: u64,
        pointer_scan_expand_response: PointerScanExpandResponse,
    ) {
        if self.session_state_revision != session_state_revision {
            return;
        }

        self.pending_parent_node_ids
            .remove(&pointer_scan_expand_response.parent_node_id);

        if self
            .pointer_scan_summary
            .as_ref()
            .map(PointerScanSummary::get_session_id)
            != Some(pointer_scan_expand_response.session_id)
        {
            return;
        }

        self.loaded_parent_node_ids
            .insert(pointer_scan_expand_response.parent_node_id);

        let node_ids = pointer_scan_expand_response
            .pointer_scan_nodes
            .iter()
            .map(PointerScanNode::get_node_id)
            .collect::<Vec<_>>();

        for pointer_scan_node in pointer_scan_expand_response.pointer_scan_nodes {
            self.nodes_by_id
                .insert(pointer_scan_node.get_node_id(), pointer_scan_node);
        }

        if let Some(parent_node_id) = pointer_scan_expand_response.parent_node_id {
            self.child_node_ids_by_parent_id
                .insert(parent_node_id, node_ids);
        } else {
            self.root_node_ids = node_ids;
        }

        if self.selected_node_id.is_none() {
            self.selected_node_id = self.root_node_ids.first().copied();
        }
    }

    fn build_tree_row(
        &self,
        node_id: u64,
        tree_depth: usize,
    ) -> Option<PointerScannerTreeRow> {
        let pointer_scan_node = self.nodes_by_id.get(&node_id)?;
        let is_expanded = self.expanded_node_ids.contains(&node_id);
        let is_selected = self.selected_node_id == Some(node_id);

        Some(PointerScannerTreeRow {
            node_id,
            tree_depth,
            has_children: pointer_scan_node.has_children(),
            is_expanded,
            is_selected,
            module_base_text: Self::build_module_base_text(pointer_scan_node),
            offset_chain_text: self.build_pointer_chain_text(node_id).unwrap_or_default(),
            resolved_address_text: Self::format_address(pointer_scan_node.get_resolved_target_address()),
            depth_text: pointer_scan_node.get_depth().to_string(),
            state_text: Self::format_pointer_scan_node_type(pointer_scan_node.get_pointer_scan_node_type()).to_string(),
        })
    }

    fn append_visible_rows_in_range(
        &self,
        node_id: u64,
        tree_depth: usize,
        row_range: &Range<usize>,
        visible_row_index: &mut usize,
        pointer_scanner_tree_rows: &mut Vec<PointerScannerTreeRow>,
    ) -> bool {
        let Some(pointer_scanner_tree_row) = self.build_tree_row(node_id, tree_depth) else {
            return false;
        };
        let current_row_index = *visible_row_index;
        *visible_row_index = visible_row_index.saturating_add(1);

        if current_row_index >= row_range.end {
            return true;
        }

        let is_expanded = pointer_scanner_tree_row.is_expanded;

        if row_range.contains(&current_row_index) {
            pointer_scanner_tree_rows.push(pointer_scanner_tree_row);
        }

        if !is_expanded {
            return false;
        }

        if let Some(child_node_ids) = self.child_node_ids_by_parent_id.get(&node_id) {
            for child_node_id in child_node_ids {
                if self.append_visible_rows_in_range(
                    *child_node_id,
                    tree_depth.saturating_add(1),
                    row_range,
                    visible_row_index,
                    pointer_scanner_tree_rows,
                ) {
                    return true;
                }
            }
        }

        false
    }

    fn count_visible_rows(
        &self,
        node_id: u64,
    ) -> usize {
        if !self.nodes_by_id.contains_key(&node_id) {
            return 0;
        }

        let mut visible_row_count = 1_usize;

        if self.expanded_node_ids.contains(&node_id) {
            if let Some(child_node_ids) = self.child_node_ids_by_parent_id.get(&node_id) {
                visible_row_count = visible_row_count.saturating_add(
                    child_node_ids
                        .iter()
                        .map(|child_node_id| self.count_visible_rows(*child_node_id))
                        .sum(),
                );
            }
        }

        visible_row_count
    }

    fn build_selected_chain_text(&self) -> Option<String> {
        let selected_node_id = self.selected_node_id?;

        self.build_pointer_chain_text(selected_node_id)
    }

    pub fn has_active_session(&self) -> bool {
        self.pointer_scan_summary.is_some()
    }

    fn build_selected_leaf_pointer(&self) -> Option<Pointer> {
        let selected_node_id = self.selected_node_id?;
        let selected_pointer_scan_node = self.nodes_by_id.get(&selected_node_id)?;

        if selected_pointer_scan_node.has_children() {
            log::warn!("Select a leaf pointer node before copying, exporting, or adding it to the project.");
            return None;
        }

        self.build_pointer_for_node(selected_node_id)
    }

    fn build_pointer_for_node(
        &self,
        node_id: u64,
    ) -> Option<Pointer> {
        let pointer_scan_summary = self.pointer_scan_summary.as_ref()?;
        let pointer_chain = self.collect_node_path(node_id)?;
        let root_pointer_scan_node = pointer_chain.first()?;
        let pointer_offsets = pointer_chain
            .iter()
            .map(PointerScanNode::get_pointer_offset)
            .collect::<Vec<_>>();

        let (root_address, module_name) = if root_pointer_scan_node.get_pointer_scan_node_type() == PointerScanNodeType::Static {
            (root_pointer_scan_node.get_module_offset(), root_pointer_scan_node.get_module_name().to_string())
        } else {
            (root_pointer_scan_node.get_pointer_address(), String::new())
        };

        Some(Pointer::new_with_size(
            root_address,
            pointer_offsets,
            module_name,
            pointer_scan_summary.get_pointer_size(),
        ))
    }

    fn collect_node_path(
        &self,
        node_id: u64,
    ) -> Option<Vec<PointerScanNode>> {
        let mut pointer_chain = Vec::new();
        let mut current_node_id = Some(node_id);

        while let Some(node_id) = current_node_id {
            let pointer_scan_node = self.nodes_by_id.get(&node_id)?.clone();
            current_node_id = pointer_scan_node.get_parent_node_id();
            pointer_chain.push(pointer_scan_node);
        }

        pointer_chain.reverse();

        Some(pointer_chain)
    }

    fn build_pointer_chain_text(
        &self,
        node_id: u64,
    ) -> Option<String> {
        let pointer_chain = self.collect_node_path(node_id)?;
        let root_pointer_scan_node = pointer_chain.first()?;
        let mut pointer_chain_segments = vec![Self::build_module_base_text(root_pointer_scan_node)];

        for pointer_scan_node in &pointer_chain {
            pointer_chain_segments.push(Self::format_pointer_offset(pointer_scan_node.get_pointer_offset()));
        }

        Some(pointer_chain_segments.join(" -> "))
    }

    fn build_module_base_text(pointer_scan_node: &PointerScanNode) -> String {
        if pointer_scan_node.get_pointer_scan_node_type() == PointerScanNodeType::Static {
            format!("{}+0x{:X}", pointer_scan_node.get_module_name(), pointer_scan_node.get_module_offset())
        } else {
            Self::format_address(pointer_scan_node.get_pointer_address())
        }
    }

    fn format_pointer_offset(pointer_offset: i64) -> String {
        if pointer_offset >= 0 {
            format!("+0x{:X}", pointer_offset as u64)
        } else {
            format!("-0x{:X}", pointer_offset.unsigned_abs())
        }
    }

    fn format_pointer_scan_node_type(pointer_scan_node_type: PointerScanNodeType) -> &'static str {
        match pointer_scan_node_type {
            PointerScanNodeType::Static => "Static",
            PointerScanNodeType::Heap => "Heap",
        }
    }

    fn format_summary_status(pointer_scan_summary: &PointerScanSummary) -> String {
        format!(
            "Session {} | Target {} | Roots {} | Nodes {} (Static {} / Heap {})",
            pointer_scan_summary.get_session_id(),
            Self::format_address(pointer_scan_summary.get_target_address()),
            pointer_scan_summary.get_root_node_count(),
            pointer_scan_summary.get_total_node_count(),
            pointer_scan_summary.get_total_static_node_count(),
            pointer_scan_summary.get_total_heap_node_count(),
        )
    }

    fn format_refreshing_summary_status(session_id: Option<u64>) -> String {
        match session_id {
            Some(session_id) => format!("Refreshing pointer scan session {}...", session_id),
            None => String::from("Refreshing pointer scan summary..."),
        }
    }

    fn format_starting_scan_status(
        target_address_input: &AnonymousValueString,
        pointer_size: PointerScanPointerSize,
        max_depth: u64,
        offset_radius: u64,
    ) -> String {
        format!(
            "Starting pointer scan | Target {} | Pointer Size {} | Depth {} | Offset {}",
            Self::format_status_input_value(target_address_input),
            pointer_size,
            max_depth,
            Self::format_hexadecimal(offset_radius),
        )
    }

    fn format_validating_scan_status(
        session_id: u64,
        validation_target_address_input: &AnonymousValueString,
    ) -> String {
        format!(
            "Validating pointer scan session {} | Target {}",
            session_id,
            Self::format_status_input_value(validation_target_address_input),
        )
    }

    fn format_resetting_scan_status() -> String {
        String::from("Clearing pointer scan session...")
    }

    fn format_status_input_value(anonymous_value_string: &AnonymousValueString) -> String {
        let input_text = anonymous_value_string.get_anonymous_value_string().trim();

        if input_text.is_empty() {
            String::from("<empty>")
        } else {
            input_text.to_string()
        }
    }

    fn create_hex_input(value_text: String) -> AnonymousValueString {
        AnonymousValueString::new(value_text, AnonymousValueStringFormat::Hexadecimal, ContainerType::None)
    }

    fn create_unsigned_input(value_text: String) -> AnonymousValueString {
        AnonymousValueString::new(value_text, AnonymousValueStringFormat::Decimal, ContainerType::None)
    }

    fn spawn_request_thread(
        thread_name: &'static str,
        thread_entry: impl FnOnce() + Send + 'static,
    ) -> bool {
        match thread::Builder::new()
            .name(thread_name.to_string())
            .spawn(thread_entry)
        {
            Ok(_join_handle) => true,
            Err(error) => {
                log::error!("Failed to spawn {} thread: {}", thread_name, error);
                false
            }
        }
    }

    fn clear_start_scan_request_state(
        pointer_scanner_view_data: Dependency<Self>,
        error_context: &'static str,
    ) {
        if let Some(mut pointer_scanner_view_data_guard) = pointer_scanner_view_data.write(error_context) {
            pointer_scanner_view_data_guard.is_starting_scan = false;
            pointer_scanner_view_data_guard.status_message = String::from("Pointer scan start failed. See logs.");
            pointer_scanner_view_data_guard.request_repaint();
        }
    }

    fn clear_summary_request_state(
        pointer_scanner_view_data: Dependency<Self>,
        error_context: &'static str,
    ) {
        if let Some(mut pointer_scanner_view_data_guard) = pointer_scanner_view_data.write(error_context) {
            pointer_scanner_view_data_guard.is_querying_summary = false;
            pointer_scanner_view_data_guard.status_message = String::from("Pointer scan summary refresh failed. See logs.");
            pointer_scanner_view_data_guard.request_repaint();
        }
    }

    fn clear_validate_scan_request_state(
        pointer_scanner_view_data: Dependency<Self>,
        error_context: &'static str,
    ) {
        if let Some(mut pointer_scanner_view_data_guard) = pointer_scanner_view_data.write(error_context) {
            pointer_scanner_view_data_guard.is_validating_scan = false;
            pointer_scanner_view_data_guard.status_message = String::from("Pointer scan validation failed. See logs.");
            pointer_scanner_view_data_guard.request_repaint();
        }
    }

    fn clear_reset_scan_request_state(
        pointer_scanner_view_data: Dependency<Self>,
        error_context: &'static str,
    ) {
        if let Some(mut pointer_scanner_view_data_guard) = pointer_scanner_view_data.write(error_context) {
            pointer_scanner_view_data_guard.is_resetting_scan = false;
            pointer_scanner_view_data_guard.status_message = String::from("Pointer scan reset failed. See logs.");
            pointer_scanner_view_data_guard.request_repaint();
        }
    }

    fn clear_expand_request_state(
        pointer_scanner_view_data: Dependency<Self>,
        parent_node_id: Option<u64>,
        error_context: &'static str,
    ) {
        if let Some(mut pointer_scanner_view_data_guard) = pointer_scanner_view_data.write(error_context) {
            pointer_scanner_view_data_guard
                .pending_parent_node_ids
                .remove(&parent_node_id);
            pointer_scanner_view_data_guard.request_repaint();
        }
    }

    fn request_repaint(&self) {
        if let Some(repaint_request_callback) = &self.repaint_request_callback {
            repaint_request_callback();
        }
    }

    pub fn synchronize_pointer_size_with_process_bitness(
        &mut self,
        process_bitness: Bitness,
    ) {
        if self.pointer_scan_summary.is_some() {
            return;
        }

        self.apply_pointer_size_from_process_bitness(process_bitness);
    }

    pub fn force_pointer_size_from_process_bitness(
        &mut self,
        process_bitness: Bitness,
    ) {
        self.apply_pointer_size_from_process_bitness(process_bitness);
    }

    fn apply_pointer_size_from_process_bitness(
        &mut self,
        process_bitness: Bitness,
    ) {
        self.pointer_size = match process_bitness {
            Bitness::Bit32 => PointerScanPointerSize::Pointer32,
            Bitness::Bit64 => PointerScanPointerSize::Pointer64,
        };
        self.pointer_size_data_type_selection
            .replace_selected_data_types(vec![Self::pointer_size_data_type_ref(self.pointer_size)]);
    }

    fn pointer_size_data_type_ref(pointer_size: PointerScanPointerSize) -> DataTypeRef {
        match pointer_size {
            PointerScanPointerSize::Pointer32 => DataTypeRef::new("u32"),
            PointerScanPointerSize::Pointer64 => DataTypeRef::new("u64"),
        }
    }

    pub fn synchronize_pointer_size_from_selection(&mut self) {
        let selected_pointer_size_data_type = self.pointer_size_data_type_selection.active_data_type().clone();

        self.pointer_size_data_type_selection
            .replace_selected_data_types(vec![selected_pointer_size_data_type.clone()]);
        self.pointer_size = if selected_pointer_size_data_type.get_data_type_id() == "u32" {
            PointerScanPointerSize::Pointer32
        } else {
            PointerScanPointerSize::Pointer64
        };
    }
}

#[cfg(test)]
mod tests {
    use super::PointerScannerViewData;
    use crossbeam_channel::{Receiver, unbounded};
    use squalr_engine_api::commands::pointer_scan::expand::pointer_scan_expand_response::PointerScanExpandResponse;
    use squalr_engine_api::commands::pointer_scan::pointer_scan_command::PointerScanCommand;
    use squalr_engine_api::commands::pointer_scan::reset::pointer_scan_reset_response::PointerScanResetResponse;
    use squalr_engine_api::commands::pointer_scan::start::pointer_scan_start_response::PointerScanStartResponse;
    use squalr_engine_api::commands::pointer_scan::summary::pointer_scan_summary_request::PointerScanSummaryRequest;
    use squalr_engine_api::commands::pointer_scan::summary::pointer_scan_summary_response::PointerScanSummaryResponse;
    use squalr_engine_api::commands::pointer_scan::validate::pointer_scan_validate_response::PointerScanValidateResponse;
    use squalr_engine_api::commands::privileged_command::PrivilegedCommand;
    use squalr_engine_api::commands::privileged_command_response::{PrivilegedCommandResponse, TypedPrivilegedCommandResponse};
    use squalr_engine_api::commands::unprivileged_command::UnprivilegedCommand;
    use squalr_engine_api::commands::unprivileged_command_response::UnprivilegedCommandResponse;
    use squalr_engine_api::dependency_injection::dependency_container::DependencyContainer;
    use squalr_engine_api::engine::engine_api_unprivileged_bindings::EngineApiUnprivilegedBindings;
    use squalr_engine_api::engine::engine_binding_error::EngineBindingError;
    use squalr_engine_api::engine::engine_execution_context::EngineExecutionContext;
    use squalr_engine_api::events::engine_event::EngineEvent;
    use squalr_engine_api::structures::data_values::anonymous_value_string_format::AnonymousValueStringFormat;
    use squalr_engine_api::structures::memory::bitness::Bitness;
    use squalr_engine_api::structures::pointer_scans::pointer_scan_node::PointerScanNode;
    use squalr_engine_api::structures::pointer_scans::pointer_scan_node_type::PointerScanNodeType;
    use squalr_engine_api::structures::pointer_scans::pointer_scan_pointer_size::PointerScanPointerSize;
    use squalr_engine_api::structures::pointer_scans::pointer_scan_summary::PointerScanSummary;
    use squalr_engine_session::engine_unprivileged_state::EngineUnprivilegedState;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::{Arc, Mutex, RwLock};
    use std::time::{Duration, Instant};

    struct DeferredPointerScannerCommand {
        privileged_command: PrivilegedCommand,
        callback: Option<Box<dyn FnOnce(PrivilegedCommandResponse) + Send + Sync + 'static>>,
    }

    struct DeferredTestPointerScannerBindings {
        queued_commands: Arc<Mutex<Vec<DeferredPointerScannerCommand>>>,
    }

    impl DeferredTestPointerScannerBindings {
        fn new() -> Self {
            Self {
                queued_commands: Arc::new(Mutex::new(Vec::new())),
            }
        }

        fn get_queued_commands(&self) -> Arc<Mutex<Vec<DeferredPointerScannerCommand>>> {
            self.queued_commands.clone()
        }

        fn respond_to_first_matching(
            queued_commands: &Arc<Mutex<Vec<DeferredPointerScannerCommand>>>,
            predicate: impl Fn(&PrivilegedCommand) -> bool,
            response: PrivilegedCommandResponse,
        ) {
            let callback = {
                let mut queued_commands_guard = queued_commands
                    .lock()
                    .expect("Expected the deferred pointer scanner queued commands lock.");
                let queued_command_index = queued_commands_guard
                    .iter()
                    .position(|queued_command| predicate(&queued_command.privileged_command))
                    .expect("Expected a queued pointer scanner command matching the predicate.");
                let mut queued_command = queued_commands_guard.remove(queued_command_index);

                queued_command
                    .callback
                    .take()
                    .expect("Expected the deferred pointer scanner callback.")
            };

            callback(response);
        }
    }

    impl EngineApiUnprivilegedBindings for DeferredTestPointerScannerBindings {
        fn dispatch_privileged_command(
            &self,
            engine_command: PrivilegedCommand,
            callback: Box<dyn FnOnce(PrivilegedCommandResponse) + Send + Sync + 'static>,
        ) -> Result<(), EngineBindingError> {
            match self.queued_commands.lock() {
                Ok(mut queued_commands) => {
                    queued_commands.push(DeferredPointerScannerCommand {
                        privileged_command: engine_command,
                        callback: Some(callback),
                    });
                }
                Err(error) => {
                    return Err(EngineBindingError::lock_failure(
                        "capturing deferred pointer scanner commands",
                        error.to_string(),
                    ));
                }
            }

            Ok(())
        }

        fn dispatch_unprivileged_command(
            &self,
            _engine_command: UnprivilegedCommand,
            _engine_unprivileged_state: &Arc<dyn EngineExecutionContext>,
            _callback: Box<dyn FnOnce(UnprivilegedCommandResponse) + Send + Sync + 'static>,
        ) -> Result<(), EngineBindingError> {
            Err(EngineBindingError::unavailable(
                "dispatching unprivileged deferred pointer scanner test command",
            ))
        }

        fn subscribe_to_engine_events(&self) -> Result<Receiver<EngineEvent>, EngineBindingError> {
            let (_event_sender, event_receiver) = unbounded();

            Ok(event_receiver)
        }
    }

    fn create_pointer_scan_summary(
        session_id: u64,
        target_address: u64,
    ) -> PointerScanSummary {
        PointerScanSummary::new(session_id, target_address, PointerScanPointerSize::Pointer64, 5, 0x100, 1, 2, 1, 1, Vec::new())
    }

    fn wait_for_condition(
        description: &str,
        condition: impl Fn() -> bool,
    ) {
        let timeout_at = Instant::now() + Duration::from_secs(2);

        while Instant::now() < timeout_at {
            if condition() {
                return;
            }

            std::thread::sleep(Duration::from_millis(10));
        }

        panic!("Timed out waiting for {}.", description);
    }

    fn create_pointer_scanner_view_data() -> PointerScannerViewData {
        let mut pointer_scanner_view_data = PointerScannerViewData::new();
        let root_node = PointerScanNode::new(
            1,
            None,
            PointerScanNodeType::Static,
            1,
            0x1010,
            0x1FF0,
            0x2000,
            0x10,
            "game.exe".to_string(),
            0x10,
            vec![2],
        );
        let child_node = PointerScanNode::new(
            2,
            Some(1),
            PointerScanNodeType::Heap,
            2,
            0x2000,
            0x3000,
            0x3010,
            -0x10,
            String::new(),
            0,
            Vec::new(),
        );

        pointer_scanner_view_data.pointer_scan_summary = Some(create_pointer_scan_summary(7, 0x3010));
        pointer_scanner_view_data.root_node_ids = vec![1];
        pointer_scanner_view_data
            .nodes_by_id
            .insert(root_node.get_node_id(), root_node);
        pointer_scanner_view_data
            .nodes_by_id
            .insert(child_node.get_node_id(), child_node);
        pointer_scanner_view_data
            .child_node_ids_by_parent_id
            .insert(1, vec![2]);
        pointer_scanner_view_data.selected_node_id = Some(2);
        pointer_scanner_view_data.expanded_node_ids.insert(1);

        pointer_scanner_view_data
    }

    fn attach_repaint_counter(pointer_scanner_view_data: &mut PointerScannerViewData) -> Arc<AtomicUsize> {
        let repaint_request_count = Arc::new(AtomicUsize::new(0));
        let repaint_request_count_clone = repaint_request_count.clone();
        pointer_scanner_view_data.set_repaint_request_callback(Arc::new(move || {
            repaint_request_count_clone.fetch_add(1, Ordering::Relaxed);
        }));

        repaint_request_count
    }

    #[test]
    fn build_project_item_create_request_uses_leaf_chain_pointer() {
        let dependency_container = DependencyContainer::new();
        let pointer_scanner_view_data = dependency_container.register(create_pointer_scanner_view_data());

        let project_item_create_request =
            PointerScannerViewData::build_project_item_create_request(pointer_scanner_view_data, Some("project_items/Pointers".into()))
                .expect("Expected leaf chain request.");
        let pointer = project_item_create_request
            .pointer
            .expect("Expected pointer payload.");

        assert_eq!(project_item_create_request.project_item_type, "pointer");
        assert_eq!(project_item_create_request.project_item_name, "game.exe+0x10 -> +0x10 -> -0x10");
        assert_eq!(pointer.get_address(), 0x10);
        assert_eq!(pointer.get_module_name(), "game.exe");
        assert_eq!(pointer.get_offsets(), &[0x10, -0x10]);
        assert_eq!(pointer.get_pointer_size(), PointerScanPointerSize::Pointer64);
    }

    #[test]
    fn build_project_item_create_request_returns_none_for_non_leaf_selection() {
        let dependency_container = DependencyContainer::new();
        let mut pointer_scanner_view_data = create_pointer_scanner_view_data();
        pointer_scanner_view_data.selected_node_id = Some(1);
        let pointer_scanner_view_data = dependency_container.register(pointer_scanner_view_data);

        let project_item_create_request =
            PointerScannerViewData::build_project_item_create_request(pointer_scanner_view_data, Some("project_items/Pointers".into()));

        assert!(project_item_create_request.is_none());
    }

    #[test]
    fn build_copy_text_returns_selected_chain_text() {
        let dependency_container = DependencyContainer::new();
        let pointer_scanner_view_data = dependency_container.register(create_pointer_scanner_view_data());

        let copy_text = PointerScannerViewData::build_copy_text(pointer_scanner_view_data).expect("Expected selected chain text.");

        assert_eq!(copy_text, "game.exe+0x10 -> +0x10 -> -0x10");
    }

    #[test]
    fn build_export_text_includes_selected_chain_metadata() {
        let dependency_container = DependencyContainer::new();
        let pointer_scanner_view_data = dependency_container.register(create_pointer_scanner_view_data());

        let export_text = PointerScannerViewData::build_export_text(pointer_scanner_view_data).expect("Expected export text for selected leaf.");

        assert!(export_text.contains("Session: 7"));
        assert!(export_text.contains("Chain: game.exe+0x10 -> +0x10 -> -0x10"));
        assert!(export_text.contains("Resolved Address: 0x3010"));
        assert!(export_text.contains("State: Heap"));
    }

    #[test]
    fn build_visible_rows_marks_expanded_and_selected_entries() {
        let dependency_container = DependencyContainer::new();
        let pointer_scanner_view_data = dependency_container.register(create_pointer_scanner_view_data());

        let pointer_scanner_tree_rows = PointerScannerViewData::build_visible_rows(pointer_scanner_view_data);

        assert_eq!(pointer_scanner_tree_rows.len(), 2);
        assert_eq!(pointer_scanner_tree_rows[0].module_base_text, "game.exe+0x10");
        assert!(pointer_scanner_tree_rows[0].is_expanded);
        assert_eq!(pointer_scanner_tree_rows[1].tree_depth, 1);
        assert!(pointer_scanner_tree_rows[1].is_selected);
    }

    #[test]
    fn get_visible_row_count_tracks_expanded_children() {
        let dependency_container = DependencyContainer::new();
        let pointer_scanner_view_data = dependency_container.register(create_pointer_scanner_view_data());

        assert_eq!(PointerScannerViewData::get_visible_row_count(pointer_scanner_view_data.clone()), 2);

        if let Some(mut pointer_scanner_view_data_guard) = pointer_scanner_view_data.write("Pointer scanner collapse root for row count test") {
            pointer_scanner_view_data_guard.expanded_node_ids.remove(&1);
        }

        assert_eq!(PointerScannerViewData::get_visible_row_count(pointer_scanner_view_data), 1);
    }

    #[test]
    fn build_visible_rows_in_range_returns_only_requested_slice() {
        let dependency_container = DependencyContainer::new();
        let mut pointer_scanner_view_data = create_pointer_scanner_view_data();
        pointer_scanner_view_data.root_node_ids = vec![1, 3];
        pointer_scanner_view_data.nodes_by_id.insert(
            3,
            PointerScanNode::new(
                3,
                None,
                PointerScanNodeType::Static,
                1,
                0x4000,
                0x5000,
                0x5010,
                0x10,
                "engine.dll".to_string(),
                0x40,
                Vec::new(),
            ),
        );
        pointer_scanner_view_data.selected_node_id = Some(3);
        let pointer_scanner_view_data = dependency_container.register(pointer_scanner_view_data);

        let pointer_scanner_tree_rows = PointerScannerViewData::build_visible_rows_in_range(pointer_scanner_view_data.clone(), 1..3);

        assert_eq!(pointer_scanner_tree_rows.len(), 2);
        assert_eq!(pointer_scanner_tree_rows[0].node_id, 2);
        assert_eq!(pointer_scanner_tree_rows[0].tree_depth, 1);
        assert_eq!(pointer_scanner_tree_rows[1].node_id, 3);
        assert!(pointer_scanner_tree_rows[1].is_selected);
        assert_eq!(PointerScannerViewData::get_visible_row_count(pointer_scanner_view_data), 3);
    }

    #[test]
    fn session_request_revisions_only_accept_the_latest_request() {
        let mut pointer_scanner_view_data = PointerScannerViewData::new();

        let first_session_request_revision = pointer_scanner_view_data.begin_session_request();
        let second_session_request_revision = pointer_scanner_view_data.begin_session_request();

        assert!(!pointer_scanner_view_data.should_apply_session_request(first_session_request_revision));
        assert!(pointer_scanner_view_data.should_apply_session_request(second_session_request_revision));
    }

    #[test]
    fn request_summary_dispatches_on_background_thread_and_queues_root_expand_until_the_next_view_pass() {
        let dependency_container = DependencyContainer::new();
        let mut pointer_scanner_view_data = PointerScannerViewData::new();
        let repaint_request_count = attach_repaint_counter(&mut pointer_scanner_view_data);
        let pointer_scanner_view_data = dependency_container.register(pointer_scanner_view_data);
        let deferred_pointer_scanner_bindings = DeferredTestPointerScannerBindings::new();
        let queued_commands = deferred_pointer_scanner_bindings.get_queued_commands();
        let engine_bindings: Arc<RwLock<dyn EngineApiUnprivilegedBindings>> = Arc::new(RwLock::new(deferred_pointer_scanner_bindings));
        let engine_unprivileged_state = EngineUnprivilegedState::new(engine_bindings);

        PointerScannerViewData::request_summary(pointer_scanner_view_data.clone(), engine_unprivileged_state.clone(), Some(7));
        wait_for_condition("pointer scanner summary dispatch", || {
            queued_commands
                .lock()
                .map(|queued_commands_guard| queued_commands_guard.len() >= 1)
                .unwrap_or(false)
        });

        {
            let pointer_scanner_view_data_guard = pointer_scanner_view_data
                .read("Pointer scanner summary pending state test")
                .expect("Expected the pointer scanner view data read guard while the summary request is pending.");
            assert!(pointer_scanner_view_data_guard.is_querying_summary);
            assert!(pointer_scanner_view_data_guard.pointer_scan_summary.is_none());
        }

        let queued_commands_after_summary = queued_commands
            .lock()
            .expect("Expected the deferred pointer scanner queued commands lock after summary.");
        assert_eq!(queued_commands_after_summary.len(), 1);
        assert!(matches!(
            queued_commands_after_summary
                .first()
                .map(|queued_command| &queued_command.privileged_command),
            Some(PrivilegedCommand::PointerScan(PointerScanCommand::Summary {
                pointer_scan_summary_request: PointerScanSummaryRequest { session_id: Some(7) },
            }))
        ));
        drop(queued_commands_after_summary);

        let repaint_request_count_before_summary_response = repaint_request_count.load(Ordering::Relaxed);

        DeferredTestPointerScannerBindings::respond_to_first_matching(
            &queued_commands,
            |privileged_command| matches!(privileged_command, PrivilegedCommand::PointerScan(PointerScanCommand::Summary { .. })),
            PointerScanSummaryResponse {
                pointer_scan_summary: Some(create_pointer_scan_summary(7, 0x3010)),
            }
            .to_engine_response(),
        );

        wait_for_condition("pointer scanner summary response application", || {
            pointer_scanner_view_data
                .read("Pointer scanner summary response wait")
                .map(|pointer_scanner_view_data_guard| {
                    !pointer_scanner_view_data_guard.is_querying_summary
                        && pointer_scanner_view_data_guard
                            .pointer_scan_summary
                            .as_ref()
                            .map(PointerScanSummary::get_session_id)
                            == Some(7)
                        && pointer_scanner_view_data_guard
                            .queued_parent_node_ids
                            .contains(&None)
                })
                .unwrap_or(false)
        });
        wait_for_condition("pointer scanner summary response repaint request", || {
            repaint_request_count.load(Ordering::Relaxed) > repaint_request_count_before_summary_response
        });

        PointerScannerViewData::dispatch_queued_expand_requests(pointer_scanner_view_data.clone(), engine_unprivileged_state);
        wait_for_condition("pointer scanner expand dispatch", || {
            queued_commands
                .lock()
                .map(|queued_commands_guard| {
                    queued_commands_guard.iter().any(|queued_command| {
                        matches!(
                            queued_command.privileged_command,
                            PrivilegedCommand::PointerScan(PointerScanCommand::Expand { .. })
                        )
                    })
                })
                .unwrap_or(false)
        });

        let queued_commands_after_expand = queued_commands
            .lock()
            .expect("Expected the deferred pointer scanner queued commands lock after expand dispatch.");
        assert_eq!(queued_commands_after_expand.len(), 1);
        assert!(matches!(
            queued_commands_after_expand
                .first()
                .map(|queued_command| &queued_command.privileged_command),
            Some(PrivilegedCommand::PointerScan(PointerScanCommand::Expand { pointer_scan_expand_request }))
                if pointer_scan_expand_request.parent_node_id.is_none()
        ));
        drop(queued_commands_after_expand);

        let repaint_request_count_before_expand_response = repaint_request_count.load(Ordering::Relaxed);

        DeferredTestPointerScannerBindings::respond_to_first_matching(
            &queued_commands,
            |privileged_command| matches!(privileged_command, PrivilegedCommand::PointerScan(PointerScanCommand::Expand { .. })),
            PointerScanExpandResponse {
                session_id: 7,
                parent_node_id: None,
                pointer_scan_nodes: vec![PointerScanNode::new(
                    1,
                    None,
                    PointerScanNodeType::Static,
                    1,
                    0x1010,
                    0x1FF0,
                    0x3010,
                    0x10,
                    "game.exe".to_string(),
                    0x10,
                    Vec::new(),
                )],
            }
            .to_engine_response(),
        );

        wait_for_condition("pointer scanner root nodes after expand", || {
            pointer_scanner_view_data
                .read("Pointer scanner root nodes after queued expand")
                .map(|pointer_scanner_view_data_guard| pointer_scanner_view_data_guard.root_node_ids == vec![1])
                .unwrap_or(false)
        });
        wait_for_condition("pointer scanner expand response repaint request", || {
            repaint_request_count.load(Ordering::Relaxed) > repaint_request_count_before_expand_response
        });

        let pointer_scanner_view_data_guard = pointer_scanner_view_data
            .read("Pointer scanner queued expand request test")
            .expect("Expected the pointer scanner view data read guard.");
        assert_eq!(pointer_scanner_view_data_guard.root_node_ids, vec![1]);
    }

    #[test]
    fn reset_scan_cancels_inflight_summary_and_ignores_stale_summary_responses() {
        let dependency_container = DependencyContainer::new();
        let pointer_scanner_view_data = dependency_container.register(PointerScannerViewData::new());
        let deferred_pointer_scanner_bindings = DeferredTestPointerScannerBindings::new();
        let queued_commands = deferred_pointer_scanner_bindings.get_queued_commands();
        let engine_bindings: Arc<RwLock<dyn EngineApiUnprivilegedBindings>> = Arc::new(RwLock::new(deferred_pointer_scanner_bindings));
        let engine_unprivileged_state = EngineUnprivilegedState::new(engine_bindings);

        PointerScannerViewData::request_summary(pointer_scanner_view_data.clone(), engine_unprivileged_state.clone(), Some(7));
        PointerScannerViewData::reset_scan(pointer_scanner_view_data.clone(), engine_unprivileged_state.clone());
        wait_for_condition("pointer scanner summary and reset dispatch", || {
            queued_commands
                .lock()
                .map(|queued_commands_guard| queued_commands_guard.len() >= 2)
                .unwrap_or(false)
        });

        {
            let queued_commands_guard = queued_commands
                .lock()
                .expect("Expected the deferred pointer scanner queued commands lock after reset.");
            assert_eq!(queued_commands_guard.len(), 2);
            assert!(matches!(
                queued_commands_guard
                    .first()
                    .map(|queued_command| &queued_command.privileged_command),
                Some(PrivilegedCommand::PointerScan(PointerScanCommand::Summary {
                    pointer_scan_summary_request: PointerScanSummaryRequest { session_id: Some(7) },
                }))
            ));
            assert!(matches!(
                queued_commands_guard
                    .get(1)
                    .map(|queued_command| &queued_command.privileged_command),
                Some(PrivilegedCommand::PointerScan(PointerScanCommand::Reset { .. }))
            ));
        }

        {
            let pointer_scanner_view_data_guard = pointer_scanner_view_data
                .read("Pointer scanner reset scan in-flight state test")
                .expect("Expected the pointer scanner view data read guard after reset.");
            assert!(pointer_scanner_view_data_guard.pointer_scan_summary.is_none());
            assert!(pointer_scanner_view_data_guard.is_resetting_scan);
            assert!(!pointer_scanner_view_data_guard.is_querying_summary);
        }

        DeferredTestPointerScannerBindings::respond_to_first_matching(
            &queued_commands,
            |privileged_command| matches!(privileged_command, PrivilegedCommand::PointerScan(PointerScanCommand::Summary { .. })),
            PointerScanSummaryResponse {
                pointer_scan_summary: Some(create_pointer_scan_summary(7, 0x4010)),
            }
            .to_engine_response(),
        );

        {
            let pointer_scanner_view_data_guard = pointer_scanner_view_data
                .read("Pointer scanner stale summary response test")
                .expect("Expected the pointer scanner view data read guard after the stale summary response.");
            assert!(pointer_scanner_view_data_guard.pointer_scan_summary.is_none());
            assert!(pointer_scanner_view_data_guard.is_resetting_scan);
        }

        DeferredTestPointerScannerBindings::respond_to_first_matching(
            &queued_commands,
            |privileged_command| matches!(privileged_command, PrivilegedCommand::PointerScan(PointerScanCommand::Reset { .. })),
            PointerScanResetResponse { success: true }.to_engine_response(),
        );

        let pointer_scanner_view_data_guard = pointer_scanner_view_data
            .read("Pointer scanner reset scan completed state test")
            .expect("Expected the pointer scanner view data read guard after the reset response.");
        assert!(pointer_scanner_view_data_guard.pointer_scan_summary.is_none());
        assert!(!pointer_scanner_view_data_guard.is_resetting_scan);
        assert_eq!(pointer_scanner_view_data_guard.status_message, "No pointer scan session.");
    }

    #[test]
    fn start_scan_dispatches_on_background_thread_and_applies_response() {
        let dependency_container = DependencyContainer::new();
        let mut pointer_scanner_view_data = PointerScannerViewData::new();
        let repaint_request_count = attach_repaint_counter(&mut pointer_scanner_view_data);
        pointer_scanner_view_data.target_address_input = PointerScannerViewData::create_hex_input("0x3010".to_string());
        let pointer_scanner_view_data = dependency_container.register(pointer_scanner_view_data);
        let deferred_pointer_scanner_bindings = DeferredTestPointerScannerBindings::new();
        let queued_commands = deferred_pointer_scanner_bindings.get_queued_commands();
        let engine_bindings: Arc<RwLock<dyn EngineApiUnprivilegedBindings>> = Arc::new(RwLock::new(deferred_pointer_scanner_bindings));
        let engine_unprivileged_state = EngineUnprivilegedState::new(engine_bindings);

        PointerScannerViewData::start_scan(pointer_scanner_view_data.clone(), engine_unprivileged_state);
        wait_for_condition("pointer scanner start dispatch", || {
            queued_commands
                .lock()
                .map(|queued_commands_guard| queued_commands_guard.len() >= 1)
                .unwrap_or(false)
        });

        {
            let queued_commands_guard = queued_commands
                .lock()
                .expect("Expected the deferred pointer scanner queued commands lock after start.");
            assert!(matches!(
                queued_commands_guard
                    .first()
                    .map(|queued_command| &queued_command.privileged_command),
                Some(PrivilegedCommand::PointerScan(PointerScanCommand::Start { .. }))
            ));
        }

        {
            let pointer_scanner_view_data_guard = pointer_scanner_view_data
                .read("Pointer scanner start pending state test")
                .expect("Expected the pointer scanner view data read guard while the start request is pending.");
            assert!(pointer_scanner_view_data_guard.is_starting_scan);
            assert!(pointer_scanner_view_data_guard.pointer_scan_summary.is_none());
            assert_eq!(
                pointer_scanner_view_data_guard.status_message,
                "Starting pointer scan | Target 0x3010 | Pointer Size u64 | Depth 5 | Offset 0x800"
            );
        }

        let repaint_request_count_before_start_response = repaint_request_count.load(Ordering::Relaxed);

        DeferredTestPointerScannerBindings::respond_to_first_matching(
            &queued_commands,
            |privileged_command| matches!(privileged_command, PrivilegedCommand::PointerScan(PointerScanCommand::Start { .. })),
            PointerScanStartResponse {
                pointer_scan_summary: Some(create_pointer_scan_summary(11, 0x3010)),
            }
            .to_engine_response(),
        );

        wait_for_condition("pointer scanner start response application", || {
            pointer_scanner_view_data
                .read("Pointer scanner start response wait")
                .map(|pointer_scanner_view_data_guard| {
                    !pointer_scanner_view_data_guard.is_starting_scan
                        && pointer_scanner_view_data_guard
                            .pointer_scan_summary
                            .as_ref()
                            .map(PointerScanSummary::get_session_id)
                            == Some(11)
                })
                .unwrap_or(false)
        });
        wait_for_condition("pointer scanner start response repaint request", || {
            repaint_request_count.load(Ordering::Relaxed) > repaint_request_count_before_start_response
        });
    }

    #[test]
    fn start_scan_with_active_session_dispatches_validate_request_and_applies_response() {
        let dependency_container = DependencyContainer::new();
        let mut pointer_scanner_view_data = PointerScannerViewData::new();
        pointer_scanner_view_data.apply_summary(Some(create_pointer_scan_summary(7, 0x3010)));
        pointer_scanner_view_data.validation_target_address_input = PointerScannerViewData::create_hex_input("0x4010".to_string());
        let pointer_scanner_view_data = dependency_container.register(pointer_scanner_view_data);
        let deferred_pointer_scanner_bindings = DeferredTestPointerScannerBindings::new();
        let queued_commands = deferred_pointer_scanner_bindings.get_queued_commands();
        let engine_bindings: Arc<RwLock<dyn EngineApiUnprivilegedBindings>> = Arc::new(RwLock::new(deferred_pointer_scanner_bindings));
        let engine_unprivileged_state = EngineUnprivilegedState::new(engine_bindings);

        PointerScannerViewData::start_scan(pointer_scanner_view_data.clone(), engine_unprivileged_state);
        wait_for_condition("pointer scanner validate dispatch", || {
            queued_commands
                .lock()
                .map(|queued_commands_guard| queued_commands_guard.len() >= 1)
                .unwrap_or(false)
        });

        {
            let queued_commands_guard = queued_commands
                .lock()
                .expect("Expected the deferred pointer scanner queued commands lock after validate.");
            assert!(matches!(
                queued_commands_guard
                    .first()
                    .map(|queued_command| &queued_command.privileged_command),
                Some(PrivilegedCommand::PointerScan(PointerScanCommand::Validate { .. }))
            ));
        }

        {
            let pointer_scanner_view_data_guard = pointer_scanner_view_data
                .read("Pointer scanner validate pending state test")
                .expect("Expected the pointer scanner view data read guard while the validate request is pending.");
            assert!(pointer_scanner_view_data_guard.is_validating_scan);
            assert_eq!(
                pointer_scanner_view_data_guard
                    .pointer_scan_summary
                    .as_ref()
                    .map(PointerScanSummary::get_session_id),
                Some(7)
            );
            assert_eq!(
                pointer_scanner_view_data_guard.status_message,
                "Validating pointer scan session 7 | Target 0x4010"
            );
        }

        DeferredTestPointerScannerBindings::respond_to_first_matching(
            &queued_commands,
            |privileged_command| matches!(privileged_command, PrivilegedCommand::PointerScan(PointerScanCommand::Validate { .. })),
            PointerScanValidateResponse {
                validation_performed: true,
                validation_target_address: Some(0x4010),
                pruned_node_count: 1,
                status_message: "Validated pointer scan session 7 against 0x4010. Pruned 1 nodes.".to_string(),
                pointer_scan_summary: Some(create_pointer_scan_summary(7, 0x4010)),
            }
            .to_engine_response(),
        );

        wait_for_condition("pointer scanner validate response application", || {
            pointer_scanner_view_data
                .read("Pointer scanner validate response wait")
                .map(|pointer_scanner_view_data_guard| {
                    !pointer_scanner_view_data_guard.is_validating_scan
                        && pointer_scanner_view_data_guard
                            .pointer_scan_summary
                            .as_ref()
                            .map(PointerScanSummary::get_target_address)
                            == Some(0x4010)
                })
                .unwrap_or(false)
        });
    }

    #[test]
    fn apply_expand_response_ignores_stale_session_state_revisions() {
        let mut pointer_scanner_view_data = PointerScannerViewData::new();
        pointer_scanner_view_data.apply_summary(Some(create_pointer_scan_summary(7, 0x3010)));
        pointer_scanner_view_data.pending_parent_node_ids.insert(None);
        let stale_session_state_revision = pointer_scanner_view_data.session_state_revision;
        pointer_scanner_view_data.apply_summary(Some(create_pointer_scan_summary(8, 0x4010)));

        pointer_scanner_view_data.apply_expand_response(
            stale_session_state_revision,
            PointerScanExpandResponse {
                session_id: 7,
                parent_node_id: None,
                pointer_scan_nodes: vec![PointerScanNode::new(
                    9,
                    None,
                    PointerScanNodeType::Static,
                    1,
                    0x5000,
                    0x5FF0,
                    0x6000,
                    0x10,
                    "old.exe".to_string(),
                    0x10,
                    Vec::new(),
                )],
            },
        );

        assert!(pointer_scanner_view_data.root_node_ids.is_empty());
        assert!(pointer_scanner_view_data.nodes_by_id.is_empty());
        assert!(pointer_scanner_view_data.pending_parent_node_ids.is_empty());
    }

    #[test]
    fn start_scan_with_invalid_max_depth_surfaces_status_message() {
        let dependency_container = DependencyContainer::new();
        let mut pointer_scanner_view_data = PointerScannerViewData::new();
        pointer_scanner_view_data.max_depth_input = PointerScannerViewData::create_unsigned_input("abc".to_string());
        let pointer_scanner_view_data = dependency_container.register(pointer_scanner_view_data);
        let deferred_pointer_scanner_bindings = DeferredTestPointerScannerBindings::new();
        let engine_bindings: Arc<RwLock<dyn EngineApiUnprivilegedBindings>> = Arc::new(RwLock::new(deferred_pointer_scanner_bindings));
        let engine_unprivileged_state = EngineUnprivilegedState::new(engine_bindings);

        PointerScannerViewData::start_scan(pointer_scanner_view_data.clone(), engine_unprivileged_state);

        let pointer_scanner_view_data_guard = pointer_scanner_view_data
            .read("Pointer scanner invalid max depth status test")
            .expect("Expected the pointer scanner view data read guard after invalid max depth.");
        assert_eq!(pointer_scanner_view_data_guard.status_message, "Cannot start pointer scan: invalid max depth.");
        assert!(!pointer_scanner_view_data_guard.is_starting_scan);
    }

    #[test]
    fn apply_expand_response_ignores_session_mismatches_and_clears_pending_requests() {
        let mut pointer_scanner_view_data = PointerScannerViewData::new();
        pointer_scanner_view_data.apply_summary(Some(create_pointer_scan_summary(7, 0x3010)));
        pointer_scanner_view_data.pending_parent_node_ids.insert(None);
        let session_state_revision = pointer_scanner_view_data.session_state_revision;

        pointer_scanner_view_data.apply_expand_response(
            session_state_revision,
            PointerScanExpandResponse {
                session_id: 8,
                parent_node_id: None,
                pointer_scan_nodes: vec![PointerScanNode::new(
                    10,
                    None,
                    PointerScanNodeType::Static,
                    1,
                    0x7000,
                    0x7FF0,
                    0x8000,
                    0x20,
                    "new.exe".to_string(),
                    0x20,
                    Vec::new(),
                )],
            },
        );

        assert!(pointer_scanner_view_data.root_node_ids.is_empty());
        assert!(pointer_scanner_view_data.nodes_by_id.is_empty());
        assert!(pointer_scanner_view_data.pending_parent_node_ids.is_empty());
        assert!(pointer_scanner_view_data.loaded_parent_node_ids.is_empty());
    }

    #[test]
    fn new_defaults_pointer_targets_and_offset_to_hexadecimal_inputs() {
        let pointer_scanner_view_data = PointerScannerViewData::new();

        assert_eq!(
            pointer_scanner_view_data
                .target_address_input
                .get_anonymous_value_string_format(),
            AnonymousValueStringFormat::Hexadecimal
        );
        assert_eq!(
            pointer_scanner_view_data
                .validation_target_address_input
                .get_anonymous_value_string_format(),
            AnonymousValueStringFormat::Hexadecimal
        );
        assert_eq!(
            pointer_scanner_view_data
                .offset_radius_input
                .get_anonymous_value_string_format(),
            AnonymousValueStringFormat::Hexadecimal
        );
        assert_eq!(
            pointer_scanner_view_data
                .offset_radius_input
                .get_anonymous_value_string(),
            "0x800"
        );
    }

    #[test]
    fn apply_summary_formats_offset_radius_as_hexadecimal_input() {
        let mut pointer_scanner_view_data = PointerScannerViewData::new();
        let pointer_scan_summary = create_pointer_scan_summary(7, 0x3010);

        pointer_scanner_view_data.apply_summary(Some(pointer_scan_summary));

        assert_eq!(
            pointer_scanner_view_data
                .target_address_input
                .get_anonymous_value_string_format(),
            AnonymousValueStringFormat::Hexadecimal
        );
        assert_eq!(
            pointer_scanner_view_data
                .target_address_input
                .get_anonymous_value_string(),
            "0x3010"
        );
        assert_eq!(
            pointer_scanner_view_data
                .offset_radius_input
                .get_anonymous_value_string(),
            "0x100"
        );
    }

    #[test]
    fn synchronize_pointer_size_with_process_bitness_updates_default_selection_without_session() {
        let mut pointer_scanner_view_data = PointerScannerViewData::new();

        pointer_scanner_view_data.synchronize_pointer_size_with_process_bitness(Bitness::Bit32);

        assert_eq!(pointer_scanner_view_data.pointer_size, PointerScanPointerSize::Pointer32);
        assert_eq!(
            pointer_scanner_view_data
                .pointer_size_data_type_selection
                .active_data_type()
                .get_data_type_id(),
            "u32"
        );
    }

    #[test]
    fn synchronize_pointer_size_with_process_bitness_preserves_active_session_selection() {
        let mut pointer_scanner_view_data = PointerScannerViewData::new();
        pointer_scanner_view_data.apply_summary(Some(create_pointer_scan_summary(7, 0x3010)));

        pointer_scanner_view_data.synchronize_pointer_size_with_process_bitness(Bitness::Bit32);

        assert_eq!(pointer_scanner_view_data.pointer_size, PointerScanPointerSize::Pointer64);
        assert_eq!(
            pointer_scanner_view_data
                .pointer_size_data_type_selection
                .active_data_type()
                .get_data_type_id(),
            "u64"
        );
    }

    #[test]
    fn force_pointer_size_from_process_bitness_overrides_stale_selection() {
        let mut pointer_scanner_view_data = PointerScannerViewData::new();
        pointer_scanner_view_data.apply_summary(Some(create_pointer_scan_summary(7, 0x3010)));

        pointer_scanner_view_data.force_pointer_size_from_process_bitness(Bitness::Bit32);

        assert_eq!(pointer_scanner_view_data.pointer_size, PointerScanPointerSize::Pointer32);
        assert_eq!(
            pointer_scanner_view_data
                .pointer_size_data_type_selection
                .active_data_type()
                .get_data_type_id(),
            "u32"
        );
    }

    #[test]
    fn apply_summary_none_clears_the_active_pointer_scan_session_state() {
        let mut pointer_scanner_view_data = create_pointer_scanner_view_data();

        pointer_scanner_view_data.apply_summary(None);

        assert!(pointer_scanner_view_data.pointer_scan_summary.is_none());
        assert!(pointer_scanner_view_data.root_node_ids.is_empty());
        assert!(pointer_scanner_view_data.nodes_by_id.is_empty());
        assert!(pointer_scanner_view_data.child_node_ids_by_parent_id.is_empty());
        assert!(pointer_scanner_view_data.expanded_node_ids.is_empty());
        assert_eq!(pointer_scanner_view_data.status_message, "No pointer scan session.");
    }

    #[test]
    fn has_active_session_reflects_pointer_scan_summary_presence() {
        let mut pointer_scanner_view_data = PointerScannerViewData::new();
        assert!(!pointer_scanner_view_data.has_active_session());

        pointer_scanner_view_data.apply_summary(Some(create_pointer_scan_summary(9, 0x4010)));

        assert!(pointer_scanner_view_data.has_active_session());
    }
}
