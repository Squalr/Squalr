use crate::commands::project_symbols::project_symbols_command::ProjectSymbolsCommand;
use crate::commands::project_symbols::project_symbols_response::ProjectSymbolsResponse;
use crate::commands::project_symbols::upsert_layout::project_symbols_upsert_layout_response::ProjectSymbolsUpsertLayoutResponse;
use crate::commands::unprivileged_command::UnprivilegedCommand;
use crate::commands::unprivileged_command_request::UnprivilegedCommandRequest;
use crate::registries::symbols::struct_layout_descriptor::StructLayoutDescriptor;
use crate::structures::structs::{
    symbolic_field_definition::SymbolicFieldDefinition,
    symbolic_struct_definition::{SymbolicLayoutKind, SymbolicStructDefinition},
};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct ProjectSymbolsUpsertLayoutRequest {
    #[serde(default)]
    pub original_struct_layout_id: Option<String>,
    pub struct_layout_id: String,

    #[serde(default = "default_layout_kind")]
    pub layout_kind: String,

    #[serde(default)]
    pub size_in_bytes: Option<u64>,

    #[serde(default)]
    pub field_definitions: Vec<String>,
}

impl ProjectSymbolsUpsertLayoutRequest {
    pub fn from_struct_layout_descriptor(
        original_struct_layout_id: Option<String>,
        struct_layout_descriptor: &StructLayoutDescriptor,
    ) -> Self {
        let struct_layout_definition = struct_layout_descriptor.get_struct_layout_definition();

        Self {
            original_struct_layout_id,
            struct_layout_id: struct_layout_descriptor.get_struct_layout_id().to_string(),
            layout_kind: match struct_layout_definition.get_layout_kind() {
                SymbolicLayoutKind::Struct => String::from("struct"),
                SymbolicLayoutKind::Union => String::from("union"),
            },
            size_in_bytes: struct_layout_definition.get_declared_size_in_bytes(),
            field_definitions: struct_layout_definition
                .get_fields()
                .iter()
                .map(ToString::to_string)
                .collect(),
        }
    }

    pub fn to_struct_layout_descriptor(&self) -> Result<StructLayoutDescriptor, String> {
        let struct_layout_id = self.struct_layout_id.trim();
        if struct_layout_id.is_empty() {
            return Err(String::from("Symbol layout id is required."));
        }

        let layout_kind = parse_layout_kind(&self.layout_kind)?;
        let field_definitions = self
            .field_definitions
            .iter()
            .map(|field_definition| SymbolicFieldDefinition::from_str(field_definition))
            .collect::<Result<Vec<_>, _>>()?;
        let symbolic_struct_definition = SymbolicStructDefinition::new_with_layout_kind(struct_layout_id.to_string(), layout_kind, field_definitions)
            .with_declared_size_in_bytes(self.size_in_bytes);

        Ok(StructLayoutDescriptor::new(struct_layout_id.to_string(), symbolic_struct_definition))
    }
}

impl UnprivilegedCommandRequest for ProjectSymbolsUpsertLayoutRequest {
    type ResponseType = ProjectSymbolsUpsertLayoutResponse;

    fn to_engine_command(&self) -> UnprivilegedCommand {
        UnprivilegedCommand::ProjectSymbols(ProjectSymbolsCommand::UpsertLayout {
            project_symbols_upsert_layout_request: self.clone(),
        })
    }
}

impl From<ProjectSymbolsUpsertLayoutResponse> for ProjectSymbolsResponse {
    fn from(project_symbols_upsert_layout_response: ProjectSymbolsUpsertLayoutResponse) -> Self {
        ProjectSymbolsResponse::UpsertLayout {
            project_symbols_upsert_layout_response,
        }
    }
}

fn default_layout_kind() -> String {
    String::from("struct")
}

fn parse_layout_kind(layout_kind: &str) -> Result<SymbolicLayoutKind, String> {
    match layout_kind.trim().to_ascii_lowercase().as_str() {
        "" | "struct" => Ok(SymbolicLayoutKind::Struct),
        "union" => Ok(SymbolicLayoutKind::Union),
        other_layout_kind => Err(format!("Unsupported symbol layout kind `{}`.", other_layout_kind)),
    }
}

#[cfg(test)]
mod tests {
    use super::ProjectSymbolsUpsertLayoutRequest;
    use crate::structures::structs::symbolic_struct_definition::SymbolicLayoutKind;

    #[test]
    fn upsert_layout_request_builds_struct_layout_descriptor() {
        let request = ProjectSymbolsUpsertLayoutRequest {
            original_struct_layout_id: None,
            struct_layout_id: String::from("player.stats"),
            layout_kind: String::from("union"),
            size_in_bytes: Some(0x20),
            field_definitions: vec![String::from("health:u32"), String::from("unassigned[4]")],
        };

        let descriptor = request
            .to_struct_layout_descriptor()
            .expect("Expected layout request to build descriptor.");

        assert_eq!(descriptor.get_struct_layout_id(), "player.stats");
        assert_eq!(descriptor.get_struct_layout_definition().get_layout_kind(), SymbolicLayoutKind::Union);
        assert_eq!(
            descriptor
                .get_struct_layout_definition()
                .get_declared_size_in_bytes(),
            Some(0x20)
        );
        assert_eq!(descriptor.get_struct_layout_definition().get_fields().len(), 2);
    }
}
