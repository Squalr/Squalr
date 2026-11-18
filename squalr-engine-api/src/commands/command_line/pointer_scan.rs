use crate as api;
use structopt::StructOpt;

#[derive(Clone, StructOpt, Debug)]
pub(crate) enum CommandLinePointerScanCommand {
    Start {
        #[structopt(flatten)]
        pointer_scan_start_request: CommandLinePointerScanStartRequest,
    },
    Reset {
        #[structopt(flatten)]
        pointer_scan_reset_request: CommandLinePointerScanResetRequest,
    },
    Summary {
        #[structopt(flatten)]
        pointer_scan_summary_request: CommandLinePointerScanSummaryRequest,
    },
    Expand {
        #[structopt(flatten)]
        pointer_scan_expand_request: CommandLinePointerScanExpandRequest,
    },
    Validate {
        #[structopt(flatten)]
        pointer_scan_validate_request: CommandLinePointerScanValidateRequest,
    },
}

#[derive(Clone, Debug, Default, StructOpt, PartialEq)]
pub(crate) struct CommandLinePointerScanTargetRequest {
    #[structopt(long = "target-address")]
    pub target_address: Option<api::structures::data_values::anonymous_value_string::AnonymousValueString>,
    #[structopt(long = "target-value")]
    pub target_value: Option<api::structures::data_values::anonymous_value_string::AnonymousValueString>,
    #[structopt(long = "target-data-type")]
    pub target_data_type: Option<api::structures::data_types::data_type_ref::DataTypeRef>,
}

#[derive(Clone, StructOpt, Debug)]
pub(crate) struct CommandLinePointerScanStartRequest {
    #[structopt(flatten)]
    pub target: CommandLinePointerScanTargetRequest,
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
pub(crate) struct CommandLinePointerScanResetRequest {}

#[derive(Clone, StructOpt, Debug, Default)]
pub(crate) struct CommandLinePointerScanSummaryRequest {
    #[structopt(short = "i", long)]
    pub session_id: Option<u64>,
}

#[derive(Clone, StructOpt, Debug)]
pub(crate) struct CommandLinePointerScanExpandRequest {
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
pub(crate) struct CommandLinePointerScanValidateRequest {
    #[structopt(short = "i", long)]
    pub session_id: u64,
    #[structopt(flatten)]
    pub target: CommandLinePointerScanTargetRequest,
}

impl From<CommandLinePointerScanTargetRequest> for api::structures::pointer_scans::pointer_scan_target_request::PointerScanTargetRequest {
    fn from(request: CommandLinePointerScanTargetRequest) -> Self {
        Self {
            target_address: request.target_address,
            target_value: request.target_value,
            target_data_type_ref: request.target_data_type,
        }
    }
}

impl From<CommandLinePointerScanCommand> for api::commands::pointer_scan::pointer_scan_command::PointerScanCommand {
    fn from(command: CommandLinePointerScanCommand) -> Self {
        match command {
            CommandLinePointerScanCommand::Start { pointer_scan_start_request } => Self::Start {
                pointer_scan_start_request: pointer_scan_start_request.into(),
            },
            CommandLinePointerScanCommand::Reset { pointer_scan_reset_request } => Self::Reset {
                pointer_scan_reset_request: pointer_scan_reset_request.into(),
            },
            CommandLinePointerScanCommand::Summary { pointer_scan_summary_request } => Self::Summary {
                pointer_scan_summary_request: pointer_scan_summary_request.into(),
            },
            CommandLinePointerScanCommand::Expand { pointer_scan_expand_request } => Self::Expand {
                pointer_scan_expand_request: pointer_scan_expand_request.into(),
            },
            CommandLinePointerScanCommand::Validate { pointer_scan_validate_request } => Self::Validate {
                pointer_scan_validate_request: pointer_scan_validate_request.into(),
            },
        }
    }
}

impl From<CommandLinePointerScanStartRequest> for api::commands::pointer_scan::start::pointer_scan_start_request::PointerScanStartRequest {
    fn from(request: CommandLinePointerScanStartRequest) -> Self {
        Self {
            target: request.target.into(),
            pointer_size: request.pointer_size,
            max_depth: request.max_depth,
            offset_radius: request.offset_radius,
            address_space: request.address_space,
        }
    }
}

impl From<CommandLinePointerScanResetRequest> for api::commands::pointer_scan::reset::pointer_scan_reset_request::PointerScanResetRequest {
    fn from(_: CommandLinePointerScanResetRequest) -> Self {
        Self {}
    }
}

impl From<CommandLinePointerScanSummaryRequest> for api::commands::pointer_scan::summary::pointer_scan_summary_request::PointerScanSummaryRequest {
    fn from(request: CommandLinePointerScanSummaryRequest) -> Self {
        Self {
            session_id: request.session_id,
        }
    }
}

impl From<CommandLinePointerScanExpandRequest> for api::commands::pointer_scan::expand::pointer_scan_expand_request::PointerScanExpandRequest {
    fn from(request: CommandLinePointerScanExpandRequest) -> Self {
        Self {
            session_id: request.session_id,
            parent_node_id: request.parent_node_id,
            page_index: request.page_index,
            page_size: request.page_size,
        }
    }
}

impl From<CommandLinePointerScanValidateRequest> for api::commands::pointer_scan::validate::pointer_scan_validate_request::PointerScanValidateRequest {
    fn from(request: CommandLinePointerScanValidateRequest) -> Self {
        Self {
            session_id: request.session_id,
            target: request.target.into(),
        }
    }
}
