use crate::{
    app_context::AppContext,
    ui::widgets::controls::{groupbox::GroupBox, icon_button::IconButtonView},
    views::{
        debugger_trace::{
            debugger_trace_entry_view::DebuggerTraceEntryView,
            view_data::debugger_trace_view_data::{DebuggerTraceInstructionKey, DebuggerTraceViewData, PendingDebuggerTraceStartRequest},
        },
        process_selector::view_data::process_selector_view_data::ProcessSelectorViewData,
        project_explorer::project_hierarchy::{
            project_hierarchy_module_address_resolver::ProjectHierarchyModuleAddressResolver, view_data::project_hierarchy_view_data::ProjectHierarchyViewData,
        },
    },
};
use eframe::egui::{Align, Align2, Button, Direction, Layout, Rect, RichText, ScrollArea, Sense, Spinner, Ui, UiBuilder, Widget, pos2, vec2};
use epaint::{CornerRadius, Margin, Stroke, Vec2};
use squalr_engine_api::{
    commands::{
        debugger::{
            attach::debugger_attach_request::DebuggerAttachRequest, pause::debugger_pause_request::DebuggerPauseRequest,
            resume::debugger_resume_request::DebuggerResumeRequest, trace_start::debugger_trace_start_request::DebuggerTraceStartRequest,
            trace_stop::debugger_trace_stop_request::DebuggerTraceStopRequest,
        },
        privileged_command_request::PrivilegedCommandRequest,
        project_items::create::project_items_create_request::ProjectItemsCreateRequest,
        unprivileged_command_request::UnprivilegedCommandRequest,
    },
    dependency_injection::dependency::Dependency,
    events::debugger::{
        session_state_changed::debugger_session_state_changed_event::DebuggerSessionStateChangedEvent,
        trace_session_updated::debugger_trace_session_updated_event::DebuggerTraceSessionUpdatedEvent,
    },
    structures::{
        debugger::{DebuggerDataBreakpointAccess, DebuggerSessionState, DebuggerTraceInstructionRecord, DebuggerTraceSessionDescriptor},
        memory::{address_display::format_module_address, bitness::Bitness},
    },
};
use std::sync::Arc;
use std::time::Duration;

#[derive(Clone)]
pub struct DebuggerTraceView {
    app_context: Arc<AppContext>,
    debugger_trace_view_data: Dependency<DebuggerTraceViewData>,
    project_hierarchy_view_data: Dependency<ProjectHierarchyViewData>,
    process_selector_view_data: Dependency<ProcessSelectorViewData>,
}

impl DebuggerTraceView {
    pub const WINDOW_ID: &'static str = "window_debugger_trace";
    const PENDING_TRACE_START_TIMEOUT: Duration = Duration::from_secs(15);

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

