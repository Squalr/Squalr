use crate::ProjectItemViewData;
use olorin_engine_api::structures::projects::project_items::project_item::ProjectItem;
use slint::ToSharedString;
use slint_mvvm::convert_to_view_data::ConvertToViewData;

pub struct ProjectItemConverter {}

impl ProjectItemConverter {
    pub fn new() -> Self {
        Self {}
    }

    fn convert_to_view_data_with_indentation(
        &self,
        project_item: &ProjectItem,
        indentation: i32,
    ) -> ProjectItemViewData {
        ProjectItemViewData {
            name: project_item.get_field_name().to_shared_string(),
            path: project_item.get_path().to_string_lossy().to_shared_string(),
            indentation,
            is_checked: project_item.get_is_activated(),
        }
    }
}

impl ConvertToViewData<ProjectItem, ProjectItemViewData> for ProjectItemConverter {
    fn convert_collection(
        &self,
        project_item_list: &Vec<ProjectItem>,
    ) -> Vec<ProjectItemViewData> {
        fn flatten_items(
            converter: &ProjectItemConverter,
            items: &Vec<ProjectItem>,
            indentation: i32,
        ) -> Vec<ProjectItemViewData> {
            items
                .iter()
                .flat_map(|item| {
                    let mut results = vec![converter.convert_to_view_data_with_indentation(item, indentation)];
                    if item.get_is_container() {
                        let children = item.get_children();

                        if !children.is_empty() {
                            results.extend(flatten_items(converter, &children, indentation + 1));
                        }
                    }

                    results
                })
                .collect()
        }

        flatten_items(self, project_item_list, 0)
    }

    fn convert_to_view_data(
        &self,
        project_item: &ProjectItem,
    ) -> ProjectItemViewData {
        ProjectItemViewData {
            name: project_item.get_field_name().to_shared_string(),
            path: project_item.get_path().to_string_lossy().to_shared_string(),
            indentation: 0,
            is_checked: project_item.get_is_activated(),
        }
    }
}
