use std::path::PathBuf;

#[derive(Clone, PartialEq)]
pub enum ProjectSelectorFrameAction {
    None,
    SelectProject(PathBuf),
    OpenProject(PathBuf, String),
}
