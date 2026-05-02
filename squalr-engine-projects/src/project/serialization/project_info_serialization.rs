use crate::project::serialization::serializable_project_file::SerializableProjectFile;
use serde::{Deserialize, Serialize};
use squalr_engine_api::plugins::PluginEnablementOverrides;
use squalr_engine_api::structures::{
    processes::process_icon::ProcessIcon,
    projects::{project::Project, project_info::ProjectInfo, project_manifest::ProjectManifest, project_symbol_catalog::ProjectSymbolCatalog},
};
use std::{
    fs::{File, OpenOptions},
    path::Path,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
enum ProjectPluginConfigurationStub {
    LegacyEnabledPluginIds(Vec<String>),
    PluginEnablementOverrides(PluginEnablementOverrides),
}

/// Represents a condensed version of project info excluding information that we do not want to serialize.
/// Note that #[serde(skip)] is insufficient, as we still want to serialize across commands,
/// So instead we use a small stub that we augment after deserialization.
#[derive(Clone, Debug, Serialize, Deserialize)]
struct ProjectInfoStub {
    /// The process icon associated with this project.
    #[serde(rename = "icon")]
    project_icon_rgba: Option<ProcessIcon>,

    /// The manifest for this project, containing the sort order of project items.
    #[serde(rename = "manifest", default)]
    project_manifest: ProjectManifest,

    /// User-authored symbolic struct definitions stored with this project.
    #[serde(rename = "symbols", default)]
    project_symbol_catalog: ProjectSymbolCatalog,

    /// Plugin enablement overrides stored with this project.
    #[serde(rename = "plugins", default, skip_serializing_if = "Option::is_none")]
    plugin_configuration: Option<ProjectPluginConfigurationStub>,
}

impl SerializableProjectFile for ProjectInfo {
    fn save_to_path(
        &mut self,
        directory: &Path,
        save_even_if_unchanged: bool,
    ) -> anyhow::Result<()> {
        if save_even_if_unchanged || self.get_has_unsaved_changes() {
            let project_file_path = directory.join(Project::PROJECT_FILE);

            let file = OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(&project_file_path)?;

            let project_info_stub = ProjectInfoStub {
                project_icon_rgba: self.get_project_icon_rgba().clone(),
                project_manifest: self.get_project_manifest().clone(),
                project_symbol_catalog: self.get_project_symbol_catalog().clone(),
                plugin_configuration: self
                    .get_plugin_enablement_overrides()
                    .cloned()
                    .map(ProjectPluginConfigurationStub::PluginEnablementOverrides),
            };

            serde_json::to_writer(file, &project_info_stub)?;

            self.set_has_unsaved_changes(false);
        }

        Ok(())
    }

    fn load_from_path(project_file_path: &Path) -> anyhow::Result<Self> {
        let project_file = File::open(project_file_path)?;
        let project_info_stub: ProjectInfoStub = serde_json::from_reader(project_file)?;

        let mut project_info = ProjectInfo::new_with_symbol_catalog(
            project_file_path.to_path_buf(),
            project_info_stub.project_icon_rgba,
            project_info_stub.project_manifest,
            project_info_stub.project_symbol_catalog,
        );
        project_info.set_plugin_enablement_overrides(match project_info_stub.plugin_configuration {
            Some(ProjectPluginConfigurationStub::LegacyEnabledPluginIds(enabled_plugin_ids)) => {
                Some(PluginEnablementOverrides::new(enabled_plugin_ids, Vec::new()))
            }
            Some(ProjectPluginConfigurationStub::PluginEnablementOverrides(plugin_enablement_overrides)) => Some(plugin_enablement_overrides),
            None => None,
        });

        Ok(project_info)
    }
}

#[cfg(test)]
mod tests {
    use super::SerializableProjectFile;
    use squalr_engine_api::plugins::PluginEnablementOverrides;
    use squalr_engine_api::registries::symbols::struct_layout_descriptor::StructLayoutDescriptor;
    use squalr_engine_api::structures::{
        data_types::data_type_ref::DataTypeRef,
        data_values::container_type::ContainerType,
        projects::{
            project::Project, project_info::ProjectInfo, project_manifest::ProjectManifest, project_symbol_catalog::ProjectSymbolCatalog,
            project_symbol_claim::ProjectSymbolClaim, project_symbol_locator::ProjectSymbolLocator,
        },
        structs::{symbolic_field_definition::SymbolicFieldDefinition, symbolic_struct_definition::SymbolicStructDefinition},
    };
    use std::fs;

    #[test]
    fn project_info_round_trip_preserves_project_symbol_catalog() {
        let temp_directory = tempfile::tempdir().expect("Expected a temporary directory.");
        let project_file_path = temp_directory.path().join(Project::PROJECT_FILE);
        let mut project_info = ProjectInfo::new_with_symbol_catalog(
            project_file_path,
            None,
            ProjectManifest::default(),
            ProjectSymbolCatalog::new(vec![StructLayoutDescriptor::new(
                String::from("player.health"),
                SymbolicStructDefinition::new(
                    String::from("player.health"),
                    vec![SymbolicFieldDefinition::new_named(
                        String::from("value"),
                        DataTypeRef::new("u32"),
                        ContainerType::None,
                    )],
                ),
            )]),
        );

        project_info
            .save_to_path(temp_directory.path(), true)
            .expect("Expected project info to save.");

        let loaded_project_info = ProjectInfo::load_from_path(&temp_directory.path().join(Project::PROJECT_FILE)).expect("Expected project info to load.");

        assert_eq!(
            loaded_project_info
                .get_project_symbol_catalog()
                .get_struct_layout_descriptors()
                .len(),
            1
        );
        assert_eq!(
            loaded_project_info
                .get_project_symbol_catalog()
                .get_struct_layout_descriptors()[0]
                .get_struct_layout_id(),
            "player.health"
        );
        assert_eq!(
            loaded_project_info
                .get_project_symbol_catalog()
                .get_struct_layout_descriptors()[0]
                .get_struct_layout_definition()
                .get_fields()[0]
                .get_field_name(),
            "value"
        );
    }

    #[test]
    fn project_info_round_trip_preserves_symbol_claims() {
        let temp_directory = tempfile::tempdir().expect("Expected a temporary directory.");
        let project_file_path = temp_directory.path().join(Project::PROJECT_FILE);
        let mut project_info = ProjectInfo::new_with_symbol_catalog(
            project_file_path,
            None,
            ProjectManifest::default(),
            ProjectSymbolCatalog::new_with_symbol_claims(
                vec![StructLayoutDescriptor::new(
                    String::from("player.stats"),
                    SymbolicStructDefinition::new(
                        String::from("player.stats"),
                        vec![SymbolicFieldDefinition::new_named(
                            String::from("health"),
                            DataTypeRef::new("u32"),
                            ContainerType::None,
                        )],
                    ),
                )],
                vec![
                    ProjectSymbolClaim::new_module_offset(String::from("Player Stats"), String::from("game.exe"), 0x1234, String::from("player.stats")),
                    ProjectSymbolClaim::new(
                        String::from("Player Absolute"),
                        ProjectSymbolLocator::new_absolute_address(0x8877_6655),
                        String::from("player.stats"),
                    ),
                ],
            ),
        );

        project_info
            .save_to_path(temp_directory.path(), true)
            .expect("Expected project info to save.");

        let loaded_project_info = ProjectInfo::load_from_path(&temp_directory.path().join(Project::PROJECT_FILE)).expect("Expected project info to load.");
        let symbol_claims = loaded_project_info
            .get_project_symbol_catalog()
            .get_symbol_claims();

        assert_eq!(symbol_claims.len(), 2);
        assert_eq!(symbol_claims[0].get_symbol_locator_key(), "module:game.exe:1234");
        assert_eq!(symbol_claims[0].get_display_name(), "Player Stats");
        assert_eq!(symbol_claims[0].get_struct_layout_id(), "player.stats");
        assert_eq!(
            symbol_claims[0].get_locator(),
            &ProjectSymbolLocator::new_module_offset(String::from("game.exe"), 0x1234)
        );
        assert_eq!(symbol_claims[1].get_locator(), &ProjectSymbolLocator::new_absolute_address(0x8877_6655));
    }

    #[test]
    fn project_info_round_trip_preserves_plugin_enablement_overrides() {
        let temp_directory = tempfile::tempdir().expect("Expected a temporary directory.");
        let project_file_path = temp_directory.path().join(Project::PROJECT_FILE);
        let mut project_info = ProjectInfo::new_with_symbol_catalog(project_file_path, None, ProjectManifest::default(), ProjectSymbolCatalog::default());

        project_info.set_plugin_enablement_overrides(Some(PluginEnablementOverrides::new(
            vec![String::from("builtin.data-type.24bit-integers")],
            vec![String::from("builtin.memory-view.dolphin")],
        )));

        project_info
            .save_to_path(temp_directory.path(), true)
            .expect("Expected project info to save.");

        let loaded_project_info = ProjectInfo::load_from_path(&temp_directory.path().join(Project::PROJECT_FILE)).expect("Expected project info to load.");

        assert_eq!(
            loaded_project_info.get_plugin_enablement_overrides(),
            Some(&PluginEnablementOverrides::new(
                vec![String::from("builtin.data-type.24bit-integers")],
                vec![String::from("builtin.memory-view.dolphin")],
            ))
        );
    }

    #[test]
    fn project_info_loads_legacy_plugin_list_as_enabled_overrides() {
        let temp_directory = tempfile::tempdir().expect("Expected a temporary directory.");
        let project_file_path = temp_directory.path().join(Project::PROJECT_FILE);
        let legacy_project_json = r#"{
            "icon": null,
            "manifest": {},
            "symbols": {},
            "plugins": ["builtin.data-type.24bit-integers", "builtin.memory-view.dolphin"]
        }"#;

        fs::write(&project_file_path, legacy_project_json).expect("Expected legacy project json to write.");

        let loaded_project_info = ProjectInfo::load_from_path(&project_file_path).expect("Expected legacy project info to load.");

        assert_eq!(
            loaded_project_info.get_plugin_enablement_overrides(),
            Some(&PluginEnablementOverrides::new(
                vec![
                    String::from("builtin.data-type.24bit-integers"),
                    String::from("builtin.memory-view.dolphin"),
                ],
                Vec::new(),
            ))
        );
    }

    #[test]
    fn project_info_loads_when_symbols_object_omits_struct_layout_descriptors() {
        let temp_directory = tempfile::tempdir().expect("Expected a temporary directory.");
        let project_file_path = temp_directory.path().join(Project::PROJECT_FILE);
        let legacy_project_json = r#"{
            "icon": null,
            "manifest": {},
            "symbols": {}
        }"#;

        fs::write(&project_file_path, legacy_project_json).expect("Expected legacy project json to write.");

        let loaded_project_info = ProjectInfo::load_from_path(&project_file_path).expect("Expected legacy project info to load.");

        assert!(loaded_project_info.get_project_symbol_catalog().is_empty());
    }
}
