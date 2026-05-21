use crate::{
    app_context::AppContext,
    ui::widgets::controls::{
        context_menu::context_menu::{ContextMenu, ContextMenuSizing},
        toolbar_menu::toolbar_menu_item_view::ToolbarMenuItemView,
    },
    views::project_explorer::project_hierarchy::{
        project_hierarchy_create_item_menu_view::ProjectHierarchyCreateItemMenuView,
        view_data::{
            project_hierarchy_frame_action::ProjectHierarchyFrameAction, project_hierarchy_menu_target::ProjectHierarchyMenuTarget,
            project_hierarchy_view_data::ProjectHierarchyViewData,
        },
    },
};
use eframe::egui::{Pos2, Ui};
use squalr_engine_api::dependency_injection::dependency::Dependency;
use std::sync::Arc;

pub struct ProjectHierarchyEmptySpaceContextMenuView<'lifetime> {
    app_context: Arc<AppContext>,
    project_hierarchy_view_data: Dependency<ProjectHierarchyViewData>,
    menu_target: Option<&'lifetime ProjectHierarchyMenuTarget>,
    menu_position: Option<Pos2>,
}

impl<'lifetime> ProjectHierarchyEmptySpaceContextMenuView<'lifetime> {
    const PROJECT_ITEM_CTX_PASTE_LABEL: &'static str = "Paste";
    const PROJECT_ITEM_CTX_PASTE_ID: &'static str = "project_hierarchy_empty_ctx_paste";

    pub fn new(
        app_context: Arc<AppContext>,
        project_hierarchy_view_data: Dependency<ProjectHierarchyViewData>,
        menu_target: Option<&'lifetime ProjectHierarchyMenuTarget>,
        menu_position: Option<Pos2>,
    ) -> Self {
        Self {
            app_context,
            project_hierarchy_view_data,
            menu_target,
            menu_position,
        }
    }

    pub fn show(
        self,
        user_interface: &mut Ui,
    ) -> Vec<ProjectHierarchyFrameAction> {
        let Some(ProjectHierarchyMenuTarget::EmptySpace { target_project_item_path }) = self.menu_target else {
            return Vec::new();
        };

        let Some(menu_position) = self.menu_position else {
            return Vec::new();
        };

        let mut frame_actions = Vec::new();
        let can_paste_project_items =
            ProjectHierarchyViewData::can_paste_project_item_clipboard(self.project_hierarchy_view_data.clone(), target_project_item_path);
        let mut project_item_menu_labels = ProjectHierarchyCreateItemMenuView::labels().to_vec();

        if can_paste_project_items {
            project_item_menu_labels.push(Self::PROJECT_ITEM_CTX_PASTE_LABEL);
        }

        let project_item_menu_width = ContextMenuSizing::width_for_labels(self.app_context.as_ref(), user_interface, project_item_menu_labels.iter().copied());
        let mut open = true;

        ContextMenu::new(
            self.app_context.clone(),
            "project_hierarchy_empty_space_context_menu",
            menu_position,
            |user_interface, should_close| {
                let create_project_item_action = ProjectHierarchyCreateItemMenuView::show_items(
                    self.app_context.clone(),
                    user_interface,
                    target_project_item_path,
                    project_item_menu_width,
                    should_close,
                );

                if create_project_item_action != ProjectHierarchyFrameAction::None {
                    frame_actions.push(create_project_item_action);
                }

                if can_paste_project_items {
                    user_interface.separator();

                    if user_interface
                        .add(
                            ToolbarMenuItemView::new(
                                self.app_context.clone(),
                                Self::PROJECT_ITEM_CTX_PASTE_LABEL,
                                Self::PROJECT_ITEM_CTX_PASTE_ID,
                                &None,
                                project_item_menu_width,
                            )
                            .icon(
                                self.app_context
                                    .theme
                                    .icon_library
                                    .icon_handle_data_type_unknown
                                    .clone(),
                            ),
                        )
                        .clicked()
                    {
                        frame_actions.push(ProjectHierarchyFrameAction::PasteProjectItems {
                            target_project_item_path: target_project_item_path.clone(),
                        });
                        *should_close = true;
                    }
                }
            },
        )
        .width(project_item_menu_width)
        .corner_radius(8)
        .show(user_interface, &mut open);

        if !open {
            ProjectHierarchyViewData::hide_menu(self.project_hierarchy_view_data.clone());
        }

        frame_actions
    }
}
