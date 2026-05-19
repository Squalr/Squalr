use crate::ui::geometry::safe_clamp_ord;
use crate::ui::list_navigation::{ListNavigationDirection, resolve_next_index};
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
use squalr_engine_api::structures::data_types::{built_in_types::i32::data_type_i32::DataTypeI32, data_type_ref::DataTypeRef};
use squalr_engine_api::structures::data_values::anonymous_value_string::AnonymousValueString;
use squalr_engine_api::structures::data_values::anonymous_value_string_format::AnonymousValueStringFormat;
use squalr_engine_api::structures::data_values::container_type::ContainerType;
use squalr_engine_api::structures::memory::address_display::{is_virtual_module_address, try_resolve_virtual_module_address};
use squalr_engine_api::structures::memory::bitness::Bitness;
use squalr_engine_api::structures::pointer_scans::pointer_scan_address_space::PointerScanAddressSpace;
use squalr_engine_api::structures::pointer_scans::pointer_scan_node::PointerScanNode;
use squalr_engine_api::structures::pointer_scans::pointer_scan_node_type::PointerScanNodeType;
use squalr_engine_api::structures::pointer_scans::pointer_scan_pointer_size::PointerScanPointerSize;
use squalr_engine_api::structures::pointer_scans::pointer_scan_summary::PointerScanSummary;
use squalr_engine_api::structures::pointer_scans::pointer_scan_target_descriptor::PointerScanTargetDescriptor;
use squalr_engine_api::structures::pointer_scans::pointer_scan_target_request::PointerScanTargetRequest;
use squalr_engine_api::structures::settings::scan_settings::ScanSettings;
use squalr_engine_api::structures::structs::symbolic_field_definition::SymbolicFieldDefinition;
use squalr_engine_session::engine_unprivileged_state::EngineUnprivilegedState;
use std::collections::{HashMap, HashSet};
use std::ops::Range;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;
use std::thread;

