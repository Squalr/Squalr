use crate as api;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Clone, StructOpt, Debug)]
pub(crate) enum CommandLineProjectItemsCommand {
    Add {
        #[structopt(flatten)]
        project_items_add_request: CommandLineProjectItemsAddRequest,
    },
    Activate {
        #[structopt(flatten)]
        project_items_activate_request: CommandLineProjectItemsActivateRequest,
    },
    Create {
        #[structopt(flatten)]
        project_items_create_request: CommandLineProjectItemsCreateRequest,
    },
    Delete {
        #[structopt(flatten)]
        project_items_delete_request: CommandLineProjectItemsDeleteRequest,
    },
    Duplicate {
        #[structopt(flatten)]
        project_items_duplicate_request: CommandLineProjectItemsDuplicateRequest,
    },
    List {
        #[structopt(flatten)]
        project_items_list_request: CommandLineProjectItemsListRequest,
    },
    Move {
        #[structopt(flatten)]
        project_items_move_request: CommandLineProjectItemsMoveRequest,
    },
    PromoteSymbol {
        #[structopt(flatten)]
        project_items_promote_symbol_request: CommandLineProjectItemsPromoteSymbolRequest,
    },
    Rename {
        #[structopt(flatten)]
        project_items_rename_request: CommandLineProjectItemsRenameRequest,
    },
    Reorder {
        #[structopt(flatten)]
        project_items_reorder_request: CommandLineProjectItemsReorderRequest,
    },
    StripSymbol {
        #[structopt(flatten)]
        project_items_strip_symbol_request: CommandLineProjectItemsStripSymbolRequest,
    },
    UpdateDetails {
        #[structopt(flatten)]
        project_items_update_details_request: CommandLineProjectItemsUpdateDetailsRequest,
    },
    WriteValue {
        #[structopt(flatten)]
        project_items_write_value_request: CommandLineProjectItemsWriteValueRequest,
    },
}

#[derive(Clone, StructOpt, Debug)]
pub(crate) struct CommandLineProjectItemsActivateRequest {
    #[structopt(short = "p", long)]
    pub project_item_paths: Vec<String>,
    #[structopt(short = "a", long)]
    pub is_activated: bool,
}

#[derive(Clone, StructOpt, Debug)]
pub(crate) struct CommandLineProjectItemsAddRequest {
    #[structopt(short = "s", long)]
    pub scan_result_refs: Vec<api::structures::scan_results::scan_result_ref::ScanResultRef>,
    #[structopt(long)]
    pub target_directory_path: Option<PathBuf>,
}

#[derive(Clone, StructOpt, Debug)]
pub(crate) struct CommandLineProjectItemsCreateRequest {
    #[structopt(short = "p", long)]
    pub parent_directory_path: PathBuf,
    #[structopt(short = "n", long)]
    pub project_item_name: String,
    #[structopt(long)]
    pub is_directory: bool,
}

#[derive(Clone, StructOpt, Debug)]
pub(crate) struct CommandLineProjectItemsDeleteRequest {
    #[structopt(short = "p", long)]
    pub project_item_paths: Vec<PathBuf>,
}

#[derive(Clone, StructOpt, Debug)]
pub(crate) struct CommandLineProjectItemsDuplicateRequest {
    #[structopt(short = "p", long)]
    pub project_item_paths: Vec<PathBuf>,
    #[structopt(short = "t", long)]
    pub target_directory_path: PathBuf,
}

#[derive(Clone, Default, StructOpt, Debug)]
pub(crate) struct CommandLineProjectItemsListRequest {}

#[derive(Clone, StructOpt, Debug)]
pub(crate) struct CommandLineProjectItemsMoveRequest {
    #[structopt(short = "p", long)]
    pub project_item_paths: Vec<PathBuf>,
    #[structopt(short = "t", long)]
    pub target_directory_path: PathBuf,
}

#[derive(Clone, Default, StructOpt, Debug)]
pub(crate) struct CommandLineProjectItemsPromoteSymbolRequest {
    #[structopt(short = "p", long = "project-item-path", parse(from_os_str))]
    pub project_item_paths: Vec<PathBuf>,
    #[structopt(long = "overwrite-conflicting-symbols")]
    pub overwrite_conflicting_symbols: bool,
}

