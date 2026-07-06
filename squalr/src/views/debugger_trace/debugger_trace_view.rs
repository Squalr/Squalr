use crate::{
    app_context::AppContext,
    ui::widgets::controls::{
        combo_box::combo_box_view::ComboBoxView,
        context_menu::context_menu::{ContextMenu, ContextMenuSizing},
        data_type_selector::data_type_selector_view::DataTypeSelectorView,
        data_value_box::data_value_box_convert_item_view::DataValueBoxConvertItemView,
        groupbox::GroupBox,
        icon_button::IconButtonView,
        toolbar_menu::toolbar_menu_item_view::ToolbarMenuItemView,
    },
    views::{
        debugger_trace::{
            debugger_trace_entry_view::DebuggerTraceEntryView,
            view_data::debugger_trace_view_data::{DebuggerTraceInstructionKey, DebuggerTraceViewData, PendingDebuggerTraceStartRequest},
        },
        instruction_patch_action::InstructionPatchAction,
        process_selector::view_data::process_selector_view_data::ProcessSelectorViewData,
        project_explorer::project_hierarchy::{
            project_hierarchy_module_address_resolver::ProjectHierarchyModuleAddressResolver, view_data::project_hierarchy_view_data::ProjectHierarchyViewData,
        },
    },
};
use eframe::egui::{Align, Align2, Button, CursorIcon, Direction, Layout, Rect, Response, RichText, ScrollArea, Sense, Spinner, Ui, UiBuilder, Widget, pos2, vec2};
use epaint::{CornerRadius, Margin, Stroke, Vec2};
use squalr_engine_api::{
    commands::{
        debugger::{
            trace_pause::debugger_trace_pause_request::DebuggerTracePauseRequest, trace_resume::debugger_trace_resume_request::DebuggerTraceResumeRequest,
            trace_start::debugger_trace_start_request::DebuggerTraceStartRequest, trace_stop::debugger_trace_stop_request::DebuggerTraceStopRequest,
        },
        privileged_command_request::PrivilegedCommandRequest,
        project_items::create::project_items_create_request::ProjectItemsCreateRequest,
        unprivileged_command_request::UnprivilegedCommandRequest,
    },
    dependency_injection::dependency::Dependency,
    events::debugger::trace_session_updated::debugger_trace_session_updated_event::DebuggerTraceSessionUpdatedEvent,
    structures::{
        data_types::data_type_ref::DataTypeRef,
        data_values::{
            anonymous_value_string::AnonymousValueString, anonymous_value_string_format::AnonymousValueStringFormat, container_type::ContainerType,
        },
        debugger::{DebuggerDataBreakpointAccess, DebuggerTraceInstructionRecord, DebuggerTraceSessionDescriptor, DebuggerTraceTargetKind},
        memory::address_display::format_module_address,
        processes::target_architecture::TargetArchitecture,
        structs::{symbolic_field_definition::SymbolicFieldDefinition, symbolic_struct_definition::SymbolicStructDefinition},
    },
};
use squalr_engine_session::virtual_snapshots::{virtual_snapshot_query::VirtualSnapshotQuery, virtual_snapshot_query_result::VirtualSnapshotQueryResult};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

#[derive(Clone)]
pub struct DebuggerTraceView {
    app_context: Arc<AppContext>,
    debugger_trace_view_data: Dependency<DebuggerTraceViewData>,
    project_hierarchy_view_data: Dependency<ProjectHierarchyViewData>,
    process_selector_view_data: Dependency<ProcessSelectorViewData>,
}

#[derive(Clone, Copy)]
struct TraceColumnSplitters {
    hit_count: f32,
    instruction: f32,
    address: f32,
    value: f32,
}

impl DebuggerTraceView {
    pub const WINDOW_ID: &'static str = "window_debugger_trace";
    const PENDING_TRACE_START_TIMEOUT: Duration = Duration::from_secs(15);
    const ADD_TO_PROJECT_LABEL: &'static str = "Add to Project";
    const ADD_TO_PROJECT_ID: &'static str = "debugger_trace_ctx_add_to_project";
    const REPLACE_WITH_NO_OPERATION_LABEL: &'static str = "Replace with Code That Does Nothing";
    const REPLACE_WITH_NO_OPERATION_ID: &'static str = "debugger_trace_ctx_replace_with_nop";
    const RESTORE_ORIGINAL_CODE_LABEL: &'static str = "Restore Original Code";
    const RESTORE_ORIGINAL_CODE_ID: &'static str = "debugger_trace_ctx_restore_original_code";
    const TRACE_PREVIEW_VIRTUAL_SNAPSHOT_ID: &'static str = "debugger_trace_preview";
    const TRACE_PREVIEW_REFRESH_INTERVAL: Duration = Duration::from_millis(250);
    const VALUE_DISPLAY_FORMAT_POPUP_WIDTH: f32 = 220.0;

    fn display_format_icon(
        &self,
        anonymous_value_string_format: AnonymousValueStringFormat,
    ) -> eframe::egui::TextureHandle {
        let icon_library = &self.app_context.theme.icon_library;

        match anonymous_value_string_format {
            AnonymousValueStringFormat::Binary => icon_library.icon_handle_display_type_binary.clone(),
            AnonymousValueStringFormat::Decimal => icon_library.icon_handle_display_type_decimal.clone(),
            AnonymousValueStringFormat::Hexadecimal | AnonymousValueStringFormat::Address => icon_library.icon_handle_display_type_hexadecimal.clone(),
            AnonymousValueStringFormat::String
            | AnonymousValueStringFormat::Bool
            | AnonymousValueStringFormat::DataTypeRef
            | AnonymousValueStringFormat::Enumeration => icon_library.icon_handle_display_type_string.clone(),
        }
    }

    pub fn new(app_context: Arc<AppContext>) -> Self {
        let debugger_trace_view_data = app_context
            .dependency_container
            .register(DebuggerTraceViewData::new());
        let project_hierarchy_view_data = app_context
            .dependency_container
            .get_dependency::<ProjectHierarchyViewData>();
        let process_selector_view_data = app_context
            .dependency_container
            .get_dependency::<ProcessSelectorViewData>();
        let instance = Self {
            app_context,
            debugger_trace_view_data,
            project_hierarchy_view_data,
            process_selector_view_data,
        };

        instance.listen_for_debugger_events();

        instance
    }

    fn listen_for_debugger_events(&self) {
        let debugger_trace_view_data = self.debugger_trace_view_data.clone();
        self.app_context
            .engine_unprivileged_state
            .listen_for_engine_event::<DebuggerTraceSessionUpdatedEvent>(move |debugger_trace_session_updated_event| {
                if let Some(debugger_trace_view_data) = debugger_trace_view_data.read("Debugger trace event listener") {
                    debugger_trace_view_data.apply_trace_session_updated(debugger_trace_session_updated_event);
                }
            });
    }

