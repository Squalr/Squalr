use std::path::PathBuf;

#[derive(Clone, PartialEq)]
pub enum ProjectSelectorFrameAction {
    None,
    SelectProject(PathBuf),
    CancelRenamingProject(),
    StartRenamingProject(PathBuf, String),
    CommitRename(PathBuf, String),
    OpenProject(PathBuf, String),
}
