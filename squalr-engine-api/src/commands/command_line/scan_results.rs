use crate as api;
use structopt::StructOpt;

#[derive(Clone, StructOpt, Debug)]
pub(crate) enum CommandLineScanResultsCommand {
    List {
        #[structopt(flatten)]
        scan_results_list_request: CommandLineScanResultsListRequest,
    },
    Query {
        #[structopt(flatten)]
        scan_results_query_request: CommandLineScanResultsQueryRequest,
    },
    Refresh {
        #[structopt(flatten)]
        scan_results_refresh_request: CommandLineScanResultsRefreshRequest,
    },
    Delete {
        #[structopt(flatten)]
        scan_results_delete_request: CommandLineScanResultsDeleteRequest,
    },
    Freeze {
        #[structopt(flatten)]
        scan_results_freeze_request: CommandLineScanResultsFreezeRequest,
    },
    SetProperty {
        #[structopt(flatten)]
        results_set_property_request: CommandLineScanResultsSetPropertyRequest,
    },
}

#[derive(Clone, StructOpt, Debug)]
pub(crate) struct CommandLineScanResultsListRequest {
    #[structopt(short = "p", long)]
    pub page_index: u64,
    #[structopt(long = "data-type-filter")]
    pub data_type_filters: Option<Vec<api::structures::data_types::data_type_ref::DataTypeRef>>,
}

#[derive(Clone, StructOpt, Debug)]
pub(crate) struct CommandLineScanResultsQueryRequest {
    #[structopt(short = "p", long)]
    pub page_index: u64,
    #[structopt(long = "data-type-filter")]
    pub data_type_filters: Option<Vec<api::structures::data_types::data_type_ref::DataTypeRef>>,
}

#[derive(Clone, StructOpt, Debug)]
pub(crate) struct CommandLineScanResultsRefreshRequest {
    #[structopt(short = "r", long)]
    pub scan_result_refs: Vec<api::structures::scan_results::scan_result_ref::ScanResultRef>,
}

#[derive(Clone, StructOpt, Debug)]
pub(crate) struct CommandLineScanResultsDeleteRequest {
    #[structopt(short = "s", long)]
    pub scan_result_refs: Vec<api::structures::scan_results::scan_result_ref::ScanResultRef>,
}

#[derive(Clone, StructOpt, Debug)]
pub(crate) struct CommandLineScanResultsFreezeRequest {
    #[structopt(short = "s", long)]
    pub scan_result_refs: Vec<api::structures::scan_results::scan_result_ref::ScanResultRef>,
    #[structopt(short = "f", long)]
    pub is_frozen: bool,
}

#[derive(Clone, StructOpt, Debug)]
pub(crate) struct CommandLineScanResultsSetPropertyRequest {
    #[structopt(short = "s", long)]
    pub scan_result_refs: Vec<api::structures::scan_results::scan_result_ref::ScanResultRef>,
    #[structopt(short = "v", long)]
    pub anonymous_value_string: api::structures::data_values::anonymous_value_string::AnonymousValueString,
    #[structopt(short = "f", long)]
    pub field_namespace: String,
}

impl From<CommandLineScanResultsCommand> for api::commands::scan_results::scan_results_command::ScanResultsCommand {
    fn from(command: CommandLineScanResultsCommand) -> Self {
        match command {
            CommandLineScanResultsCommand::List { scan_results_list_request } => Self::List {
                results_list_request: scan_results_list_request.into(),
            },
            CommandLineScanResultsCommand::Query { scan_results_query_request } => Self::Query {
                results_query_request: scan_results_query_request.into(),
            },
            CommandLineScanResultsCommand::Refresh { scan_results_refresh_request } => Self::Refresh {
                results_refresh_request: scan_results_refresh_request.into(),
            },
            CommandLineScanResultsCommand::Delete { scan_results_delete_request } => Self::Delete {
                results_delete_request: scan_results_delete_request.into(),
            },
            CommandLineScanResultsCommand::Freeze { scan_results_freeze_request } => Self::Freeze {
                results_freeze_request: scan_results_freeze_request.into(),
            },
            CommandLineScanResultsCommand::SetProperty { results_set_property_request } => Self::SetProperty {
                results_set_property_request: results_set_property_request.into(),
            },
        }
    }
}

impl From<CommandLineScanResultsListRequest> for api::commands::scan_results::list::scan_results_list_request::ScanResultsListRequest {
    fn from(request: CommandLineScanResultsListRequest) -> Self {
        Self {
            page_index: request.page_index,
            data_type_filters: request.data_type_filters,
        }
    }
}

impl From<CommandLineScanResultsQueryRequest> for api::commands::scan_results::query::scan_results_query_request::ScanResultsQueryRequest {
    fn from(request: CommandLineScanResultsQueryRequest) -> Self {
        Self {
            page_index: request.page_index,
            data_type_filters: request.data_type_filters,
        }
    }
}

impl From<CommandLineScanResultsRefreshRequest> for api::commands::scan_results::refresh::scan_results_refresh_request::ScanResultsRefreshRequest {
    fn from(request: CommandLineScanResultsRefreshRequest) -> Self {
        Self {
            scan_result_refs: request.scan_result_refs,
        }
    }
}

impl From<CommandLineScanResultsDeleteRequest> for api::commands::scan_results::delete::scan_results_delete_request::ScanResultsDeleteRequest {
    fn from(request: CommandLineScanResultsDeleteRequest) -> Self {
        Self {
            scan_result_refs: request.scan_result_refs,
        }
    }
}

impl From<CommandLineScanResultsFreezeRequest> for api::commands::scan_results::freeze::scan_results_freeze_request::ScanResultsFreezeRequest {
    fn from(request: CommandLineScanResultsFreezeRequest) -> Self {
        Self {
            scan_result_refs: request.scan_result_refs,
            is_frozen: request.is_frozen,
        }
    }
}

impl From<CommandLineScanResultsSetPropertyRequest>
    for api::commands::scan_results::set_property::scan_results_set_property_request::ScanResultsSetPropertyRequest
{
    fn from(request: CommandLineScanResultsSetPropertyRequest) -> Self {
        Self {
            scan_result_refs: request.scan_result_refs,
            anonymous_value_string: request.anonymous_value_string,
            field_namespace: request.field_namespace,
        }
    }
}
