use epaint::Pos2;
use std::path::PathBuf;

#[derive(Clone, PartialEq)]
pub enum ProjectSelectorFrameAction {
    None,
    ShowContextMenu(PathBuf, Pos2),
    HideContextMenu(),
    SelectProject(PathBuf),
    CancelRenamingProject(),
    StartRenamingProject(PathBuf, String),
    CommitRename(PathBuf, String),
    OpenProject(PathBuf, String),
    DeleteProject(PathBuf, String),
}
