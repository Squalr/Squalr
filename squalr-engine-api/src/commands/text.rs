use crate as api;
use crate::commands::privileged_command::PrivilegedCommand;
use crate::commands::unprivileged_command::UnprivilegedCommand;
use std::collections::BTreeMap;
use std::ffi::OsString;
use std::path::PathBuf;
use structopt::StructOpt;
use thiserror::Error;

pub use structopt::clap;

#[derive(Clone, Debug)]
pub enum TextCommand {
    Privileged(PrivilegedCommand),
    Unprivileged(UnprivilegedCommand),
}

#[derive(Debug, Error)]
pub enum TextCommandParseError {
    #[error("Error parsing input")]
    InvalidShellWords,
    #[error("No command provided")]
    EmptyCommand,
    #[error("{0}")]
    Command(#[from] clap::Error),
}

pub fn parse_command_line(input: &str) -> Result<TextCommand, TextCommandParseError> {
    parse_command_line_with_program_name(input, "squalr")
}

pub fn parse_prompt_command_line(input: &str) -> Result<TextCommand, TextCommandParseError> {
    parse_command_line_with_program_name(input, "")
}

pub fn parse_command_line_with_program_name(
    input: &str,
    program_name: &str,
) -> Result<TextCommand, TextCommandParseError> {
    let mut command_arguments = shlex::split(input).ok_or(TextCommandParseError::InvalidShellWords)?;

    if command_arguments.is_empty() {
        return Err(TextCommandParseError::EmptyCommand);
    }

    command_arguments.insert(0, program_name.to_string());

    parse_text_command(command_arguments).map_err(TextCommandParseError::Command)
}

pub fn format_prompt_command_error(error: &clap::Error) -> String {
    let normalized_message = normalize_prompt_command_message(&error.message);

    match error.kind {
        clap::ErrorKind::HelpDisplayed | clap::ErrorKind::VersionDisplayed => normalized_message,
        _ => summarize_prompt_command_error(&normalized_message),
    }
}

fn normalize_prompt_command_message(message: &str) -> String {
    message
        .lines()
        .filter(|line| !line.trim_start().starts_with("For more information try"))
        .map(strip_prompt_command_usage_padding)
        .collect::<Vec<_>>()
        .join("\n")
        .trim()
        .to_string()
}

fn strip_prompt_command_usage_padding(line: &str) -> &str {
    if line.trim().is_empty() {
        return "";
    }

    line.strip_prefix("    ").unwrap_or(line)
}

fn summarize_prompt_command_error(message: &str) -> String {
    let mut summary_lines = Vec::new();

    if let Some(first_error_line) = message.lines().find(|line| !line.trim().is_empty()) {
        summary_lines.push(first_error_line.trim().to_string());
    }

    if let Some(usage_line) = prompt_command_usage_line(message) {
        summary_lines.push(format!("Usage: {}", usage_line.trim()));
    }

    summary_lines.join("\n")
}

fn prompt_command_usage_line(message: &str) -> Option<&str> {
    let mut lines = message.lines();

    while let Some(line) = lines.next() {
        if line.trim() == "USAGE:" {
            return lines.find(|usage_line| !usage_line.trim().is_empty());
        }
    }

    None
}

pub fn parse_text_command<I, T>(iterator: I) -> Result<TextCommand, clap::Error>
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    ConsoleCommand::from_iter_safe(iterator).map(Into::into)
}

pub fn parse_privileged_command<I, T>(iterator: I) -> Result<PrivilegedCommand, clap::Error>
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    match parse_text_command(iterator)? {
        TextCommand::Privileged(command) => Ok(command),
        TextCommand::Unprivileged(_) => Err(clap::Error::with_description(
            "Expected a privileged command.",
            clap::ErrorKind::InvalidSubcommand,
        )),
    }
}

pub fn parse_unprivileged_command<I, T>(iterator: I) -> Result<UnprivilegedCommand, clap::Error>
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    match parse_text_command(iterator)? {
        TextCommand::Privileged(_) => Err(clap::Error::with_description(
            "Expected an unprivileged command.",
            clap::ErrorKind::InvalidSubcommand,
        )),
        TextCommand::Unprivileged(command) => Ok(command),
    }
}

#[derive(Clone, StructOpt, Debug)]
enum ConsoleCommand {
    #[structopt(alias = "mem", alias = "m")]
    Memory(ConsoleMemoryCommand),
    #[structopt(alias = "plug", alias = "plugins")]
    Plugins(ConsolePluginsCommand),
    #[structopt(alias = "proc", alias = "pr")]
    Process(ConsoleProcessCommand),
    #[structopt(alias = "reg")]
    Registry(ConsoleRegistryCommand),
    #[structopt(alias = "res", alias = "r")]
    Results(ConsoleScanResultsCommand),
    #[structopt(alias = "scan", alias = "s")]
    Scan(ConsoleScanCommand),
    #[structopt(alias = "pscan")]
    PointerScan(ConsolePointerScanCommand),
    #[structopt(alias = "sscan")]
    StructScan(ConsoleStructScanCommand),
    #[structopt(alias = "set", alias = "st")]
    Settings(ConsoleSettingsCommand),
    #[structopt(alias = "tasks", alias = "tt")]
    TrackableTasks(ConsoleTrackableTasksCommand),
    #[structopt(alias = "proj", alias = "p")]
    Project(ConsoleProjectCommand),
    #[structopt(alias = "proj_items", alias = "project_items", alias = "pi")]
    ProjectItems(ConsoleProjectItemsCommand),
    #[structopt(alias = "proj_symbols", alias = "project_symbols", alias = "ps")]
    ProjectSymbols(ConsoleProjectSymbolsCommand),
}

impl From<ConsoleCommand> for TextCommand {
    fn from(command: ConsoleCommand) -> Self {
        match command {
            ConsoleCommand::Memory(command) => Self::Privileged(PrivilegedCommand::Memory(command.into())),
            ConsoleCommand::Plugins(command) => Self::Privileged(PrivilegedCommand::Plugins(command.into())),
            ConsoleCommand::Process(command) => Self::Privileged(PrivilegedCommand::Process(command.into())),
            ConsoleCommand::Registry(command) => Self::Privileged(PrivilegedCommand::Registry(command.into())),
            ConsoleCommand::Results(command) => Self::Privileged(PrivilegedCommand::Results(command.into())),
            ConsoleCommand::Scan(command) => Self::Privileged(PrivilegedCommand::Scan(command.into())),
            ConsoleCommand::PointerScan(command) => Self::Privileged(PrivilegedCommand::PointerScan(command.into())),
            ConsoleCommand::StructScan(command) => Self::Privileged(PrivilegedCommand::StructScan(command.into())),
            ConsoleCommand::Settings(command) => Self::Privileged(PrivilegedCommand::Settings(command.into())),
            ConsoleCommand::TrackableTasks(command) => Self::Privileged(PrivilegedCommand::TrackableTasks(command.into())),
            ConsoleCommand::Project(command) => Self::Unprivileged(UnprivilegedCommand::Project(command.into())),
            ConsoleCommand::ProjectItems(command) => Self::Unprivileged(UnprivilegedCommand::ProjectItems(command.into())),
            ConsoleCommand::ProjectSymbols(command) => Self::Unprivileged(UnprivilegedCommand::ProjectSymbols(command.into())),
        }
    }
}

#[derive(Clone, StructOpt, Debug)]
enum ConsoleMemoryCommand {
    Freeze {
        #[structopt(flatten)]
        memory_freeze_request: ConsoleMemoryFreezeRequest,
    },
    Query {
        #[structopt(flatten)]
        memory_query_request: ConsoleMemoryQueryRequest,
    },
    Read {
        #[structopt(flatten)]
        memory_read_request: ConsoleMemoryReadRequest,
    },
    Write {
        #[structopt(flatten)]
        memory_write_request: ConsoleMemoryWriteRequest,
    },
}

#[derive(Clone, StructOpt, Debug, Default)]
struct ConsoleMemoryFreezeRequest {
    #[structopt(short = "f", long = "frozen")]
    pub is_frozen: bool,
}

#[derive(Clone, StructOpt, Debug)]
struct ConsoleMemoryQueryRequest {
    #[structopt(short = "p", long, default_value = "usermode")]
    pub page_retrieval_mode: api::plugins::memory_view::PageRetrievalMode,
}

#[derive(Clone, StructOpt, Debug)]
struct ConsoleMemoryReadRequest {
    #[structopt(short = "a", long, parse(try_from_str = api::conversions::conversions_from_primitives::Conversions::parse_hex_or_int))]
    pub address: u64,
    #[structopt(short = "m")]
    pub module_name: String,
    #[structopt(short = "v")]
    pub symbolic_struct_definition: api::structures::structs::symbolic_struct_definition::SymbolicStructDefinition,
    #[structopt(long)]
    pub suppress_logging: bool,
}

#[derive(Clone, StructOpt, Debug)]
struct ConsoleMemoryWriteRequest {
    #[structopt(short = "a", long, parse(try_from_str = api::conversions::conversions_from_primitives::Conversions::parse_hex_or_int))]
    pub address: u64,
    #[structopt(short = "m")]
    pub module_name: String,
    #[structopt(short = "v")]
    pub value: Vec<u8>,
}

impl From<ConsoleMemoryCommand> for api::commands::memory::memory_command::MemoryCommand {
    fn from(command: ConsoleMemoryCommand) -> Self {
        match command {
            ConsoleMemoryCommand::Freeze { memory_freeze_request } => Self::Freeze {
                memory_freeze_request: memory_freeze_request.into(),
            },
            ConsoleMemoryCommand::Query { memory_query_request } => Self::Query {
                memory_query_request: memory_query_request.into(),
            },
            ConsoleMemoryCommand::Read { memory_read_request } => Self::Read {
                memory_read_request: memory_read_request.into(),
            },
            ConsoleMemoryCommand::Write { memory_write_request } => Self::Write {
                memory_write_request: memory_write_request.into(),
            },
        }
    }
}