    fn stop_trace_session(
        &self,
        trace_session_id: &str,
    ) {
        let Some(debugger_trace_view_data) = self.debugger_trace_view_data.read("Debugger trace stop begin") else {
            return;
        };

        if !debugger_trace_view_data.begin_trace_control(trace_session_id) {
            return;
        }

        let debugger_trace_view_data = self.debugger_trace_view_data.clone();
        let trace_session_id_for_callback = trace_session_id.to_string();
        let is_dispatched = DebuggerTraceStopRequest {
            trace_session_id: trace_session_id.to_string(),
        }
        .send(&self.app_context.engine_unprivileged_state, move |debugger_trace_stop_response| {
            if let Some(debugger_trace_view_data) = debugger_trace_view_data.read("Debugger trace stop complete") {
                debugger_trace_view_data.complete_trace_control(&trace_session_id_for_callback);
            }

            if !debugger_trace_stop_response.status.get_success() {
                log::warn!(
                    "Debugger trace stop failed: {}.",
                    debugger_trace_stop_response
                        .status
                        .get_message()
                        .unwrap_or("unknown error")
                );
            }
        });

        if !is_dispatched {
            if let Some(debugger_trace_view_data) = self
                .debugger_trace_view_data
                .read("Debugger trace stop dispatch failure")
            {
                debugger_trace_view_data.complete_trace_control(trace_session_id);
            }
            log::warn!("Debugger trace stop failed: command dispatch failed.");
        }
    }

    fn pause_trace_collection(
        &self,
        trace_session_id: &str,
    ) {
        let Some(debugger_trace_view_data) = self
            .debugger_trace_view_data
            .read("Debugger trace collection pause begin")
        else {
            return;
        };

        if !debugger_trace_view_data.begin_trace_control(trace_session_id) {
            return;
        }

        let debugger_trace_view_data = self.debugger_trace_view_data.clone();
        let trace_session_id_for_callback = trace_session_id.to_string();
        let is_dispatched = DebuggerTracePauseRequest {
            trace_session_id: trace_session_id.to_string(),
        }
        .send(&self.app_context.engine_unprivileged_state, move |debugger_trace_pause_response| {
            if let Some(debugger_trace_view_data) = debugger_trace_view_data.read("Debugger trace collection pause complete") {
                debugger_trace_view_data.complete_trace_control(&trace_session_id_for_callback);
            }

            if !debugger_trace_pause_response.status.get_success() {
                log::warn!(
                    "Debugger trace collection pause failed: {}.",
                    debugger_trace_pause_response
                        .status
                        .get_message()
                        .unwrap_or("unknown error")
                );
            }
        });

        if !is_dispatched {
            if let Some(debugger_trace_view_data) = self
                .debugger_trace_view_data
                .read("Debugger trace collection pause dispatch failure")
            {
                debugger_trace_view_data.complete_trace_control(trace_session_id);
            }
            log::warn!("Debugger trace collection pause failed: command dispatch failed.");
        }
    }

    fn resume_trace_collection(
        &self,
        trace_session_id: &str,
    ) {
        let Some(debugger_trace_view_data) = self
            .debugger_trace_view_data
            .read("Debugger trace collection resume begin")
        else {
            return;
        };

        if !debugger_trace_view_data.begin_trace_control(trace_session_id) {
            return;
        }

        let debugger_trace_view_data = self.debugger_trace_view_data.clone();
        let trace_session_id_for_callback = trace_session_id.to_string();
        let is_dispatched = DebuggerTraceResumeRequest {
            trace_session_id: trace_session_id.to_string(),
        }
        .send(&self.app_context.engine_unprivileged_state, move |debugger_trace_resume_response| {
            if let Some(debugger_trace_view_data) = debugger_trace_view_data.read("Debugger trace collection resume complete") {
                debugger_trace_view_data.complete_trace_control(&trace_session_id_for_callback);
            }

            if !debugger_trace_resume_response.status.get_success() {
                log::warn!(
                    "Debugger trace collection resume failed: {}.",
                    debugger_trace_resume_response
                        .status
                        .get_message()
                        .unwrap_or("unknown error")
                );
            }
        });

        if !is_dispatched {
            if let Some(debugger_trace_view_data) = self
                .debugger_trace_view_data
                .read("Debugger trace collection resume dispatch failure")
            {
                debugger_trace_view_data.complete_trace_control(trace_session_id);
            }
            log::warn!("Debugger trace collection resume failed: command dispatch failed.");
        }
    }

