use olorin_engine_api::structures::projects::project_items::project_item_ref::ProjectItemRef;
use serde::{Deserialize, Serialize};

/// Represents a unique reference to a project item in an opened project.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ProjectTree {
    /// The identifier for the project item.
    project_item_ref: ProjectItemRef,

    /// The child project items underneath the referenced project item.
    children: Vec<ProjectTree>,

    /// A value indicating whether the referenced project item accepts children.
    is_container: bool,

    /// A value indicating whether the referenced project item has unsaved changes.
    has_unsaved_changes: bool,
}

impl ProjectTree {
    pub fn new(
        project_item_ref: ProjectItemRef,
        is_container: bool,
    ) -> Self {
        Self {
            project_item_ref,
            children: vec![],
            is_container,
            has_unsaved_changes: true,
        }
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

    pub fn get_children(&self) -> &Vec<ProjectTree> {
        debug_assert!(self.is_container);

        &self.children
    }

    pub fn append_child(
        &mut self,
        new_child: ProjectTree,
    ) {
        debug_assert!(self.is_container);

        self.children.push(new_child);
    }

    pub fn get_children_mut(&mut self) -> &mut Vec<ProjectTree> {
        debug_assert!(self.is_container);

        &mut self.children
    }
}