impl From<ConsoleMemoryFreezeRequest> for api::commands::memory::freeze::memory_freeze_request::MemoryFreezeRequest {
    fn from(request: ConsoleMemoryFreezeRequest) -> Self {
        Self {
            freeze_targets: Vec::new(),
            is_frozen: request.is_frozen,
        }
    }
}

impl From<ConsoleMemoryQueryRequest> for api::commands::memory::query::memory_query_request::MemoryQueryRequest {
    fn from(request: ConsoleMemoryQueryRequest) -> Self {
        Self {
            page_retrieval_mode: request.page_retrieval_mode,
        }
    }
}

impl From<ConsoleMemoryReadRequest> for api::commands::memory::read::memory_read_request::MemoryReadRequest {
    fn from(request: ConsoleMemoryReadRequest) -> Self {
        Self {
            address: request.address,
            module_name: request.module_name,
            symbolic_struct_definition: request.symbolic_struct_definition,
            suppress_logging: request.suppress_logging,
        }
    }
}

impl From<ConsoleMemoryWriteRequest> for api::commands::memory::write::memory_write_request::MemoryWriteRequest {
    fn from(request: ConsoleMemoryWriteRequest) -> Self {
        Self {
            address: request.address,
            module_name: request.module_name,
            value: request.value,
        }
    }
}

#[derive(Clone, StructOpt, Debug)]
enum ConsolePluginsCommand {
    List {
        #[structopt(flatten)]
        plugin_list_request: ConsolePluginListRequest,
    },
    SetEnabled {
        #[structopt(flatten)]
        plugin_set_enabled_request: ConsolePluginSetEnabledRequest,
    },
}

#[derive(Clone, StructOpt, Debug, Default)]
struct ConsolePluginListRequest {}

#[derive(Clone, StructOpt, Debug)]
struct ConsolePluginSetEnabledRequest {
    #[structopt(long = "plugin-id")]
    pub plugin_id: String,
    #[structopt(long = "enabled")]
    pub is_enabled: bool,
}

impl From<ConsolePluginsCommand> for api::commands::plugins::plugins_command::PluginsCommand {
    fn from(command: ConsolePluginsCommand) -> Self {
        match command {
            ConsolePluginsCommand::List { plugin_list_request } => Self::List {
                plugin_list_request: plugin_list_request.into(),
            },
            ConsolePluginsCommand::SetEnabled { plugin_set_enabled_request } => Self::SetEnabled {
                plugin_set_enabled_request: plugin_set_enabled_request.into(),
            },
        }
    }
}

impl From<ConsolePluginListRequest> for api::commands::plugins::list::plugin_list_request::PluginListRequest {
    fn from(_: ConsolePluginListRequest) -> Self {
        Self {}
    }
}

impl From<ConsolePluginSetEnabledRequest> for api::commands::plugins::set_enabled::plugin_set_enabled_request::PluginSetEnabledRequest {
    fn from(request: ConsolePluginSetEnabledRequest) -> Self {
        Self {
            plugin_id: request.plugin_id,
            is_enabled: request.is_enabled,
        }
    }
}

#[derive(Clone, StructOpt, Debug)]
enum ConsoleProcessCommand {
    Open {
        #[structopt(flatten)]
        process_open_request: ConsoleProcessOpenRequest,
    },
    List {
        #[structopt(flatten)]
        process_list_request: ConsoleProcessListRequest,
    },
    Close {
        #[structopt(flatten)]
        process_close_request: ConsoleProcessCloseRequest,
    },
}

#[derive(Clone, StructOpt, Debug)]
struct ConsoleProcessOpenRequest {
    #[structopt(short = "p", long)]
    pub process_id: Option<u32>,
    #[structopt(short = "n", long)]
    pub search_name: Option<String>,
    #[structopt(short = "m", long)]
    pub match_case: bool,
}

#[derive(Clone, StructOpt, Debug)]
struct ConsoleProcessListRequest {
    #[structopt(short = "w", long)]
    pub require_windowed: bool,
    #[structopt(short = "n", long)]
    pub search_name: Option<String>,
    #[structopt(short = "m", long)]
    pub match_case: bool,
    #[structopt(short = "l", long)]
    pub limit: Option<u64>,
    #[structopt(short = "i", long)]
    pub fetch_icons: bool,
}

#[derive(Clone, StructOpt, Debug)]
struct ConsoleProcessCloseRequest {}

impl From<ConsoleProcessCommand> for api::commands::process::process_command::ProcessCommand {
    fn from(command: ConsoleProcessCommand) -> Self {
        match command {
            ConsoleProcessCommand::Open { process_open_request } => Self::Open {
                process_open_request: process_open_request.into(),
            },
            ConsoleProcessCommand::List { process_list_request } => Self::List {
                process_list_request: process_list_request.into(),
            },
            ConsoleProcessCommand::Close { process_close_request } => Self::Close {
                process_close_request: process_close_request.into(),
            },
        }
    }
}

impl From<ConsoleProcessOpenRequest> for api::commands::process::open::process_open_request::ProcessOpenRequest {
    fn from(request: ConsoleProcessOpenRequest) -> Self {
        Self {
            process_id: request.process_id,
            search_name: request.search_name,
            match_case: request.match_case,
        }
    }
}

impl From<ConsoleProcessListRequest> for api::commands::process::list::process_list_request::ProcessListRequest {
    fn from(request: ConsoleProcessListRequest) -> Self {
        Self {
            require_windowed: request.require_windowed,
            search_name: request.search_name,
            match_case: request.match_case,
            limit: request.limit,
            fetch_icons: request.fetch_icons,
        }
    }
}

impl From<ConsoleProcessCloseRequest> for api::commands::process::close::process_close_request::ProcessCloseRequest {
    fn from(_: ConsoleProcessCloseRequest) -> Self {
        Self {}
    }
}

#[derive(Clone, StructOpt, Debug)]
enum ConsoleRegistryCommand {
    GetMetadata {
        #[structopt(flatten)]
        registry_get_metadata_request: ConsoleRegistryGetMetadataRequest,
    },
}

#[derive(Clone, Default, StructOpt, Debug)]
struct ConsoleRegistryGetMetadataRequest {}

impl From<ConsoleRegistryCommand> for api::commands::registry::registry_command::RegistryCommand {
    fn from(command: ConsoleRegistryCommand) -> Self {
        match command {
            ConsoleRegistryCommand::GetMetadata { registry_get_metadata_request } => Self::GetMetadata {
                registry_get_metadata_request: registry_get_metadata_request.into(),
            },
        }
    }
}

impl From<ConsoleRegistryGetMetadataRequest> for api::commands::registry::get_metadata::registry_get_metadata_request::RegistryGetMetadataRequest {
    fn from(_: ConsoleRegistryGetMetadataRequest) -> Self {
        Self {}
    }
}

#[derive(Clone, StructOpt, Debug)]
enum ConsoleScanCommand {
    Reset {
        #[structopt(flatten)]
        scan_reset_request: ConsoleScanResetRequest,
    },
    New {
        #[structopt(flatten)]
        scan_new_request: ConsoleScanNewRequest,
    },
    CollectValues {
        #[structopt(flatten)]
        scan_value_collector_request: ConsoleScanCollectValuesRequest,
    },
    ElementScan {
        #[structopt(flatten)]
        element_scan_request: ConsoleElementScanRequest,
    },
}

#[derive(Clone, StructOpt, Debug)]
struct ConsoleScanResetRequest {}

#[derive(Clone, StructOpt, Debug)]
struct ConsoleScanNewRequest {}

#[derive(Clone, StructOpt, Debug)]
struct ConsoleScanCollectValuesRequest {
    #[structopt(short = "d", long)]
    pub data_type_refs: Vec<api::structures::data_types::data_type_ref::DataTypeRef>,
}

#[derive(Clone, StructOpt, Debug)]
struct ConsoleElementScanRequest {
    #[structopt(short = "c", long)]
    pub scan_constraints: Vec<api::structures::scanning::constraints::anonymous_scan_constraint::AnonymousScanConstraint>,
    #[structopt(short = "d", long)]
    pub data_type_refs: Vec<api::structures::data_types::data_type_ref::DataTypeRef>,
}

impl From<ConsoleScanCommand> for api::commands::scan::scan_command::ScanCommand {
    fn from(command: ConsoleScanCommand) -> Self {
        match command {
            ConsoleScanCommand::Reset { scan_reset_request } => Self::Reset {
                scan_reset_request: scan_reset_request.into(),
            },
            ConsoleScanCommand::New { scan_new_request } => Self::New {
                scan_new_request: scan_new_request.into(),
            },
            ConsoleScanCommand::CollectValues { scan_value_collector_request } => Self::CollectValues {
                scan_value_collector_request: scan_value_collector_request.into(),
            },
            ConsoleScanCommand::ElementScan { element_scan_request } => Self::ElementScan {
                element_scan_request: element_scan_request.into(),
            },
        }
    }
}

impl From<ConsoleScanResetRequest> for api::commands::scan::reset::scan_reset_request::ScanResetRequest {
    fn from(_: ConsoleScanResetRequest) -> Self {
        Self {}
    }
}

impl From<ConsoleScanNewRequest> for api::commands::scan::new::scan_new_request::ScanNewRequest {
    fn from(_: ConsoleScanNewRequest) -> Self {
        Self {}
    }
}

