use crate::commands::project_symbols::project_symbols_command::ProjectSymbolsCommand;
use crate::commands::project_symbols::project_symbols_response::ProjectSymbolsResponse;
use crate::commands::project_symbols::upsert_resolver::project_symbols_upsert_resolver_response::ProjectSymbolsUpsertResolverResponse;
use crate::commands::unprivileged_command::UnprivilegedCommand;
use crate::commands::unprivileged_command_request::UnprivilegedCommandRequest;
use crate::registries::symbols::symbolic_resolver_descriptor::SymbolicResolverDescriptor;
use crate::structures::structs::symbolic_resolver_definition::SymbolicResolverDefinition;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct ProjectSymbolsUpsertResolverRequest {
    #[serde(default)]
    pub original_resolver_id: Option<String>,
    pub resolver_id: String,
    pub resolver_definition_json: String,
}

impl ProjectSymbolsUpsertResolverRequest {
    pub fn from_resolver_descriptor(
        original_resolver_id: Option<String>,
        resolver_descriptor: &SymbolicResolverDescriptor,
    ) -> Result<Self, String> {
        Ok(Self {
            original_resolver_id,
            resolver_id: resolver_descriptor.get_resolver_id().to_string(),
            resolver_definition_json: serde_json::to_string(resolver_descriptor.get_resolver_definition())
                .map_err(|error| format!("Failed to serialize resolver definition: {error}"))?,
        })
    }

    pub fn to_resolver_descriptor(&self) -> Result<SymbolicResolverDescriptor, String> {
        let resolver_id = self.resolver_id.trim();
        if resolver_id.is_empty() {
            return Err(String::from("Resolver id is required."));
        }

        let resolver_definition = serde_json::from_str::<SymbolicResolverDefinition>(&self.resolver_definition_json)
            .map_err(|error| format!("Invalid resolver definition JSON: {error}"))?;

        Ok(SymbolicResolverDescriptor::new(resolver_id.to_string(), resolver_definition))
    }
}

impl UnprivilegedCommandRequest for ProjectSymbolsUpsertResolverRequest {
    type ResponseType = ProjectSymbolsUpsertResolverResponse;

    fn to_engine_command(&self) -> UnprivilegedCommand {
        UnprivilegedCommand::ProjectSymbols(ProjectSymbolsCommand::UpsertResolver {
            project_symbols_upsert_resolver_request: self.clone(),
        })
    }
}

impl From<ProjectSymbolsUpsertResolverResponse> for ProjectSymbolsResponse {
    fn from(project_symbols_upsert_resolver_response: ProjectSymbolsUpsertResolverResponse) -> Self {
        ProjectSymbolsResponse::UpsertResolver {
            project_symbols_upsert_resolver_response,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ProjectSymbolsUpsertResolverRequest;
    use crate::structures::structs::symbolic_resolver_definition::SymbolicResolverNode;

    #[test]
    fn upsert_resolver_request_builds_resolver_descriptor() {
        let request = ProjectSymbolsUpsertResolverRequest {
            original_resolver_id: None,
            resolver_id: String::from("inventory.count"),
            resolver_definition_json: String::from(r#"{"root_node":{"Literal":4}}"#),
        };

        let descriptor = request
            .to_resolver_descriptor()
            .expect("Expected resolver request to build descriptor.");

        assert_eq!(descriptor.get_resolver_id(), "inventory.count");
        assert_eq!(descriptor.get_resolver_definition().get_root_node(), &SymbolicResolverNode::Literal(4));
    }
}
