use crate as api;
use std::collections::BTreeMap;
use structopt::StructOpt;

#[derive(Clone, StructOpt, Debug)]
pub(crate) enum CommandLineProjectSymbolsCommand {
    Create {
        #[structopt(flatten)]
        project_symbols_create_request: CommandLineProjectSymbolsCreateRequest,
    },
    CreateModule {
        #[structopt(flatten)]
        project_symbols_create_module_request: CommandLineProjectSymbolsCreateModuleRequest,
    },
    Delete {
        #[structopt(flatten)]
        project_symbols_delete_request: CommandLineProjectSymbolsDeleteRequest,
    },
    DeleteLayout {
        #[structopt(flatten)]
        project_symbols_delete_layout_request: CommandLineProjectSymbolsDeleteLayoutRequest,
    },
    DeleteResolver {
        #[structopt(flatten)]
        project_symbols_delete_resolver_request: CommandLineProjectSymbolsDeleteResolverRequest,
    },
    List {
        #[structopt(flatten)]
        project_symbols_list_request: CommandLineProjectSymbolsListRequest,
    },
    Rename {
        #[structopt(flatten)]
        project_symbols_rename_request: CommandLineProjectSymbolsRenameRequest,
    },
    RenameModule {
        #[structopt(flatten)]
        project_symbols_rename_module_request: CommandLineProjectSymbolsRenameModuleRequest,
    },
    Update {
        #[structopt(flatten)]
        project_symbols_update_request: CommandLineProjectSymbolsUpdateRequest,
    },
    UpsertLayout {
        #[structopt(flatten)]
        project_symbols_upsert_layout_request: CommandLineProjectSymbolsUpsertLayoutRequest,
    },
    UpsertResolver {
        #[structopt(flatten)]
        project_symbols_upsert_resolver_request: CommandLineProjectSymbolsUpsertResolverRequest,
    },
    WriteValue {
        #[structopt(flatten)]
        project_symbols_write_value_request: CommandLineProjectSymbolsWriteValueRequest,
    },
}