type RepaintRequestCallback = Arc<dyn Fn() + Send + Sync>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PointerScannerTreeRow {
    pub node_id: u64,
    pub has_children: bool,
    pub is_navigate_up_row: bool,
    pub is_selected: bool,
    pub primary_text: String,
    pub value_text: String,
    pub resolved_address_text: String,
    pub depth_text: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct PointerScannerPageRequest {
    parent_node_id: Option<u64>,
    page_index: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct PointerScannerLoadedPage {
    node_ids: Vec<u64>,
    last_page_index: u64,
    total_node_count: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct PointerScannerNavigationState {
    parent_node_id: Option<u64>,
    page_index: u64,
    last_page_index: u64,
    total_node_count: u64,
    selected_node_id: Option<u64>,
}

#[derive(Clone)]
pub struct PointerScannerViewData {
    pub primary_splitter_ratio: f32,
    pub value_splitter_ratio: f32,
    pub resolved_splitter_ratio: f32,
    pub target_address_input: AnonymousValueString,
    pub validation_target_address_input: AnonymousValueString,
    pub target_value_input: AnonymousValueString,
    pub validation_target_value_input: AnonymousValueString,
    pub pointer_scan_address_space: PointerScanAddressSpace,
    pub pointer_size: PointerScanPointerSize,
    pub pointer_size_data_type_selection: DataTypeSelection,
    pub target_data_type_selection: DataTypeSelection,
    pub max_depth_input: AnonymousValueString,
    pub offset_radius_input: AnonymousValueString,
    pub status_message: String,
    pub pointer_scan_summary: Option<PointerScanSummary>,
    browse_page_size: u64,
    current_context_parent_node_id: Option<u64>,
    current_context_node_ids: Vec<u64>,
    current_page_index: u64,
    cached_last_page_index: u64,
    current_context_total_node_count: u64,
    navigation_stack: Vec<PointerScannerNavigationState>,
    pub nodes_by_id: HashMap<u64, PointerScanNode>,
    loaded_pages_by_request: HashMap<PointerScannerPageRequest, PointerScannerLoadedPage>,
    pending_page_requests: HashSet<PointerScannerPageRequest>,
    queued_page_requests: HashSet<PointerScannerPageRequest>,
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
    pub const DEFAULT_PRIMARY_SPLITTER_RATIO: f32 = 0.40;
    pub const DEFAULT_VALUE_SPLITTER_RATIO: f32 = 0.64;
    pub const DEFAULT_RESOLVED_SPLITTER_RATIO: f32 = 0.84;

    pub fn new() -> Self {
        let pointer_size = PointerScanPointerSize::Pointer64;

        Self {
            primary_splitter_ratio: Self::DEFAULT_PRIMARY_SPLITTER_RATIO,
            value_splitter_ratio: Self::DEFAULT_VALUE_SPLITTER_RATIO,
            resolved_splitter_ratio: Self::DEFAULT_RESOLVED_SPLITTER_RATIO,
            target_address_input: Self::create_hex_input(String::new()),
            validation_target_address_input: Self::create_hex_input(String::new()),
            target_value_input: Self::create_unsigned_input(String::new()),
            validation_target_value_input: Self::create_unsigned_input(String::new()),
            pointer_scan_address_space: PointerScanAddressSpace::Auto,
            pointer_size,
            pointer_size_data_type_selection: DataTypeSelection::new(Self::pointer_size_data_type_ref(pointer_size)),
            target_data_type_selection: DataTypeSelection::new(Self::target_data_type_ref(DataTypeI32::DATA_TYPE_ID)),
            max_depth_input: Self::create_unsigned_input(String::from("5")),
            offset_radius_input: Self::create_unsigned_input(String::from("2048")),
            status_message: String::from("No pointer scan session."),
            pointer_scan_summary: None,
            browse_page_size: ScanSettings::default().results_page_size as u64,
            current_context_parent_node_id: None,
            current_context_node_ids: Vec::new(),
            current_page_index: 0,
            cached_last_page_index: 0,
            current_context_total_node_count: 0,
            navigation_stack: Vec::new(),
            nodes_by_id: HashMap::new(),
            loaded_pages_by_request: HashMap::new(),
            pending_page_requests: HashSet::new(),
            queued_page_requests: HashSet::new(),
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
                            pointer_scanner_view_data_guard.queue_expand_request(PointerScannerPageRequest {
                                parent_node_id: None,
                                page_index: 0,
                            });
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
            Self::validate_address_scan(pointer_scanner_view_data, engine_unprivileged_state);

            return;
        }

        Self::start_new_address_scan(pointer_scanner_view_data, engine_unprivileged_state);
    }

    pub fn start_value_scan(
        pointer_scanner_view_data: Dependency<Self>,
        engine_unprivileged_state: Arc<EngineUnprivilegedState>,
    ) {
        let should_validate_scan = {
            let pointer_scanner_view_data_guard = match pointer_scanner_view_data.read("Pointer scanner start value scan mode") {
                Some(pointer_scanner_view_data_guard) => pointer_scanner_view_data_guard,
                None => return,
            };

            if pointer_scanner_view_data_guard.is_resetting_scan {
                return;
            }

            pointer_scanner_view_data_guard.has_active_session()
        };

        if should_validate_scan {
            Self::validate_value_scan(pointer_scanner_view_data, engine_unprivileged_state);

            return;
        }

        Self::start_new_value_scan(pointer_scanner_view_data, engine_unprivileged_state);
    }

    fn start_new_address_scan(
        pointer_scanner_view_data: Dependency<Self>,
        engine_unprivileged_state: Arc<EngineUnprivilegedState>,
    ) {
        let (target_address_input, pointer_size, max_depth, offset_radius, address_space, session_request_revision) = {
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

            let address_space = pointer_scanner_view_data_guard.resolve_requested_address_space_for_new_address_scan();
            pointer_scanner_view_data_guard.is_starting_scan = true;
            let session_request_revision = pointer_scanner_view_data_guard.begin_session_request();
            pointer_scanner_view_data_guard.status_message = Self::format_starting_scan_status(
                &pointer_scanner_view_data_guard.target_address_input,
                pointer_scanner_view_data_guard.pointer_size,
                max_depth,
                offset_radius,
                address_space,
            );
            pointer_scanner_view_data_guard.request_repaint();

            (
                pointer_scanner_view_data_guard.target_address_input.clone(),
                pointer_scanner_view_data_guard.pointer_size,
                max_depth,
                offset_radius,
                address_space,
                session_request_revision,
            )
        };
        let pointer_scan_start_request = PointerScanStartRequest {
            target: PointerScanTargetRequest::address(target_address_input),
            pointer_size,
            max_depth,
            offset_radius,
            address_space,
        };
        let pointer_scanner_view_data_clone = pointer_scanner_view_data.clone();
        let pointer_scanner_view_data_for_dispatch = pointer_scanner_view_data.clone();

        let did_spawn_thread = Self::spawn_request_thread("pointer-scan-start", move || {
            let did_dispatch = pointer_scan_start_request.send(&engine_unprivileged_state, move |pointer_scan_start_response| {
                let pointer_scan_summary = pointer_scan_start_response.pointer_scan_summary.clone();

                if let Some(mut pointer_scanner_view_data_guard) = pointer_scanner_view_data_clone.write("Pointer scanner start scan response") {
                    pointer_scanner_view_data_guard.is_starting_scan = false;

                    if pointer_scanner_view_data_guard.should_apply_session_request(session_request_revision) && pointer_scan_start_response.success {
                        pointer_scanner_view_data_guard.apply_summary(pointer_scan_summary.clone());
                        if pointer_scan_summary.is_some() {
                            pointer_scanner_view_data_guard.queue_expand_request(PointerScannerPageRequest {
                                parent_node_id: None,
                                page_index: 0,
                            });
                        }

                        if let Some(pointer_scan_summary) = pointer_scan_summary.as_ref() {
                            pointer_scanner_view_data_guard.status_message = Self::format_start_completed_status(pointer_scan_summary);
                        }
                    } else if pointer_scanner_view_data_guard.should_apply_session_request(session_request_revision) {
                        pointer_scanner_view_data_guard.status_message = String::from("Cannot start pointer scan without an opened process.");
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

    fn start_new_value_scan(
        pointer_scanner_view_data: Dependency<Self>,
        engine_unprivileged_state: Arc<EngineUnprivilegedState>,
    ) {
        let (target_value_input, target_data_type_ref, pointer_size, max_depth, offset_radius, address_space, session_request_revision) = {
            let mut pointer_scanner_view_data_guard = match pointer_scanner_view_data.write("Pointer scanner start value scan") {
                Some(pointer_scanner_view_data_guard) => pointer_scanner_view_data_guard,
                None => return,
            };

            if pointer_scanner_view_data_guard.is_starting_scan || pointer_scanner_view_data_guard.is_resetting_scan {
                return;
            }

            let Some(max_depth) = Self::parse_unsigned_input(&pointer_scanner_view_data_guard.max_depth_input) else {
                pointer_scanner_view_data_guard.status_message = String::from("Cannot start pointer scan: invalid max depth.");
                pointer_scanner_view_data_guard.request_repaint();
                return;
            };
            let Some(offset_radius) = Self::parse_unsigned_input(&pointer_scanner_view_data_guard.offset_radius_input) else {
                pointer_scanner_view_data_guard.status_message = String::from("Cannot start pointer scan: invalid offset.");
                pointer_scanner_view_data_guard.request_repaint();
                return;
            };

            let address_space = pointer_scanner_view_data_guard.resolve_requested_address_space_for_new_value_scan();
            pointer_scanner_view_data_guard.is_starting_scan = true;
            let session_request_revision = pointer_scanner_view_data_guard.begin_session_request();
            pointer_scanner_view_data_guard.status_message = Self::format_starting_value_scan_status(
                &pointer_scanner_view_data_guard.target_value_input,
                pointer_scanner_view_data_guard
                    .target_data_type_selection
                    .active_data_type(),
                pointer_scanner_view_data_guard.pointer_size,
                max_depth,
                offset_radius,
                address_space,
            );
            pointer_scanner_view_data_guard.request_repaint();

            (
                pointer_scanner_view_data_guard.target_value_input.clone(),
                pointer_scanner_view_data_guard
                    .target_data_type_selection
                    .active_data_type()
                    .clone(),
                pointer_scanner_view_data_guard.pointer_size,
                max_depth,
                offset_radius,
                address_space,
                session_request_revision,
            )
        };
        let pointer_scan_start_request = PointerScanStartRequest {
            target: PointerScanTargetRequest::value(target_value_input, target_data_type_ref),
            pointer_size,
            max_depth,
            offset_radius,
            address_space,
        };
        let pointer_scanner_view_data_clone = pointer_scanner_view_data.clone();
        let pointer_scanner_view_data_for_dispatch = pointer_scanner_view_data.clone();

        let did_spawn_thread = Self::spawn_request_thread("pointer-scan-start-value", move || {
            let did_dispatch = pointer_scan_start_request.send(&engine_unprivileged_state, move |pointer_scan_start_response| {
                let pointer_scan_summary = pointer_scan_start_response.pointer_scan_summary.clone();

                if let Some(mut pointer_scanner_view_data_guard) = pointer_scanner_view_data_clone.write("Pointer scanner start value scan response") {
                    pointer_scanner_view_data_guard.is_starting_scan = false;

                    if pointer_scanner_view_data_guard.should_apply_session_request(session_request_revision) && pointer_scan_start_response.success {
                        pointer_scanner_view_data_guard.apply_summary(pointer_scan_summary.clone());
                        if pointer_scan_summary.is_some() {
                            pointer_scanner_view_data_guard.queue_expand_request(PointerScannerPageRequest {
                                parent_node_id: None,
                                page_index: 0,
                            });
                        }

                        if let Some(pointer_scan_summary) = pointer_scan_summary.as_ref() {
                            pointer_scanner_view_data_guard.status_message = Self::format_start_completed_status(pointer_scan_summary);
                        }
                    } else if pointer_scanner_view_data_guard.should_apply_session_request(session_request_revision) {
                        pointer_scanner_view_data_guard.status_message = String::from("Cannot start pointer scan without an opened process.");
                    }

                    pointer_scanner_view_data_guard.request_repaint();
                }
            });

            if !did_dispatch {
                Self::clear_start_scan_request_state(pointer_scanner_view_data_for_dispatch, "Pointer scanner start value scan dispatch failure");
            }
        });

        if !did_spawn_thread {
            Self::clear_start_scan_request_state(pointer_scanner_view_data, "Pointer scanner start value scan thread spawn failure");
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
            pointer_scanner_view_data_guard.clear_session_state_preserving_inputs();
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
                        pointer_scanner_view_data_guard.clear_session_state_preserving_inputs();
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

    fn validate_address_scan(
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
            target: PointerScanTargetRequest::address(validation_target_address_input),
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
                            pointer_scanner_view_data_guard.queue_expand_request(PointerScannerPageRequest {
                                parent_node_id: None,
                                page_index: 0,
                            });
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

    fn validate_value_scan(
        pointer_scanner_view_data: Dependency<Self>,
        engine_unprivileged_state: Arc<EngineUnprivilegedState>,
    ) {
        let (session_id, validation_target_value_input, validation_target_data_type_ref, session_request_revision) = {
            let mut pointer_scanner_view_data_guard = match pointer_scanner_view_data.write("Pointer scanner validate value scan") {
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
                return;
            };

            if pointer_scanner_view_data_guard.is_validating_scan || pointer_scanner_view_data_guard.is_resetting_scan {
                return;
            }

            pointer_scanner_view_data_guard.is_validating_scan = true;
            let session_request_revision = pointer_scanner_view_data_guard.begin_session_request();
            pointer_scanner_view_data_guard.status_message = Self::format_validating_value_scan_status(
                session_id,
                &pointer_scanner_view_data_guard.validation_target_value_input,
                pointer_scanner_view_data_guard
                    .target_data_type_selection
                    .active_data_type(),
            );
            pointer_scanner_view_data_guard.request_repaint();

            (
                session_id,
                pointer_scanner_view_data_guard
                    .validation_target_value_input
                    .clone(),
                pointer_scanner_view_data_guard
                    .target_data_type_selection
                    .active_data_type()
                    .clone(),
                session_request_revision,
            )
        };
        let pointer_scan_validate_request = PointerScanValidateRequest {
            session_id,
            target: PointerScanTargetRequest::value(validation_target_value_input, validation_target_data_type_ref),
        };
        let pointer_scanner_view_data_clone = pointer_scanner_view_data.clone();
        let pointer_scanner_view_data_for_dispatch = pointer_scanner_view_data.clone();

        let did_spawn_thread = Self::spawn_request_thread("pointer-scan-validate-value", move || {
            let did_dispatch = pointer_scan_validate_request.send(&engine_unprivileged_state, move |pointer_scan_validate_response| {
                let pointer_scan_summary = pointer_scan_validate_response.pointer_scan_summary.clone();

                if let Some(mut pointer_scanner_view_data_guard) = pointer_scanner_view_data_clone.write("Pointer scanner validate value scan response") {
                    pointer_scanner_view_data_guard.is_validating_scan = false;

                    if pointer_scanner_view_data_guard.should_apply_session_request(session_request_revision) {
                        pointer_scanner_view_data_guard.apply_summary(pointer_scan_summary.clone());
                        if pointer_scan_summary.is_some() {
                            pointer_scanner_view_data_guard.queue_expand_request(PointerScannerPageRequest {
                                parent_node_id: None,
                                page_index: 0,
                            });
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
                Self::clear_validate_scan_request_state(pointer_scanner_view_data_for_dispatch, "Pointer scanner validate value scan dispatch failure");
            }
        });

        if !did_spawn_thread {
            Self::clear_validate_scan_request_state(pointer_scanner_view_data, "Pointer scanner validate value scan thread spawn failure");
        }
    }

    pub fn dispatch_queued_expand_requests(
        pointer_scanner_view_data: Dependency<Self>,
        engine_unprivileged_state: Arc<EngineUnprivilegedState>,
    ) {
        let queued_page_requests = {
            let mut pointer_scanner_view_data_guard = match pointer_scanner_view_data.write("Pointer scanner dispatch queued expand requests") {
                Some(pointer_scanner_view_data_guard) => pointer_scanner_view_data_guard,
                None => return,
            };

            if pointer_scanner_view_data_guard.queued_page_requests.is_empty() {
                return;
            }

            pointer_scanner_view_data_guard
                .queued_page_requests
                .drain()
                .collect::<Vec<_>>()
        };

        for queued_page_request in queued_page_requests {
            Self::request_expand(pointer_scanner_view_data.clone(), engine_unprivileged_state.clone(), queued_page_request);
        }
    }

    fn request_expand(
        pointer_scanner_view_data: Dependency<Self>,
        engine_unprivileged_state: Arc<EngineUnprivilegedState>,
        page_request: PointerScannerPageRequest,
    ) {
        let (session_id, session_state_revision, page_size) = {
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
                .pending_page_requests
                .contains(&page_request)
            {
                return;
            }

            if pointer_scanner_view_data_guard
                .loaded_pages_by_request
                .contains_key(&page_request)
            {
                return;
            }

            pointer_scanner_view_data_guard
                .pending_page_requests
                .insert(page_request.clone());
            pointer_scanner_view_data_guard.request_repaint();

            (
                session_id,
                pointer_scanner_view_data_guard.session_state_revision,
                pointer_scanner_view_data_guard.browse_page_size,
            )
        };
        let pointer_scan_expand_request = PointerScanExpandRequest {
            session_id,
            parent_node_id: page_request.parent_node_id,
            page_index: page_request.page_index,
            page_size,
        };
        let pointer_scanner_view_data_clone = pointer_scanner_view_data.clone();
        let pointer_scanner_view_data_for_dispatch = pointer_scanner_view_data.clone();
        let page_request_for_dispatch = page_request.clone();

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
                    page_request_for_dispatch,
                    "Pointer scanner request expand dispatch failure",
                );
            }
        });

        if !did_spawn_thread {
            Self::clear_expand_request_state(pointer_scanner_view_data, page_request, "Pointer scanner request expand thread spawn failure");
        }
    }

    pub fn toggle_node_expansion(
        pointer_scanner_view_data: Dependency<Self>,
        _engine_unprivileged_state: Arc<EngineUnprivilegedState>,
        node_id: u64,
    ) {
        Self::navigate_into_node_context(pointer_scanner_view_data, node_id);
    }

    pub fn navigate_back(pointer_scanner_view_data: Dependency<Self>) {
        if let Some(mut pointer_scanner_view_data_guard) = pointer_scanner_view_data.write("Pointer scanner navigate back") {
            let Some(navigation_state) = pointer_scanner_view_data_guard.navigation_stack.pop() else {
                return;
            };

            let page_request = PointerScannerPageRequest {
                parent_node_id: navigation_state.parent_node_id,
                page_index: navigation_state.page_index,
            };

            pointer_scanner_view_data_guard.set_current_context_page(
                page_request,
                navigation_state.last_page_index,
                navigation_state.total_node_count,
                navigation_state.selected_node_id,
            );
            pointer_scanner_view_data_guard.request_repaint();
        }
    }

    pub fn can_navigate_back(pointer_scanner_view_data: Dependency<Self>) -> bool {
        pointer_scanner_view_data
            .read("Pointer scanner can navigate back")
            .map(|pointer_scanner_view_data_guard| !pointer_scanner_view_data_guard.navigation_stack.is_empty())
            .unwrap_or(false)
    }

    pub fn navigate_into_node_context(
        pointer_scanner_view_data: Dependency<Self>,
        node_id: u64,
    ) {
        if let Some(mut pointer_scanner_view_data_guard) = pointer_scanner_view_data.write("Pointer scanner navigate into node context") {
            let Some(pointer_scan_node) = pointer_scanner_view_data_guard.nodes_by_id.get(&node_id) else {
                return;
            };

            if !pointer_scan_node.has_children() {
                return;
            }

            pointer_scanner_view_data_guard.push_navigation_state();
            pointer_scanner_view_data_guard.set_current_context_page(
                PointerScannerPageRequest {
                    parent_node_id: Some(node_id),
                    page_index: 0,
                },
                0,
                0,
                None,
            );
            pointer_scanner_view_data_guard.request_repaint();
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

    pub fn navigate_node_selection(
        pointer_scanner_view_data: Dependency<Self>,
        direction: ListNavigationDirection,
    ) -> Option<u64> {
        let mut pointer_scanner_view_data_guard = pointer_scanner_view_data.write("Pointer scanner navigate node selection")?;
        let selected_node_index = pointer_scanner_view_data_guard
            .selected_node_id
            .and_then(|selected_node_id| {
                pointer_scanner_view_data_guard
                    .current_context_node_ids
                    .iter()
                    .position(|node_id| *node_id == selected_node_id)
            });
        let next_selection_index = resolve_next_index(selected_node_index, pointer_scanner_view_data_guard.current_context_node_ids.len(), direction)?;
        let next_node_id = *pointer_scanner_view_data_guard
            .current_context_node_ids
            .get(next_selection_index)?;

        pointer_scanner_view_data_guard.selected_node_id = Some(next_node_id);
        pointer_scanner_view_data_guard.request_repaint();

        Some(next_node_id)
    }

    pub fn navigate_into_selected_node_context(pointer_scanner_view_data: Dependency<Self>) {
        let selected_node_id = pointer_scanner_view_data
            .read("Pointer scanner read selected node for navigation")
            .and_then(|pointer_scanner_view_data_guard| pointer_scanner_view_data_guard.selected_node_id);

        if let Some(selected_node_id) = selected_node_id {
            Self::navigate_into_node_context(pointer_scanner_view_data, selected_node_id);
        }
    }

    pub fn build_visible_rows(pointer_scanner_view_data: Dependency<Self>) -> Vec<PointerScannerTreeRow> {
        Self::build_visible_rows_in_range(pointer_scanner_view_data.clone(), 0..Self::get_visible_row_count(pointer_scanner_view_data))
    }

    pub fn navigate_first_root_page(pointer_scanner_view_data: Dependency<Self>) {
        Self::navigate_first_page(pointer_scanner_view_data);
    }

    pub fn navigate_last_root_page(pointer_scanner_view_data: Dependency<Self>) {
        Self::navigate_last_page(pointer_scanner_view_data);
    }

    pub fn navigate_previous_root_page(pointer_scanner_view_data: Dependency<Self>) {
        Self::navigate_previous_page(pointer_scanner_view_data);
    }

    pub fn navigate_next_root_page(pointer_scanner_view_data: Dependency<Self>) {
        Self::navigate_next_page(pointer_scanner_view_data);
    }

    pub fn set_root_page_index_string(
        pointer_scanner_view_data: Dependency<Self>,
        new_page_index_text: &str,
    ) {
        Self::set_page_index_string(pointer_scanner_view_data, new_page_index_text);
    }

    pub fn navigate_first_page(pointer_scanner_view_data: Dependency<Self>) {
        Self::set_page_index(pointer_scanner_view_data, 0);
    }

    pub fn navigate_last_page(pointer_scanner_view_data: Dependency<Self>) {
        let cached_last_page_index = match pointer_scanner_view_data.read("Pointer scanner navigation last page") {
            Some(pointer_scanner_view_data_guard) => pointer_scanner_view_data_guard.cached_last_page_index,
            None => return,
        };

        Self::set_page_index(pointer_scanner_view_data, cached_last_page_index);
    }

    pub fn navigate_previous_page(pointer_scanner_view_data: Dependency<Self>) {
        let current_page_index = match pointer_scanner_view_data.read("Pointer scanner navigation previous page") {
            Some(pointer_scanner_view_data_guard) => pointer_scanner_view_data_guard.current_page_index,
            None => return,
        };

        Self::set_page_index(pointer_scanner_view_data, current_page_index.saturating_sub(1));
    }

    pub fn navigate_next_page(pointer_scanner_view_data: Dependency<Self>) {
        let current_page_index = match pointer_scanner_view_data.read("Pointer scanner navigation next page") {
            Some(pointer_scanner_view_data_guard) => pointer_scanner_view_data_guard.current_page_index,
            None => return,
        };

        Self::set_page_index(pointer_scanner_view_data, current_page_index.saturating_add(1));
    }

    pub fn set_page_index_string(
        pointer_scanner_view_data: Dependency<Self>,
        new_page_index_text: &str,
    ) {
        let new_page_index = new_page_index_text
            .chars()
            .take_while(|character| character.is_ascii_digit())
            .collect::<String>()
            .parse::<u64>()
            .unwrap_or(0);

        Self::set_page_index(pointer_scanner_view_data, new_page_index);
    }

    pub fn build_root_page_label(pointer_scanner_view_data: Dependency<Self>) -> String {
        Self::build_page_label(pointer_scanner_view_data)
    }

    pub fn build_page_label(pointer_scanner_view_data: Dependency<Self>) -> String {
        let pointer_scanner_view_data_guard = match pointer_scanner_view_data.read("Pointer scanner build page label") {
            Some(pointer_scanner_view_data_guard) => pointer_scanner_view_data_guard,
            None => return String::from("0"),
        };

        pointer_scanner_view_data_guard.current_page_index.to_string()
    }

    pub fn build_root_page_stats_text(pointer_scanner_view_data: Dependency<Self>) -> String {
        Self::build_page_stats_text(pointer_scanner_view_data)
    }

    pub fn build_page_stats_text(pointer_scanner_view_data: Dependency<Self>) -> String {
        let pointer_scanner_view_data_guard = match pointer_scanner_view_data.read("Pointer scanner build page stats") {
            Some(pointer_scanner_view_data_guard) => pointer_scanner_view_data_guard,
            None => return String::from("Entries 0-0 of 0"),
        };

        pointer_scanner_view_data_guard.build_page_stats_text_internal()
    }

    pub fn build_current_context_text(pointer_scanner_view_data: Dependency<Self>) -> String {
        let pointer_scanner_view_data_guard = match pointer_scanner_view_data.read("Pointer scanner build current context text") {
            Some(pointer_scanner_view_data_guard) => pointer_scanner_view_data_guard,
            None => return String::from("Roots"),
        };

        pointer_scanner_view_data_guard.build_current_context_text_internal()
    }

    pub fn is_root_context(pointer_scanner_view_data: Dependency<Self>) -> bool {
        pointer_scanner_view_data
            .read("Pointer scanner is root context")
            .map(|pointer_scanner_view_data_guard| {
                pointer_scanner_view_data_guard
                    .current_context_parent_node_id
                    .is_none()
            })
            .unwrap_or(true)
    }

    pub fn get_visible_row_count(pointer_scanner_view_data: Dependency<Self>) -> usize {
        let pointer_scanner_view_data_guard = match pointer_scanner_view_data.read("Pointer scanner build visible rows") {
            Some(pointer_scanner_view_data_guard) => pointer_scanner_view_data_guard,
            None => return 0,
        };

        pointer_scanner_view_data_guard.current_context_node_ids.len()
            + usize::from(
                pointer_scanner_view_data_guard
                    .current_context_parent_node_id
                    .is_some(),
            )
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
        let start_index = row_range
            .start
            .min(pointer_scanner_view_data_guard.get_visible_row_count_internal());
        let end_index = row_range
            .end
            .min(pointer_scanner_view_data_guard.get_visible_row_count_internal());
        let mut pointer_scanner_tree_rows = Vec::with_capacity(end_index.saturating_sub(start_index));

        for row_index in start_index..end_index {
            if let Some(pointer_scanner_tree_row) = pointer_scanner_view_data_guard.build_tree_row_at_row_index(row_index) {
                pointer_scanner_tree_rows.push(pointer_scanner_tree_row);
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
            Self::format_target_descriptor(summary.get_target_descriptor()),
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
        let selected_node_id = pointer_scanner_view_data_guard.selected_node_id?;
        let selected_pointer_scan_node = pointer_scanner_view_data_guard
            .nodes_by_id
            .get(&selected_node_id)?;
        if selected_pointer_scan_node.has_children() {
            log::warn!("Select a leaf pointer node before adding it to the project.");
            return None;
        }
        let project_item_name = pointer_scanner_view_data_guard.build_selected_project_item_name()?;

        Some(ProjectItemsCreateRequest {
            parent_directory_path: target_directory_path.unwrap_or_default(),
            project_item_name,
            is_directory: false,
            address: Some(selected_pointer_scan_node.get_resolved_target_address()),
            module_name: Some(String::new()),
            data_type_id: Some(pointer_scanner_view_data_guard.get_target_data_type_id()),
            pointer_offsets: None,
        })
    }

    pub fn build_project_item_create_request_for_node(
        pointer_scanner_view_data: Dependency<Self>,
        node_id: u64,
        target_directory_path: Option<PathBuf>,
    ) -> Option<ProjectItemsCreateRequest> {
        let pointer_scanner_view_data_guard = pointer_scanner_view_data.read("Pointer scanner build project item create request for node")?;
        let pointer_scan_node = pointer_scanner_view_data_guard.nodes_by_id.get(&node_id)?;

        if pointer_scan_node.has_children() {
            return None;
        }

        let project_item_name = pointer_scanner_view_data_guard.build_project_item_name(node_id)?;

        Some(ProjectItemsCreateRequest {
            parent_directory_path: target_directory_path.unwrap_or_default(),
            project_item_name,
            is_directory: false,
            address: Some(pointer_scan_node.get_resolved_target_address()),
            module_name: Some(String::new()),
            data_type_id: Some(pointer_scanner_view_data_guard.get_target_data_type_id()),
            pointer_offsets: None,
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
        data_type_id: &str,
    ) {
        if let Some(mut pointer_scanner_view_data_guard) = pointer_scanner_view_data.write("Pointer scanner set scan target from project address") {
            let normalized_target_data_type_ref = Self::normalize_project_target_data_type_ref(data_type_id).unwrap_or_else(|| {
                pointer_scanner_view_data_guard
                    .target_data_type_selection
                    .active_data_type()
                    .clone()
            });
            let resolved_virtual_address = try_resolve_virtual_module_address(module_name, address);
            let formatted_address = Self::format_address(resolved_virtual_address.unwrap_or(address));
            pointer_scanner_view_data_guard.target_address_input = Self::create_hex_input(formatted_address.clone());
            pointer_scanner_view_data_guard.validation_target_address_input = Self::create_hex_input(formatted_address);
            if resolved_virtual_address.is_some() {
                let default_pointer_size = Self::default_pointer_size_for_module_name(module_name);

                pointer_scanner_view_data_guard.pointer_scan_address_space = PointerScanAddressSpace::GameMemory;
                pointer_scanner_view_data_guard.pointer_size = default_pointer_size;
                pointer_scanner_view_data_guard
                    .pointer_size_data_type_selection
                    .replace_selected_data_types(vec![Self::pointer_size_data_type_ref(default_pointer_size)]);
            }
            pointer_scanner_view_data_guard
                .target_data_type_selection
                .replace_selected_data_types(vec![normalized_target_data_type_ref]);
            pointer_scanner_view_data_guard.status_message = if let Some(virtual_address) = resolved_virtual_address {
                format!(
                    "Pointer scanner target autofilled from {}+0x{:X} as guest address {}.",
                    module_name,
                    address,
                    Self::format_address(virtual_address)
                )
            } else if module_name.trim().is_empty() {
                String::from("Pointer scanner target autofilled from the project explorer.")
            } else {
                format!(
                    "Pointer scanner target autofilled from {}+0x{:X}. Stored module-relative addresses are not resolved here, so verify the live target before starting.",
                    module_name, address
                )
            };
        }
    }

    fn normalize_project_target_data_type_ref(data_type_id: &str) -> Option<DataTypeRef> {
        let trimmed_data_type_id = data_type_id.trim();

        if trimmed_data_type_id.is_empty() {
            return None;
        }

        SymbolicFieldDefinition::from_str(data_type_id)
            .map(|symbolic_field_definition| symbolic_field_definition.get_data_type_ref().clone())
            .ok()
            .or_else(|| Some(Self::target_data_type_ref(trimmed_data_type_id)))
    }

    fn clear_session_state_preserving_inputs(&mut self) {
        self.session_state_revision = self.session_state_revision.saturating_add(1);
        self.pointer_scan_summary = None;
        self.current_context_parent_node_id = None;
        self.current_context_node_ids.clear();
        self.current_page_index = 0;
        self.cached_last_page_index = 0;
        self.current_context_total_node_count = 0;
        self.navigation_stack.clear();
        self.nodes_by_id.clear();
        self.loaded_pages_by_request.clear();
        self.pending_page_requests.clear();
        self.queued_page_requests.clear();
        self.selected_node_id = None;
        self.status_message = String::from("No pointer scan session.");
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
        self.current_context_parent_node_id = None;
        self.current_context_node_ids.clear();
        self.current_page_index = 0;
        self.cached_last_page_index = 0;
        self.current_context_total_node_count = 0;
        self.navigation_stack.clear();
        self.nodes_by_id.clear();
        self.loaded_pages_by_request.clear();
        self.pending_page_requests.clear();
        self.queued_page_requests.clear();
        self.selected_node_id = None;

        if let Some(pointer_scan_summary) = pointer_scan_summary {
            self.pointer_scan_address_space = pointer_scan_summary.get_address_space();
            self.pointer_size = pointer_scan_summary.get_pointer_size();
            self.pointer_size_data_type_selection
                .replace_selected_data_types(vec![Self::pointer_size_data_type_ref(self.pointer_size)]);
            self.max_depth_input = Self::create_unsigned_input(pointer_scan_summary.get_max_depth().to_string());
            self.offset_radius_input = Self::create_unsigned_input(pointer_scan_summary.get_offset_radius().to_string());
            self.current_context_total_node_count = pointer_scan_summary.get_root_node_count();
            self.cached_last_page_index = Self::calculate_last_page_index(pointer_scan_summary.get_root_node_count(), self.browse_page_size);
            self.status_message = Self::format_summary_status(&pointer_scan_summary);

            match pointer_scan_summary.get_target_descriptor() {
                PointerScanTargetDescriptor::Address { target_address } => {
                    let formatted_target_address = Self::format_address(*target_address);
                    self.target_address_input = Self::create_hex_input(formatted_target_address.clone());
                    self.validation_target_address_input = Self::create_hex_input(formatted_target_address);
                    self.target_value_input = Self::create_unsigned_input(String::new());
                    self.validation_target_value_input = Self::create_unsigned_input(String::new());
                }
                PointerScanTargetDescriptor::Value {
                    target_value, data_type_ref, ..
                } => {
                    self.target_address_input = Self::create_hex_input(String::new());
                    self.validation_target_address_input = Self::create_hex_input(String::new());
                    self.target_value_input = target_value.clone();
                    self.validation_target_value_input = target_value.clone();
                    self.target_data_type_selection
                        .replace_selected_data_types(vec![data_type_ref.clone()]);
                }
            }
        } else {
            self.pointer_scan_address_space = PointerScanAddressSpace::Auto;
            self.target_address_input = Self::create_hex_input(String::new());
            self.validation_target_address_input = Self::create_hex_input(String::new());
            self.target_value_input = Self::create_unsigned_input(String::new());
            self.validation_target_value_input = Self::create_unsigned_input(String::new());
            self.status_message = String::from("No pointer scan session.");
        }
    }

    fn get_target_data_type_id(&self) -> String {
        self.target_data_type_selection
            .active_data_type()
            .get_data_type_id()
            .to_string()
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
        page_request: PointerScannerPageRequest,
    ) {
        self.queued_page_requests.insert(page_request);
    }

    fn apply_expand_response(
        &mut self,
        session_state_revision: u64,
        pointer_scan_expand_response: PointerScanExpandResponse,
    ) {
        if self.session_state_revision != session_state_revision {
            return;
        }

        let page_request = PointerScannerPageRequest {
            parent_node_id: pointer_scan_expand_response.parent_node_id,
            page_index: pointer_scan_expand_response.page_index,
        };
        self.pending_page_requests.remove(&page_request);

        if self
            .pointer_scan_summary
            .as_ref()
            .map(PointerScanSummary::get_session_id)
            != Some(pointer_scan_expand_response.session_id)
        {
            return;
        }

        let node_ids = pointer_scan_expand_response
            .pointer_scan_nodes
            .iter()
            .map(PointerScanNode::get_node_id)
            .collect::<Vec<_>>();

        for pointer_scan_node in pointer_scan_expand_response.pointer_scan_nodes {
            self.nodes_by_id
                .insert(pointer_scan_node.get_node_id(), pointer_scan_node);
        }

        self.loaded_pages_by_request.insert(
            page_request.clone(),
            PointerScannerLoadedPage {
                node_ids: node_ids.clone(),
                last_page_index: pointer_scan_expand_response.last_page_index,
                total_node_count: pointer_scan_expand_response.total_node_count,
            },
        );

        if self.current_page_request() == page_request {
            self.current_context_node_ids = node_ids;
            self.cached_last_page_index = pointer_scan_expand_response.last_page_index;
            self.current_context_total_node_count = pointer_scan_expand_response.total_node_count;
            self.ensure_selection_on_current_page(self.selected_node_id);
        }
    }

    fn set_page_index(
        pointer_scanner_view_data: Dependency<Self>,
        new_page_index: u64,
    ) {
        let mut pointer_scanner_view_data_guard = match pointer_scanner_view_data.write("Pointer scanner set page index") {
            Some(pointer_scanner_view_data_guard) => pointer_scanner_view_data_guard,
            None => return,
        };
        let bounded_page_index = safe_clamp_ord(new_page_index, 0, pointer_scanner_view_data_guard.cached_last_page_index);

        if bounded_page_index == pointer_scanner_view_data_guard.current_page_index {
            return;
        }

        let current_context_parent_node_id = pointer_scanner_view_data_guard.current_context_parent_node_id;
        let cached_last_page_index = pointer_scanner_view_data_guard.cached_last_page_index;
        let current_context_total_node_count = pointer_scanner_view_data_guard.current_context_total_node_count;

        pointer_scanner_view_data_guard.set_current_context_page(
            PointerScannerPageRequest {
                parent_node_id: current_context_parent_node_id,
                page_index: bounded_page_index,
            },
            cached_last_page_index,
            current_context_total_node_count,
            None,
        );
        pointer_scanner_view_data_guard.request_repaint();
    }

    fn push_navigation_state(&mut self) {
        self.navigation_stack.push(PointerScannerNavigationState {
            parent_node_id: self.current_context_parent_node_id,
            page_index: self.current_page_index,
            last_page_index: self.cached_last_page_index,
            total_node_count: self.current_context_total_node_count,
            selected_node_id: self.selected_node_id,
        });
    }

    fn set_current_context_page(
        &mut self,
        page_request: PointerScannerPageRequest,
        last_page_index: u64,
        total_node_count: u64,
        preferred_selected_node_id: Option<u64>,
    ) {
        self.current_context_parent_node_id = page_request.parent_node_id;
        self.current_page_index = page_request.page_index;

        if let Some(loaded_page) = self.loaded_pages_by_request.get(&page_request).cloned() {
            self.current_context_node_ids = loaded_page.node_ids;
            self.cached_last_page_index = loaded_page.last_page_index;
            self.current_context_total_node_count = loaded_page.total_node_count;
            self.ensure_selection_on_current_page(preferred_selected_node_id);
        } else {
            self.current_context_node_ids.clear();
            self.cached_last_page_index = last_page_index;
            self.current_context_total_node_count = total_node_count;
            self.selected_node_id = None;
            self.queue_expand_request(page_request);
        }
    }

    fn current_page_request(&self) -> PointerScannerPageRequest {
        PointerScannerPageRequest {
            parent_node_id: self.current_context_parent_node_id,
            page_index: self.current_page_index,
        }
    }

    fn ensure_selection_on_current_page(
        &mut self,
        preferred_selected_node_id: Option<u64>,
    ) {
        let preferred_selected_node_id = preferred_selected_node_id.or(self.selected_node_id);

        if preferred_selected_node_id
            .map(|selected_node_id| self.is_node_visible_on_current_page(selected_node_id))
            .unwrap_or(false)
        {
            self.selected_node_id = preferred_selected_node_id;
            return;
        }

        self.selected_node_id = self.current_context_node_ids.first().copied();
    }

    fn is_node_visible_on_current_page(
        &self,
        node_id: u64,
    ) -> bool {
        self.current_context_node_ids.contains(&node_id)
    }

    fn build_page_stats_text_internal(&self) -> String {
        let label = if self.current_context_parent_node_id.is_some() { "Offsets" } else { "Roots" };
        let total_node_count = if self.current_context_parent_node_id.is_none() {
            self.pointer_scan_summary
                .as_ref()
                .map(PointerScanSummary::get_root_node_count)
                .unwrap_or(self.current_context_total_node_count)
        } else {
            self.current_context_total_node_count
        };

        if total_node_count == 0 {
            return format!("{label} 0-0 of 0");
        }

        if self.current_context_node_ids.is_empty() {
            return format!("{label} loading (0 of {total_node_count})");
        }

        let start_index = (self.current_page_index as usize)
            .saturating_mul(self.browse_page_size.max(1) as usize)
            .saturating_add(1);
        let end_index = start_index
            .saturating_add(self.current_context_node_ids.len())
            .saturating_sub(1);

        format!("{label} {start_index}-{end_index} of {total_node_count}")
    }

    fn build_current_context_text_internal(&self) -> String {
        match self.current_context_parent_node_id {
            Some(parent_node_id) => self
                .build_pointer_context_text(parent_node_id)
                .unwrap_or_else(|| String::from("Context unavailable")),
            None => String::from("Roots"),
        }
    }

    fn get_visible_row_count_internal(&self) -> usize {
        self.current_context_node_ids.len() + usize::from(self.current_context_parent_node_id.is_some())
    }

    fn build_tree_row_at_row_index(
        &self,
        row_index: usize,
    ) -> Option<PointerScannerTreeRow> {
        if self.current_context_parent_node_id.is_some() && row_index == 0 {
            return Some(self.build_navigate_up_row());
        }

        let node_index = row_index.saturating_sub(usize::from(self.current_context_parent_node_id.is_some()));
        let node_id = *self.current_context_node_ids.get(node_index)?;

        self.build_tree_row(node_id)
    }

    fn build_navigate_up_row(&self) -> PointerScannerTreeRow {
        let primary_text = self
            .current_context_parent_node_id
            .and_then(|parent_node_id| self.nodes_by_id.get(&parent_node_id))
            .map(|parent_pointer_scan_node| format!("Return to depth {}", parent_pointer_scan_node.get_depth()))
            .unwrap_or_else(|| String::from("Return"));

        PointerScannerTreeRow {
            node_id: 0,
            has_children: false,
            is_navigate_up_row: true,
            is_selected: false,
            primary_text,
            value_text: String::new(),
            resolved_address_text: String::new(),
            depth_text: String::new(),
        }
    }

    fn build_tree_row(
        &self,
        node_id: u64,
    ) -> Option<PointerScannerTreeRow> {
        let pointer_scan_node = self.nodes_by_id.get(&node_id)?;
        let is_selected = self.selected_node_id == Some(node_id);
        let is_root_context = self.current_context_parent_node_id.is_none();
        let primary_text = if is_root_context {
            Self::build_module_base_text(pointer_scan_node)
        } else {
            Self::format_pointer_offset(pointer_scan_node.get_pointer_offset())
        };
        let value_text = Self::format_address(pointer_scan_node.get_pointer_value());
        Some(PointerScannerTreeRow {
            node_id,
            has_children: pointer_scan_node.has_children(),
            is_navigate_up_row: false,
            is_selected,
            primary_text,
            value_text,
            resolved_address_text: Self::format_address(pointer_scan_node.get_resolved_target_address()),
            depth_text: format!("{} of {}", pointer_scan_node.get_depth(), pointer_scan_node.get_branch_total_depth()),
        })
    }

    fn build_selected_chain_text(&self) -> Option<String> {
        let selected_node_id = self.selected_node_id?;

        self.build_pointer_chain_text(selected_node_id)
    }

    fn build_selected_project_item_name(&self) -> Option<String> {
        let selected_node_id = self.selected_node_id?;

        self.build_project_item_name(selected_node_id)
    }

    pub fn has_active_session(&self) -> bool {
        self.pointer_scan_summary.is_some()
    }

    pub fn has_mutating_session_request_in_progress(&self) -> bool {
        self.is_starting_scan || self.is_validating_scan || self.is_resetting_scan
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
            if !Self::should_include_node_offset_in_chain(pointer_scan_node) {
                continue;
            }

            pointer_chain_segments.push(Self::format_pointer_offset(pointer_scan_node.get_pointer_offset()));
        }

        Some(pointer_chain_segments.join(" -> "))
    }

    fn build_project_item_name(
        &self,
        node_id: u64,
    ) -> Option<String> {
        let pointer_chain = self.collect_node_path(node_id)?;
        let root_pointer_scan_node = pointer_chain.first()?;
        let pointer_depth = pointer_chain
            .iter()
            .filter(|pointer_scan_node| Self::should_include_node_offset_in_chain(pointer_scan_node))
            .count();

        Some(format!("{} [{}]", Self::build_module_base_text(root_pointer_scan_node), pointer_depth))
    }

    fn build_pointer_context_text(
        &self,
        node_id: u64,
    ) -> Option<String> {
        let pointer_chain = self.collect_node_path(node_id)?;
        let root_pointer_scan_node = pointer_chain.first()?;
        let root_text = Self::build_module_base_text(root_pointer_scan_node);
        let offset_chain_text = pointer_chain
            .iter()
            .skip(1)
            .filter(|pointer_scan_node| Self::should_include_node_offset_in_chain(pointer_scan_node))
            .map(|pointer_scan_node| Self::format_pointer_offset(pointer_scan_node.get_pointer_offset()))
            .collect::<Vec<_>>()
            .join(" -> ");

        if offset_chain_text.is_empty() {
            Some(root_text)
        } else {
            Some(format!("{root_text} | {offset_chain_text}"))
        }
    }

    fn should_include_node_offset_in_chain(pointer_scan_node: &PointerScanNode) -> bool {
        pointer_scan_node.get_parent_node_id().is_some() || !pointer_scan_node.has_children()
    }

    fn build_module_base_text(pointer_scan_node: &PointerScanNode) -> String {
        if pointer_scan_node.get_pointer_scan_node_type() == PointerScanNodeType::Static {
            let module_name = pointer_scan_node.get_module_name();

            if module_name.is_empty() {
                Self::format_address(pointer_scan_node.get_pointer_address())
            } else {
                format!("{}+0x{:X}", module_name, pointer_scan_node.get_module_offset())
            }
        } else {
            Self::format_address(pointer_scan_node.get_pointer_address())
        }
    }

    fn format_pointer_offset(pointer_offset: i64) -> String {
        if pointer_offset >= 0 {
            format!("0x{:X}", pointer_offset as u64)
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
            "Session {} | Target {} | Space {} | Roots {} | Nodes {} (Static {} / Heap {})",
            pointer_scan_summary.get_session_id(),
            Self::format_target_descriptor(pointer_scan_summary.get_target_descriptor()),
            pointer_scan_summary.get_address_space(),
            pointer_scan_summary.get_root_node_count(),
            pointer_scan_summary.get_total_node_count(),
            pointer_scan_summary.get_total_static_node_count(),
            pointer_scan_summary.get_total_heap_node_count(),
        )
    }

    fn format_start_completed_status(pointer_scan_summary: &PointerScanSummary) -> String {
        format!("Pointer scan complete | {}", Self::format_summary_status(pointer_scan_summary))
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
        address_space: PointerScanAddressSpace,
    ) -> String {
        format!(
            "Starting pointer scan | Target {} | Space {} | Pointer Size {} | Depth {} | Offset {}",
            Self::format_status_input_value(target_address_input),
            address_space,
            pointer_size,
            max_depth,
            Self::format_hexadecimal(offset_radius),
        )
    }

    fn format_starting_value_scan_status(
        target_value_input: &AnonymousValueString,
        target_data_type_ref: &DataTypeRef,
        pointer_size: PointerScanPointerSize,
        max_depth: u64,
        offset_radius: u64,
        address_space: PointerScanAddressSpace,
    ) -> String {
        format!(
            "Starting pointer scan | Value {} | Type {} | Space {} | Pointer Size {} | Depth {} | Offset {}",
            Self::format_status_input_value(target_value_input),
            target_data_type_ref.get_data_type_id(),
            address_space,
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

    fn format_validating_value_scan_status(
        session_id: u64,
        validation_target_value_input: &AnonymousValueString,
        target_data_type_ref: &DataTypeRef,
    ) -> String {
        format!(
            "Validating pointer scan session {} | Value {} | Type {}",
            session_id,
            Self::format_status_input_value(validation_target_value_input),
            target_data_type_ref.get_data_type_id(),
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

    fn format_target_descriptor(pointer_scan_target_descriptor: &PointerScanTargetDescriptor) -> String {
        match pointer_scan_target_descriptor {
            PointerScanTargetDescriptor::Address { target_address } => Self::format_address(*target_address),
            PointerScanTargetDescriptor::Value {
                target_value,
                data_type_ref,
                target_address_count,
            } => format!(
                "{} ({}, {} matches)",
                Self::format_status_input_value(target_value),
                data_type_ref.get_data_type_id(),
                target_address_count,
            ),
        }
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
        page_request: PointerScannerPageRequest,
        error_context: &'static str,
    ) {
        if let Some(mut pointer_scanner_view_data_guard) = pointer_scanner_view_data.write(error_context) {
            pointer_scanner_view_data_guard
                .pending_page_requests
                .remove(&page_request);
            pointer_scanner_view_data_guard.request_repaint();
        }
    }

    fn calculate_last_page_index(
        total_node_count: u64,
        page_size: u64,
    ) -> u64 {
        if total_node_count == 0 || page_size == 0 {
            0
        } else {
            total_node_count
                .saturating_sub(1)
                .checked_div(page_size)
                .unwrap_or(0)
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
        self.pointer_size = PointerScanPointerSize::from_process_bitness(process_bitness);
        self.pointer_size_data_type_selection
            .replace_selected_data_types(vec![Self::pointer_size_data_type_ref(self.pointer_size)]);
    }

    fn pointer_size_data_type_ref(pointer_size: PointerScanPointerSize) -> DataTypeRef {
        pointer_size.to_data_type_ref()
    }

    fn default_pointer_size_for_module_name(module_name: &str) -> PointerScanPointerSize {
        if module_name.eq_ignore_ascii_case("gc_wii")
            || module_name.eq_ignore_ascii_case("MEM1")
            || module_name.eq_ignore_ascii_case("MEM2")
            || module_name.eq_ignore_ascii_case("GC")
            || module_name.eq_ignore_ascii_case("Wii")
            || module_name.to_ascii_lowercase().starts_with("gba_wm_")
            || module_name.to_ascii_lowercase().starts_with("gba_im_")
        {
            PointerScanPointerSize::Pointer32be
        } else {
            PointerScanPointerSize::Pointer64
        }
    }

    pub fn resolve_requested_address_space_for_new_address_scan(&self) -> PointerScanAddressSpace {
        match self.pointer_scan_address_space {
            PointerScanAddressSpace::Auto => Self::parse_unsigned_input(&self.target_address_input)
                .filter(|target_address| is_virtual_module_address(*target_address))
                .map(|_| PointerScanAddressSpace::GameMemory)
                .unwrap_or(PointerScanAddressSpace::EmulatorMemory),
            PointerScanAddressSpace::GameMemory => PointerScanAddressSpace::GameMemory,
            PointerScanAddressSpace::EmulatorMemory => PointerScanAddressSpace::EmulatorMemory,
        }
    }

    pub fn resolve_requested_address_space_for_new_value_scan(&self) -> PointerScanAddressSpace {
        match self.pointer_scan_address_space {
            PointerScanAddressSpace::Auto => PointerScanAddressSpace::EmulatorMemory,
            PointerScanAddressSpace::GameMemory => PointerScanAddressSpace::GameMemory,
            PointerScanAddressSpace::EmulatorMemory => PointerScanAddressSpace::EmulatorMemory,
        }
    }

    fn target_data_type_ref(data_type_id: &str) -> DataTypeRef {
        let trimmed_data_type_id = data_type_id.trim();

        if trimmed_data_type_id.is_empty() {
            DataTypeRef::new(DataTypeI32::DATA_TYPE_ID)
        } else {
            DataTypeRef::new(trimmed_data_type_id)
        }
    }

    pub fn synchronize_pointer_size_from_selection(&mut self) {
        let selected_pointer_size_data_type = self.pointer_size_data_type_selection.active_data_type().clone();
        let selected_pointer_size = PointerScanPointerSize::from_data_type_ref(&selected_pointer_size_data_type).unwrap_or(self.pointer_size);

        self.pointer_size_data_type_selection
            .replace_selected_data_types(vec![selected_pointer_size_data_type.clone()]);
        self.pointer_size = selected_pointer_size;
    }
}
