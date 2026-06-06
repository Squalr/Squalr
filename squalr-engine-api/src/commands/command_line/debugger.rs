use crate as api;
use structopt::StructOpt;

#[derive(Clone, StructOpt, Debug)]
pub(crate) enum CommandLineDebuggerCommand {
    Attach {
        #[structopt(flatten)]
        debugger_attach_request: CommandLineDebuggerAttachRequest,
    },
    Detach {
        #[structopt(flatten)]
        debugger_detach_request: CommandLineDebuggerDetachRequest,
    },
    Pause {
        #[structopt(flatten)]
        debugger_pause_request: CommandLineDebuggerPauseRequest,
    },
    Resume {
        #[structopt(flatten)]
        debugger_resume_request: CommandLineDebuggerResumeRequest,
    },
    BreakpointSet {
        #[structopt(flatten)]
        debugger_breakpoint_set_request: CommandLineDebuggerBreakpointSetRequest,
    },
    BreakpointRemove {
        #[structopt(flatten)]
        debugger_breakpoint_remove_request: CommandLineDebuggerBreakpointRemoveRequest,
    },
    BreakpointList {
        #[structopt(flatten)]
        debugger_breakpoint_list_request: CommandLineDebuggerBreakpointListRequest,
    },
    RegistersRead {
        #[structopt(flatten)]
        debugger_registers_read_request: CommandLineDebuggerRegistersReadRequest,
    },
    RegisterWrite {
        #[structopt(flatten)]
        debugger_register_write_request: CommandLineDebuggerRegisterWriteRequest,
    },
    TraceStart {
        #[structopt(flatten)]
        debugger_trace_start_request: CommandLineDebuggerTraceStartRequest,
    },
    TraceStop {
        #[structopt(flatten)]
        debugger_trace_stop_request: CommandLineDebuggerTraceStopRequest,
    },
    TraceList {
        #[structopt(flatten)]
        debugger_trace_list_request: CommandLineDebuggerTraceListRequest,
    },
    #[structopt(alias = "what-reads")]
    FindWhatReads {
        #[structopt(flatten)]
        debugger_trace_target_request: CommandLineDebuggerTraceTargetRequest,
    },
    #[structopt(alias = "what-writes")]
    FindWhatWrites {
        #[structopt(flatten)]
        debugger_trace_target_request: CommandLineDebuggerTraceTargetRequest,
    },
    #[structopt(alias = "what-accesses")]
    FindWhatAccesses {
        #[structopt(flatten)]
        debugger_trace_target_request: CommandLineDebuggerTraceTargetRequest,
    },
}

#[derive(Clone, StructOpt, Debug, Default)]
pub(crate) struct CommandLineDebuggerAttachRequest {
    #[structopt(long = "plugin-id")]
    pub plugin_id: Option<String>,
}

#[derive(Clone, StructOpt, Debug, Default)]
pub(crate) struct CommandLineDebuggerDetachRequest {}

#[derive(Clone, StructOpt, Debug, Default)]
pub(crate) struct CommandLineDebuggerPauseRequest {}

#[derive(Clone, StructOpt, Debug, Default)]
pub(crate) struct CommandLineDebuggerResumeRequest {}

#[derive(Clone, StructOpt, Debug)]
pub(crate) struct CommandLineDebuggerBreakpointSetRequest {
    #[structopt(short = "a", long, parse(try_from_str = api::conversions::conversions_from_primitives::Conversions::parse_hex_or_int))]
    pub address: u64,
    #[structopt(short = "s", long, default_value = "1")]
    pub size_in_bytes: u8,
    #[structopt(long = "access", default_value = "write")]
    pub access: api::structures::debugger::DebuggerDataBreakpointAccess,
    #[structopt(long)]
    pub label: Option<String>,
}

#[derive(Clone, StructOpt, Debug)]
pub(crate) struct CommandLineDebuggerBreakpointRemoveRequest {
    #[structopt(long = "breakpoint-id")]
    pub breakpoint_id: String,
}

#[derive(Clone, StructOpt, Debug, Default)]
pub(crate) struct CommandLineDebuggerBreakpointListRequest {}

#[derive(Clone, StructOpt, Debug, Default)]
pub(crate) struct CommandLineDebuggerRegistersReadRequest {}

#[derive(Clone, StructOpt, Debug)]
pub(crate) struct CommandLineDebuggerRegisterWriteRequest {
    #[structopt(long = "register")]
    pub register_name: String,
    #[structopt(short = "v", long, parse(try_from_str = api::conversions::conversions_from_primitives::Conversions::parse_hex_or_int))]
    pub value: u64,
}

