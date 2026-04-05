use crate::structures::processes::process_icon::ProcessIcon;
use crate::structures::projects::project_manifest::ProjectManifest;
use crate::structures::projects::project_symbol_catalog::ProjectSymbolCatalog;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct ProjectInfo {
    /// The name of this project. This is derived from the folder containing the project json.
    project_name: String,

    /// The path of the main project file.
    project_file_path: PathBuf,

    /// The process icon associated with this project.
    project_icon_rgba: Option<ProcessIcon>,

    /// The manifest for this project, containing the sort order of project items.
    project_manifest: ProjectManifest,

    /// User-authored symbolic struct definitions stored with this project.
    project_symbol_catalog: ProjectSymbolCatalog,

    #[serde(skip)]
    has_unsaved_changes: bool,
}

impl ProjectInfo {
    pub fn new(
        project_file_path: PathBuf,
        project_icon_rgba: Option<ProcessIcon>,
        project_manifest: ProjectManifest,
    ) -> Self {
        Self::new_with_symbol_catalog(project_file_path, project_icon_rgba, project_manifest, ProjectSymbolCatalog::default())
    }

    pub fn new_with_symbol_catalog(
        project_file_path: PathBuf,
        project_icon_rgba: Option<ProcessIcon>,
        project_manifest: ProjectManifest,
        project_symbol_catalog: ProjectSymbolCatalog,
    ) -> Self {
        let project_name = project_file_path
            .parent()
            .and_then(|parent_path| parent_path.file_name())
            .and_then(|parent_path| parent_path.to_str())
            .unwrap_or("<Unknown project name>")
            .to_string();

        Self {
            project_name,
            project_file_path,
            project_icon_rgba,
            project_manifest,
            project_symbol_catalog,
            has_unsaved_changes: true,
        }
    }

    pub fn get_name(&self) -> &str {
        &self.project_name
    }

    pub fn get_project_file_path(&self) -> &PathBuf {
        &self.project_file_path
    }

    pub fn get_project_directory(&self) -> Option<PathBuf> {
        self.project_file_path
            .parent()
            .map(|parent_path| parent_path.to_path_buf())
    }

    pub fn get_project_icon_rgba(&self) -> &Option<ProcessIcon> {
        &self.project_icon_rgba
    }

    pub fn set_project_icon(
        &mut self,
        project_icon: Option<ProcessIcon>,
    ) {
        self.project_icon_rgba = project_icon;
    }

    pub fn get_project_manifest(&self) -> &ProjectManifest {
        &self.project_manifest
    }

    pub fn get_project_manifest_mut(&mut self) -> &mut ProjectManifest {
        &mut self.project_manifest
    }

    pub fn get_project_symbol_catalog(&self) -> &ProjectSymbolCatalog {
        &self.project_symbol_catalog
    }

    pub fn get_project_symbol_catalog_mut(&mut self) -> &mut ProjectSymbolCatalog {
        &mut self.project_symbol_catalog
    }

    pub fn get_has_unsaved_changes(&self) -> bool {
        self.has_unsaved_changes
    }

    pub fn set_has_unsaved_changes(
        &mut self,
        has_unsaved_changes: bool,
    ) {
        self.has_unsaved_changes = has_unsaved_changes;
    }
}