        let debugger_trace_view_data = self.debugger_trace_view_data.clone();
        self.app_context
            .engine_unprivileged_state
            .listen_for_engine_event::<DebuggerSessionStateChangedEvent>(move |debugger_session_state_changed_event| {
                if let Some(debugger_trace_view_data) = debugger_trace_view_data.read("Debugger session state event listener") {
                    debugger_trace_view_data.apply_debugger_session_state_changed(debugger_session_state_changed_event);
                }
            });
    }

    fn stop_trace_session(
        &self,
        trace_session_id: &str,
    ) {
        let is_dispatched = DebuggerTraceStopRequest {
            trace_session_id: trace_session_id.to_string(),
        }
        .send(&self.app_context.engine_unprivileged_state, |debugger_trace_stop_response| {
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
            log::warn!("Debugger trace stop failed: command dispatch failed.");
        }
    }

    fn pause_debugger(&self) {
        let debugger_trace_view_data = self.debugger_trace_view_data.clone();
        DebuggerPauseRequest {}.send(&self.app_context.engine_unprivileged_state, move |debugger_pause_response| {
            if !debugger_pause_response.status.get_success() {
                log::warn!(
                    "Debugger pause failed: {}.",
                    debugger_pause_response
                        .status
                        .get_message()
                        .unwrap_or("unknown error")
                );
            } else if let Some(debugger_trace_view_data) = debugger_trace_view_data.read("Debugger pause response") {
                debugger_trace_view_data.set_debugger_session_state(debugger_pause_response.session_state);
            }
        });
    }

    fn resume_debugger(&self) {
        let debugger_trace_view_data = self.debugger_trace_view_data.clone();
        DebuggerResumeRequest {}.send(&self.app_context.engine_unprivileged_state, move |debugger_resume_response| {
            if !debugger_resume_response.status.get_success() {
                log::warn!(
                    "Debugger resume failed: {}.",
                    debugger_resume_response
                        .status
                        .get_message()
                        .unwrap_or("unknown error")
                );
            } else if let Some(debugger_trace_view_data) = debugger_trace_view_data.read("Debugger resume response") {
                debugger_trace_view_data.set_debugger_session_state(debugger_resume_response.session_state);
            }
        });
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
        format!(
            "{} 0x{:X} [{} bytes]",
            Self::access_label(trace_session.get_access()),
            trace_session.get_address(),
            trace_session.get_size_in_bytes()
        )
    }

    fn format_pending_trace_target(pending_trace_start_request: &PendingDebuggerTraceStartRequest) -> String {
        let label = pending_trace_start_request
            .get_label()
            .filter(|label| !label.is_empty())
            .unwrap_or("selected address");

        format!(
            "{} for {} at 0x{:X} [{} bytes]",
            Self::prompt_action_label(pending_trace_start_request.get_access()),
            label,
            pending_trace_start_request.get_address(),
            pending_trace_start_request.get_size_in_bytes()
        )
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
                                self.show_attach_prompt_buttons(user_interface);
                            }
                        })
                        .desired_width(panel_width),
                    );
                });
            },
        );
    }

    fn show_attach_prompt_buttons(
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

                let attach_response = user_interface.add_sized(
                    button_size,
                    Button::new(RichText::new("Attach").color(theme.foreground))
                        .fill(theme.background_control_primary)
                        .stroke(Stroke::new(1.0, theme.background_control_primary_dark)),
                );

                if attach_response.clicked() {
                    self.confirm_pending_trace_start();
                }
            });
        });
    }

    fn confirm_pending_trace_start(&self) {
        let pending_trace_start_operation = self
            .debugger_trace_view_data
            .read("Debugger trace begin attach prompt")
            .and_then(|debugger_trace_view_data| debugger_trace_view_data.begin_pending_trace_start());
        let Some(pending_trace_start_operation) = pending_trace_start_operation else {
            return;
        };
        let operation_id = pending_trace_start_operation.get_operation_id();
        let pending_trace_start_request = pending_trace_start_operation.into_request();

        let engine_unprivileged_state = self.app_context.engine_unprivileged_state.clone();
        let debugger_trace_view_data = self.debugger_trace_view_data.clone();
        let dispatch_failure_debugger_trace_view_data = debugger_trace_view_data.clone();
        let is_attach_dispatched = DebuggerAttachRequest { plugin_id: None }.send(&engine_unprivileged_state.clone(), move |debugger_attach_response| {
            if !debugger_attach_response.status.get_success() {
                let status_message = format!(
                    "Debugger attach failed: {}.",
                    debugger_attach_response
                        .status
                        .get_message()
                        .unwrap_or("unknown error")
                );

                if let Some(debugger_trace_view_data) = debugger_trace_view_data.read("Debugger trace attach failed") {
                    debugger_trace_view_data.fail_pending_trace_start(operation_id, status_message);
                }

                return;
            }

            let debugger_trace_view_data = debugger_trace_view_data.clone();
            let trace_start_dispatch_failure_debugger_trace_view_data = debugger_trace_view_data.clone();
            let is_trace_start_dispatched = DebuggerTraceStartRequest {
                address: pending_trace_start_request.get_address(),
                size_in_bytes: pending_trace_start_request.get_size_in_bytes(),
                access: pending_trace_start_request.get_access(),
                label: pending_trace_start_request.get_label().map(String::from),
            }
            .send(&engine_unprivileged_state, move |debugger_trace_start_response| {
                if debugger_trace_start_response.status.get_success() {
                    if let Some(debugger_trace_view_data) = debugger_trace_view_data.read("Debugger trace start completed") {
                        debugger_trace_view_data.complete_pending_trace_start(operation_id);
                    }
                } else {
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
                }
            });

            if !is_trace_start_dispatched {
                if let Some(debugger_trace_view_data) = trace_start_dispatch_failure_debugger_trace_view_data.read("Debugger trace start dispatch failed") {
                    debugger_trace_view_data.fail_pending_trace_start(operation_id, String::from("Debugger trace start failed: command dispatch failed."));
                }
            }
        });

        if !is_attach_dispatched {
            if let Some(debugger_trace_view_data) = dispatch_failure_debugger_trace_view_data.read("Debugger attach dispatch failed") {
                debugger_trace_view_data.fail_pending_trace_start(operation_id, String::from("Debugger attach failed: command dispatch failed."));
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

        let content_width = content_rectangle.width().max(1.0);
        let hit_count_splitter_position_x = content_rectangle.min.x + 36.0;
        let instruction_splitter_position_x = content_rectangle.min.x + content_width * 0.18;
        let address_splitter_position_x = content_rectangle.min.x + content_width * 0.72;
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

        paint_header_label(user_interface, hit_count_splitter_position_x, "Count");
        paint_header_label(user_interface, instruction_splitter_position_x, "Instruction");
        paint_header_label(user_interface, address_splitter_position_x, "Address");
    }

    fn show_trace_session_header(
        &self,
        user_interface: &mut Ui,
        trace_session: &DebuggerTraceSessionDescriptor,
        debugger_session_state: DebuggerSessionState,
    ) {
        let theme = &self.app_context.theme;
        let header_height = 28.0;
        let horizontal_padding = 8.0;
        let button_size = vec2(24.0, 24.0);
        let button_spacing = 4.0;
        let (header_rectangle, _) = user_interface.allocate_exact_size(vec2(user_interface.available_width(), header_height), Sense::hover());
        let mut text_position_x = header_rectangle.min.x + horizontal_padding;
        let status_label = if trace_session.get_is_active() {
            match debugger_session_state {
                DebuggerSessionState::Paused => "Paused",
                _ => "Active",
            }
        } else {
            "Stopped"
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

        if trace_session.get_is_active() {
            let stop_button_rectangle = Rect::from_min_size(
                pos2(header_rectangle.min.x + horizontal_padding, header_rectangle.center().y - button_size.y * 0.5),
                button_size,
            );
            let stop_response = user_interface.place(
                stop_button_rectangle,
                IconButtonView::new(theme, &theme.icon_library.icon_handle_navigation_stop, "Stop trace."),
            );

            if stop_response.clicked() {
                self.stop_trace_session(trace_session.get_trace_session_id());
            }

            let control_button_rectangle = Rect::from_min_size(
                pos2(stop_button_rectangle.max.x + button_spacing, header_rectangle.center().y - button_size.y * 0.5),
                button_size,
            );

            if debugger_session_state == DebuggerSessionState::Paused {
                let resume_response = user_interface.place(
                    control_button_rectangle,
                    IconButtonView::new(theme, &theme.icon_library.icon_handle_navigation_right_arrows, "Resume target."),
                );

                if resume_response.clicked() {
                    self.resume_debugger();
                }
            } else {
                let pause_response = user_interface.place(
                    control_button_rectangle,
                    IconButtonView::new(theme, &theme.icon_library.icon_handle_navigation_pause, "Pause target."),
                );

                if pause_response.clicked() {
                    self.pause_debugger();
                }
            }

            text_position_x = control_button_rectangle.max.x + horizontal_padding;
        }

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
        debugger_session_state: DebuggerSessionState,
    ) {
        let theme = &self.app_context.theme;
        self.show_trace_session_header(user_interface, trace_session, debugger_session_state);

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
        let content_width = content_rectangle.width().max(1.0);
        let hit_count_splitter_position_x = content_rectangle.min.x + 36.0;
        let instruction_splitter_position_x = content_rectangle.min.x + content_width * 0.18;
        let address_splitter_position_x = content_rectangle.min.x + content_width * 0.72;

        for instruction_record in instruction_records {
            let instruction_key = DebuggerTraceInstructionKey::from_record(instruction_record);
            let is_selected = selected_instruction_keys.contains(&instruction_key);
            let row_response = user_interface.add(DebuggerTraceEntryView::new(
                self.app_context.clone(),
                instruction_record,
                &instruction_key,
                is_selected,
                hit_count_splitter_position_x,
                instruction_splitter_position_x,
                address_splitter_position_x,
            ));

            if row_response.clicked() {
                if let Some(debugger_trace_view_data) = self
                    .debugger_trace_view_data
                    .read("Debugger trace select instruction")
                {
                    debugger_trace_view_data.set_single_instruction_selection(instruction_key.clone());
                }
            }

            if row_response.double_clicked() {
                self.add_instruction_record_to_project(instruction_record);
            }
        }
    }

    fn add_instruction_record_to_project(
        &self,
        instruction_record: &DebuggerTraceInstructionRecord,
    ) {
        let Some(instruction_address) = instruction_record.get_instruction_address() else {
            log::warn!("Cannot add debugger trace instruction without an instruction address.");
            return;
        };
        let target_directory_path = ProjectHierarchyViewData::get_selected_directory_path(self.project_hierarchy_view_data.clone()).unwrap_or_default();
        let (project_item_address, project_item_module_name) = ProjectHierarchyModuleAddressResolver::resolve_absolute_address_to_project_item_address(
            &self.app_context.engine_unprivileged_state,
            instruction_address,
        );
        let project_item_name = Self::build_instruction_project_item_name(project_item_address, &project_item_module_name, instruction_record);
        let project_items_create_request = ProjectItemsCreateRequest {
            parent_directory_path: target_directory_path,
            project_item_name,
            is_directory: false,
            address: Some(project_item_address),
            module_name: Some(project_item_module_name),
            data_type_id: Some(self.instruction_data_type_id().to_string()),
            pointer_offsets: None,
        };

        project_items_create_request.send(&self.app_context.engine_unprivileged_state, |project_items_create_response| {
            if !project_items_create_response.success {
                log::warn!("Debugger trace add-to-project command failed.");
            }
        });
    }

    fn instruction_data_type_id(&self) -> &'static str {
        let process_bitness = self
            .process_selector_view_data
            .read("Debugger trace process bitness")
            .and_then(|process_selector_view_data| {
                process_selector_view_data
                    .opened_process
                    .as_ref()
                    .map(|opened_process_info| opened_process_info.get_bitness())
            });

        match process_bitness.unwrap_or(Bitness::Bit64) {
            Bitness::Bit32 => "i_x86",
            Bitness::Bit64 => "i_x64",
        }
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

                            self.show_instruction_records(
                                user_interface,
                                trace_session,
                                &session_instruction_records,
                                &snapshot.selected_instruction_keys,
                                snapshot.debugger_session_state,
                            );
                        }
                    });
            })
            .response;

        response
    }
}
