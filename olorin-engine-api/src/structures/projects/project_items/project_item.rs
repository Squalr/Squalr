use crate::structures::{projects::project_items::project_item_type_ref::ProjectItemTypeRef, structs::valued_struct::ValuedStruct};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Represents a unique reference to a project item in an opened project.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProjectItem {
    /// The unique path to this project item.
    path: PathBuf,

    item_type: ProjectItemTypeRef,

    /// The container for all properties on this project item.
    properties: ValuedStruct,

    /// A value indicating whether this item has been activated / enabled.
    is_activated: bool,

    /// The child project items underneath this project item.
    children: Vec<ProjectItem>,

    /// A value indicating whether this project item accepts children.
    is_container_type: bool,

    /// A value indicating whether this project item has unsaved changes.
    has_unsaved_changes: bool,
}

impl ProjectItem {
    pub fn new(
        path: PathBuf,
        item_type: ProjectItemTypeRef,
        is_container_type: bool,
    ) -> Self {
        Self {
            path,
            item_type,
            properties: ValuedStruct::new_anonymous(vec![]),
            is_activated: false,
            children: vec![],
            is_container_type,
            has_unsaved_changes: true,
        }
    }

    pub fn get_path(&self) -> &PathBuf {
        &self.path
    }

    pub fn get_properties(&self) -> &ValuedStruct {
        &self.properties
    }

    pub fn get_properties_mut(&mut self) -> &mut ValuedStruct {
        &mut self.properties
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

    pub fn get_is_activated(&self) -> bool {
        self.is_activated
    }

    pub fn toggle_activated(&mut self) {
        self.is_activated = !self.is_activated
    }

    pub fn set_activated(
        &mut self,
        is_activated: bool,
    ) {
        self.is_activated = is_activated;
    }

    pub fn get_is_container_type(&self) -> bool {
        self.is_container_type
    }

    pub fn get_children(&self) -> &Vec<ProjectItem> {
        debug_assert!(self.is_container_type);

        &self.children
    }

    pub fn append_child(
        &mut self,
        new_child: ProjectItem,
    ) {
        debug_assert!(self.is_container_type);

        self.children.push(new_child);
    }

    pub fn get_children_mut(&mut self) -> &mut Vec<ProjectItem> {
        debug_assert!(self.is_container_type);

        &mut self.children
    }
}
