use crate::commands::memory::memory_command::MemoryCommand;
use crate::commands::memory::memory_response::MemoryResponse;
use crate::commands::memory::query::memory_query_response::MemoryQueryResponse;
use crate::commands::privileged_command::PrivilegedCommand;
use crate::commands::privileged_command_request::PrivilegedCommandRequest;
use crate::plugins::memory_view::PageRetrievalMode;
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

#[derive(Clone, StructOpt, Debug, Serialize, Deserialize)]
pub struct MemoryQueryRequest {
    #[structopt(short = "p", long, default_value = "usermode")]
    pub page_retrieval_mode: PageRetrievalMode,
}

impl Default for MemoryQueryRequest {
    fn default() -> Self {
        Self {
            page_retrieval_mode: PageRetrievalMode::FromUserMode,
        }
    }
}

impl PrivilegedCommandRequest for MemoryQueryRequest {
    type ResponseType = MemoryQueryResponse;

    fn to_engine_command(&self) -> PrivilegedCommand {
        PrivilegedCommand::Memory(MemoryCommand::Query {
            memory_query_request: self.clone(),
        })
    }
}

impl From<MemoryQueryResponse> for MemoryResponse {
    fn from(memory_query_response: MemoryQueryResponse) -> Self {
        MemoryResponse::Query { memory_query_response }
    }
}