#[derive(Clone, StructOpt, Debug)]
pub(crate) struct CommandLineDebuggerTraceStartRequest {
    #[structopt(short = "a", long, parse(try_from_str = api::conversions::conversions_from_primitives::Conversions::parse_hex_or_int))]
    pub address: u64,
    #[structopt(short = "s", long, default_value = "1")]
    pub size_in_bytes: u8,
    #[structopt(long = "access", default_value = "write")]
    pub access: api::structures::debugger::DebuggerDataBreakpointAccess,
    #[structopt(long)]
    pub label: Option<String>,
}

#[derive(Clone, StructOpt, Debug)]
pub(crate) struct CommandLineDebuggerTraceTargetRequest {
    #[structopt(short = "a", long, parse(try_from_str = api::conversions::conversions_from_primitives::Conversions::parse_hex_or_int))]
    pub address: u64,
    #[structopt(short = "s", long, default_value = "1")]
    pub size_in_bytes: u8,
    #[structopt(long)]
    pub label: Option<String>,
}

#[derive(Clone, StructOpt, Debug)]
pub(crate) struct CommandLineDebuggerTraceStopRequest {
    #[structopt(long = "trace-session-id")]
    pub trace_session_id: String,
}

#[derive(Clone, StructOpt, Debug, Default)]
pub(crate) struct CommandLineDebuggerTraceListRequest {}

impl From<CommandLineDebuggerCommand> for api::commands::debugger::debugger_command::DebuggerCommand {
    fn from(command: CommandLineDebuggerCommand) -> Self {
        match command {
            CommandLineDebuggerCommand::Attach { debugger_attach_request } => Self::Attach {
                debugger_attach_request: debugger_attach_request.into(),
            },
            CommandLineDebuggerCommand::Detach { debugger_detach_request } => Self::Detach {
                debugger_detach_request: debugger_detach_request.into(),
            },
            CommandLineDebuggerCommand::Pause { debugger_pause_request } => Self::Pause {
                debugger_pause_request: debugger_pause_request.into(),
            },
            CommandLineDebuggerCommand::Resume { debugger_resume_request } => Self::Resume {
                debugger_resume_request: debugger_resume_request.into(),
            },
            CommandLineDebuggerCommand::BreakpointSet {
                debugger_breakpoint_set_request,
            } => Self::BreakpointSet {
                debugger_breakpoint_set_request: debugger_breakpoint_set_request.into(),
            },
            CommandLineDebuggerCommand::BreakpointRemove {
                debugger_breakpoint_remove_request,
            } => Self::BreakpointRemove {
                debugger_breakpoint_remove_request: debugger_breakpoint_remove_request.into(),
            },
            CommandLineDebuggerCommand::BreakpointList {
                debugger_breakpoint_list_request,
            } => Self::BreakpointList {
                debugger_breakpoint_list_request: debugger_breakpoint_list_request.into(),
            },
            CommandLineDebuggerCommand::RegistersRead {
                debugger_registers_read_request,
            } => Self::RegistersRead {
                debugger_registers_read_request: debugger_registers_read_request.into(),
            },
            CommandLineDebuggerCommand::RegisterWrite {
                debugger_register_write_request,
            } => Self::RegisterWrite {
                debugger_register_write_request: debugger_register_write_request.into(),
            },
            CommandLineDebuggerCommand::TraceStart { debugger_trace_start_request } => Self::TraceStart {
                debugger_trace_start_request: debugger_trace_start_request.into(),
            },
            CommandLineDebuggerCommand::TraceStop { debugger_trace_stop_request } => Self::TraceStop {
                debugger_trace_stop_request: debugger_trace_stop_request.into(),
            },
            CommandLineDebuggerCommand::TraceList { debugger_trace_list_request } => Self::TraceList {
                debugger_trace_list_request: debugger_trace_list_request.into(),
            },
            CommandLineDebuggerCommand::FindWhatReads { debugger_trace_target_request } => Self::TraceStart {
                debugger_trace_start_request: debugger_trace_target_request
                    .into_trace_start_request(api::structures::debugger::DebuggerDataBreakpointAccess::Read),
            },
            CommandLineDebuggerCommand::FindWhatWrites { debugger_trace_target_request } => Self::TraceStart {
                debugger_trace_start_request: debugger_trace_target_request
                    .into_trace_start_request(api::structures::debugger::DebuggerDataBreakpointAccess::Write),
            },
            CommandLineDebuggerCommand::FindWhatAccesses { debugger_trace_target_request } => Self::TraceStart {
                debugger_trace_start_request: debugger_trace_target_request
                    .into_trace_start_request(api::structures::debugger::DebuggerDataBreakpointAccess::ReadWrite),
            },
        }
    }
}

