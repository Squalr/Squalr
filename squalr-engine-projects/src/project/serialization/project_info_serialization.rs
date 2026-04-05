use crate::project::serialization::serializable_project_file::SerializableProjectFile;
use serde::{Deserialize, Serialize};
use squalr_engine_api::structures::{
    processes::process_icon::ProcessIcon,
    projects::{project::Project, project_info::ProjectInfo, project_manifest::ProjectManifest, project_symbol_catalog::ProjectSymbolCatalog},
};
use std::{
    fs::{File, OpenOptions},
    path::Path,
};

/// Represents a condensed version of project info excluding information that we do not want to serialize.
/// Note that #[serde(skip)] is insufficient, as we still want to serialize across commands,
/// So instead we use a small stub that we augment after deserialization.
#[derive(Clone, Debug, Serialize, Deserialize)]
struct ProjectInfoStub {
    /// The process icon associated with this project.
    #[serde(rename = "icon")]
    project_icon_rgba: Option<ProcessIcon>,

    /// The manifest for this project, containing the sort order of project items.
    #[serde(rename = "manifest")]
    project_manifest: ProjectManifest,

    /// User-authored symbolic struct definitions stored with this project.
    #[serde(rename = "symbols", default)]
    project_symbol_catalog: ProjectSymbolCatalog,
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
            };

            serde_json::to_writer(file, &project_info_stub)?;

            self.set_has_unsaved_changes(false);
        }

        Ok(())
    }

    fn load_from_path(project_file_path: &Path) -> anyhow::Result<Self> {
        let project_file = File::open(project_file_path)?;
        let project_info_stub: ProjectInfoStub = serde_json::from_reader(project_file)?;

        Ok(ProjectInfo::new_with_symbol_catalog(
            project_file_path.to_path_buf(),
            project_info_stub.project_icon_rgba,
            project_info_stub.project_manifest,
            project_info_stub.project_symbol_catalog,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::SerializableProjectFile;
    use squalr_engine_api::registries::symbols::struct_layout_descriptor::StructLayoutDescriptor;
    use squalr_engine_api::structures::{
        data_types::data_type_ref::DataTypeRef,
        data_values::container_type::ContainerType,
        projects::{project::Project, project_info::ProjectInfo, project_manifest::ProjectManifest, project_symbol_catalog::ProjectSymbolCatalog},
        structs::{symbolic_field_definition::SymbolicFieldDefinition, symbolic_struct_definition::SymbolicStructDefinition},
    };

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
                    vec![SymbolicFieldDefinition::new(
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
    }
}
