use crate as api;
use structopt::StructOpt;

#[derive(Clone, StructOpt, Debug)]
pub(crate) struct CommandLineStructScanCommand {
    #[structopt(flatten)]
    pub struct_scan_request: CommandLineStructScanRequest,
}

#[derive(Clone, StructOpt, Debug)]
pub(crate) struct CommandLineStructScanRequest {
    #[structopt(short = "v", long)]
    pub scan_value: Option<api::structures::data_values::anonymous_value_string::AnonymousValueString>,
    #[structopt(short = "d", long)]
    pub data_type_ids: Vec<String>,
    #[structopt(short = "c", long)]
    pub compare_type: api::structures::scanning::comparisons::scan_compare_type::ScanCompareType,
}

impl From<CommandLineStructScanCommand> for api::commands::struct_scan::struct_scan_command::StructScanCommand {
    fn from(command: CommandLineStructScanCommand) -> Self {
        Self {
            struct_scan_request: command.struct_scan_request.into(),
        }
    }
}

impl From<CommandLineStructScanRequest> for api::commands::struct_scan::struct_scan_request::StructScanRequest {
    fn from(request: CommandLineStructScanRequest) -> Self {
        Self {
            scan_value: request.scan_value,
            data_type_ids: request.data_type_ids,
            compare_type: request.compare_type,
        }
    }
}