impl From<ConsoleScanCollectValuesRequest> for api::commands::scan::collect_values::scan_collect_values_request::ScanCollectValuesRequest {
    fn from(request: ConsoleScanCollectValuesRequest) -> Self {
        Self {
            data_type_refs: request.data_type_refs,
        }
    }
}

impl From<ConsoleElementScanRequest> for api::commands::scan::element_scan::element_scan_request::ElementScanRequest {
    fn from(request: ConsoleElementScanRequest) -> Self {
        Self {
            scan_constraints: request.scan_constraints,
            data_type_refs: request.data_type_refs,
        }
    }
}

#[derive(Clone, StructOpt, Debug)]
enum ConsolePointerScanCommand {
    Start {
        #[structopt(flatten)]
        pointer_scan_start_request: ConsolePointerScanStartRequest,
    },
    Reset {
        #[structopt(flatten)]
        pointer_scan_reset_request: ConsolePointerScanResetRequest,
    },
    Summary {
        #[structopt(flatten)]
        pointer_scan_summary_request: ConsolePointerScanSummaryRequest,
    },
    Expand {
        #[structopt(flatten)]
        pointer_scan_expand_request: ConsolePointerScanExpandRequest,
    },
    Validate {
        #[structopt(flatten)]
        pointer_scan_validate_request: ConsolePointerScanValidateRequest,
    },
}

#[derive(Clone, Debug, Default, StructOpt, PartialEq)]
struct ConsolePointerScanTargetRequest {
    #[structopt(long = "target-address")]
    pub target_address: Option<api::structures::data_values::anonymous_value_string::AnonymousValueString>,
    #[structopt(long = "target-value")]
    pub target_value: Option<api::structures::data_values::anonymous_value_string::AnonymousValueString>,
    #[structopt(long = "target-data-type")]
    pub target_data_type: Option<api::structures::data_types::data_type_ref::DataTypeRef>,
}

#[derive(Clone, StructOpt, Debug)]
struct ConsolePointerScanStartRequest {
    #[structopt(flatten)]
    pub target: ConsolePointerScanTargetRequest,
    #[structopt(short = "s", long)]
    pub pointer_size: api::structures::pointer_scans::pointer_scan_pointer_size::PointerScanPointerSize,
    #[structopt(short = "d", long)]
    pub max_depth: u64,
    #[structopt(short = "o", long)]
    pub offset_radius: u64,
    #[structopt(long = "address-space", default_value = "emulator")]
    pub address_space: api::structures::pointer_scans::pointer_scan_address_space::PointerScanAddressSpace,
}

#[derive(Clone, StructOpt, Debug)]
struct ConsolePointerScanResetRequest {}

#[derive(Clone, StructOpt, Debug, Default)]
struct ConsolePointerScanSummaryRequest {
    #[structopt(short = "i", long)]
    pub session_id: Option<u64>,
}

#[derive(Clone, StructOpt, Debug)]
struct ConsolePointerScanExpandRequest {
    #[structopt(short = "i", long)]
    pub session_id: u64,
    #[structopt(long)]
    pub parent_node_id: Option<u64>,
    #[structopt(long, default_value = "0")]
    pub page_index: u64,
    #[structopt(long, default_value = "22")]
    pub page_size: u64,
}

#[derive(Clone, StructOpt, Debug)]
struct ConsolePointerScanValidateRequest {
    #[structopt(short = "i", long)]
    pub session_id: u64,
    #[structopt(flatten)]
    pub target: ConsolePointerScanTargetRequest,
}

impl From<ConsolePointerScanTargetRequest> for api::structures::pointer_scans::pointer_scan_target_request::PointerScanTargetRequest {
    fn from(request: ConsolePointerScanTargetRequest) -> Self {
        Self {
            target_address: request.target_address,
            target_value: request.target_value,
            target_data_type_ref: request.target_data_type,
        }
    }
}

impl From<ConsolePointerScanCommand> for api::commands::pointer_scan::pointer_scan_command::PointerScanCommand {
    fn from(command: ConsolePointerScanCommand) -> Self {
        match command {
            ConsolePointerScanCommand::Start { pointer_scan_start_request } => Self::Start {
                pointer_scan_start_request: pointer_scan_start_request.into(),
            },
            ConsolePointerScanCommand::Reset { pointer_scan_reset_request } => Self::Reset {
                pointer_scan_reset_request: pointer_scan_reset_request.into(),
            },
            ConsolePointerScanCommand::Summary { pointer_scan_summary_request } => Self::Summary {
                pointer_scan_summary_request: pointer_scan_summary_request.into(),
            },
            ConsolePointerScanCommand::Expand { pointer_scan_expand_request } => Self::Expand {
                pointer_scan_expand_request: pointer_scan_expand_request.into(),
            },
            ConsolePointerScanCommand::Validate { pointer_scan_validate_request } => Self::Validate {
                pointer_scan_validate_request: pointer_scan_validate_request.into(),
            },
        }
    }
}

impl From<ConsolePointerScanStartRequest> for api::commands::pointer_scan::start::pointer_scan_start_request::PointerScanStartRequest {
    fn from(request: ConsolePointerScanStartRequest) -> Self {
        Self {
            target: request.target.into(),
            pointer_size: request.pointer_size,
            max_depth: request.max_depth,
            offset_radius: request.offset_radius,
            address_space: request.address_space,
        }
    }
}

impl From<ConsolePointerScanResetRequest> for api::commands::pointer_scan::reset::pointer_scan_reset_request::PointerScanResetRequest {
    fn from(_: ConsolePointerScanResetRequest) -> Self {
        Self {}
    }
}

impl From<ConsolePointerScanSummaryRequest> for api::commands::pointer_scan::summary::pointer_scan_summary_request::PointerScanSummaryRequest {
    fn from(request: ConsolePointerScanSummaryRequest) -> Self {
        Self {
            session_id: request.session_id,
        }
    }
}

impl From<ConsolePointerScanExpandRequest> for api::commands::pointer_scan::expand::pointer_scan_expand_request::PointerScanExpandRequest {
    fn from(request: ConsolePointerScanExpandRequest) -> Self {
        Self {
            session_id: request.session_id,
            parent_node_id: request.parent_node_id,
            page_index: request.page_index,
            page_size: request.page_size,
        }
    }
}

impl From<ConsolePointerScanValidateRequest> for api::commands::pointer_scan::validate::pointer_scan_validate_request::PointerScanValidateRequest {
    fn from(request: ConsolePointerScanValidateRequest) -> Self {
        Self {
            session_id: request.session_id,
            target: request.target.into(),
        }
    }
}

#[derive(Clone, StructOpt, Debug)]
enum ConsoleScanResultsCommand {
    List {
        #[structopt(flatten)]
        scan_results_list_request: ConsoleScanResultsListRequest,
    },
    Query {
        #[structopt(flatten)]
        scan_results_query_request: ConsoleScanResultsQueryRequest,
    },
    Refresh {
        #[structopt(flatten)]
        scan_results_refresh_request: ConsoleScanResultsRefreshRequest,
    },
    Delete {
        #[structopt(flatten)]
        scan_results_delete_request: ConsoleScanResultsDeleteRequest,
    },
    Freeze {
        #[structopt(flatten)]
        scan_results_freeze_request: ConsoleScanResultsFreezeRequest,
    },
    SetProperty {
        #[structopt(flatten)]
        results_set_property_request: ConsoleScanResultsSetPropertyRequest,
    },
}

#[derive(Clone, StructOpt, Debug)]
struct ConsoleScanResultsListRequest {
    #[structopt(short = "p", long)]
    pub page_index: u64,
    #[structopt(long = "data-type-filter")]
    pub data_type_filters: Option<Vec<api::structures::data_types::data_type_ref::DataTypeRef>>,
}

#[derive(Clone, StructOpt, Debug)]
struct ConsoleScanResultsQueryRequest {
    #[structopt(short = "p", long)]
    pub page_index: u64,
    #[structopt(long = "data-type-filter")]
    pub data_type_filters: Option<Vec<api::structures::data_types::data_type_ref::DataTypeRef>>,
}

#[derive(Clone, StructOpt, Debug)]
struct ConsoleScanResultsRefreshRequest {
    #[structopt(short = "r", long)]
    pub scan_result_refs: Vec<api::structures::scan_results::scan_result_ref::ScanResultRef>,
}

#[derive(Clone, StructOpt, Debug)]
struct ConsoleScanResultsDeleteRequest {
    #[structopt(short = "s", long)]
    pub scan_result_refs: Vec<api::structures::scan_results::scan_result_ref::ScanResultRef>,
}

#[derive(Clone, StructOpt, Debug)]
struct ConsoleScanResultsFreezeRequest {
    #[structopt(short = "s", long)]
    pub scan_result_refs: Vec<api::structures::scan_results::scan_result_ref::ScanResultRef>,
    #[structopt(short = "f", long)]
    pub is_frozen: bool,
}

#[derive(Clone, StructOpt, Debug)]
struct ConsoleScanResultsSetPropertyRequest {
    #[structopt(short = "s", long)]
    pub scan_result_refs: Vec<api::structures::scan_results::scan_result_ref::ScanResultRef>,
    #[structopt(short = "v", long)]
    pub anonymous_value_string: api::structures::data_values::anonymous_value_string::AnonymousValueString,
    #[structopt(short = "f", long)]
    pub field_namespace: String,
}

