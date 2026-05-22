use crate as api;
use structopt::StructOpt;

#[derive(Clone, StructOpt, Debug)]
pub(crate) enum CommandLineMemoryCommand {
    Freeze {
        #[structopt(flatten)]
        memory_freeze_request: CommandLineMemoryFreezeRequest,
    },
    Query {
        #[structopt(flatten)]
        memory_query_request: CommandLineMemoryQueryRequest,
    },
    Read {
        #[structopt(flatten)]
        memory_read_request: CommandLineMemoryReadRequest,
    },
    Write {
        #[structopt(flatten)]
        memory_write_request: CommandLineMemoryWriteRequest,
    },
}

#[derive(Clone, StructOpt, Debug, Default)]
pub(crate) struct CommandLineMemoryFreezeRequest {
    #[structopt(short = "f", long = "frozen")]
    pub is_frozen: bool,
}

#[derive(Clone, StructOpt, Debug)]
pub(crate) struct CommandLineMemoryQueryRequest {
    #[structopt(short = "p", long, default_value = "usermode")]
    pub page_retrieval_mode: api::plugins::memory_view::PageRetrievalMode,
}

#[derive(Clone, StructOpt, Debug)]
pub(crate) struct CommandLineMemoryReadRequest {
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
pub(crate) struct CommandLineMemoryWriteRequest {
    #[structopt(short = "a", long, parse(try_from_str = api::conversions::conversions_from_primitives::Conversions::parse_hex_or_int))]
    pub address: u64,
    #[structopt(short = "m")]
    pub module_name: String,
    #[structopt(short = "v")]
    pub value: Vec<u8>,
}

impl From<CommandLineMemoryCommand> for api::commands::memory::memory_command::MemoryCommand {
    fn from(command: CommandLineMemoryCommand) -> Self {
        match command {
            CommandLineMemoryCommand::Freeze { memory_freeze_request } => Self::Freeze {
                memory_freeze_request: memory_freeze_request.into(),
            },
            CommandLineMemoryCommand::Query { memory_query_request } => Self::Query {
                memory_query_request: memory_query_request.into(),
            },
            CommandLineMemoryCommand::Read { memory_read_request } => Self::Read {
                memory_read_request: memory_read_request.into(),
            },
            CommandLineMemoryCommand::Write { memory_write_request } => Self::Write {
                memory_write_request: memory_write_request.into(),
            },
        }
    }
}

impl From<CommandLineMemoryFreezeRequest> for api::commands::memory::freeze::memory_freeze_request::MemoryFreezeRequest {
    fn from(request: CommandLineMemoryFreezeRequest) -> Self {
        Self {
            freeze_targets: Vec::new(),
            is_frozen: request.is_frozen,
        }
    }
}

impl From<CommandLineMemoryQueryRequest> for api::commands::memory::query::memory_query_request::MemoryQueryRequest {
    fn from(request: CommandLineMemoryQueryRequest) -> Self {
        Self {
            page_retrieval_mode: request.page_retrieval_mode,
        }
    }
}

impl From<CommandLineMemoryReadRequest> for api::commands::memory::read::memory_read_request::MemoryReadRequest {
    fn from(request: CommandLineMemoryReadRequest) -> Self {
        Self {
            address: request.address,
            module_name: request.module_name,
            symbolic_struct_definition: request.symbolic_struct_definition,
            suppress_logging: request.suppress_logging,
        }
    }
}

impl From<CommandLineMemoryWriteRequest> for api::commands::memory::write::memory_write_request::MemoryWriteRequest {
    fn from(request: CommandLineMemoryWriteRequest) -> Self {
        Self {
            address: request.address,
            module_name: request.module_name,
            value: request.value,
        }
    }
}
