use crate::project::serialization::serializable_project_file::SerializableProjectFile;
use squalr_engine_api::structures::projects::{
    project::Project,
    project_info::ProjectInfo,
    project_items::{built_in_types::project_item_type_directory::ProjectItemTypeDirectory, project_item::ProjectItem, project_item_ref::ProjectItemRef},
};
use std::{collections::HashMap, path::Path};

impl SerializableProjectFile for Project {
    fn save_to_path(
        &mut self,
        directory: &Path,
        save_even_if_unchanged: bool,
    ) -> Result<(), anyhow::Error> {
        // Save the main project info file.
        self.get_project_info_mut()
            .save_to_path(directory, save_even_if_unchanged)?;

        // Save all project items.
        for project_item_pair in self.get_project_items_mut() {
            let project_item_ref = project_item_pair.0;
            let project_item = project_item_pair.1;
            let project_item_path = project_item_ref.get_project_item_path();

            if project_item_path.is_file() {
                if let Err(error) = project_item.save_to_path(project_item_path, save_even_if_unchanged) {
                    log::error!("Failed to serialize project item: {}", error)
                }
            }
        }

        Ok(())
    }

    fn load_from_path(project_directory_path: &Path) -> anyhow::Result<Self> {
        let project_info = ProjectInfo::load_from_path(&project_directory_path.join(Project::PROJECT_FILE))?;
        let mut project_items = HashMap::new();
        let project_root_directory_path = resolve_project_root_directory_path(project_directory_path);
        let project_root_ref = ProjectItemRef::new(project_root_directory_path.clone());
        let project_root = ProjectItemTypeDirectory::new_project_item(&project_root_ref);

        project_items.insert(project_root_ref.clone(), project_root);

        fn load_recursive(
            current_path: &Path,
            project_items: &mut HashMap<ProjectItemRef, ProjectItem>,
        ) -> anyhow::Result<()> {
            for entry in std::fs::read_dir(current_path)? {
                let entry = entry?;
                let entry_path = entry.path();
                if entry_path.is_dir() {
                    let dir_ref = ProjectItemRef::new(entry_path.clone());
                    let dir_item = ProjectItemTypeDirectory::new_project_item(&dir_ref);

                    project_items.insert(dir_ref.clone(), dir_item);

                    load_recursive(&entry_path, project_items)?;
                } else if is_project_item_file_path(&entry_path) {
                    let item_ref = ProjectItemRef::new(entry_path.clone());
                    let project_item = ProjectItem::load_from_path(&entry_path)?;

                    project_items.insert(item_ref, project_item);
                } else {
                    log::debug!("Skipping non-project item during deserialization: {:?}", entry_path)
                }
            }

            Ok(())
        }

        if !project_root_directory_path.exists() {
            std::fs::create_dir_all(&project_root_directory_path)?;
        }

        load_recursive(&project_root_directory_path, &mut project_items)?;

        Ok(Project::new(project_info, project_items, project_root_ref))
    }
}

fn resolve_project_root_directory_path(project_directory_path: &Path) -> std::path::PathBuf {
    project_directory_path.join(Project::PROJECT_DIR)
}

fn is_project_item_file_path(project_item_path: &Path) -> bool {
    let expected_extension = Project::PROJECT_ITEM_EXTENSION.trim_start_matches('.');

    project_item_path
        .extension()
        .and_then(|extension| extension.to_str())
        .map(|extension| extension.eq_ignore_ascii_case(expected_extension))
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::is_project_item_file_path;
    use std::path::PathBuf;

    #[test]
    fn is_project_item_file_path_accepts_json_extension() {
        let project_item_path = PathBuf::from("C:/Projects/TestProject/project_items/Addresses/health.json");

        assert!(is_project_item_file_path(&project_item_path));
    }

    #[test]
    fn is_project_item_file_path_rejects_non_json_extension() {
        let project_item_path = PathBuf::from("C:/Projects/TestProject/project_items/Addresses/notes.txt");

        assert!(!is_project_item_file_path(&project_item_path));
    }
}
