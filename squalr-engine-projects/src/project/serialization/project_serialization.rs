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

        let mut project = Project::new(project_info, project_items, project_root_ref);
        project.set_has_unsaved_changes(false);

        Ok(project)
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
    use crate::project::serialization::serializable_project_file::SerializableProjectFile;
    use squalr_engine_api::structures::data_types::built_in_types::u8::data_type_u8::DataTypeU8;
    use squalr_engine_api::structures::projects::project::Project;
    use squalr_engine_api::structures::projects::project_items::built_in_types::project_item_type_address::ProjectItemTypeAddress;
    use std::fs::{self, File};
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

    #[test]
    fn load_from_path_marks_project_and_items_clean() {
        let temp_directory = tempfile::tempdir().expect("Expected temporary project directory.");
        let project_directory_path = temp_directory.path();
        let project_items_directory_path = project_directory_path.join(Project::PROJECT_DIR);
        fs::create_dir_all(&project_items_directory_path).expect("Expected project items directory to be created.");
        fs::write(
            project_directory_path.join(Project::PROJECT_FILE),
            r#"{"icon":null,"manifest":{"sort_order":[]},"symbols":{}}"#,
        )
        .expect("Expected project info file to be written.");
        let project_item_path = project_items_directory_path.join("health.json");
        let project_item = ProjectItemTypeAddress::new_project_item("Health", 0x1234, "module", "", DataTypeU8::get_value_from_primitive(7));
        let project_item_file = File::create(project_item_path).expect("Expected project item file to be created.");
        serde_json::to_writer_pretty(project_item_file, &project_item).expect("Expected project item to be written.");

        let project = Project::load_from_path(project_directory_path).expect("Expected project to load.");

        assert!(!project.get_has_unsaved_changes());
        assert!(
            project
                .get_project_items()
                .values()
                .all(|project_item| !project_item.get_has_unsaved_changes())
        );
    }

    #[test]
    fn save_to_path_marks_written_project_items_clean() {
        let temp_directory = tempfile::tempdir().expect("Expected temporary project directory.");
        let project_directory_path = temp_directory.path();
        let project_items_directory_path = project_directory_path.join(Project::PROJECT_DIR);
        fs::create_dir_all(&project_items_directory_path).expect("Expected project items directory to be created.");
        fs::write(
            project_directory_path.join(Project::PROJECT_FILE),
            r#"{"icon":null,"manifest":{"sort_order":[]},"symbols":{}}"#,
        )
        .expect("Expected project info file to be written.");
        let project_item_path = project_items_directory_path.join("health.json");
        let project_item = ProjectItemTypeAddress::new_project_item("Health", 0x1234, "module", "", DataTypeU8::get_value_from_primitive(7));
        let project_item_file = File::create(&project_item_path).expect("Expected project item file to be created.");
        serde_json::to_writer_pretty(project_item_file, &project_item).expect("Expected project item to be written.");
        let mut project = Project::load_from_path(project_directory_path).expect("Expected project to load.");
        let project_item_ref = squalr_engine_api::structures::projects::project_items::project_item_ref::ProjectItemRef::new(project_item_path);
        project
            .get_project_item_mut(&project_item_ref)
            .expect("Expected project item to be available.")
            .set_has_unsaved_changes(true);

        project
            .save_to_path(project_directory_path, false)
            .expect("Expected project to save.");

        assert!(!project.get_has_unsaved_changes());
    }
}
