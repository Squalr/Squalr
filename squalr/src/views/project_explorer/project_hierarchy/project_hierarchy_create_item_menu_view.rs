use crate::{
    app_context::AppContext,
    ui::widgets::controls::toolbar_menu::toolbar_menu_item_view::ToolbarMenuItemView,
    views::project_explorer::project_hierarchy::view_data::{
        project_hierarchy_create_item_kind::ProjectHierarchyCreateItemKind, project_hierarchy_frame_action::ProjectHierarchyFrameAction,
    },
};
use eframe::egui::Ui;
use std::path::Path;
use std::sync::Arc;

pub struct ProjectHierarchyCreateItemMenuView;

impl ProjectHierarchyCreateItemMenuView {
    pub const NEW_ADDRESS_LABEL: &'static str = "New Address";
    pub const NEW_FOLDER_LABEL: &'static str = "New Folder";
    const NEW_ADDRESS_ID: &'static str = "project_hierarchy_ctx_new_project_item";
    const NEW_FOLDER_ID: &'static str = "project_hierarchy_ctx_new_folder";

    pub fn labels() -> [&'static str; 2] {
        [Self::NEW_ADDRESS_LABEL, Self::NEW_FOLDER_LABEL]
    }

    pub fn show_items(
        app_context: Arc<AppContext>,
        user_interface: &mut Ui,
        target_project_item_path: &Path,
        project_item_menu_width: f32,
        should_close: &mut bool,
    ) -> ProjectHierarchyFrameAction {
        for (label, item_id, create_item_kind) in [
            (Self::NEW_ADDRESS_LABEL, Self::NEW_ADDRESS_ID, ProjectHierarchyCreateItemKind::Address),
            (Self::NEW_FOLDER_LABEL, Self::NEW_FOLDER_ID, ProjectHierarchyCreateItemKind::Directory),
        ] {
            if user_interface
                .add(
                    ToolbarMenuItemView::new(app_context.clone(), label, item_id, &None, project_item_menu_width).icon(match create_item_kind {
                        ProjectHierarchyCreateItemKind::Directory => app_context
                            .theme
                            .icon_library
                            .icon_handle_file_system_open_folder
                            .clone(),
                        ProjectHierarchyCreateItemKind::Address => app_context
                            .theme
                            .icon_library
                            .icon_handle_data_type_blue_blocks_4
                            .clone(),
                    }),
                )
                .clicked()
            {
                *should_close = true;

                return ProjectHierarchyFrameAction::CreateProjectItem {
                    target_project_item_path: target_project_item_path.to_path_buf(),
                    create_item_kind,
                };
            }
        }

        ProjectHierarchyFrameAction::None
    }
}
