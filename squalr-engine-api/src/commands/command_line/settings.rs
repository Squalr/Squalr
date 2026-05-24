use crate as api;
use structopt::StructOpt;

#[derive(Clone, StructOpt, Debug)]
pub(crate) enum CommandLineSettingsCommand {
    General {
        #[structopt(flatten)]
        general_settings_command: CommandLineGeneralSettingsCommand,
    },
    Memory {
        #[structopt(flatten)]
        memory_settings_command: CommandLineMemorySettingsCommand,
    },
    Scan {
        #[structopt(flatten)]
        scan_settings_command: CommandLineScanSettingsCommand,
    },
}

#[derive(Clone, StructOpt, Debug)]
pub(crate) enum CommandLineGeneralSettingsCommand {
    List {
        #[structopt(flatten)]
        general_settings_list_request: CommandLineGeneralSettingsListRequest,
    },
    Set {
        #[structopt(flatten)]
        general_settings_set_request: CommandLineGeneralSettingsSetRequest,
    },
}

#[derive(Clone, StructOpt, Debug)]
pub(crate) struct CommandLineGeneralSettingsListRequest {}

#[derive(Clone, StructOpt, Debug, Default)]
pub(crate) struct CommandLineGeneralSettingsSetRequest {
    #[structopt(short = "r_delay", long)]
    pub engine_request_delay: Option<u64>,
}

#[derive(Clone, StructOpt, Debug)]
pub(crate) enum CommandLineMemorySettingsCommand {
    List {
        #[structopt(flatten)]
        memory_settings_list_request: CommandLineMemorySettingsListRequest,
    },
    Set {
        #[structopt(flatten)]
        memory_settings_set_request: CommandLineMemorySettingsSetRequest,
    },
}

#[derive(Clone, StructOpt, Debug)]
pub(crate) struct CommandLineMemorySettingsListRequest {}

#[derive(Clone, StructOpt, Debug, Default)]
pub(crate) struct CommandLineMemorySettingsSetRequest {
    #[structopt(long)]
    pub memory_type_none: Option<bool>,
    #[structopt(long)]
    pub memory_type_private: Option<bool>,
    #[structopt(long)]
    pub memory_type_image: Option<bool>,
    #[structopt(long)]
    pub memory_type_mapped: Option<bool>,
    #[structopt(long)]
    pub required_write: Option<bool>,
    #[structopt(long)]
    pub required_execute: Option<bool>,
    #[structopt(long)]
    pub required_copy_on_write: Option<bool>,
    #[structopt(long)]
    pub excluded_write: Option<bool>,
    #[structopt(long)]
    pub excluded_execute: Option<bool>,
    #[structopt(long)]
    pub excluded_copy_on_write: Option<bool>,
    #[structopt(long)]
    pub start_address: Option<u64>,
    #[structopt(long)]
    pub end_address: Option<u64>,
    #[structopt(long)]
    pub only_query_usermode: Option<bool>,
}

#[derive(Clone, StructOpt, Debug)]
pub(crate) enum CommandLineScanSettingsCommand {
    List {
        #[structopt(flatten)]
        scan_settings_list_request: CommandLineScanSettingsListRequest,
    },
    Set {
        #[structopt(flatten)]
        scan_settings_set_request: CommandLineScanSettingsSetRequest,
    },
}

#[derive(Clone, StructOpt, Debug)]
pub(crate) struct CommandLineScanSettingsListRequest {}

#[derive(Clone, StructOpt, Debug, Default)]
pub(crate) struct CommandLineScanSettingsSetRequest {
    #[structopt(long)]
    pub page_retrieval_mode: Option<api::plugins::memory_view::PageRetrievalMode>,
    #[structopt(long)]
    pub results_page_size: Option<u32>,
    #[structopt(long)]
    pub results_read_interval_ms: Option<u64>,
    #[structopt(long)]
    pub project_read_interval_ms: Option<u64>,
    #[structopt(long)]
    pub project_file_system_watch_enabled: Option<bool>,
    #[structopt(long)]
    pub freeze_interval_ms: Option<u64>,
    #[structopt(long)]
    pub memory_alignment: Option<api::structures::memory::memory_alignment::MemoryAlignment>,
    #[structopt(long)]
    pub memory_read_mode: Option<api::structures::scanning::memory_read_mode::MemoryReadMode>,
    #[structopt(long)]
    pub floating_point_tolerance: Option<api::structures::data_types::floating_point_tolerance::FloatingPointTolerance>,
    #[structopt(long)]
    pub is_single_threaded_scan: Option<bool>,
    #[structopt(long)]
    pub debug_perform_validation_scan: Option<bool>,
}

