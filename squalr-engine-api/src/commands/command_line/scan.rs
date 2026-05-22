use crate as api;
use structopt::StructOpt;

#[derive(Clone, StructOpt, Debug)]
pub(crate) enum CommandLineScanCommand {
    Reset {
        #[structopt(flatten)]
        scan_reset_request: CommandLineScanResetRequest,
    },
    New {
        #[structopt(flatten)]
        scan_new_request: CommandLineScanNewRequest,
    },
    CollectValues {
        #[structopt(flatten)]
        scan_value_collector_request: CommandLineScanCollectValuesRequest,
    },
    ElementScan {
        #[structopt(flatten)]
        element_scan_request: CommandLineElementScanRequest,
    },
}

#[derive(Clone, StructOpt, Debug)]
pub(crate) struct CommandLineScanResetRequest {}

#[derive(Clone, StructOpt, Debug)]
pub(crate) struct CommandLineScanNewRequest {}

#[derive(Clone, StructOpt, Debug)]
pub(crate) struct CommandLineScanCollectValuesRequest {
    #[structopt(short = "d", long)]
    pub data_type_refs: Vec<api::structures::data_types::data_type_ref::DataTypeRef>,
}

#[derive(Clone, StructOpt, Debug)]
pub(crate) struct CommandLineElementScanRequest {
    #[structopt(short = "c", long)]
    pub scan_constraints: Vec<api::structures::scanning::constraints::anonymous_scan_constraint::AnonymousScanConstraint>,
    #[structopt(short = "d", long)]
    pub data_type_refs: Vec<api::structures::data_types::data_type_ref::DataTypeRef>,
}

impl From<CommandLineScanCommand> for api::commands::scan::scan_command::ScanCommand {
    fn from(command: CommandLineScanCommand) -> Self {
        match command {
            CommandLineScanCommand::Reset { scan_reset_request } => Self::Reset {
                scan_reset_request: scan_reset_request.into(),
            },
            CommandLineScanCommand::New { scan_new_request } => Self::New {
                scan_new_request: scan_new_request.into(),
            },
            CommandLineScanCommand::CollectValues { scan_value_collector_request } => Self::CollectValues {
                scan_value_collector_request: scan_value_collector_request.into(),
            },
            CommandLineScanCommand::ElementScan { element_scan_request } => Self::ElementScan {
                element_scan_request: element_scan_request.into(),
            },
        }
    }
}

impl From<CommandLineScanResetRequest> for api::commands::scan::reset::scan_reset_request::ScanResetRequest {
    fn from(_: CommandLineScanResetRequest) -> Self {
        Self {}
    }
}

impl From<CommandLineScanNewRequest> for api::commands::scan::new::scan_new_request::ScanNewRequest {
    fn from(_: CommandLineScanNewRequest) -> Self {
        Self {}
    }
}

impl From<CommandLineScanCollectValuesRequest> for api::commands::scan::collect_values::scan_collect_values_request::ScanCollectValuesRequest {
    fn from(request: CommandLineScanCollectValuesRequest) -> Self {
        Self {
            data_type_refs: request.data_type_refs,
        }
    }
}

impl From<CommandLineElementScanRequest> for api::commands::scan::element_scan::element_scan_request::ElementScanRequest {
    fn from(request: CommandLineElementScanRequest) -> Self {
        Self {
            scan_constraints: request.scan_constraints,
            data_type_refs: request.data_type_refs,
        }
    }
}