impl From<ConsoleScanResultsCommand> for api::commands::scan_results::scan_results_command::ScanResultsCommand {
    fn from(command: ConsoleScanResultsCommand) -> Self {
        match command {
            ConsoleScanResultsCommand::List { scan_results_list_request } => Self::List {
                results_list_request: scan_results_list_request.into(),
            },
            ConsoleScanResultsCommand::Query { scan_results_query_request } => Self::Query {
                results_query_request: scan_results_query_request.into(),
            },
            ConsoleScanResultsCommand::Refresh { scan_results_refresh_request } => Self::Refresh {
                results_refresh_request: scan_results_refresh_request.into(),
            },
            ConsoleScanResultsCommand::Delete { scan_results_delete_request } => Self::Delete {
                results_delete_request: scan_results_delete_request.into(),
            },
            ConsoleScanResultsCommand::Freeze { scan_results_freeze_request } => Self::Freeze {
                results_freeze_request: scan_results_freeze_request.into(),
            },
            ConsoleScanResultsCommand::SetProperty { results_set_property_request } => Self::SetProperty {
                results_set_property_request: results_set_property_request.into(),
            },
        }
    }
}

impl From<ConsoleScanResultsListRequest> for api::commands::scan_results::list::scan_results_list_request::ScanResultsListRequest {
    fn from(request: ConsoleScanResultsListRequest) -> Self {
        Self {
            page_index: request.page_index,
            data_type_filters: request.data_type_filters,
        }
    }
}

impl From<ConsoleScanResultsQueryRequest> for api::commands::scan_results::query::scan_results_query_request::ScanResultsQueryRequest {
    fn from(request: ConsoleScanResultsQueryRequest) -> Self {
        Self {
            page_index: request.page_index,
            data_type_filters: request.data_type_filters,
        }
    }
}

impl From<ConsoleScanResultsRefreshRequest> for api::commands::scan_results::refresh::scan_results_refresh_request::ScanResultsRefreshRequest {
    fn from(request: ConsoleScanResultsRefreshRequest) -> Self {
        Self {
            scan_result_refs: request.scan_result_refs,
        }
    }
}

impl From<ConsoleScanResultsDeleteRequest> for api::commands::scan_results::delete::scan_results_delete_request::ScanResultsDeleteRequest {
    fn from(request: ConsoleScanResultsDeleteRequest) -> Self {
        Self {
            scan_result_refs: request.scan_result_refs,
        }
    }
}

impl From<ConsoleScanResultsFreezeRequest> for api::commands::scan_results::freeze::scan_results_freeze_request::ScanResultsFreezeRequest {
    fn from(request: ConsoleScanResultsFreezeRequest) -> Self {
        Self {
            scan_result_refs: request.scan_result_refs,
            is_frozen: request.is_frozen,
        }
    }
}

impl From<ConsoleScanResultsSetPropertyRequest>
    for api::commands::scan_results::set_property::scan_results_set_property_request::ScanResultsSetPropertyRequest
{
    fn from(request: ConsoleScanResultsSetPropertyRequest) -> Self {
        Self {
            scan_result_refs: request.scan_result_refs,
            anonymous_value_string: request.anonymous_value_string,
            field_namespace: request.field_namespace,
        }
    }
}

#[derive(Clone, StructOpt, Debug)]
struct ConsoleStructScanCommand {
    #[structopt(flatten)]
    pub struct_scan_request: ConsoleStructScanRequest,
}

#[derive(Clone, StructOpt, Debug)]
struct ConsoleStructScanRequest {
    #[structopt(short = "v", long)]
    pub scan_value: Option<api::structures::data_values::anonymous_value_string::AnonymousValueString>,
    #[structopt(short = "d", long)]
    pub data_type_ids: Vec<String>,
    #[structopt(short = "c", long)]
    pub compare_type: api::structures::scanning::comparisons::scan_compare_type::ScanCompareType,
}

impl From<ConsoleStructScanCommand> for api::commands::struct_scan::struct_scan_command::StructScanCommand {
    fn from(command: ConsoleStructScanCommand) -> Self {
        Self {
            struct_scan_request: command.struct_scan_request.into(),
        }
    }
}

impl From<ConsoleStructScanRequest> for api::commands::struct_scan::struct_scan_request::StructScanRequest {
    fn from(request: ConsoleStructScanRequest) -> Self {
        Self {
            scan_value: request.scan_value,
            data_type_ids: request.data_type_ids,
            compare_type: request.compare_type,
        }
    }
}

#[derive(Clone, StructOpt, Debug)]
enum ConsoleTrackableTasksCommand {
    List {
        #[structopt(flatten)]
        trackable_tasks_list_request: ConsoleTrackableTasksListRequest,
    },
    Cancel {
        #[structopt(flatten)]
        trackable_tasks_cancel_request: ConsoleTrackableTasksCancelRequest,
    },
}

#[derive(Clone, StructOpt, Debug)]
struct ConsoleTrackableTasksListRequest {}

#[derive(Clone, StructOpt, Debug)]
struct ConsoleTrackableTasksCancelRequest {
    #[structopt(short = "t", long)]
    pub task_id: String,
}

impl From<ConsoleTrackableTasksCommand> for api::commands::trackable_tasks::trackable_tasks_command::TrackableTasksCommand {
    fn from(command: ConsoleTrackableTasksCommand) -> Self {
        match command {
            ConsoleTrackableTasksCommand::List { trackable_tasks_list_request } => Self::List {
                trackable_tasks_list_request: trackable_tasks_list_request.into(),
            },
            ConsoleTrackableTasksCommand::Cancel {
                trackable_tasks_cancel_request,
            } => Self::Cancel {
                trackable_tasks_cancel_request: trackable_tasks_cancel_request.into(),
            },
        }
    }
}

impl From<ConsoleTrackableTasksListRequest> for api::commands::trackable_tasks::list::trackable_tasks_list_request::TrackableTasksListRequest {
    fn from(_: ConsoleTrackableTasksListRequest) -> Self {
        Self {}
    }
}

impl From<ConsoleTrackableTasksCancelRequest> for api::commands::trackable_tasks::cancel::trackable_tasks_cancel_request::TrackableTasksCancelRequest {
    fn from(request: ConsoleTrackableTasksCancelRequest) -> Self {
        Self { task_id: request.task_id }
    }
}

#[derive(Clone, StructOpt, Debug)]
enum ConsoleSettingsCommand {
    General {
        #[structopt(flatten)]
        general_settings_command: ConsoleGeneralSettingsCommand,
    },
    Memory {
        #[structopt(flatten)]
        memory_settings_command: ConsoleMemorySettingsCommand,
    },
    Scan {
        #[structopt(flatten)]
        scan_settings_command: ConsoleScanSettingsCommand,
    },
}

#[derive(Clone, StructOpt, Debug)]
enum ConsoleGeneralSettingsCommand {
    List {
        #[structopt(flatten)]
        general_settings_list_request: ConsoleGeneralSettingsListRequest,
    },
    Set {
        #[structopt(flatten)]
        general_settings_set_request: ConsoleGeneralSettingsSetRequest,
    },
}

#[derive(Clone, StructOpt, Debug)]
struct ConsoleGeneralSettingsListRequest {}

#[derive(Clone, StructOpt, Debug, Default)]
struct ConsoleGeneralSettingsSetRequest {
    #[structopt(short = "r_delay", long)]
    pub engine_request_delay: Option<u64>,
}

#[derive(Clone, StructOpt, Debug)]
enum ConsoleMemorySettingsCommand {
    List {
        #[structopt(flatten)]
        memory_settings_list_request: ConsoleMemorySettingsListRequest,
    },
    Set {
        #[structopt(flatten)]
        memory_settings_set_request: ConsoleMemorySettingsSetRequest,
    },
}

#[derive(Clone, StructOpt, Debug)]
struct ConsoleMemorySettingsListRequest {}