impl From<CommandLineSettingsCommand> for api::commands::settings::settings_command::SettingsCommand {
    fn from(command: CommandLineSettingsCommand) -> Self {
        match command {
            CommandLineSettingsCommand::General { general_settings_command } => Self::General {
                general_settings_command: general_settings_command.into(),
            },
            CommandLineSettingsCommand::Memory { memory_settings_command } => Self::Memory {
                memory_settings_command: memory_settings_command.into(),
            },
            CommandLineSettingsCommand::Scan { scan_settings_command } => Self::Scan {
                scan_settings_command: scan_settings_command.into(),
            },
        }
    }
}

impl From<CommandLineGeneralSettingsCommand> for api::commands::settings::general::general_settings_command::GeneralSettingsCommand {
    fn from(command: CommandLineGeneralSettingsCommand) -> Self {
        match command {
            CommandLineGeneralSettingsCommand::List { general_settings_list_request } => Self::List {
                general_settings_list_request: general_settings_list_request.into(),
            },
            CommandLineGeneralSettingsCommand::Set { general_settings_set_request } => Self::Set {
                general_settings_set_request: general_settings_set_request.into(),
            },
        }
    }
}

impl From<CommandLineGeneralSettingsListRequest> for api::commands::settings::general::list::general_settings_list_request::GeneralSettingsListRequest {
    fn from(_: CommandLineGeneralSettingsListRequest) -> Self {
        Self {}
    }
}

impl From<CommandLineGeneralSettingsSetRequest> for api::commands::settings::general::set::general_settings_set_request::GeneralSettingsSetRequest {
    fn from(request: CommandLineGeneralSettingsSetRequest) -> Self {
        Self {
            engine_request_delay: request.engine_request_delay,
        }
    }
}

impl From<CommandLineMemorySettingsCommand> for api::commands::settings::memory::memory_settings_command::MemorySettingsCommand {
    fn from(command: CommandLineMemorySettingsCommand) -> Self {
        match command {
            CommandLineMemorySettingsCommand::List { memory_settings_list_request } => Self::List {
                memory_settings_list_request: memory_settings_list_request.into(),
            },
            CommandLineMemorySettingsCommand::Set { memory_settings_set_request } => Self::Set {
                memory_settings_set_request: memory_settings_set_request.into(),
            },
        }
    }
}

impl From<CommandLineMemorySettingsListRequest> for api::commands::settings::memory::list::memory_settings_list_request::MemorySettingsListRequest {
    fn from(_: CommandLineMemorySettingsListRequest) -> Self {
        Self {}
    }
}

impl From<CommandLineMemorySettingsSetRequest> for api::commands::settings::memory::set::memory_settings_set_request::MemorySettingsSetRequest {
    fn from(request: CommandLineMemorySettingsSetRequest) -> Self {
        Self {
            memory_type_none: request.memory_type_none,
            memory_type_private: request.memory_type_private,
            memory_type_image: request.memory_type_image,
            memory_type_mapped: request.memory_type_mapped,
            required_write: request.required_write,
            required_execute: request.required_execute,
            required_copy_on_write: request.required_copy_on_write,
            excluded_write: request.excluded_write,
            excluded_execute: request.excluded_execute,
            excluded_copy_on_write: request.excluded_copy_on_write,
            start_address: request.start_address,
            end_address: request.end_address,
            only_query_usermode: request.only_query_usermode,
        }
    }
}

impl From<CommandLineScanSettingsCommand> for api::commands::settings::scan::scan_settings_command::ScanSettingsCommand {
    fn from(command: CommandLineScanSettingsCommand) -> Self {
        match command {
            CommandLineScanSettingsCommand::List { scan_settings_list_request } => Self::List {
                scan_settings_list_request: scan_settings_list_request.into(),
            },
            CommandLineScanSettingsCommand::Set { scan_settings_set_request } => Self::Set {
                scan_settings_set_request: scan_settings_set_request.into(),
            },
        }
    }
}

impl From<CommandLineScanSettingsListRequest> for api::commands::settings::scan::list::scan_settings_list_request::ScanSettingsListRequest {
    fn from(_: CommandLineScanSettingsListRequest) -> Self {
        Self {}
    }
}

impl From<CommandLineScanSettingsSetRequest> for api::commands::settings::scan::set::scan_settings_set_request::ScanSettingsSetRequest {
    fn from(request: CommandLineScanSettingsSetRequest) -> Self {
        Self {
            page_retrieval_mode: request.page_retrieval_mode,
            results_page_size: request.results_page_size,
            results_read_interval_ms: request.results_read_interval_ms,
            project_read_interval_ms: request.project_read_interval_ms,
            project_file_system_watch_enabled: request.project_file_system_watch_enabled,
            freeze_interval_ms: request.freeze_interval_ms,
            memory_alignment: request.memory_alignment,
            memory_read_mode: request.memory_read_mode,
            floating_point_tolerance: request.floating_point_tolerance,
            is_single_threaded_scan: request.is_single_threaded_scan,
            debug_perform_validation_scan: request.debug_perform_validation_scan,
        }
    }
}
