use std::path::PathBuf;

#[derive(Clone, PartialEq)]
pub enum ProjectSelectorFrameAction {
    None,
    OpenProject(PathBuf, String),
}