impl From<CommandLineDebuggerAttachRequest> for api::commands::debugger::attach::debugger_attach_request::DebuggerAttachRequest {
    fn from(request: CommandLineDebuggerAttachRequest) -> Self {
        Self { plugin_id: request.plugin_id }
    }
}

impl From<CommandLineDebuggerDetachRequest> for api::commands::debugger::detach::debugger_detach_request::DebuggerDetachRequest {
    fn from(_: CommandLineDebuggerDetachRequest) -> Self {
        Self {}
    }
}

impl From<CommandLineDebuggerPauseRequest> for api::commands::debugger::pause::debugger_pause_request::DebuggerPauseRequest {
    fn from(_: CommandLineDebuggerPauseRequest) -> Self {
        Self {}
    }
}

impl From<CommandLineDebuggerResumeRequest> for api::commands::debugger::resume::debugger_resume_request::DebuggerResumeRequest {
    fn from(_: CommandLineDebuggerResumeRequest) -> Self {
        Self {}
    }
}

impl From<CommandLineDebuggerBreakpointSetRequest> for api::commands::debugger::breakpoint_set::debugger_breakpoint_set_request::DebuggerBreakpointSetRequest {
    fn from(request: CommandLineDebuggerBreakpointSetRequest) -> Self {
        Self {
            address: request.address,
            kind: api::structures::debugger::DebuggerBreakpointKind::hardware_data(request.access, request.size_in_bytes),
            label: request.label,
        }
    }
}

impl From<CommandLineDebuggerBreakpointRemoveRequest>
    for api::commands::debugger::breakpoint_remove::debugger_breakpoint_remove_request::DebuggerBreakpointRemoveRequest
{
    fn from(request: CommandLineDebuggerBreakpointRemoveRequest) -> Self {
        Self {
            breakpoint_id: request.breakpoint_id,
        }
    }
}

impl From<CommandLineDebuggerBreakpointListRequest>
    for api::commands::debugger::breakpoint_list::debugger_breakpoint_list_request::DebuggerBreakpointListRequest
{
    fn from(_: CommandLineDebuggerBreakpointListRequest) -> Self {
        Self {}
    }
}

impl From<CommandLineDebuggerRegistersReadRequest> for api::commands::debugger::registers_read::debugger_registers_read_request::DebuggerRegistersReadRequest {
    fn from(_: CommandLineDebuggerRegistersReadRequest) -> Self {
        Self {}
    }
}

impl From<CommandLineDebuggerRegisterWriteRequest> for api::commands::debugger::register_write::debugger_register_write_request::DebuggerRegisterWriteRequest {
    fn from(request: CommandLineDebuggerRegisterWriteRequest) -> Self {
        Self {
            register_name: request.register_name,
            value: request.value,
        }
    }
}

impl From<CommandLineDebuggerTraceStartRequest> for api::commands::debugger::trace_start::debugger_trace_start_request::DebuggerTraceStartRequest {
    fn from(request: CommandLineDebuggerTraceStartRequest) -> Self {
        Self {
            address: request.address,
            size_in_bytes: request.size_in_bytes,
            access: request.access,
            label: request.label,
        }
    }
}

impl CommandLineDebuggerTraceTargetRequest {
    fn into_trace_start_request(
        self,
        access: api::structures::debugger::DebuggerDataBreakpointAccess,
    ) -> api::commands::debugger::trace_start::debugger_trace_start_request::DebuggerTraceStartRequest {
        api::commands::debugger::trace_start::debugger_trace_start_request::DebuggerTraceStartRequest {
            address: self.address,
            size_in_bytes: self.size_in_bytes,
            access,
            label: self.label,
        }
    }
}

impl From<CommandLineDebuggerTraceStopRequest> for api::commands::debugger::trace_stop::debugger_trace_stop_request::DebuggerTraceStopRequest {
    fn from(request: CommandLineDebuggerTraceStopRequest) -> Self {
        Self {
            trace_session_id: request.trace_session_id,
        }
    }
}

impl From<CommandLineDebuggerTraceListRequest> for api::commands::debugger::trace_list::debugger_trace_list_request::DebuggerTraceListRequest {
    fn from(_: CommandLineDebuggerTraceListRequest) -> Self {
        Self {}
    }
}
