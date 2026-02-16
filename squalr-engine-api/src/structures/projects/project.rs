use crate::structures::{
    processes::process_icon::ProcessIcon,
    projects::{
        project_info::ProjectInfo,
        project_items::{project_item::ProjectItem, project_item_ref::ProjectItemRef},
        project_manifest::ProjectManifest,
    },
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents a full project in memory that can be serialized to the filesystem as distinct files, or exported as a single file.
#[derive(Serialize, Deserialize)]
pub struct Project {
    /// The metadata about this project.
    project_info: ProjectInfo,

    // The full list of all project items, indexed by their relative paths.
    project_items: HashMap<ProjectItemRef, ProjectItem>,

    /// The reference to the root project item, which is always a directory.
    #[serde(skip)]
    project_root_ref: ProjectItemRef,
}

impl Project {
    pub const PROJECT_FILE: &'static str = "project.json";
    pub const PROJECT_DIR: &'static str = "project_items";
    pub const PROJECT_ITEM_EXTENSION: &'static str = ".json";

    pub fn new(
        project_info: ProjectInfo,
        project_items: HashMap<ProjectItemRef, ProjectItem>,
        project_root_ref: ProjectItemRef,
    ) -> Self {
        Self {
            project_info,
            project_items,
            project_root_ref,
        }
    }

    pub fn get_name(&self) -> &str {
        self.project_info.get_name()
    }

    pub fn get_project_manifest(&self) -> &ProjectManifest {
        &self.project_info.get_project_manifest()
    }

    pub fn get_project_manifest_mut(&mut self) -> &mut ProjectManifest {
        self.project_info.get_project_manifest_mut()
    }

    pub fn get_project_info(&self) -> &ProjectInfo {
        &self.project_info
    }

    pub fn get_project_info_mut(&mut self) -> &mut ProjectInfo {
        &mut self.project_info
    }

    pub fn set_project_icon(
        &mut self,
        project_icon: Option<ProcessIcon>,
    ) {
        self.project_info.set_project_icon(project_icon);
    }

    pub fn get_project_items(&self) -> &HashMap<ProjectItemRef, ProjectItem> {
        &self.project_items
    }

    pub fn get_project_items_mut(&mut self) -> &mut HashMap<ProjectItemRef, ProjectItem> {
        &mut self.project_items
    }

    pub fn get_project_root(&self) -> Option<&ProjectItem> {
        self.project_items.get(&self.project_root_ref)
    }

    pub fn get_project_root_ref(&self) -> &ProjectItemRef {
        &self.project_root_ref
    }

    pub fn get_project_root_mut(&mut self) -> Option<&mut ProjectItem> {
        self.project_items.get_mut(&self.project_root_ref)
    }

    pub fn get_project_item_mut(
        &mut self,
        project_item_ref: &ProjectItemRef,
    ) -> Option<&mut ProjectItem> {
        self.project_items.get_mut(project_item_ref)
    }
}