    fn access_label(access: DebuggerDataBreakpointAccess) -> &'static str {
        match access {
            DebuggerDataBreakpointAccess::Read => "Reads",
            DebuggerDataBreakpointAccess::Write => "Writes",
            DebuggerDataBreakpointAccess::ReadWrite => "Accesses",
        }
    }

    fn prompt_action_label(access: DebuggerDataBreakpointAccess) -> &'static str {
        match access {
            DebuggerDataBreakpointAccess::Read => "Find What Reads",
            DebuggerDataBreakpointAccess::Write => "Find What Writes",
            DebuggerDataBreakpointAccess::ReadWrite => "Find What Accesses",
        }
    }

    fn instruction_access_label(access: DebuggerDataBreakpointAccess) -> &'static str {
        match access {
            DebuggerDataBreakpointAccess::Read => "Reads From",
            DebuggerDataBreakpointAccess::Write => "Writes To",
            DebuggerDataBreakpointAccess::ReadWrite => "Accesses",
        }
    }

    fn instruction_prompt_action_label(access: DebuggerDataBreakpointAccess) -> &'static str {
        match access {
            DebuggerDataBreakpointAccess::Read => "Find What This Reads From",
            DebuggerDataBreakpointAccess::Write => "Find What This Writes To",
            DebuggerDataBreakpointAccess::ReadWrite => "Find What This Accesses",
        }
    }

    fn instruction_text(instruction_record: &DebuggerTraceInstructionRecord) -> String {
        instruction_record
            .get_instruction_text()
            .filter(|instruction_text| !instruction_text.is_empty())
            .map(String::from)
            .unwrap_or_else(|| {
                instruction_record
                    .get_instruction_bytes()
                    .iter()
                    .map(|instruction_byte| format!("{:02X}", instruction_byte))
                    .collect::<Vec<_>>()
                    .join(" ")
            })
    }

    fn format_trace_target(trace_session: &DebuggerTraceSessionDescriptor) -> String {
        match trace_session.get_target_kind() {
            DebuggerTraceTargetKind::Instruction => format!(
                "Instruction at 0x{:X} {}",
                trace_session.get_address(),
                Self::instruction_access_label(trace_session.get_access())
            ),
            DebuggerTraceTargetKind::Address => format!(
                "{} 0x{:X} [{} bytes]",
                Self::access_label(trace_session.get_access()),
                trace_session.get_address(),
                trace_session.get_size_in_bytes()
            ),
        }
    }

    fn format_pending_trace_target(pending_trace_start_request: &PendingDebuggerTraceStartRequest) -> String {
        let label = pending_trace_start_request
            .get_label()
            .filter(|label| !label.is_empty())
            .unwrap_or("selected address");

        match pending_trace_start_request.get_target_kind() {
            DebuggerTraceTargetKind::Instruction => format!(
                "{} for {} at 0x{:X}",
                Self::instruction_prompt_action_label(pending_trace_start_request.get_access()),
                label,
                pending_trace_start_request.get_address()
            ),
            DebuggerTraceTargetKind::Address => format!(
                "{} for {} at 0x{:X} [{} bytes]",
                Self::prompt_action_label(pending_trace_start_request.get_access()),
                label,
                pending_trace_start_request.get_address(),
                pending_trace_start_request.get_size_in_bytes()
            ),
        }
    }

    /// Reads the live value at each instruction-directed record's accessed address through the virtual-snapshot pipeline
    /// (the same path the project explorer / scanner use), interpreting the memory with the user-selected data type and
    /// display format. Values are cached per address and only the snapshot queries — not the cached values — are reset
    /// when the address set or interpretation changes, so cells never blank between refreshes (avoids flicker). Keyed by
    /// accessed address; address-directed records have no data address and so get no value.
    /// The memory address whose value the Value column should show for a record, by trace direction:
    /// - Instruction-directed: the per-record accessed address (the data the instruction touched).
    /// - Address-directed: the session's watched address (the same for every row — the value being written to).
    fn record_value_address(
        instruction_record: &DebuggerTraceInstructionRecord,
        trace_session: &DebuggerTraceSessionDescriptor,
    ) -> Option<u64> {
        match trace_session.get_target_kind() {
            DebuggerTraceTargetKind::Instruction => instruction_record.get_accessed_address(),
            DebuggerTraceTargetKind::Address => Some(trace_session.get_address()),
        }
    }

    fn read_record_preview_values(
        &self,
        instruction_records: &[DebuggerTraceInstructionRecord],
        trace_session: &DebuggerTraceSessionDescriptor,
    ) -> HashMap<u64, String> {
        let engine_unprivileged_state = &self.app_context.engine_unprivileged_state;
        let (value_data_type_id, stored_display_format) = self
            .debugger_trace_view_data
            .read("Debugger trace preview values")
            .map(|debugger_trace_view_data| (debugger_trace_view_data.get_value_data_type_id(), debugger_trace_view_data.get_value_display_format()))
            .unwrap_or_else(|| (String::from("i32"), None));
        let value_data_type_ref = DataTypeRef::new(&value_data_type_id);
        let active_display_format =
            stored_display_format.unwrap_or_else(|| engine_unprivileged_state.get_default_anonymous_value_string_format(&value_data_type_ref));

        // Unique value addresses in a STABLE (sorted) order. Stable order matters: the snapshot's set_queries is a no-op
        // when the query list is unchanged, so identical ordering frame-to-frame avoids needlessly clearing the results
        // (which caused the earlier flicker).
        let mut accessed_addresses = instruction_records
            .iter()
            .filter_map(|instruction_record| Self::record_value_address(instruction_record, trace_session))
            .collect::<Vec<_>>();
        accessed_addresses.sort_unstable();
        accessed_addresses.dedup();

        // Same set/refresh/get pattern the project explorer uses for live address previews (and reads route through the
        // privileged worker). Driven every frame; set_queries no-ops when unchanged, request_refresh self-throttles.
        if accessed_addresses.is_empty() {
            engine_unprivileged_state.set_virtual_snapshot_queries(Self::TRACE_PREVIEW_VIRTUAL_SNAPSHOT_ID, Self::TRACE_PREVIEW_REFRESH_INTERVAL, Vec::new());
            return HashMap::new();
        }

        let symbolic_struct_definition = SymbolicStructDefinition::new_anonymous(vec![SymbolicFieldDefinition::new(value_data_type_ref, ContainerType::None)]);
        let virtual_snapshot_queries = accessed_addresses
            .iter()
            .map(|accessed_address| VirtualSnapshotQuery::Address {
                query_id: format!("0x{:X}", accessed_address),
                address: *accessed_address,
                module_name: String::new(),
                symbolic_struct_definition: symbolic_struct_definition.clone(),
            })
            .collect::<Vec<_>>();

        engine_unprivileged_state.set_virtual_snapshot_queries(Self::TRACE_PREVIEW_VIRTUAL_SNAPSHOT_ID, Self::TRACE_PREVIEW_REFRESH_INTERVAL, virtual_snapshot_queries);
        engine_unprivileged_state.request_virtual_snapshot_refresh(Self::TRACE_PREVIEW_VIRTUAL_SNAPSHOT_ID);

        let Some(virtual_snapshot) = engine_unprivileged_state.get_virtual_snapshot(Self::TRACE_PREVIEW_VIRTUAL_SNAPSHOT_ID) else {
            return HashMap::new();
        };

        accessed_addresses
            .into_iter()
            .filter_map(|accessed_address| {
                let query_result = virtual_snapshot.get_query_results().get(&format!("0x{:X}", accessed_address))?;

                Some((accessed_address, self.format_preview_value(query_result, active_display_format)?))
            })
            .collect()
    }

    /// Interprets the first read field as the active data type and renders it in the active display format via the
    /// engine anonymizer (same path the project explorer uses). Returns None on failure so the row shows the "??"
    /// placeholder rather than a stale/blank cell.
    fn format_preview_value(
        &self,
        query_result: &VirtualSnapshotQueryResult,
        active_display_format: AnonymousValueStringFormat,
    ) -> Option<String> {
        let memory_read_response = query_result.memory_read_response.as_ref()?;

        if !memory_read_response.success {
            return None;
        }

        let data_value = memory_read_response.valued_struct.get_fields().first()?.get_data_value()?;
        let anonymous_value_string = self
            .app_context
            .engine_unprivileged_state
            .anonymize_value(data_value, active_display_format)
            .ok()?;
        let formatted = anonymous_value_string.get_anonymous_value_string();

        if formatted.is_empty() { None } else { Some(formatted.to_string()) }
    }

    fn show_attach_prompt(
        &self,
        user_interface: &mut Ui,
        pending_trace_start_request: &PendingDebuggerTraceStartRequest,
        status_message: Option<&str>,
        is_starting_pending_trace: bool,
    ) {
        let theme = &self.app_context.theme;
        let panel_width = user_interface.available_width().clamp(320.0, 560.0);

        user_interface.allocate_ui_with_layout(
            user_interface.available_size(),
            Layout::centered_and_justified(Direction::TopDown),
            |user_interface| {
                user_interface.horizontal(|user_interface| {
                    let side_spacing = ((user_interface.available_width() - panel_width) * 0.5).max(0.0);
                    user_interface.add_space(side_spacing);
                    user_interface.add(
                        GroupBox::new_from_theme(theme, "Attach debugger?", |user_interface| {
                            user_interface.label(
                                RichText::new(Self::format_pending_trace_target(pending_trace_start_request))
                                    .font(theme.font_library.font_noto_sans.font_normal.clone())
                                    .color(theme.foreground),
                            );
                            user_interface.add_space(8.0);
                            user_interface.label(
                                RichText::new("Starting this trace requires attaching a debugger to the opened process.")
                                    .font(theme.font_library.font_noto_sans.font_small.clone())
                                    .color(theme.foreground_preview),
                            );

                            if let Some(status_message) = status_message {
                                user_interface.add_space(8.0);
                                user_interface.label(
                                    RichText::new(status_message)
                                        .font(theme.font_library.font_noto_sans.font_small.clone())
                                        .color(theme.background_control_danger),
                                );
                            }

                            user_interface.add_space(12.0);
                            if is_starting_pending_trace {
                                user_interface.allocate_ui_with_layout(
                                    vec2(user_interface.available_width(), 32.0),
                                    Layout::centered_and_justified(Direction::LeftToRight),
                                    |user_interface| {
                                        user_interface.add(Spinner::new().color(theme.foreground));
                                    },
                                );
                            } else {
                                self.show_trace_start_prompt_buttons(user_interface);
                            }
                        })
                        .desired_width(panel_width),
                    );
                });
            },
        );
    }

    fn show_trace_start_prompt_buttons(
        &self,
        user_interface: &mut Ui,
    ) {
        let theme = &self.app_context.theme;
        let button_size = vec2(136.0, 28.0);
        let button_spacing = 12.0;
        let total_button_row_width = button_size.x * 2.0 + button_spacing;
        let side_spacing = ((user_interface.available_width() - total_button_row_width) * 0.5).max(0.0);

        user_interface.allocate_ui(vec2(user_interface.available_width(), 32.0), |user_interface| {
            user_interface.horizontal(|user_interface| {
                user_interface.add_space(side_spacing);
                user_interface.spacing_mut().item_spacing.x = button_spacing;

                let cancel_response = user_interface.add_sized(
                    button_size,
                    Button::new(RichText::new("Cancel").color(theme.foreground))
                        .fill(theme.background_control_secondary)
                        .stroke(Stroke::new(1.0, theme.background_control_secondary_dark)),
                );

                if cancel_response.clicked() {
                    if let Some(debugger_trace_view_data) = self
                        .debugger_trace_view_data
                        .read("Debugger trace cancel attach prompt")
                    {
                        debugger_trace_view_data.cancel_pending_trace_start();
                    }
                }

                let start_response = user_interface.add_sized(
                    button_size,
                    Button::new(RichText::new("Start").color(theme.foreground))
                        .fill(theme.background_control_primary)
                        .stroke(Stroke::new(1.0, theme.background_control_primary_dark)),
                );

                if start_response.clicked() {
                    self.confirm_pending_trace_start();
                }
            });
        });
    }

    fn confirm_pending_trace_start(&self) {
        let pending_trace_start_operation = self
            .debugger_trace_view_data
            .read("Debugger trace begin start prompt")
            .and_then(|debugger_trace_view_data| debugger_trace_view_data.begin_pending_trace_start());
        let Some(pending_trace_start_operation) = pending_trace_start_operation else {
            return;
        };
        let operation_id = pending_trace_start_operation.get_operation_id();
        let pending_trace_start_request = pending_trace_start_operation.into_request();

        let engine_unprivileged_state = self.app_context.engine_unprivileged_state.clone();
        let debugger_trace_view_data = self.debugger_trace_view_data.clone();
        let dispatch_failure_debugger_trace_view_data = debugger_trace_view_data.clone();
        let is_trace_start_dispatched = DebuggerTraceStartRequest {
            address: pending_trace_start_request.get_address(),
            size_in_bytes: pending_trace_start_request.get_size_in_bytes(),
            access: pending_trace_start_request.get_access(),
            label: pending_trace_start_request.get_label().map(String::from),
            target_kind: pending_trace_start_request.get_target_kind(),
        }
        .send(&engine_unprivileged_state, move |debugger_trace_start_response| {
            if debugger_trace_start_response.status.get_success() {
                if let Some(debugger_trace_view_data) = debugger_trace_view_data.read("Debugger trace start completed") {
                    debugger_trace_view_data.complete_pending_trace_start(operation_id);
                }

                return;
            }

            let status_message = format!(
                "Debugger trace start failed: {}.",
                debugger_trace_start_response
                    .status
                    .get_message()
                    .unwrap_or("unknown error")
            );

            if let Some(debugger_trace_view_data) = debugger_trace_view_data.read("Debugger trace start failed") {
                debugger_trace_view_data.fail_pending_trace_start(operation_id, status_message);
            }
        });

        if !is_trace_start_dispatched {
            if let Some(debugger_trace_view_data) = dispatch_failure_debugger_trace_view_data.read("Debugger trace start dispatch failed") {
                debugger_trace_view_data.fail_pending_trace_start(operation_id, String::from("Debugger trace start failed: command dispatch failed."));
            }
        }
    }

    fn show_trace_header(
        &self,
        user_interface: &mut Ui,
        content_rectangle: Rect,
    ) {
        let theme = &self.app_context.theme;
        let header_height = 28.0;
        let (header_rectangle, _) = user_interface.allocate_exact_size(vec2(user_interface.available_width().max(1.0), header_height), Sense::hover());
        let separator_rectangle = Rect::from_min_max(
            pos2(header_rectangle.min.x, header_rectangle.max.y),
            pos2(header_rectangle.max.x, header_rectangle.max.y + 3.0),
        );

        user_interface
            .painter()
            .rect_filled(header_rectangle, CornerRadius::ZERO, theme.background_primary);
        user_interface
            .painter()
            .rect_filled(separator_rectangle, CornerRadius::ZERO, theme.background_control);

        let column_ratios = self.column_splitter_ratios();
        let column_splitters = Self::trace_column_splitter_positions(content_rectangle, column_ratios);
        let text_left_padding = 8.0;
        let paint_header_label = |user_interface: &mut Ui, x_position: f32, label: &str| {
            user_interface.painter().text(
                pos2(x_position + text_left_padding, header_rectangle.center().y),
                Align2::LEFT_CENTER,
                label,
                theme.font_library.font_noto_sans.font_header.clone(),
                theme.foreground,
            );
        };

        // A compact "#" keeps the hit-count column narrow.
        paint_header_label(user_interface, column_splitters.hit_count, "#");
        paint_header_label(user_interface, column_splitters.instruction, "Instruction");
        paint_header_label(user_interface, column_splitters.address, "Address");

        // Type and Value are a single column: its header hosts the data-type selector (showing the active type) on the
        // left and a compact display-format picker on the far right. The rows below show only the formatted value.
        let control_height = 20.0;
        let control_center_y = header_rectangle.center().y;
        let format_selector_size = vec2(24.0, control_height);
        let format_selector_rectangle = Rect::from_min_size(
            pos2(
                (content_rectangle.max.x - format_selector_size.x - 6.0).max(column_splitters.value + text_left_padding),
                control_center_y - control_height * 0.5,
            ),
            format_selector_size,
        );
        let type_selector_rectangle = Rect::from_min_max(
            pos2(column_splitters.value + text_left_padding, control_center_y - control_height * 0.5),
            pos2(
                (format_selector_rectangle.min.x - 6.0).max(column_splitters.value + text_left_padding + 1.0),
                control_center_y + control_height * 0.5,
            ),
        );

        let stored_data_type_id = self
            .debugger_trace_view_data
            .read("Debugger trace header value data type")
            .map(|debugger_trace_view_data| debugger_trace_view_data.get_value_data_type_id())
            .unwrap_or_else(|| String::from("i32"));
        let default_display_format = self
            .app_context
            .engine_unprivileged_state
            .get_default_anonymous_value_string_format(&DataTypeRef::new(&stored_data_type_id));

        if let Some(debugger_trace_view_data) = self.debugger_trace_view_data.read("Debugger trace header value format controls") {
            debugger_trace_view_data.with_value_format_controls(default_display_format, |data_type_selection, display_format| {
                user_interface.put(
                    type_selector_rectangle,
                    DataTypeSelectorView::new(self.app_context.clone(), data_type_selection, "debugger_trace_value_data_type")
                        .single_select()
                        .width(type_selector_rectangle.width())
                        .height(type_selector_rectangle.height()),
                );

                let active_data_type_ref = data_type_selection.active_data_type().clone();
                let mut supported_display_formats = self
                    .app_context
                    .engine_unprivileged_state
                    .get_supported_anonymous_value_string_formats(&active_data_type_ref);

                if supported_display_formats.is_empty() {
                    supported_display_formats.push(*display_format);
                }
                if !supported_display_formats.contains(display_format) {
                    *display_format = supported_display_formats[0];
                }

                let mut header_display_format_value = AnonymousValueString::new(String::new(), *display_format, ContainerType::None);

                user_interface.put(
                    format_selector_rectangle,
                    ComboBoxView::new(
                        self.app_context.clone(),
                        String::new(),
                        "debugger_trace_value_display_format",
                        Some(self.display_format_icon(*display_format)),
                        |popup_user_interface, should_close| {
                            for anonymous_value_string_format in &supported_display_formats {
                                if popup_user_interface
                                    .add(
                                        DataValueBoxConvertItemView::new(
                                            self.app_context.clone(),
                                            &mut header_display_format_value,
                                            anonymous_value_string_format,
                                            None,
                                            false,
                                            false,
                                            Self::VALUE_DISPLAY_FORMAT_POPUP_WIDTH,
                                        )
                                        .width(Self::VALUE_DISPLAY_FORMAT_POPUP_WIDTH),
                                    )
                                    .clicked()
                                {
                                    *should_close = true;
                                }
                            }
                        },
                    )
                    .width(format_selector_rectangle.width())
                    .height(format_selector_rectangle.height())
                    .show_dropdown_arrow(false),
                );

                *display_format = header_display_format_value.get_anonymous_value_string_format();
            });
        }
    }

    /// Paints and handles the draggable column dividers. Called AFTER the header and all rows are drawn so the dividers
    /// span the full table height and win the pointer over the rows (mirrors the scan results table). `splitter_bottom_y`
    /// is the bottom of the rows just rendered.
    fn show_column_splitters(
        &self,
        user_interface: &mut Ui,
        content_rectangle: Rect,
        splitter_bottom_y: f32,
        column_splitters: TraceColumnSplitters,
    ) {
        let theme = &self.app_context.theme;
        let bar_thickness = 4.0;
        let content_min_x = content_rectangle.min.x;
        let content_width = content_rectangle.width().max(1.0);
        let (instruction_ratio, address_ratio, value_ratio) = self.column_splitter_ratios();
        let top_y = content_rectangle.min.y;
        let bottom_y = splitter_bottom_y.max(top_y + 1.0);

        let splitter_bar = |user_interface: &mut Ui, splitter_position_x: f32, id_suffix: &str| -> Response {
            let splitter_rectangle = Rect::from_min_max(pos2(splitter_position_x - bar_thickness * 0.5, top_y), pos2(splitter_position_x + bar_thickness * 0.5, bottom_y));
            let splitter_id = user_interface.id().with(("debugger_trace_column_splitter", id_suffix));
            let splitter_response = user_interface.interact(splitter_rectangle, splitter_id, Sense::drag());
            let splitter_color = if splitter_response.hovered() || splitter_response.dragged() {
                theme.selected_border
            } else {
                theme.background_control
            };

            user_interface.painter().rect_filled(splitter_rectangle, 0.0, splitter_color);

            splitter_response.on_hover_cursor(CursorIcon::ResizeHorizontal)
        };

        let instruction_response = splitter_bar(user_interface, column_splitters.instruction, "instruction");
        let address_response = splitter_bar(user_interface, column_splitters.address, "address");
        let value_response = splitter_bar(user_interface, column_splitters.value, "value");

        let mut new_instruction_ratio = instruction_ratio;
        let mut new_address_ratio = address_ratio;
        let mut new_value_ratio = value_ratio;
        let mut did_drag = false;

        if instruction_response.dragged() {
            new_instruction_ratio = (column_splitters.instruction + instruction_response.drag_delta().x - content_min_x) / content_width;
            did_drag = true;
        }
        if address_response.dragged() {
            new_address_ratio = (column_splitters.address + address_response.drag_delta().x - content_min_x) / content_width;
            did_drag = true;
        }
        if value_response.dragged() {
            new_value_ratio = (column_splitters.value + value_response.drag_delta().x - content_min_x) / content_width;
            did_drag = true;
        }

        if did_drag {
            self.update_column_splitter_ratios(new_instruction_ratio, new_address_ratio, new_value_ratio);
        }
    }

    fn update_column_splitter_ratios(
        &self,
        instruction_splitter_ratio: f32,
        address_splitter_ratio: f32,
        value_splitter_ratio: f32,
    ) {
        if let Some(debugger_trace_view_data) = self.debugger_trace_view_data.read("Debugger trace set column splitter ratios") {
            debugger_trace_view_data.set_column_splitter_ratios(instruction_splitter_ratio, address_splitter_ratio, value_splitter_ratio);
        }
    }

    fn trace_column_splitter_positions(
        content_rectangle: Rect,
        column_splitter_ratios: (f32, f32, f32),
    ) -> TraceColumnSplitters {
        let content_width = content_rectangle.width().max(1.0);
        let (instruction_ratio, address_ratio, value_ratio) = column_splitter_ratios;

        // The count column needs room for several digits even on narrow windows, so enforce a pixel minimum and cascade
        // the minimum widths rightward so columns never collapse below something readable.
        let hit_count = content_rectangle.min.x + 28.0;
        let minimum_count_width = 42.0;
        let minimum_column_width = 64.0;
        let instruction = (content_rectangle.min.x + content_width * instruction_ratio).max(hit_count + minimum_count_width);
        let address = (content_rectangle.min.x + content_width * address_ratio).max(instruction + minimum_column_width);
        let value = (content_rectangle.min.x + content_width * value_ratio).max(address + minimum_column_width);

        TraceColumnSplitters {
            hit_count,
            instruction,
            address,
            value,
        }
    }

    fn column_splitter_ratios(&self) -> (f32, f32, f32) {
        self.debugger_trace_view_data
            .read("Debugger trace column splitter ratios")
            .map(|debugger_trace_view_data| debugger_trace_view_data.get_column_splitter_ratios())
            .unwrap_or((0.14, 0.42, 0.66))
    }

    fn show_trace_session_header(
        &self,
        user_interface: &mut Ui,
        trace_session: &DebuggerTraceSessionDescriptor,
        has_pending_control: bool,
    ) {
        let theme = &self.app_context.theme;
        let header_height = 28.0;
        let horizontal_padding = 8.0;
        let button_size = vec2(24.0, 24.0);
        let button_spacing = 4.0;
        let (header_rectangle, _) = user_interface.allocate_exact_size(vec2(user_interface.available_width(), header_height), Sense::hover());
        let status_label = if !trace_session.get_is_active() {
            "Stopped"
        } else if trace_session.get_breakpoint().get_is_enabled() {
            "Collecting"
        } else {
            "Collection Paused"
        };
        let header_label = format!(
            "{} | {} | {}",
            trace_session.get_trace_session_id(),
            Self::format_trace_target(trace_session),
            status_label
        );

        user_interface
            .painter()
            .rect_filled(header_rectangle, CornerRadius::ZERO, theme.background_primary);

        let controls_are_disabled = has_pending_control || !trace_session.get_is_active();
        let stop_button_rectangle = Rect::from_min_size(
            pos2(header_rectangle.min.x + horizontal_padding, header_rectangle.center().y - button_size.y * 0.5),
            button_size,
        );
        let stop_response = user_interface.place(
            stop_button_rectangle,
            IconButtonView::new(theme, &theme.icon_library.icon_handle_navigation_stop, "Stop trace.").disabled(controls_are_disabled),
        );

        if stop_response.clicked() && !controls_are_disabled {
            self.stop_trace_session(trace_session.get_trace_session_id());
        }

        let control_button_rectangle = Rect::from_min_size(
            pos2(stop_button_rectangle.max.x + button_spacing, header_rectangle.center().y - button_size.y * 0.5),
            button_size,
        );

        if trace_session.get_is_active() && !trace_session.get_breakpoint().get_is_enabled() {
            let resume_response = user_interface.place(
                control_button_rectangle,
                IconButtonView::new(theme, &theme.icon_library.icon_handle_navigation_right_arrows, "Resume collection.").disabled(controls_are_disabled),
            );

            if resume_response.clicked() && !controls_are_disabled {
                self.resume_trace_collection(trace_session.get_trace_session_id());
            }
        } else {
            let pause_response = user_interface.place(
                control_button_rectangle,
                IconButtonView::new(theme, &theme.icon_library.icon_handle_navigation_pause, "Pause collection.").disabled(controls_are_disabled),
            );

            if pause_response.clicked() && !controls_are_disabled {
                self.pause_trace_collection(trace_session.get_trace_session_id());
            }
        }

        let text_position_x = control_button_rectangle.max.x + horizontal_padding;

        user_interface.painter().text(
            pos2(text_position_x, header_rectangle.center().y),
            Align2::LEFT_CENTER,
            header_label,
            theme.font_library.font_noto_sans.font_normal.clone(),
            theme.foreground_preview,
        );
    }

    fn show_instruction_records(
        &self,
        user_interface: &mut Ui,
        trace_session: &DebuggerTraceSessionDescriptor,
        instruction_records: &[DebuggerTraceInstructionRecord],
        selected_instruction_keys: &[DebuggerTraceInstructionKey],
        has_pending_control: bool,
    ) {
        let theme = &self.app_context.theme;
        self.show_trace_session_header(user_interface, trace_session, has_pending_control);

        if instruction_records.is_empty() {
            user_interface.allocate_ui_with_layout(
                vec2(user_interface.available_width(), 32.0),
                Layout::centered_and_justified(Direction::LeftToRight),
                |user_interface| {
                    user_interface.label(
                        RichText::new("Waiting for breakpoint hits.")
                            .font(theme.font_library.font_noto_sans.font_small.clone())
                            .color(theme.foreground_preview),
                    );
                },
            );
            return;
        }

        let content_rectangle = user_interface.available_rect_before_wrap();
        self.show_trace_header(user_interface, content_rectangle);
        let column_splitters = Self::trace_column_splitter_positions(content_rectangle, self.column_splitter_ratios());
        let record_preview_values = self.read_record_preview_values(instruction_records, trace_session);

        for instruction_record in instruction_records {
            let instruction_key = DebuggerTraceInstructionKey::from_record(instruction_record);
            let is_selected = selected_instruction_keys.contains(&instruction_key);
            // The value at this row's address: the accessed address (instruction-directed) or the watched address
            // (address-directed). Show "??" while a read for a known address hasn't landed yet; blank if there is no
            // address at all (e.g. an instruction-directed record whose accessed address could not be resolved).
            let preview_value = match Self::record_value_address(instruction_record, trace_session) {
                Some(value_address) => record_preview_values
                    .get(&value_address)
                    .cloned()
                    .unwrap_or_else(|| String::from("??")),
                None => String::new(),
            };
            let row_response = user_interface.add(DebuggerTraceEntryView::new(
                self.app_context.clone(),
                instruction_record,
                &instruction_key,
                is_selected,
                column_splitters.hit_count,
                column_splitters.instruction,
                column_splitters.address,
                column_splitters.value,
                preview_value,
            ));

            if row_response.clicked() {
                if let Some(debugger_trace_view_data) = self
                    .debugger_trace_view_data
                    .read("Debugger trace select instruction")
                {
                    debugger_trace_view_data.set_single_instruction_selection(instruction_key.clone());
                }
            }

            if row_response.secondary_clicked() {
                if let Some(debugger_trace_view_data) = self
                    .debugger_trace_view_data
                    .read("Debugger trace instruction context menu")
                {
                    debugger_trace_view_data.show_instruction_context_menu(
                        instruction_key.clone(),
                        row_response
                            .hover_pos()
                            .unwrap_or(row_response.rect.left_bottom()),
                    );
                }
            }

            if row_response.double_clicked() {
                self.add_instruction_record_to_project(instruction_record);
            }
        }

        // Draw + handle the resizable column dividers last, spanning the full available height (not just the rows) so the
        // bars fill the space and win the pointer over the rows.
        self.show_column_splitters(user_interface, content_rectangle, content_rectangle.max.y, column_splitters);
    }

    fn show_instruction_context_menu(
        &self,
        user_interface: &mut Ui,
        snapshot_instruction_records: &[DebuggerTraceInstructionRecord],
        snapshot_selected_instruction_keys: &[DebuggerTraceInstructionKey],
        snapshot_context_menu_target: Option<&crate::views::debugger_trace::view_data::debugger_trace_view_data::DebuggerTraceInstructionContextMenuTarget>,
    ) {
        let Some(context_menu_target) = snapshot_context_menu_target else {
            return;
        };
        let Some(instruction_record) = snapshot_instruction_records
            .iter()
            .find(|instruction_record| DebuggerTraceInstructionKey::from_record(instruction_record) == *context_menu_target.get_instruction_key())
        else {
            if let Some(debugger_trace_view_data) = self
                .debugger_trace_view_data
                .read("Debugger trace hide stale instruction context menu")
            {
                debugger_trace_view_data.hide_instruction_context_menu();
            }
            return;
        };
        let context_menu_labels = [
            Self::ADD_TO_PROJECT_LABEL,
            Self::REPLACE_WITH_NO_OPERATION_LABEL,
            Self::RESTORE_ORIGINAL_CODE_LABEL,
        ];
        let context_menu_width = ContextMenuSizing::width_for_labels(self.app_context.as_ref(), user_interface, context_menu_labels);
        let mut open = true;

        ContextMenu::new(
            self.app_context.clone(),
            "debugger_trace_instruction_context_menu",
            context_menu_target.get_position(),
            |user_interface, should_close| {
                if user_interface
                    .add(ToolbarMenuItemView::new(
                        self.app_context.clone(),
                        Self::ADD_TO_PROJECT_LABEL,
                        Self::ADD_TO_PROJECT_ID,
                        &None,
                        context_menu_width,
                    ))
                    .clicked()
                {
                    self.add_instruction_record_to_project(instruction_record);
                    *should_close = true;
                }

                if user_interface
                    .add(ToolbarMenuItemView::new(
                        self.app_context.clone(),
                        Self::REPLACE_WITH_NO_OPERATION_LABEL,
                        Self::REPLACE_WITH_NO_OPERATION_ID,
                        &None,
                        context_menu_width,
                    ))
                    .clicked()
                {
                    self.replace_instruction_record_with_no_operation(instruction_record, snapshot_selected_instruction_keys);
                    *should_close = true;
                }

                if user_interface
                    .add(ToolbarMenuItemView::new(
                        self.app_context.clone(),
                        Self::RESTORE_ORIGINAL_CODE_LABEL,
                        Self::RESTORE_ORIGINAL_CODE_ID,
                        &None,
                        context_menu_width,
                    ))
                    .clicked()
                {
                    self.restore_instruction_record_original_code(instruction_record);
                    *should_close = true;
                }
            },
        )
        .width(context_menu_width)
        .corner_radius(8)
        .show(user_interface, &mut open);

        if !open {
            if let Some(debugger_trace_view_data) = self
                .debugger_trace_view_data
                .read("Debugger trace close instruction context menu")
            {
                debugger_trace_view_data.hide_instruction_context_menu();
            }
        }
    }

    fn replace_instruction_record_with_no_operation(
        &self,
        instruction_record: &DebuggerTraceInstructionRecord,
        _selected_instruction_keys: &[DebuggerTraceInstructionKey],
    ) {
        let Some(instruction_address) = instruction_record.get_instruction_address() else {
            log::warn!("Cannot replace debugger trace instruction without an instruction address.");
            return;
        };

        InstructionPatchAction::replace_known_instruction_with_no_operation(
            self.app_context.clone(),
            self.process_selector_view_data.clone(),
            instruction_address,
            String::new(),
            instruction_record.get_instruction_bytes().to_vec(),
            Some(Self::instruction_text(instruction_record)),
        );
    }

    fn restore_instruction_record_original_code(
        &self,
        instruction_record: &DebuggerTraceInstructionRecord,
    ) {
        let Some(instruction_address) = instruction_record.get_instruction_address() else {
            log::warn!("Cannot restore debugger trace instruction without an instruction address.");
            return;
        };

        InstructionPatchAction::restore_no_operation_patch_at_address(self.app_context.clone(), instruction_address, String::new());
    }

    fn add_instruction_record_to_project(
        &self,
        instruction_record: &DebuggerTraceInstructionRecord,
    ) {
        // Instruction-directed records: double-clicking an accessed-address row adds that DATA address (the thing the
        // instruction touched) as a plain address item. Address-directed records add the accessing instruction itself.
        let is_accessed_address_record = instruction_record.get_accessed_address().is_some();
        let Some(target_address) = instruction_record
            .get_accessed_address()
            .or_else(|| instruction_record.get_instruction_address())
        else {
            log::warn!("Cannot add debugger trace record without a resolvable address.");
            return;
        };
        let target_directory_path = ProjectHierarchyViewData::get_selected_directory_path(self.project_hierarchy_view_data.clone()).unwrap_or_default();
        let (project_item_address, project_item_module_name) = ProjectHierarchyModuleAddressResolver::resolve_absolute_address_to_project_item_address(
            &self.app_context.engine_unprivileged_state,
            target_address,
        );
        let (project_item_name, data_type_id, initial_preview_value) = if is_accessed_address_record {
            (format!("Accessed 0x{:X}", project_item_address), String::from("i32"), None)
        } else {
            (
                Self::build_instruction_project_item_name(project_item_address, &project_item_module_name, instruction_record),
                self.instruction_data_type_id(instruction_record),
                Some(Self::instruction_text(instruction_record)),
            )
        };
        let project_items_create_request = ProjectItemsCreateRequest {
            parent_directory_path: target_directory_path,
            project_item_name,
            is_directory: false,
            address: Some(project_item_address),
            module_name: Some(project_item_module_name),
            data_type_id: Some(data_type_id),
            pointer_offsets: None,
            initial_preview_value,
        };
        let app_context = self.app_context.clone();
        let project_hierarchy_view_data = self.project_hierarchy_view_data.clone();
        let project_hierarchy_view_data_for_refresh = project_hierarchy_view_data.clone();

        let is_dispatched = project_items_create_request.send(&self.app_context.engine_unprivileged_state, move |project_items_create_response| {
            if !project_items_create_response.success {
                log::warn!("Debugger trace add-to-project command failed.");
                return;
            }

            if let Some(mut project_hierarchy_view_data) = project_hierarchy_view_data.write("Debugger trace select created instruction project item") {
                project_hierarchy_view_data.select_created_project_item(&project_items_create_response.created_project_item_path);
            }

            ProjectHierarchyViewData::refresh_project_items(project_hierarchy_view_data_for_refresh, app_context);
        });

        if !is_dispatched {
            log::warn!("Debugger trace add-to-project command dispatch failed.");
        }
    }

    fn instruction_data_type_id(
        &self,
        instruction_record: &DebuggerTraceInstructionRecord,
    ) -> String {
        self.resolve_instruction_record_target_architecture(instruction_record)
            .get_instruction_data_type_id()
            .to_string()
    }

    fn resolve_instruction_record_target_architecture(
        &self,
        instruction_record: &DebuggerTraceInstructionRecord,
    ) -> TargetArchitecture {
        instruction_record
            .get_target_architecture()
            .cloned()
            .or_else(|| {
                self.process_selector_view_data
                    .read("Debugger trace target architecture")
                    .and_then(|process_selector_view_data| {
                        process_selector_view_data
                            .opened_process
                            .as_ref()
                            .map(|opened_process_info| opened_process_info.get_target_architecture().clone())
                    })
            })
            .unwrap_or_else(TargetArchitecture::default)
    }

    fn build_instruction_project_item_name(
        project_item_address: u64,
        project_item_module_name: &str,
        instruction_record: &DebuggerTraceInstructionRecord,
    ) -> String {
        let address_text = if project_item_module_name.is_empty() {
            format!("0x{:X}", project_item_address)
        } else {
            format_module_address(project_item_module_name, project_item_address)
        };
        let instruction_text = Self::instruction_text(instruction_record);

        if instruction_text.is_empty() {
            format!("Instruction {}", address_text)
        } else {
            format!("{} {}", instruction_text, address_text)
        }
    }
}

