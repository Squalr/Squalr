use crate::{
    app_context::AppContext,
    views::debugger_trace::view_data::debugger_trace_view_data::{DebuggerTraceInstructionKey, DebuggerTraceViewData},
};
use eframe::egui::{Align, Button, Grid, Layout, RichText, ScrollArea, Sense, Ui, Widget};
use squalr_engine_api::{
    commands::{debugger::trace_stop::debugger_trace_stop_request::DebuggerTraceStopRequest, privileged_command_request::PrivilegedCommandRequest},
    dependency_injection::dependency::Dependency,
    events::debugger::trace_session_updated::debugger_trace_session_updated_event::DebuggerTraceSessionUpdatedEvent,
    structures::debugger::{DebuggerDataBreakpointAccess, DebuggerTraceInstructionRecord, DebuggerTraceSessionDescriptor},
};
use std::sync::Arc;

#[derive(Clone)]
pub struct DebuggerTraceView {
    app_context: Arc<AppContext>,
    debugger_trace_view_data: Dependency<DebuggerTraceViewData>,
}

impl DebuggerTraceView {
    pub const WINDOW_ID: &'static str = "window_debugger_trace";

    pub fn new(app_context: Arc<AppContext>) -> Self {
        let debugger_trace_view_data = app_context
            .dependency_container
            .register(DebuggerTraceViewData::new());
        let instance = Self {
            app_context,
            debugger_trace_view_data,
        };

        instance.listen_for_trace_session_updates();

        instance
    }

    fn listen_for_trace_session_updates(&self) {
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
        DebuggerTraceStopRequest {
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
    }

    fn access_label(access: DebuggerDataBreakpointAccess) -> &'static str {
        match access {
            DebuggerDataBreakpointAccess::Read => "Reads",
            DebuggerDataBreakpointAccess::Write => "Writes",
            DebuggerDataBreakpointAccess::ReadWrite => "Accesses",
        }
    }

    fn format_address(address: Option<u64>) -> String {
        address
            .map(|address| format!("0x{:X}", address))
            .unwrap_or_else(|| String::from("-"))
    }

    fn format_trace_target(trace_session: Option<&DebuggerTraceSessionDescriptor>) -> String {
        trace_session
            .map(|trace_session| {
                format!(
                    "{} 0x{:X} [{} bytes]",
                    Self::access_label(trace_session.get_access()),
                    trace_session.get_address(),
                    trace_session.get_size_in_bytes()
                )
            })
            .unwrap_or_else(|| String::from("-"))
    }

    fn record_instruction_text(instruction_record: &DebuggerTraceInstructionRecord) -> String {
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
                let snapshot = debugger_trace_view_data.get_snapshot();
                let has_active_trace_session = snapshot
                    .trace_sessions
                    .iter()
                    .any(DebuggerTraceSessionDescriptor::get_is_active);

                user_interface.horizontal(|user_interface| {
                    if user_interface
                        .add_enabled(!snapshot.selected_instruction_keys.is_empty(), Button::new("Add Selected to Project"))
                        .on_disabled_hover_text("Instruction project items are not wired yet.")
                        .clicked()
                    {
                        log::warn!("Adding debugger trace instructions to the project is not implemented yet.");
                    }

                    if user_interface
                        .add_enabled(!snapshot.selected_instruction_keys.is_empty(), Button::new("Clear Selection"))
                        .clicked()
                    {
                        debugger_trace_view_data.clear_selection();
                    }

                    if user_interface
                        .add_enabled(!snapshot.trace_sessions.is_empty() && !has_active_trace_session, Button::new("Clear View"))
                        .on_disabled_hover_text("Stop active traces before clearing the view.")
                        .clicked()
                    {
                        debugger_trace_view_data.clear();
                    }
                });

                if snapshot.trace_sessions.is_empty() {
                    user_interface.label(
                        RichText::new("No debugger trace sessions.")
                            .font(theme.font_library.font_noto_sans.font_normal.clone())
                            .color(theme.foreground),
                    );
                    return;
                }

                user_interface.separator();

                ScrollArea::vertical()
                    .id_salt("debugger_trace_scroll")
                    .auto_shrink([false, false])
                    .show(user_interface, |user_interface| {
                        for trace_session in &snapshot.trace_sessions {
                            user_interface.horizontal(|user_interface| {
                                let session_label = format!("{} | {}", trace_session.get_trace_session_id(), Self::format_trace_target(Some(trace_session)));

                                user_interface.label(
                                    RichText::new(session_label)
                                        .font(theme.font_library.font_noto_sans.font_header.clone())
                                        .color(theme.foreground),
                                );

                                if trace_session.get_is_active() && user_interface.button("Stop").clicked() {
                                    self.stop_trace_session(trace_session.get_trace_session_id());
                                }
                            });

                            Grid::new(format!("debugger_trace_grid_{}", trace_session.get_trace_session_id()))
                                .striped(true)
                                .min_col_width(64.0)
                                .show(user_interface, |user_interface| {
                                    user_interface.label(RichText::new("").strong());
                                    user_interface.label(RichText::new("Hits").strong());
                                    user_interface.label(RichText::new("Instruction").strong());
                                    user_interface.label(RichText::new("Address").strong());
                                    user_interface.label(RichText::new("Backend").strong());
                                    user_interface.end_row();

                                    for instruction_record in snapshot
                                        .instruction_records
                                        .iter()
                                        .filter(|instruction_record| instruction_record.get_trace_session_id() == trace_session.get_trace_session_id())
                                    {
                                        let instruction_key = DebuggerTraceInstructionKey::from_record(instruction_record);
                                        let is_selected = snapshot.selected_instruction_keys.contains(&instruction_key);
                                        let mut selected = is_selected;

                                        if user_interface.checkbox(&mut selected, "").changed() {
                                            debugger_trace_view_data.set_instruction_selected(instruction_key.clone(), selected);
                                        }

                                        user_interface.label(instruction_record.get_hit_count().to_string());

                                        let instruction_response = user_interface
                                            .add(eframe::egui::Label::new(Self::record_instruction_text(instruction_record)).sense(Sense::click()));

                                        if instruction_response.clicked() {
                                            debugger_trace_view_data.set_instruction_selected(instruction_key.clone(), !is_selected);
                                        }

                                        user_interface.label(Self::format_address(instruction_record.get_instruction_address()));
                                        user_interface.label(instruction_record.get_last_backend_message().unwrap_or("-"));
                                        user_interface.end_row();
                                    }
                                });

                            user_interface.separator();
                        }
                    });
            })
            .response;

        response
    }
}