#[derive(Clone, StructOpt, Debug, Default)]
struct ConsoleMemorySettingsSetRequest {
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
enum ConsoleScanSettingsCommand {
    List {
        #[structopt(flatten)]
        scan_settings_list_request: ConsoleScanSettingsListRequest,
    },
    Set {
        #[structopt(flatten)]
        scan_settings_set_request: ConsoleScanSettingsSetRequest,
    },
}

#[derive(Clone, StructOpt, Debug)]
struct ConsoleScanSettingsListRequest {}

#[derive(Clone, StructOpt, Debug, Default)]
struct ConsoleScanSettingsSetRequest {
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

impl From<ConsoleSettingsCommand> for api::commands::settings::settings_command::SettingsCommand {
    fn from(command: ConsoleSettingsCommand) -> Self {
        match command {
            ConsoleSettingsCommand::General { general_settings_command } => Self::General {
                general_settings_command: general_settings_command.into(),
            },
            ConsoleSettingsCommand::Memory { memory_settings_command } => Self::Memory {
                memory_settings_command: memory_settings_command.into(),
            },
            ConsoleSettingsCommand::Scan { scan_settings_command } => Self::Scan {
                scan_settings_command: scan_settings_command.into(),
            },
        }
    }
}

impl From<ConsoleGeneralSettingsCommand> for api::commands::settings::general::general_settings_command::GeneralSettingsCommand {
    fn from(command: ConsoleGeneralSettingsCommand) -> Self {
        match command {
            ConsoleGeneralSettingsCommand::List { general_settings_list_request } => Self::List {
                general_settings_list_request: general_settings_list_request.into(),
            },
            ConsoleGeneralSettingsCommand::Set { general_settings_set_request } => Self::Set {
                general_settings_set_request: general_settings_set_request.into(),
            },
        }
    }
}

impl From<ConsoleGeneralSettingsListRequest> for api::commands::settings::general::list::general_settings_list_request::GeneralSettingsListRequest {
    fn from(_: ConsoleGeneralSettingsListRequest) -> Self {
        Self {}
    }
}

impl From<ConsoleGeneralSettingsSetRequest> for api::commands::settings::general::set::general_settings_set_request::GeneralSettingsSetRequest {
    fn from(request: ConsoleGeneralSettingsSetRequest) -> Self {
        Self {
            engine_request_delay: request.engine_request_delay,
        }
    }
}

impl From<ConsoleMemorySettingsCommand> for api::commands::settings::memory::memory_settings_command::MemorySettingsCommand {
    fn from(command: ConsoleMemorySettingsCommand) -> Self {
        match command {
            ConsoleMemorySettingsCommand::List { memory_settings_list_request } => Self::List {
                memory_settings_list_request: memory_settings_list_request.into(),
            },
            ConsoleMemorySettingsCommand::Set { memory_settings_set_request } => Self::Set {
                memory_settings_set_request: memory_settings_set_request.into(),
            },
        }
    }
}

impl From<ConsoleMemorySettingsListRequest> for api::commands::settings::memory::list::memory_settings_list_request::MemorySettingsListRequest {
    fn from(_: ConsoleMemorySettingsListRequest) -> Self {
        Self {}
    }
}

impl From<ConsoleMemorySettingsSetRequest> for api::commands::settings::memory::set::memory_settings_set_request::MemorySettingsSetRequest {
    fn from(request: ConsoleMemorySettingsSetRequest) -> Self {
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

impl From<ConsoleScanSettingsCommand> for api::commands::settings::scan::scan_settings_command::ScanSettingsCommand {
    fn from(command: ConsoleScanSettingsCommand) -> Self {
        match command {
            ConsoleScanSettingsCommand::List { scan_settings_list_request } => Self::List {
                scan_settings_list_request: scan_settings_list_request.into(),
            },
            ConsoleScanSettingsCommand::Set { scan_settings_set_request } => Self::Set {
                scan_settings_set_request: scan_settings_set_request.into(),
            },
        }
    }
}

impl From<ConsoleScanSettingsListRequest> for api::commands::settings::scan::list::scan_settings_list_request::ScanSettingsListRequest {
    fn from(_: ConsoleScanSettingsListRequest) -> Self {
        Self {}
    }
}

impl From<ConsoleScanSettingsSetRequest> for api::commands::settings::scan::set::scan_settings_set_request::ScanSettingsSetRequest {
    fn from(request: ConsoleScanSettingsSetRequest) -> Self {
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

#[derive(Clone, StructOpt, Debug)]
enum ConsoleProjectCommand {
    List {
        #[structopt(flatten)]
        project_list_request: ConsoleProjectListRequest,
    },
    Open {
        #[structopt(flatten)]
        project_open_request: ConsoleProjectOpenRequest,
    },
    Close {
        #[structopt(flatten)]
        project_close_request: ConsoleProjectCloseRequest,
    },
    Create {
        #[structopt(flatten)]
        project_create_request: ConsoleProjectCreateRequest,
    },
    Delete {
        #[structopt(flatten)]
        project_delete_request: ConsoleProjectDeleteRequest,
    },
    Rename {
        #[structopt(flatten)]
        project_rename_request: ConsoleProjectRenameRequest,
    },
    Save {
        #[structopt(flatten)]
        project_save_request: ConsoleProjectSaveRequest,
    },
    Export {
        #[structopt(flatten)]
        project_export_request: ConsoleProjectExportRequest,
    },
}

#[derive(Clone, StructOpt, Debug)]
struct ConsoleProjectListRequest {}

#[derive(Clone, StructOpt, Debug)]
struct ConsoleProjectOpenRequest {
    #[structopt(short = "b", long)]
    pub open_file_browser: bool,
    #[structopt(short = "p", long)]
    pub project_directory_path: Option<PathBuf>,
    #[structopt(short = "n", long)]
    pub project_name: Option<String>,
}

#[derive(Clone, StructOpt, Debug)]
struct ConsoleProjectCloseRequest {}

#[derive(Clone, StructOpt, Debug)]
struct ConsoleProjectCreateRequest {
    #[structopt(short = "p", long)]
    pub project_directory_path: Option<PathBuf>,
    #[structopt(short = "n", long)]
    pub project_name: Option<String>,
}

#[derive(Clone, StructOpt, Debug)]
struct ConsoleProjectDeleteRequest {
    #[structopt(short = "p", long)]
    pub project_directory_path: Option<PathBuf>,
    #[structopt(short = "n", long)]
    pub project_name: Option<String>,
}

#[derive(Clone, StructOpt, Debug)]
struct ConsoleProjectRenameRequest {
    #[structopt(short = "p", long)]
    pub project_directory_path: PathBuf,
    #[structopt(short = "n", long)]
    pub new_project_name: String,
}

#[derive(Clone, StructOpt, Debug)]
struct ConsoleProjectSaveRequest {}

#[derive(Clone, StructOpt, Debug)]
struct ConsoleProjectExportRequest {
    #[structopt(short = "p", long)]
    pub project_directory_path: Option<PathBuf>,
    #[structopt(short = "n", long)]
    pub project_name: Option<String>,
    #[structopt(short = "o", long)]
    pub open_export_folder: bool,
}

impl From<ConsoleProjectCommand> for api::commands::project::project_command::ProjectCommand {
    fn from(command: ConsoleProjectCommand) -> Self {
        match command {
            ConsoleProjectCommand::List { project_list_request } => Self::List {
                project_list_request: project_list_request.into(),
            },
            ConsoleProjectCommand::Open { project_open_request } => Self::Open {
                project_open_request: project_open_request.into(),
            },
            ConsoleProjectCommand::Close { project_close_request } => Self::Close {
                project_close_request: project_close_request.into(),
            },
            ConsoleProjectCommand::Create { project_create_request } => Self::Create {
                project_create_request: project_create_request.into(),
            },
            ConsoleProjectCommand::Delete { project_delete_request } => Self::Delete {
                project_delete_request: project_delete_request.into(),
            },
            ConsoleProjectCommand::Rename { project_rename_request } => Self::Rename {
                project_rename_request: project_rename_request.into(),
            },
            ConsoleProjectCommand::Save { project_save_request } => Self::Save {
                project_save_request: project_save_request.into(),
            },
            ConsoleProjectCommand::Export { project_export_request } => Self::Export {
                project_export_request: project_export_request.into(),
            },
        }
    }
}

impl From<ConsoleProjectListRequest> for api::commands::project::list::project_list_request::ProjectListRequest {
    fn from(_: ConsoleProjectListRequest) -> Self {
        Self {}
    }
}

impl From<ConsoleProjectOpenRequest> for api::commands::project::open::project_open_request::ProjectOpenRequest {
    fn from(request: ConsoleProjectOpenRequest) -> Self {
        Self {
            open_file_browser: request.open_file_browser,
            project_directory_path: request.project_directory_path,
            project_name: request.project_name,
        }
    }
}

impl From<ConsoleProjectCloseRequest> for api::commands::project::close::project_close_request::ProjectCloseRequest {
    fn from(_: ConsoleProjectCloseRequest) -> Self {
        Self {}
    }
}

impl From<ConsoleProjectCreateRequest> for api::commands::project::create::project_create_request::ProjectCreateRequest {
    fn from(request: ConsoleProjectCreateRequest) -> Self {
        Self {
            project_directory_path: request.project_directory_path,
            project_name: request.project_name,
        }
    }
}

impl From<ConsoleProjectDeleteRequest> for api::commands::project::delete::project_delete_request::ProjectDeleteRequest {
    fn from(request: ConsoleProjectDeleteRequest) -> Self {
        Self {
            project_directory_path: request.project_directory_path,
            project_name: request.project_name,
        }
    }
}

impl From<ConsoleProjectRenameRequest> for api::commands::project::rename::project_rename_request::ProjectRenameRequest {
    fn from(request: ConsoleProjectRenameRequest) -> Self {
        Self {
            project_directory_path: request.project_directory_path,
            new_project_name: request.new_project_name,
        }
    }
}

impl From<ConsoleProjectSaveRequest> for api::commands::project::save::project_save_request::ProjectSaveRequest {
    fn from(_: ConsoleProjectSaveRequest) -> Self {
        Self {}
    }
}

impl From<ConsoleProjectExportRequest> for api::commands::project::export::project_export_request::ProjectExportRequest {
    fn from(request: ConsoleProjectExportRequest) -> Self {
        Self {
            project_directory_path: request.project_directory_path,
            project_name: request.project_name,
            open_export_folder: request.open_export_folder,
        }
    }
}

#[derive(Clone, StructOpt, Debug)]
enum ConsoleProjectItemsCommand {
    Add {
        #[structopt(flatten)]
        project_items_add_request: ConsoleProjectItemsAddRequest,
    },
    Activate {
        #[structopt(flatten)]
        project_items_activate_request: ConsoleProjectItemsActivateRequest,
    },
    Create {
        #[structopt(flatten)]
        project_items_create_request: ConsoleProjectItemsCreateRequest,
    },
    Delete {
        #[structopt(flatten)]
        project_items_delete_request: ConsoleProjectItemsDeleteRequest,
    },
    Duplicate {
        #[structopt(flatten)]
        project_items_duplicate_request: ConsoleProjectItemsDuplicateRequest,
    },
    List {
        #[structopt(flatten)]
        project_items_list_request: ConsoleProjectItemsListRequest,
    },
    Move {
        #[structopt(flatten)]
        project_items_move_request: ConsoleProjectItemsMoveRequest,
    },
    PromoteSymbol {
        #[structopt(flatten)]
        project_items_promote_symbol_request: ConsoleProjectItemsPromoteSymbolRequest,
    },
    Rename {
        #[structopt(flatten)]
        project_items_rename_request: ConsoleProjectItemsRenameRequest,
    },
    Reorder {
        #[structopt(flatten)]
        project_items_reorder_request: ConsoleProjectItemsReorderRequest,
    },
    StripSymbol {
        #[structopt(flatten)]
        project_items_strip_symbol_request: ConsoleProjectItemsStripSymbolRequest,
    },
    UpdateDetails {
        #[structopt(flatten)]
        project_items_update_details_request: ConsoleProjectItemsUpdateDetailsRequest,
    },
    WriteValue {
        #[structopt(flatten)]
        project_items_write_value_request: ConsoleProjectItemsWriteValueRequest,
    },
}

#[derive(Clone, StructOpt, Debug)]
struct ConsoleProjectItemsActivateRequest {
    #[structopt(short = "p", long)]
    pub project_item_paths: Vec<String>,
    #[structopt(short = "a", long)]
    pub is_activated: bool,
}

#[derive(Clone, StructOpt, Debug)]
struct ConsoleProjectItemsAddRequest {
    #[structopt(short = "s", long)]
    pub scan_result_refs: Vec<api::structures::scan_results::scan_result_ref::ScanResultRef>,
    #[structopt(long)]
    pub target_directory_path: Option<PathBuf>,
}

#[derive(Clone, StructOpt, Debug)]
struct ConsoleProjectItemsCreateRequest {
    #[structopt(short = "p", long)]
    pub parent_directory_path: PathBuf,
    #[structopt(short = "n", long)]
    pub project_item_name: String,
    #[structopt(long)]
    pub is_directory: bool,
}

#[derive(Clone, StructOpt, Debug)]
struct ConsoleProjectItemsDeleteRequest {
    #[structopt(short = "p", long)]
    pub project_item_paths: Vec<PathBuf>,
}

#[derive(Clone, StructOpt, Debug)]
struct ConsoleProjectItemsDuplicateRequest {
    #[structopt(short = "p", long)]
    pub project_item_paths: Vec<PathBuf>,
    #[structopt(short = "t", long)]
    pub target_directory_path: PathBuf,
}

#[derive(Clone, Default, StructOpt, Debug)]
struct ConsoleProjectItemsListRequest {}

#[derive(Clone, StructOpt, Debug)]
struct ConsoleProjectItemsMoveRequest {
    #[structopt(short = "p", long)]
    pub project_item_paths: Vec<PathBuf>,
    #[structopt(short = "t", long)]
    pub target_directory_path: PathBuf,
}

#[derive(Clone, Default, StructOpt, Debug)]
struct ConsoleProjectItemsPromoteSymbolRequest {
    #[structopt(short = "p", long = "project-item-path", parse(from_os_str))]
    pub project_item_paths: Vec<PathBuf>,
    #[structopt(long = "overwrite-conflicting-symbols")]
    pub overwrite_conflicting_symbols: bool,
}

#[derive(Clone, StructOpt, Debug)]
struct ConsoleProjectItemsRenameRequest {
    #[structopt(short = "p", long)]
    pub project_item_path: PathBuf,
    #[structopt(short = "n", long)]
    pub project_item_name: String,
}

#[derive(Clone, StructOpt, Debug)]
struct ConsoleProjectItemsReorderRequest {
    #[structopt(short = "p", long)]
    pub project_item_paths: Vec<PathBuf>,
}

#[derive(Clone, StructOpt, Debug)]
struct ConsoleProjectItemsStripSymbolRequest {
    #[structopt(short = "p", long = "project-item-path", parse(from_os_str))]
    pub project_item_paths: Vec<PathBuf>,
}

#[derive(Clone, Default, StructOpt, Debug)]
struct ConsoleProjectItemsUpdateDetailsRequest {
    #[structopt(short = "p", long = "project-item-path", parse(from_os_str))]
    pub project_item_paths: Vec<PathBuf>,
    #[structopt(long = "property")]
    pub property_name: Option<String>,
    #[structopt(long = "address-target-property")]
    pub address_target_property_name: Option<String>,
    #[structopt(short = "v", long = "value")]
    pub anonymous_value_string: Option<api::structures::data_values::anonymous_value_string::AnonymousValueString>,
}

#[derive(Clone, StructOpt, Debug)]
struct ConsoleProjectItemsWriteValueRequest {
    #[structopt(short = "p", long = "project-item-path", parse(from_os_str))]
    pub project_item_path: PathBuf,
    #[structopt(long = "field", default_value = "value")]
    pub field_name: String,
    #[structopt(short = "v", long)]
    pub anonymous_value_string: api::structures::data_values::anonymous_value_string::AnonymousValueString,
}

impl From<ConsoleProjectItemsCommand> for api::commands::project_items::project_items_command::ProjectItemsCommand {
    fn from(command: ConsoleProjectItemsCommand) -> Self {
        match command {
            ConsoleProjectItemsCommand::Add { project_items_add_request } => Self::Add {
                project_items_add_request: project_items_add_request.into(),
            },
            ConsoleProjectItemsCommand::Activate {
                project_items_activate_request,
            } => Self::Activate {
                project_items_activate_request: project_items_activate_request.into(),
            },
            ConsoleProjectItemsCommand::Create { project_items_create_request } => Self::Create {
                project_items_create_request: project_items_create_request.into(),
            },
            ConsoleProjectItemsCommand::Delete { project_items_delete_request } => Self::Delete {
                project_items_delete_request: project_items_delete_request.into(),
            },
            ConsoleProjectItemsCommand::Duplicate {
                project_items_duplicate_request,
            } => Self::Duplicate {
                project_items_duplicate_request: project_items_duplicate_request.into(),
            },
            ConsoleProjectItemsCommand::List { project_items_list_request } => Self::List {
                project_items_list_request: project_items_list_request.into(),
            },
            ConsoleProjectItemsCommand::Move { project_items_move_request } => Self::Move {
                project_items_move_request: project_items_move_request.into(),
            },
            ConsoleProjectItemsCommand::PromoteSymbol {
                project_items_promote_symbol_request,
            } => Self::PromoteSymbol {
                project_items_promote_symbol_request: project_items_promote_symbol_request.into(),
            },
            ConsoleProjectItemsCommand::Rename { project_items_rename_request } => Self::Rename {
                project_items_rename_request: project_items_rename_request.into(),
            },
            ConsoleProjectItemsCommand::Reorder { project_items_reorder_request } => Self::Reorder {
                project_items_reorder_request: project_items_reorder_request.into(),
            },
            ConsoleProjectItemsCommand::StripSymbol {
                project_items_strip_symbol_request,
            } => Self::StripSymbol {
                project_items_strip_symbol_request: project_items_strip_symbol_request.into(),
            },
            ConsoleProjectItemsCommand::UpdateDetails {
                project_items_update_details_request,
            } => Self::UpdateDetails {
                project_items_update_details_request: project_items_update_details_request.into(),
            },
            ConsoleProjectItemsCommand::WriteValue {
                project_items_write_value_request,
            } => Self::WriteValue {
                project_items_write_value_request: project_items_write_value_request.into(),
            },
        }
    }
}

impl From<ConsoleProjectItemsActivateRequest> for api::commands::project_items::activate::project_items_activate_request::ProjectItemsActivateRequest {
    fn from(request: ConsoleProjectItemsActivateRequest) -> Self {
        Self {
            project_item_paths: request.project_item_paths,
            is_activated: request.is_activated,
        }
    }
}

impl From<ConsoleProjectItemsAddRequest> for api::commands::project_items::add::project_items_add_request::ProjectItemsAddRequest {
    fn from(request: ConsoleProjectItemsAddRequest) -> Self {
        Self {
            scan_result_refs: request.scan_result_refs,
            target_directory_path: request.target_directory_path,
        }
    }
}

impl From<ConsoleProjectItemsCreateRequest> for api::commands::project_items::create::project_items_create_request::ProjectItemsCreateRequest {
    fn from(request: ConsoleProjectItemsCreateRequest) -> Self {
        Self {
            parent_directory_path: request.parent_directory_path,
            project_item_name: request.project_item_name,
            is_directory: request.is_directory,
            address: None,
            module_name: None,
            data_type_id: None,
            pointer_offsets: None,
        }
    }
}

impl From<ConsoleProjectItemsDeleteRequest> for api::commands::project_items::delete::project_items_delete_request::ProjectItemsDeleteRequest {
    fn from(request: ConsoleProjectItemsDeleteRequest) -> Self {
        Self {
            project_item_paths: request.project_item_paths,
        }
    }
}

impl From<ConsoleProjectItemsDuplicateRequest> for api::commands::project_items::duplicate::project_items_duplicate_request::ProjectItemsDuplicateRequest {
    fn from(request: ConsoleProjectItemsDuplicateRequest) -> Self {
        Self {
            project_item_paths: request.project_item_paths,
            target_directory_path: request.target_directory_path,
        }
    }
}

impl From<ConsoleProjectItemsListRequest> for api::commands::project_items::list::project_items_list_request::ProjectItemsListRequest {
    fn from(_: ConsoleProjectItemsListRequest) -> Self {
        Self {
            preview_project_item_paths: None,
        }
    }
}

impl From<ConsoleProjectItemsMoveRequest> for api::commands::project_items::move_item::project_items_move_request::ProjectItemsMoveRequest {
    fn from(request: ConsoleProjectItemsMoveRequest) -> Self {
        Self {
            project_item_paths: request.project_item_paths,
            target_directory_path: request.target_directory_path,
        }
    }
}

impl From<ConsoleProjectItemsPromoteSymbolRequest>
    for api::commands::project_items::promote_symbol::project_items_promote_symbol_request::ProjectItemsPromoteSymbolRequest
{
    fn from(request: ConsoleProjectItemsPromoteSymbolRequest) -> Self {
        Self {
            project_item_paths: request.project_item_paths,
            overwrite_conflicting_symbols: request.overwrite_conflicting_symbols,
        }
    }
}

impl From<ConsoleProjectItemsRenameRequest> for api::commands::project_items::rename::project_items_rename_request::ProjectItemsRenameRequest {
    fn from(request: ConsoleProjectItemsRenameRequest) -> Self {
        Self {
            project_item_path: request.project_item_path,
            project_item_name: request.project_item_name,
        }
    }
}

impl From<ConsoleProjectItemsReorderRequest> for api::commands::project_items::reorder::project_items_reorder_request::ProjectItemsReorderRequest {
    fn from(request: ConsoleProjectItemsReorderRequest) -> Self {
        Self {
            project_item_paths: request.project_item_paths,
        }
    }
}

impl From<ConsoleProjectItemsStripSymbolRequest>
    for api::commands::project_items::strip_symbol::project_items_strip_symbol_request::ProjectItemsStripSymbolRequest
{
    fn from(request: ConsoleProjectItemsStripSymbolRequest) -> Self {
        Self {
            project_item_paths: request.project_item_paths,
        }
    }
}

impl From<ConsoleProjectItemsUpdateDetailsRequest>
    for api::commands::project_items::update_details::project_items_update_details_request::ProjectItemsUpdateDetailsRequest
{
    fn from(request: ConsoleProjectItemsUpdateDetailsRequest) -> Self {
        Self {
            project_item_paths: request.project_item_paths,
            property_name: request.property_name,
            address_target_property_name: request.address_target_property_name,
            anonymous_value_string: request.anonymous_value_string,
            details_field_source: api::structures::details::DetailsFieldSource::Unknown,
            details_value: api::structures::details::DetailsValue::Empty,
        }
    }
}

impl From<ConsoleProjectItemsWriteValueRequest>
    for api::commands::project_items::write_value::project_items_write_value_request::ProjectItemsWriteValueRequest
{
    fn from(request: ConsoleProjectItemsWriteValueRequest) -> Self {
        Self {
            project_item_path: request.project_item_path,
            field_name: request.field_name,
            anonymous_value_string: request.anonymous_value_string,
        }
    }
}

#[derive(Clone, StructOpt, Debug)]
enum ConsoleProjectSymbolsCommand {
    Create {
        #[structopt(flatten)]
        project_symbols_create_request: ConsoleProjectSymbolsCreateRequest,
    },
    CreateModule {
        #[structopt(flatten)]
        project_symbols_create_module_request: ConsoleProjectSymbolsCreateModuleRequest,
    },
    Delete {
        #[structopt(flatten)]
        project_symbols_delete_request: ConsoleProjectSymbolsDeleteRequest,
    },
    DeleteLayout {
        #[structopt(flatten)]
        project_symbols_delete_layout_request: ConsoleProjectSymbolsDeleteLayoutRequest,
    },
    DeleteResolver {
        #[structopt(flatten)]
        project_symbols_delete_resolver_request: ConsoleProjectSymbolsDeleteResolverRequest,
    },
    List {
        #[structopt(flatten)]
        project_symbols_list_request: ConsoleProjectSymbolsListRequest,
    },
    Rename {
        #[structopt(flatten)]
        project_symbols_rename_request: ConsoleProjectSymbolsRenameRequest,
    },
    RenameModule {
        #[structopt(flatten)]
        project_symbols_rename_module_request: ConsoleProjectSymbolsRenameModuleRequest,
    },
    Update {
        #[structopt(flatten)]
        project_symbols_update_request: ConsoleProjectSymbolsUpdateRequest,
    },
    UpsertLayout {
        #[structopt(flatten)]
        project_symbols_upsert_layout_request: ConsoleProjectSymbolsUpsertLayoutRequest,
    },
    UpsertResolver {
        #[structopt(flatten)]
        project_symbols_upsert_resolver_request: ConsoleProjectSymbolsUpsertResolverRequest,
    },
    WriteValue {
        #[structopt(flatten)]
        project_symbols_write_value_request: ConsoleProjectSymbolsWriteValueRequest,
    },
}

#[derive(Clone, Default, StructOpt, Debug)]
struct ConsoleProjectSymbolsCreateRequest {
    #[structopt(short = "n", long = "name")]
    pub display_name: String,
    #[structopt(short = "t", long = "type")]
    pub struct_layout_id: String,
    #[structopt(short = "a", long = "address")]
    pub address: Option<u64>,
    #[structopt(short = "m", long = "module")]
    pub module_name: Option<String>,
    #[structopt(short = "o", long = "offset")]
    pub offset: Option<u64>,
}

#[derive(Clone, Default, StructOpt, Debug)]
struct ConsoleProjectSymbolsCreateModuleRequest {
    #[structopt(short = "m", long = "module")]
    pub module_name: String,
    #[structopt(short = "s", long = "size")]
    pub size: u64,
}

#[derive(Clone, Default, StructOpt, Debug)]
struct ConsoleProjectSymbolsDeleteRequest {
    #[structopt(short = "k", long = "key")]
    pub symbol_locator_keys: Vec<String>,
    #[structopt(short = "m", long = "module")]
    pub module_names: Vec<String>,
}

#[derive(Clone, Default, StructOpt, Debug)]
struct ConsoleProjectSymbolsDeleteLayoutRequest {
    #[structopt(short = "i", long = "id")]
    pub struct_layout_id: String,
    #[structopt(long = "replacement-type", default_value = "u8")]
    pub replacement_data_type_id: String,
}

#[derive(Clone, Default, StructOpt, Debug)]
struct ConsoleProjectSymbolsDeleteResolverRequest {
    #[structopt(short = "i", long = "id")]
    pub resolver_id: String,
}

#[derive(Clone, Default, StructOpt, Debug)]
struct ConsoleProjectSymbolsListRequest {}

#[derive(Clone, Default, StructOpt, Debug)]
struct ConsoleProjectSymbolsRenameRequest {
    #[structopt(short = "k", long = "key")]
    pub symbol_locator_key: String,
    #[structopt(short = "n", long = "name")]
    pub display_name: String,
}

#[derive(Clone, Default, StructOpt, Debug)]
struct ConsoleProjectSymbolsRenameModuleRequest {
    #[structopt(short = "m", long = "module")]
    pub module_name: String,
    #[structopt(short = "n", long = "new-name")]
    pub new_module_name: String,
}

#[derive(Clone, Default, StructOpt, Debug)]
struct ConsoleProjectSymbolsUpdateRequest {
    #[structopt(short = "k", long = "key")]
    pub symbol_locator_key: String,
    #[structopt(long = "name")]
    pub display_name: Option<String>,
    #[structopt(long = "type")]
    pub struct_layout_id: Option<String>,
}

#[derive(Clone, Default, StructOpt, Debug)]
struct ConsoleProjectSymbolsUpsertLayoutRequest {
    #[structopt(long = "original-id")]
    pub original_struct_layout_id: Option<String>,
    #[structopt(short = "i", long = "id")]
    pub struct_layout_id: String,
    #[structopt(short = "k", long = "kind", default_value = "struct")]
    pub layout_kind: String,
    #[structopt(short = "s", long = "size")]
    pub size_in_bytes: Option<u64>,
    #[structopt(short = "f", long = "field")]
    pub field_definitions: Vec<String>,
}

#[derive(Clone, Default, StructOpt, Debug)]
struct ConsoleProjectSymbolsUpsertResolverRequest {
    #[structopt(long = "original-id")]
    pub original_resolver_id: Option<String>,
    #[structopt(short = "i", long = "id")]
    pub resolver_id: String,
    #[structopt(short = "d", long = "definition-json")]
    pub resolver_definition_json: String,
}

#[derive(Clone, StructOpt, Debug)]
struct ConsoleProjectSymbolsWriteValueRequest {
    #[structopt(long = "address")]
    pub address: u64,
    #[structopt(long = "module", default_value = "")]
    pub module_name: String,
    #[structopt(long = "type")]
    pub symbol_type_id: String,
    #[structopt(long = "container", default_value = "")]
    pub container_type: api::structures::data_values::container_type::ContainerType,
    #[structopt(long = "field")]
    pub field_name: String,
    #[structopt(short = "v", long)]
    pub anonymous_value_string: api::structures::data_values::anonymous_value_string::AnonymousValueString,
}

impl From<ConsoleProjectSymbolsCommand> for api::commands::project_symbols::project_symbols_command::ProjectSymbolsCommand {
    fn from(command: ConsoleProjectSymbolsCommand) -> Self {
        match command {
            ConsoleProjectSymbolsCommand::Create {
                project_symbols_create_request,
            } => Self::Create {
                project_symbols_create_request: project_symbols_create_request.into(),
            },
            ConsoleProjectSymbolsCommand::CreateModule {
                project_symbols_create_module_request,
            } => Self::CreateModule {
                project_symbols_create_module_request: project_symbols_create_module_request.into(),
            },
            ConsoleProjectSymbolsCommand::Delete {
                project_symbols_delete_request,
            } => Self::Delete {
                project_symbols_delete_request: project_symbols_delete_request.into(),
            },
            ConsoleProjectSymbolsCommand::DeleteLayout {
                project_symbols_delete_layout_request,
            } => Self::DeleteLayout {
                project_symbols_delete_layout_request: project_symbols_delete_layout_request.into(),
            },
            ConsoleProjectSymbolsCommand::DeleteResolver {
                project_symbols_delete_resolver_request,
            } => Self::DeleteResolver {
                project_symbols_delete_resolver_request: project_symbols_delete_resolver_request.into(),
            },
            ConsoleProjectSymbolsCommand::List { project_symbols_list_request } => Self::List {
                project_symbols_list_request: project_symbols_list_request.into(),
            },
            ConsoleProjectSymbolsCommand::Rename {
                project_symbols_rename_request,
            } => Self::Rename {
                project_symbols_rename_request: project_symbols_rename_request.into(),
            },
            ConsoleProjectSymbolsCommand::RenameModule {
                project_symbols_rename_module_request,
            } => Self::RenameModule {
                project_symbols_rename_module_request: project_symbols_rename_module_request.into(),
            },
            ConsoleProjectSymbolsCommand::Update {
                project_symbols_update_request,
            } => Self::Update {
                project_symbols_update_request: project_symbols_update_request.into(),
            },
            ConsoleProjectSymbolsCommand::UpsertLayout {
                project_symbols_upsert_layout_request,
            } => Self::UpsertLayout {
                project_symbols_upsert_layout_request: project_symbols_upsert_layout_request.into(),
            },
            ConsoleProjectSymbolsCommand::UpsertResolver {
                project_symbols_upsert_resolver_request,
            } => Self::UpsertResolver {
                project_symbols_upsert_resolver_request: project_symbols_upsert_resolver_request.into(),
            },
            ConsoleProjectSymbolsCommand::WriteValue {
                project_symbols_write_value_request,
            } => Self::WriteValue {
                project_symbols_write_value_request: project_symbols_write_value_request.into(),
            },
        }
    }
}

impl From<ConsoleProjectSymbolsCreateRequest> for api::commands::project_symbols::create::project_symbols_create_request::ProjectSymbolsCreateRequest {
    fn from(request: ConsoleProjectSymbolsCreateRequest) -> Self {
        Self {
            display_name: request.display_name,
            struct_layout_id: request.struct_layout_id,
            address: request.address,
            module_name: request.module_name,
            offset: request.offset,
            metadata: BTreeMap::new(),
        }
    }
}

impl From<ConsoleProjectSymbolsCreateModuleRequest>
    for api::commands::project_symbols::create_module::project_symbols_create_module_request::ProjectSymbolsCreateModuleRequest
{
    fn from(request: ConsoleProjectSymbolsCreateModuleRequest) -> Self {
        Self {
            module_name: request.module_name,
            size: request.size,
        }
    }
}

impl From<ConsoleProjectSymbolsDeleteRequest> for api::commands::project_symbols::delete::project_symbols_delete_request::ProjectSymbolsDeleteRequest {
    fn from(request: ConsoleProjectSymbolsDeleteRequest) -> Self {
        Self {
            symbol_locator_keys: request.symbol_locator_keys,
            module_names: request.module_names,
            module_ranges: Vec::new(),
        }
    }
}

impl From<ConsoleProjectSymbolsDeleteLayoutRequest>
    for api::commands::project_symbols::delete_layout::project_symbols_delete_layout_request::ProjectSymbolsDeleteLayoutRequest
{
    fn from(request: ConsoleProjectSymbolsDeleteLayoutRequest) -> Self {
        Self {
            struct_layout_id: request.struct_layout_id,
            replacement_data_type_id: request.replacement_data_type_id,
        }
    }
}

impl From<ConsoleProjectSymbolsDeleteResolverRequest>
    for api::commands::project_symbols::delete_resolver::project_symbols_delete_resolver_request::ProjectSymbolsDeleteResolverRequest
{
    fn from(request: ConsoleProjectSymbolsDeleteResolverRequest) -> Self {
        Self {
            resolver_id: request.resolver_id,
        }
    }
}

impl From<ConsoleProjectSymbolsListRequest> for api::commands::project_symbols::list::project_symbols_list_request::ProjectSymbolsListRequest {
    fn from(_: ConsoleProjectSymbolsListRequest) -> Self {
        Self {}
    }
}

impl From<ConsoleProjectSymbolsRenameRequest> for api::commands::project_symbols::rename::project_symbols_rename_request::ProjectSymbolsRenameRequest {
    fn from(request: ConsoleProjectSymbolsRenameRequest) -> Self {
        Self {
            symbol_locator_key: request.symbol_locator_key,
            display_name: request.display_name,
        }
    }
}

impl From<ConsoleProjectSymbolsRenameModuleRequest>
    for api::commands::project_symbols::rename_module::project_symbols_rename_module_request::ProjectSymbolsRenameModuleRequest
{
    fn from(request: ConsoleProjectSymbolsRenameModuleRequest) -> Self {
        Self {
            module_name: request.module_name,
            new_module_name: request.new_module_name,
        }
    }
}

impl From<ConsoleProjectSymbolsUpdateRequest> for api::commands::project_symbols::update::project_symbols_update_request::ProjectSymbolsUpdateRequest {
    fn from(request: ConsoleProjectSymbolsUpdateRequest) -> Self {
        Self {
            symbol_locator_key: request.symbol_locator_key,
            display_name: request.display_name,
            struct_layout_id: request.struct_layout_id,
        }
    }
}

impl From<ConsoleProjectSymbolsUpsertLayoutRequest>
    for api::commands::project_symbols::upsert_layout::project_symbols_upsert_layout_request::ProjectSymbolsUpsertLayoutRequest
{
    fn from(request: ConsoleProjectSymbolsUpsertLayoutRequest) -> Self {
        Self {
            original_struct_layout_id: request.original_struct_layout_id,
            struct_layout_id: request.struct_layout_id,
            layout_kind: request.layout_kind,
            size_in_bytes: request.size_in_bytes,
            field_definitions: request.field_definitions,
        }
    }
}

impl From<ConsoleProjectSymbolsUpsertResolverRequest>
    for api::commands::project_symbols::upsert_resolver::project_symbols_upsert_resolver_request::ProjectSymbolsUpsertResolverRequest
{
    fn from(request: ConsoleProjectSymbolsUpsertResolverRequest) -> Self {
        Self {
            original_resolver_id: request.original_resolver_id,
            resolver_id: request.resolver_id,
            resolver_definition_json: request.resolver_definition_json,
        }
    }
}

impl From<ConsoleProjectSymbolsWriteValueRequest>
    for api::commands::project_symbols::write_value::project_symbols_write_value_request::ProjectSymbolsWriteValueRequest
{
    fn from(request: ConsoleProjectSymbolsWriteValueRequest) -> Self {
        Self {
            address: request.address,
            module_name: request.module_name,
            symbol_type_id: request.symbol_type_id,
            container_type: request.container_type,
            field_name: request.field_name,
            anonymous_value_string: request.anonymous_value_string,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use api::commands::process::process_command::ProcessCommand;
    use api::commands::project::project_command::ProjectCommand;

    #[test]
    fn parse_text_command_routes_privileged_namespace_directly() {
        let parsed_command = parse_text_command(["squalr-cli", "process", "list"]).expect("Expected process list to parse.");

        assert!(matches!(
            parsed_command,
            TextCommand::Privileged(api::commands::privileged_command::PrivilegedCommand::Process(ProcessCommand::List { .. }))
        ));
    }

    #[test]
    fn parse_text_command_routes_unprivileged_namespace_directly() {
        let parsed_command = parse_text_command(["squalr-cli", "project", "list"]).expect("Expected project list to parse.");

        assert!(matches!(
            parsed_command,
            TextCommand::Unprivileged(api::commands::unprivileged_command::UnprivilegedCommand::Project(ProjectCommand::List { .. }))
        ));
    }

    #[test]
    fn parse_command_line_handles_shell_words_and_project_aliases() {
        let parsed_command = parse_command_line("p create --project-name 'quoted name'").expect("Expected project create alias to parse.");

        assert!(matches!(
            parsed_command,
            TextCommand::Unprivileged(api::commands::unprivileged_command::UnprivilegedCommand::Project(ProjectCommand::Create { .. }))
        ));
    }

    #[test]
    fn parse_command_line_with_program_name_uses_caller_program_name_in_help() {
        let parse_error = parse_command_line_with_program_name("process open unexpected", "squalr-gui").expect_err("Expected parse failure.");

        assert!(parse_error.to_string().contains("squalr-gui process open"));
    }

    #[test]
    fn prompt_command_line_omits_program_name_from_usage() {
        let parse_error = parse_prompt_command_line("process open unexpected").expect_err("Expected parse failure.");
        let prompt_error_message = match parse_error {
            TextCommandParseError::Command(error) => format_prompt_command_error(&error),
            error => error.to_string(),
        };

        assert!(prompt_error_message.contains("process open"));
        assert!(!prompt_error_message.contains("squalr process open"));
        assert!(!prompt_error_message.contains("For more information try"));
    }

    #[test]
    fn prompt_command_error_summary_keeps_usage_without_full_help_footer() {
        let parse_error = parse_prompt_command_line("process open unexpected").expect_err("Expected parse failure.");
        let TextCommandParseError::Command(parse_error) = parse_error else {
            panic!("Expected clap parse error.");
        };

        let prompt_error_message = format_prompt_command_error(&parse_error);

        assert!(prompt_error_message.starts_with("error:"));
        assert!(prompt_error_message.contains("Usage: process open"));
        assert!(!prompt_error_message.contains("USAGE:"));
        assert!(!prompt_error_message.contains("For more information try"));
    }

    #[test]
    fn specific_privileged_parser_rejects_unprivileged_commands() {
        let parse_error = parse_privileged_command(["squalr-cli", "project", "list"]).expect_err("Expected unprivileged command to be rejected.");

        assert!(matches!(parse_error.kind, clap::ErrorKind::InvalidSubcommand));
    }

    #[test]
    fn specific_unprivileged_parser_rejects_privileged_commands() {
        let parse_error = parse_unprivileged_command(["squalr-cli", "process", "list"]).expect_err("Expected privileged command to be rejected.");

        assert!(matches!(parse_error.kind, clap::ErrorKind::InvalidSubcommand));
    }
}
