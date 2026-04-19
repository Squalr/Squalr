use std::path::{Path, PathBuf};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ProjectHierarchyClipboardMode {
    Copy,
    Cut,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ProjectHierarchyClipboard {
    project_file_path: Option<PathBuf>,
    project_item_paths: Vec<PathBuf>,
    mode: Option<ProjectHierarchyClipboardMode>,
}

impl ProjectHierarchyClipboard {
    pub fn clear(&mut self) {
        self.project_file_path = None;
        self.project_item_paths.clear();
        self.mode = None;
    }

    pub fn set(
        &mut self,
        project_file_path: Option<PathBuf>,
        project_item_paths: Vec<PathBuf>,
        mode: ProjectHierarchyClipboardMode,
    ) {
        self.project_file_path = project_file_path;
        self.project_item_paths = project_item_paths;
        self.mode = Some(mode);
    }

    pub fn retain_valid_paths(
        &mut self,
        valid_project_item_paths: &[PathBuf],
    ) {
        self.project_item_paths
            .retain(|project_item_path| valid_project_item_paths.contains(project_item_path));

        if self.project_item_paths.is_empty() {
            self.clear();
        }
    }

    pub fn update_path_prefix(
        &mut self,
        previous_project_item_path: &Path,
        renamed_project_item_path: &Path,
    ) {
        for project_item_path in self.project_item_paths.iter_mut() {
            if project_item_path == previous_project_item_path || project_item_path.starts_with(previous_project_item_path) {
                let relative_suffix = project_item_path
                    .strip_prefix(previous_project_item_path)
                    .unwrap_or(Path::new(""));
                *project_item_path = renamed_project_item_path.join(relative_suffix);
            }
        }
    }

    pub fn get_project_file_path(&self) -> Option<&PathBuf> {
        self.project_file_path.as_ref()
    }

    pub fn get_project_item_paths(&self) -> &[PathBuf] {
        &self.project_item_paths
    }

    pub fn get_mode(&self) -> Option<&ProjectHierarchyClipboardMode> {
        self.mode.as_ref()
    }

    pub fn is_cut(&self) -> bool {
        self.mode == Some(ProjectHierarchyClipboardMode::Cut)
    }

    pub fn is_empty(&self) -> bool {
        self.project_item_paths.is_empty() || self.mode.is_none()
    }
}
