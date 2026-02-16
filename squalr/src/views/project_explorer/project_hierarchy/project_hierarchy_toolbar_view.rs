use crate::{
    app_context::AppContext,
    ui::{draw::icon_draw::IconDraw, widgets::controls::button::Button},
    views::project_explorer::project_hierarchy::view_data::{
        project_hierarchy_pending_operation::ProjectHierarchyPendingOperation, project_hierarchy_take_over_state::ProjectHierarchyTakeOverState,
        project_hierarchy_view_data::ProjectHierarchyViewData,
    },
    views::project_explorer::project_selector::view_data::project_selector_view_data::ProjectSelectorViewData,
};
use eframe::egui::{Align, Layout, Response, Sense, Ui, UiBuilder, Widget};
use epaint::{Color32, CornerRadius, vec2};
use squalr_engine_api::dependency_injection::dependency::Dependency;
use std::sync::Arc;

#[derive(Clone)]
pub struct ProjectHierarchyToolbarView {
    app_context: Arc<AppContext>,
    project_hierarchy_view_data: Dependency<ProjectHierarchyViewData>,
}

impl ProjectHierarchyToolbarView {
    pub fn new(app_context: Arc<AppContext>) -> Self {
        let project_hierarchy_view_data = app_context
            .dependency_container
            .get_dependency::<ProjectHierarchyViewData>();
        let instance = Self {
            app_context,
            project_hierarchy_view_data,
        };

        instance
    }
}

impl Widget for ProjectHierarchyToolbarView {
    fn ui(
        self,
        user_interface: &mut Ui,
    ) -> Response {
        let height = 28.0;
        let (allocated_size_rectangle, response) = user_interface.allocate_exact_size(vec2(user_interface.available_width(), height), Sense::empty());
        let theme = &self.app_context.theme;

        user_interface
            .painter()
            .rect_filled(allocated_size_rectangle, CornerRadius::ZERO, theme.background_primary);

        // Create a child ui constrained to the title bar.
        let builder = UiBuilder::new()
            .max_rect(allocated_size_rectangle)
            .layout(Layout::left_to_right(Align::Center));
        let mut toolbar_user_interface = user_interface.new_child(builder);

        toolbar_user_interface.with_layout(Layout::left_to_right(Align::Center), |user_interface| {
            let button_size = vec2(36.0, 28.0);
            let (has_selected_project_item, is_busy, has_take_over_state) = self
                .project_hierarchy_view_data
                .read("Project hierarchy toolbar state")
                .map(|project_hierarchy_view_data| {
                    (
                        !project_hierarchy_view_data
                            .selected_project_item_paths
                            .is_empty(),
                        project_hierarchy_view_data.pending_operation != ProjectHierarchyPendingOperation::None,
                        project_hierarchy_view_data.take_over_state != ProjectHierarchyTakeOverState::None,
                    )
                })
                .unwrap_or((false, false, false));

            // Close project.
            let button_refresh = user_interface.add_sized(
                button_size,
                Button::new_from_theme(&theme)
                    .with_tooltip_text("Close this project.")
                    .background_color(Color32::TRANSPARENT),
            );
            IconDraw::draw(user_interface, button_refresh.rect, &theme.icon_library.icon_handle_close);

            if button_refresh.clicked() {
                ProjectSelectorViewData::close_current_project(self.app_context.clone());
            }

            // Delete selected project item.
            let button_delete = user_interface.add_sized(
                button_size,
                Button::new_from_theme(&theme)
                    .with_tooltip_text("Delete selected project item.")
                    .background_color(Color32::TRANSPARENT)
                    .disabled(!has_selected_project_item || is_busy || has_take_over_state),
            );
            IconDraw::draw(user_interface, button_delete.rect, &theme.icon_library.icon_handle_common_delete);

            if button_delete.clicked() {
                ProjectHierarchyViewData::request_delete_confirmation_for_selected_project_item(self.project_hierarchy_view_data.clone());
            }
        });

        response
    }
}