#[derive(Clone, Default, StructOpt, Debug)]
pub(crate) struct CommandLineProjectSymbolsCreateRequest {
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
pub(crate) struct CommandLineProjectSymbolsCreateModuleRequest {
    #[structopt(short = "m", long = "module")]
    pub module_name: String,
    #[structopt(short = "s", long = "size")]
    pub size: u64,
}

#[derive(Clone, Default, StructOpt, Debug)]
pub(crate) struct CommandLineProjectSymbolsDeleteRequest {
    #[structopt(short = "k", long = "key")]
    pub symbol_locator_keys: Vec<String>,
    #[structopt(short = "m", long = "module")]
    pub module_names: Vec<String>,
}

#[derive(Clone, Default, StructOpt, Debug)]
pub(crate) struct CommandLineProjectSymbolsDeleteLayoutRequest {
    #[structopt(short = "i", long = "id")]
    pub struct_layout_id: String,
    #[structopt(long = "replacement-type", default_value = "u8")]
    pub replacement_data_type_id: String,
}

#[derive(Clone, Default, StructOpt, Debug)]
pub(crate) struct CommandLineProjectSymbolsDeleteResolverRequest {
    #[structopt(short = "i", long = "id")]
    pub resolver_id: String,
}

#[derive(Clone, Default, StructOpt, Debug)]
pub(crate) struct CommandLineProjectSymbolsListRequest {}

#[derive(Clone, Default, StructOpt, Debug)]
pub(crate) struct CommandLineProjectSymbolsRenameRequest {
    #[structopt(short = "k", long = "key")]
    pub symbol_locator_key: String,
    #[structopt(short = "n", long = "name")]
    pub display_name: String,
}

#[derive(Clone, Default, StructOpt, Debug)]
pub(crate) struct CommandLineProjectSymbolsRenameModuleRequest {
    #[structopt(short = "m", long = "module")]
    pub module_name: String,
    #[structopt(short = "n", long = "new-name")]
    pub new_module_name: String,
}

#[derive(Clone, Default, StructOpt, Debug)]
pub(crate) struct CommandLineProjectSymbolsUpdateRequest {
    #[structopt(short = "k", long = "key")]
    pub symbol_locator_key: String,
    #[structopt(long = "name")]
    pub display_name: Option<String>,
    #[structopt(long = "type")]
    pub struct_layout_id: Option<String>,
}

#[derive(Clone, Default, StructOpt, Debug)]
pub(crate) struct CommandLineProjectSymbolsUpsertLayoutRequest {
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
pub(crate) struct CommandLineProjectSymbolsUpsertResolverRequest {
    #[structopt(long = "original-id")]
    pub original_resolver_id: Option<String>,
    #[structopt(short = "i", long = "id")]
    pub resolver_id: String,
    #[structopt(short = "d", long = "definition-json")]
    pub resolver_definition_json: String,
}

#[derive(Clone, StructOpt, Debug)]
pub(crate) struct CommandLineProjectSymbolsWriteValueRequest {
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

impl From<CommandLineProjectSymbolsCommand> for api::commands::project_symbols::project_symbols_command::ProjectSymbolsCommand {
    fn from(command: CommandLineProjectSymbolsCommand) -> Self {
        match command {
            CommandLineProjectSymbolsCommand::Create {
                project_symbols_create_request,
            } => Self::Create {
                project_symbols_create_request: project_symbols_create_request.into(),
            },
            CommandLineProjectSymbolsCommand::CreateModule {
                project_symbols_create_module_request,
            } => Self::CreateModule {
                project_symbols_create_module_request: project_symbols_create_module_request.into(),
            },
            CommandLineProjectSymbolsCommand::Delete {
                project_symbols_delete_request,
            } => Self::Delete {
                project_symbols_delete_request: project_symbols_delete_request.into(),
            },
            CommandLineProjectSymbolsCommand::DeleteLayout {
                project_symbols_delete_layout_request,
            } => Self::DeleteLayout {
                project_symbols_delete_layout_request: project_symbols_delete_layout_request.into(),
            },
            CommandLineProjectSymbolsCommand::DeleteResolver {
                project_symbols_delete_resolver_request,
            } => Self::DeleteResolver {
                project_symbols_delete_resolver_request: project_symbols_delete_resolver_request.into(),
            },
            CommandLineProjectSymbolsCommand::List { project_symbols_list_request } => Self::List {
                project_symbols_list_request: project_symbols_list_request.into(),
            },
            CommandLineProjectSymbolsCommand::Rename {
                project_symbols_rename_request,
            } => Self::Rename {
                project_symbols_rename_request: project_symbols_rename_request.into(),
            },
            CommandLineProjectSymbolsCommand::RenameModule {
                project_symbols_rename_module_request,
            } => Self::RenameModule {
                project_symbols_rename_module_request: project_symbols_rename_module_request.into(),
            },
            CommandLineProjectSymbolsCommand::Update {
                project_symbols_update_request,
            } => Self::Update {
                project_symbols_update_request: project_symbols_update_request.into(),
            },
            CommandLineProjectSymbolsCommand::UpsertLayout {
                project_symbols_upsert_layout_request,
            } => Self::UpsertLayout {
                project_symbols_upsert_layout_request: project_symbols_upsert_layout_request.into(),
            },
            CommandLineProjectSymbolsCommand::UpsertResolver {
                project_symbols_upsert_resolver_request,
            } => Self::UpsertResolver {
                project_symbols_upsert_resolver_request: project_symbols_upsert_resolver_request.into(),
            },
            CommandLineProjectSymbolsCommand::WriteValue {
                project_symbols_write_value_request,
            } => Self::WriteValue {
                project_symbols_write_value_request: project_symbols_write_value_request.into(),
            },
        }
    }
}

impl From<CommandLineProjectSymbolsCreateRequest> for api::commands::project_symbols::create::project_symbols_create_request::ProjectSymbolsCreateRequest {
    fn from(request: CommandLineProjectSymbolsCreateRequest) -> Self {
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

impl From<CommandLineProjectSymbolsCreateModuleRequest>
    for api::commands::project_symbols::create_module::project_symbols_create_module_request::ProjectSymbolsCreateModuleRequest
{
    fn from(request: CommandLineProjectSymbolsCreateModuleRequest) -> Self {
        Self {
            module_name: request.module_name,
            size: request.size,
        }
    }
}

impl From<CommandLineProjectSymbolsDeleteRequest> for api::commands::project_symbols::delete::project_symbols_delete_request::ProjectSymbolsDeleteRequest {
    fn from(request: CommandLineProjectSymbolsDeleteRequest) -> Self {
        Self {
            symbol_locator_keys: request.symbol_locator_keys,
            module_names: request.module_names,
            module_ranges: Vec::new(),
        }
    }
}

impl From<CommandLineProjectSymbolsDeleteLayoutRequest>
    for api::commands::project_symbols::delete_layout::project_symbols_delete_layout_request::ProjectSymbolsDeleteLayoutRequest
{
    fn from(request: CommandLineProjectSymbolsDeleteLayoutRequest) -> Self {
        Self {
            struct_layout_id: request.struct_layout_id,
            replacement_data_type_id: request.replacement_data_type_id,
        }
    }
}

impl From<CommandLineProjectSymbolsDeleteResolverRequest>
    for api::commands::project_symbols::delete_resolver::project_symbols_delete_resolver_request::ProjectSymbolsDeleteResolverRequest
{
    fn from(request: CommandLineProjectSymbolsDeleteResolverRequest) -> Self {
        Self {
            resolver_id: request.resolver_id,
        }
    }
}

impl From<CommandLineProjectSymbolsListRequest> for api::commands::project_symbols::list::project_symbols_list_request::ProjectSymbolsListRequest {
    fn from(_: CommandLineProjectSymbolsListRequest) -> Self {
        Self {}
    }
}

impl From<CommandLineProjectSymbolsRenameRequest> for api::commands::project_symbols::rename::project_symbols_rename_request::ProjectSymbolsRenameRequest {
    fn from(request: CommandLineProjectSymbolsRenameRequest) -> Self {
        Self {
            symbol_locator_key: request.symbol_locator_key,
            display_name: request.display_name,
        }
    }
}

impl From<CommandLineProjectSymbolsRenameModuleRequest>
    for api::commands::project_symbols::rename_module::project_symbols_rename_module_request::ProjectSymbolsRenameModuleRequest
{
    fn from(request: CommandLineProjectSymbolsRenameModuleRequest) -> Self {
        Self {
            module_name: request.module_name,
            new_module_name: request.new_module_name,
        }
    }
}

impl From<CommandLineProjectSymbolsUpdateRequest> for api::commands::project_symbols::update::project_symbols_update_request::ProjectSymbolsUpdateRequest {
    fn from(request: CommandLineProjectSymbolsUpdateRequest) -> Self {
        Self {
            symbol_locator_key: request.symbol_locator_key,
            display_name: request.display_name,
            struct_layout_id: request.struct_layout_id,
        }
    }
}

impl From<CommandLineProjectSymbolsUpsertLayoutRequest>
    for api::commands::project_symbols::upsert_layout::project_symbols_upsert_layout_request::ProjectSymbolsUpsertLayoutRequest
{
    fn from(request: CommandLineProjectSymbolsUpsertLayoutRequest) -> Self {
        Self {
            original_struct_layout_id: request.original_struct_layout_id,
            struct_layout_id: request.struct_layout_id,
            layout_kind: request.layout_kind,
            size_in_bytes: request.size_in_bytes,
            field_definitions: request.field_definitions,
        }
    }
}

impl From<CommandLineProjectSymbolsUpsertResolverRequest>
    for api::commands::project_symbols::upsert_resolver::project_symbols_upsert_resolver_request::ProjectSymbolsUpsertResolverRequest
{
    fn from(request: CommandLineProjectSymbolsUpsertResolverRequest) -> Self {
        Self {
            original_resolver_id: request.original_resolver_id,
            resolver_id: request.resolver_id,
            resolver_definition_json: request.resolver_definition_json,
        }
    }
}

impl From<CommandLineProjectSymbolsWriteValueRequest>
    for api::commands::project_symbols::write_value::project_symbols_write_value_request::ProjectSymbolsWriteValueRequest
{
    fn from(request: CommandLineProjectSymbolsWriteValueRequest) -> Self {
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