#[derive(Clone, StructOpt, Debug)]
pub(crate) struct CommandLineProjectItemsRenameRequest {
    #[structopt(short = "p", long)]
    pub project_item_path: PathBuf,
    #[structopt(short = "n", long)]
    pub project_item_name: String,
}

#[derive(Clone, StructOpt, Debug)]
pub(crate) struct CommandLineProjectItemsReorderRequest {
    #[structopt(short = "p", long)]
    pub project_item_paths: Vec<PathBuf>,
}

#[derive(Clone, StructOpt, Debug)]
pub(crate) struct CommandLineProjectItemsStripSymbolRequest {
    #[structopt(short = "p", long = "project-item-path", parse(from_os_str))]
    pub project_item_paths: Vec<PathBuf>,
}

#[derive(Clone, Default, StructOpt, Debug)]
pub(crate) struct CommandLineProjectItemsUpdateDetailsRequest {
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
pub(crate) struct CommandLineProjectItemsWriteValueRequest {
    #[structopt(short = "p", long = "project-item-path", parse(from_os_str))]
    pub project_item_path: PathBuf,
    #[structopt(long = "field", default_value = "value")]
    pub field_name: String,
    #[structopt(short = "v", long)]
    pub anonymous_value_string: api::structures::data_values::anonymous_value_string::AnonymousValueString,
}

impl From<CommandLineProjectItemsCommand> for api::commands::project_items::project_items_command::ProjectItemsCommand {
    fn from(command: CommandLineProjectItemsCommand) -> Self {
        match command {
            CommandLineProjectItemsCommand::Add { project_items_add_request } => Self::Add {
                project_items_add_request: project_items_add_request.into(),
            },
            CommandLineProjectItemsCommand::Activate {
                project_items_activate_request,
            } => Self::Activate {
                project_items_activate_request: project_items_activate_request.into(),
            },
            CommandLineProjectItemsCommand::Create { project_items_create_request } => Self::Create {
                project_items_create_request: project_items_create_request.into(),
            },
            CommandLineProjectItemsCommand::Delete { project_items_delete_request } => Self::Delete {
                project_items_delete_request: project_items_delete_request.into(),
            },
            CommandLineProjectItemsCommand::Duplicate {
                project_items_duplicate_request,
            } => Self::Duplicate {
                project_items_duplicate_request: project_items_duplicate_request.into(),
            },
            CommandLineProjectItemsCommand::List { project_items_list_request } => Self::List {
                project_items_list_request: project_items_list_request.into(),
            },
            CommandLineProjectItemsCommand::Move { project_items_move_request } => Self::Move {
                project_items_move_request: project_items_move_request.into(),
            },
            CommandLineProjectItemsCommand::PromoteSymbol {
                project_items_promote_symbol_request,
            } => Self::PromoteSymbol {
                project_items_promote_symbol_request: project_items_promote_symbol_request.into(),
            },
            CommandLineProjectItemsCommand::Rename { project_items_rename_request } => Self::Rename {
                project_items_rename_request: project_items_rename_request.into(),
            },
            CommandLineProjectItemsCommand::Reorder { project_items_reorder_request } => Self::Reorder {
                project_items_reorder_request: project_items_reorder_request.into(),
            },
            CommandLineProjectItemsCommand::StripSymbol {
                project_items_strip_symbol_request,
            } => Self::StripSymbol {
                project_items_strip_symbol_request: project_items_strip_symbol_request.into(),
            },
            CommandLineProjectItemsCommand::UpdateDetails {
                project_items_update_details_request,
            } => Self::UpdateDetails {
                project_items_update_details_request: project_items_update_details_request.into(),
            },
            CommandLineProjectItemsCommand::WriteValue {
                project_items_write_value_request,
            } => Self::WriteValue {
                project_items_write_value_request: project_items_write_value_request.into(),
            },
        }
    }
}

impl From<CommandLineProjectItemsActivateRequest> for api::commands::project_items::activate::project_items_activate_request::ProjectItemsActivateRequest {
    fn from(request: CommandLineProjectItemsActivateRequest) -> Self {
        Self {
            project_item_paths: request.project_item_paths,
            is_activated: request.is_activated,
        }
    }
}

impl From<CommandLineProjectItemsAddRequest> for api::commands::project_items::add::project_items_add_request::ProjectItemsAddRequest {
    fn from(request: CommandLineProjectItemsAddRequest) -> Self {
        Self {
            scan_result_refs: request.scan_result_refs,
            target_directory_path: request.target_directory_path,
        }
    }
}

impl From<CommandLineProjectItemsCreateRequest> for api::commands::project_items::create::project_items_create_request::ProjectItemsCreateRequest {
    fn from(request: CommandLineProjectItemsCreateRequest) -> Self {
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

impl From<CommandLineProjectItemsDeleteRequest> for api::commands::project_items::delete::project_items_delete_request::ProjectItemsDeleteRequest {
    fn from(request: CommandLineProjectItemsDeleteRequest) -> Self {
        Self {
            project_item_paths: request.project_item_paths,
        }
    }
}

impl From<CommandLineProjectItemsDuplicateRequest> for api::commands::project_items::duplicate::project_items_duplicate_request::ProjectItemsDuplicateRequest {
    fn from(request: CommandLineProjectItemsDuplicateRequest) -> Self {
        Self {
            project_item_paths: request.project_item_paths,
            target_directory_path: request.target_directory_path,
        }
    }
}

impl From<CommandLineProjectItemsListRequest> for api::commands::project_items::list::project_items_list_request::ProjectItemsListRequest {
    fn from(_: CommandLineProjectItemsListRequest) -> Self {
        Self {
            preview_project_item_paths: None,
        }
    }
}

impl From<CommandLineProjectItemsMoveRequest> for api::commands::project_items::move_item::project_items_move_request::ProjectItemsMoveRequest {
    fn from(request: CommandLineProjectItemsMoveRequest) -> Self {
        Self {
            project_item_paths: request.project_item_paths,
            target_directory_path: request.target_directory_path,
        }
    }
}

impl From<CommandLineProjectItemsPromoteSymbolRequest>
    for api::commands::project_items::promote_symbol::project_items_promote_symbol_request::ProjectItemsPromoteSymbolRequest
{
    fn from(request: CommandLineProjectItemsPromoteSymbolRequest) -> Self {
        Self {
            project_item_paths: request.project_item_paths,
            overwrite_conflicting_symbols: request.overwrite_conflicting_symbols,
        }
    }
}

impl From<CommandLineProjectItemsRenameRequest> for api::commands::project_items::rename::project_items_rename_request::ProjectItemsRenameRequest {
    fn from(request: CommandLineProjectItemsRenameRequest) -> Self {
        Self {
            project_item_path: request.project_item_path,
            project_item_name: request.project_item_name,
        }
    }
}

impl From<CommandLineProjectItemsReorderRequest> for api::commands::project_items::reorder::project_items_reorder_request::ProjectItemsReorderRequest {
    fn from(request: CommandLineProjectItemsReorderRequest) -> Self {
        Self {
            project_item_paths: request.project_item_paths,
        }
    }
}

impl From<CommandLineProjectItemsStripSymbolRequest>
    for api::commands::project_items::strip_symbol::project_items_strip_symbol_request::ProjectItemsStripSymbolRequest
{
    fn from(request: CommandLineProjectItemsStripSymbolRequest) -> Self {
        Self {
            project_item_paths: request.project_item_paths,
        }
    }
}

impl From<CommandLineProjectItemsUpdateDetailsRequest>
    for api::commands::project_items::update_details::project_items_update_details_request::ProjectItemsUpdateDetailsRequest
{
    fn from(request: CommandLineProjectItemsUpdateDetailsRequest) -> Self {
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

impl From<CommandLineProjectItemsWriteValueRequest>
    for api::commands::project_items::write_value::project_items_write_value_request::ProjectItemsWriteValueRequest
{
    fn from(request: CommandLineProjectItemsWriteValueRequest) -> Self {
        Self {
            project_item_path: request.project_item_path,
            field_name: request.field_name,
            anonymous_value_string: request.anonymous_value_string,
        }
    }
}
