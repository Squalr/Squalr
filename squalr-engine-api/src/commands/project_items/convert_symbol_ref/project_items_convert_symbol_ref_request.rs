use crate::commands::project_items::convert_symbol_ref::project_items_convert_symbol_ref_response::ProjectItemsConvertSymbolRefResponse;
use crate::commands::project_items::project_items_command::ProjectItemsCommand;
use crate::commands::project_items::project_items_response::ProjectItemsResponse;
use crate::commands::unprivileged_command::UnprivilegedCommand;
use crate::commands::unprivileged_command_request::UnprivilegedCommandRequest;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum ProjectItemSymbolRefConversionTarget {
    Inferred,
    Address,
    Pointer,
}

#[derive(Clone, Default, StructOpt, Debug, Serialize, Deserialize)]
pub struct ProjectItemsConvertSymbolRefRequest {
    #[structopt(short = "p", long = "project-item-path", parse(from_os_str))]
    pub project_item_paths: Vec<PathBuf>,

    #[structopt(short = "t", long = "target")]
    pub target: ProjectItemSymbolRefConversionTarget,
}

impl Default for ProjectItemSymbolRefConversionTarget {
    fn default() -> Self {
        Self::Inferred
    }
}

impl std::str::FromStr for ProjectItemSymbolRefConversionTarget {
    type Err = String;

    fn from_str(source: &str) -> Result<Self, Self::Err> {
        match source.trim().to_ascii_lowercase().as_str() {
            "inferred" | "source" | "original" => Ok(Self::Inferred),
            "address" => Ok(Self::Address),
            "pointer" => Ok(Self::Pointer),
            _ => Err(format!("Unsupported symbol-ref conversion target: {}", source)),
        }
    }
}

impl UnprivilegedCommandRequest for ProjectItemsConvertSymbolRefRequest {
    type ResponseType = ProjectItemsConvertSymbolRefResponse;

    fn to_engine_command(&self) -> UnprivilegedCommand {
        UnprivilegedCommand::ProjectItems(ProjectItemsCommand::ConvertSymbolRef {
            project_items_convert_symbol_ref_request: self.clone(),
        })
    }
}

impl From<ProjectItemsConvertSymbolRefResponse> for ProjectItemsResponse {
    fn from(project_items_convert_symbol_ref_response: ProjectItemsConvertSymbolRefResponse) -> Self {
        ProjectItemsResponse::ConvertSymbolRef {
            project_items_convert_symbol_ref_response,
        }
    }
}