impl Widget for DebuggerTraceView {
    fn ui(
        self,
        user_interface: &mut Ui,
    ) -> eframe::egui::Response {
        let response = user_interface
            .allocate_ui_with_layout(user_interface.available_size(), Layout::top_down(Align::Min), |user_interface| {
                let theme = &self.app_context.theme;
                let Some(debugger_trace_view_data) = self.debugger_trace_view_data.read("Debugger trace view") else {
                    return;
                };
                debugger_trace_view_data.expire_stale_pending_trace_start(Self::PENDING_TRACE_START_TIMEOUT);
                let snapshot = debugger_trace_view_data.get_snapshot();
                drop(debugger_trace_view_data);

                if let Some(pending_trace_start_request) = snapshot.pending_trace_start_request.as_ref() {
                    self.show_attach_prompt(
                        user_interface,
                        pending_trace_start_request,
                        snapshot.pending_trace_start_status_message.as_deref(),
                        snapshot.is_starting_pending_trace,
                    );
                    return;
                }

                if snapshot.trace_sessions.is_empty() {
                    user_interface.allocate_ui_with_layout(
                        user_interface.available_size(),
                        Layout::centered_and_justified(Direction::TopDown),
                        |user_interface| {
                            user_interface.label(
                                RichText::new("No debugger trace sessions.")
                                    .font(theme.font_library.font_noto_sans.font_normal.clone())
                                    .color(theme.foreground_preview),
                            );
                        },
                    );
                    return;
                }

                let mut content_user_interface = user_interface.new_child(
                    UiBuilder::new()
                        .max_rect(user_interface.available_rect_before_wrap())
                        .layout(Layout::top_down(Align::Min)),
                );
                content_user_interface.spacing_mut().menu_margin = Margin::ZERO;
                content_user_interface.spacing_mut().window_margin = Margin::ZERO;
                content_user_interface.spacing_mut().menu_spacing = 0.0;
                content_user_interface.spacing_mut().item_spacing = Vec2::ZERO;

                ScrollArea::vertical()
                    .id_salt("debugger_trace_entries")
                    .auto_shrink([false, false])
                    .show(&mut content_user_interface, |user_interface| {
                        for trace_session in &snapshot.trace_sessions {
                            let session_instruction_records = snapshot
                                .instruction_records
                                .iter()
                                .filter(|instruction_record| instruction_record.get_trace_session_id() == trace_session.get_trace_session_id())
                                .cloned()
                                .collect::<Vec<_>>();

                            let has_pending_control = snapshot
                                .pending_control_trace_session_ids
                                .contains(trace_session.get_trace_session_id());

                            self.show_instruction_records(
                                user_interface,
                                trace_session,
                                &session_instruction_records,
                                &snapshot.selected_instruction_keys,
                                has_pending_control,
                            );
                        }
                    });

                self.show_instruction_context_menu(
                    user_interface,
                    &snapshot.instruction_records,
                    &snapshot.selected_instruction_keys,
                    snapshot.instruction_context_menu_target.as_ref(),
                );
            })
            .response;

        response
    }
}
