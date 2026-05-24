use crate::structures::data_values::anonymous_value_string_format::AnonymousValueStringFormat;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::PathBuf;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ProjectManifest {
    #[serde(rename = "sort_order", default)]
    project_item_sort_order: Vec<PathBuf>,
    #[serde(rename = "symbol_display_formats", default, skip_serializing_if = "BTreeMap::is_empty")]
    symbol_display_formats_by_node_key: BTreeMap<String, AnonymousValueStringFormat>,
}

impl ProjectManifest {
    pub fn new(project_item_sort_order: Vec<PathBuf>) -> Self {
        Self {
            project_item_sort_order,
            symbol_display_formats_by_node_key: BTreeMap::new(),
        }
    }

    pub fn get_project_item_sort_order(&self) -> &Vec<PathBuf> {
        &self.project_item_sort_order
    }

    pub fn set_project_item_sort_order(
        &mut self,
        project_item_sort_order: Vec<PathBuf>,
    ) {
        self.project_item_sort_order = project_item_sort_order;
    }

    pub fn get_symbol_display_format(
        &self,
        symbol_node_key: &str,
    ) -> Option<AnonymousValueStringFormat> {
        self.symbol_display_formats_by_node_key
            .get(symbol_node_key)
            .copied()
    }

    pub fn set_symbol_display_format(
        &mut self,
        symbol_node_key: String,
        display_format: AnonymousValueStringFormat,
    ) {
        self.symbol_display_formats_by_node_key
            .insert(symbol_node_key, display_format);
    }
}

#[cfg(test)]
mod tests {
    use super::ProjectManifest;
    use crate::structures::data_values::anonymous_value_string_format::AnonymousValueStringFormat;

    #[test]
    fn symbol_display_formats_round_trip_through_manifest_json() {
        let mut project_manifest = ProjectManifest::new(Vec::new());

        project_manifest.set_symbol_display_format(String::from("module.exe::health"), AnonymousValueStringFormat::Hexadecimal);

        let serialized_project_manifest = serde_json::to_string(&project_manifest).expect("Expected project manifest to serialize.");
        let deserialized_project_manifest: ProjectManifest =
            serde_json::from_str(&serialized_project_manifest).expect("Expected project manifest to deserialize.");

        assert_eq!(
            deserialized_project_manifest.get_symbol_display_format("module.exe::health"),
            Some(AnonymousValueStringFormat::Hexadecimal)
        );
    }
}
