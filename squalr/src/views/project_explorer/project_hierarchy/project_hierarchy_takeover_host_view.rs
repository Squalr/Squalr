use crate::app_context::AppContext;
use crate::ui::{geometry::safe_clamp_f32, widgets::controls::groupbox::GroupBox};
use crate::views::project_explorer::project_hierarchy::{
    project_item_details::ProjectItemDetails,
    project_item_value_edit_take_over_view::ProjectItemValueEditTakeOverView,
    view_data::{project_hierarchy_take_over_state::ProjectHierarchyTakeOverState, project_hierarchy_tree_entry::ProjectHierarchyTreeEntry},
};
use eframe::egui::{Align, Button, Direction, Id, Layout, RichText, ScrollArea, Ui, vec2};
use epaint::Stroke;
use squalr_engine_api::commands::project_items::promote_symbol::project_items_promote_symbol_response::ProjectItemsPromoteSymbolConflict;
use squalr_engine_api::structures::data_types::data_type_ref::DataTypeRef;
use squalr_engine_api::structures::data_values::anonymous_value_string::AnonymousValueString;
use squalr_engine_api::structures::projects::project_info::ProjectInfo;
use std::path::{Path, PathBuf};
use std::sync::Arc;

pub enum ProjectHierarchyTakeoverHostAction {
    None,
    Cancel,
    DeleteProjectItems(Vec<PathBuf>),
    PromoteSymbolOverwrite(Vec<PathBuf>),
    SubmitValueEdit {
        project_item_path: PathBuf,
        value_field_name: String,
        validation_data_type_ref: DataTypeRef,
        value_edit: AnonymousValueString,
    },
}

pub struct ProjectHierarchyTakeoverHostView<'a> {
    app_context: Arc<AppContext>,
    opened_project_info: Option<&'a ProjectInfo>,
    tree_entries: &'a [ProjectHierarchyTreeEntry],
    take_over_state: &'a ProjectHierarchyTakeOverState,
}

impl<'a> ProjectHierarchyTakeoverHostView<'a> {
    pub fn new(
        app_context: Arc<AppContext>,
        opened_project_info: Option<&'a ProjectInfo>,
        tree_entries: &'a [ProjectHierarchyTreeEntry],
        take_over_state: &'a ProjectHierarchyTakeOverState,
    ) -> Self {
        Self {
            app_context,
            opened_project_info,
            tree_entries,
            take_over_state,
        }
    }

    pub fn show(
        self,
        user_interface: &mut Ui,
    ) -> ProjectHierarchyTakeoverHostAction {
        match self.take_over_state {
            ProjectHierarchyTakeOverState::EditProjectItemValue { project_item_path } => self.show_value_edit(user_interface, project_item_path),
            ProjectHierarchyTakeOverState::DeleteConfirmation { project_item_paths } => self.show_delete_confirmation(user_interface, project_item_paths),
            ProjectHierarchyTakeOverState::PromoteSymbolConflict { project_item_paths, conflicts } => {
                self.show_promote_symbol_conflicts(user_interface, project_item_paths, conflicts)
            }
            ProjectHierarchyTakeOverState::None | ProjectHierarchyTakeOverState::RenameProjectItem { .. } => ProjectHierarchyTakeoverHostAction::None,
        }
    }

    pub fn project_item_value_edit_storage_id(project_item_path: &Path) -> Id {
        Id::new(("project_hierarchy_value_edit", project_item_path.to_path_buf()))
    }

    pub fn clear_project_item_value_edit_state(
        user_interface: &Ui,
        project_item_path: &Path,
    ) {
        let value_edit_storage_id = Self::project_item_value_edit_storage_id(project_item_path);

        user_interface.ctx().data_mut(|data| {
            data.remove::<AnonymousValueString>(value_edit_storage_id);
        });
    }

    fn show_value_edit(
        &self,
        user_interface: &mut Ui,
        project_item_path: &PathBuf,
    ) -> ProjectHierarchyTakeoverHostAction {
        let project_item = self
            .tree_entries
            .iter()
            .find(|tree_entry| tree_entry.project_item_path == *project_item_path)
            .map(|tree_entry| tree_entry.project_item.clone());
        let Some(project_item) = project_item else {
            return ProjectHierarchyTakeoverHostAction::Cancel;
        };
        let Some(value_edit_context) =
            ProjectItemDetails::build_project_item_value_edit_context(&self.app_context.engine_unprivileged_state, self.opened_project_info, &project_item)
        else {
            return ProjectHierarchyTakeoverHostAction::Cancel;
        };
        let value_edit_storage_id = Self::project_item_value_edit_storage_id(project_item_path);
        let mut value_edit = user_interface
            .ctx()
            .data_mut(|data| data.get_temp::<AnonymousValueString>(value_edit_storage_id))
            .unwrap_or_else(|| value_edit_context.initial_value_edit.clone());
        let value_edit_display_values = ProjectItemDetails::build_project_item_value_edit_display_values(
            &self.app_context.engine_unprivileged_state,
            &value_edit_context.validation_data_type_ref,
            &value_edit,
        );
        let value_editor_id = format!("project_hierarchy_value_editor_{}", project_item_path.to_string_lossy());
        let panel_width = safe_clamp_f32(user_interface.available_width(), 320.0, 560.0);
        let mut action = ProjectHierarchyTakeoverHostAction::None;

        user_interface.add_space(12.0);
        user_interface.horizontal(|user_interface| {
            let side_spacing = ((user_interface.available_width() - panel_width) * 0.5).max(0.0);
            user_interface.add_space(side_spacing);
            user_interface.allocate_ui_with_layout(vec2(panel_width, 0.0), Layout::top_down(Align::Min), |user_interface| {
                let value_edit_take_over_response = ProjectItemValueEditTakeOverView::new(
                    self.app_context.clone(),
                    &value_edit_context.project_item_name,
                    &mut value_edit,
                    &value_edit_context.validation_data_type_ref,
                    &value_edit_display_values,
                    &value_editor_id,
                )
                .show(user_interface);

                if value_edit_take_over_response.should_commit {
                    action = ProjectHierarchyTakeoverHostAction::SubmitValueEdit {
                        project_item_path: project_item_path.clone(),
                        value_field_name: value_edit_context.value_field_name.clone(),
                        validation_data_type_ref: value_edit_context.validation_data_type_ref.clone(),
                        value_edit: value_edit.clone(),
                    };
                }

                if value_edit_take_over_response.should_cancel {
                    action = ProjectHierarchyTakeoverHostAction::Cancel;
                }
            });
        });

        user_interface
            .ctx()
            .data_mut(|data| data.insert_temp(value_edit_storage_id, value_edit));

        action
    }

    fn show_delete_confirmation(
        &self,
        user_interface: &mut Ui,
        project_item_paths: &[PathBuf],
    ) -> ProjectHierarchyTakeoverHostAction {
        let theme = &self.app_context.theme;
        let mut action = ProjectHierarchyTakeoverHostAction::None;

        user_interface.allocate_ui_with_layout(
            user_interface.available_size(),
            Layout::centered_and_justified(Direction::TopDown),
            |user_interface| {
                let groupbox_side_padding = 8.0;
                let panel_width = (user_interface.available_width() - groupbox_side_padding * 2.0).max(0.0);

                user_interface.horizontal(|user_interface| {
                    user_interface.add_space(groupbox_side_padding);
                    user_interface.add(
                        GroupBox::new_from_theme(theme, "Delete project item(s)", |user_interface| {
                            ScrollArea::vertical()
                                .id_salt("project_hierarchy_delete_confirmation")
                                .max_height(160.0)
                                .auto_shrink([false, false])
                                .show(user_interface, |user_interface| {
                                    user_interface.vertical_centered(|user_interface| {
                                        for project_item_path in project_item_paths {
                                            let project_item_name = project_item_path
                                                .file_name()
                                                .and_then(|value| value.to_str())
                                                .unwrap_or_default();
                                            user_interface.label(
                                                RichText::new(project_item_name)
                                                    .font(theme.font_library.font_ubuntu_mono_bold.font_header.clone())
                                                    .color(theme.foreground),
                                            );
                                        }
                                    });
                                });

                            user_interface.add_space(12.0);
                            self.show_binary_action_buttons(user_interface, "Delete", true, |button_action| {
                                action = button_action.map_or(ProjectHierarchyTakeoverHostAction::Cancel, |_| {
                                    ProjectHierarchyTakeoverHostAction::DeleteProjectItems(project_item_paths.to_vec())
                                });
                            });
                        })
                        .desired_width(panel_width),
                    );
                });
            },
        );

        action
    }

    fn show_promote_symbol_conflicts(
        &self,
        user_interface: &mut Ui,
        project_item_paths: &[PathBuf],
        conflicts: &[ProjectItemsPromoteSymbolConflict],
    ) -> ProjectHierarchyTakeoverHostAction {
        let theme = &self.app_context.theme;
        let mut action = ProjectHierarchyTakeoverHostAction::None;

        user_interface.add_space(12.0);
        user_interface.vertical_centered(|user_interface| {
            user_interface.label(
                RichText::new("Overwrite conflicting symbol claim(s)?")
                    .font(theme.font_library.font_noto_sans.font_normal.clone())
                    .color(theme.foreground),
            );
        });
        user_interface.add_space(8.0);

        ScrollArea::vertical()
            .id_salt("project_hierarchy_promote_symbol_conflicts")
            .max_height(180.0)
            .auto_shrink([false, false])
            .show(user_interface, |user_interface| {
                user_interface.vertical(|user_interface| {
                    for conflict in conflicts {
                        user_interface.label(
                            RichText::new(format!(
                                "{} -> {} ({})",
                                conflict.requested_display_name, conflict.symbol_locator_key, conflict.existing_locator_display
                            ))
                            .font(theme.font_library.font_ubuntu_mono_bold.font_normal.clone())
                            .color(theme.foreground),
                        );
                    }
                });
            });

        user_interface.add_space(8.0);
        self.show_binary_action_buttons(user_interface, "Overwrite", false, |button_action| {
            action = button_action.map_or(ProjectHierarchyTakeoverHostAction::Cancel, |_| {
                ProjectHierarchyTakeoverHostAction::PromoteSymbolOverwrite(project_item_paths.to_vec())
            });
        });

        action
    }

    fn show_binary_action_buttons(
        &self,
        user_interface: &mut Ui,
        confirm_label: &str,
        confirm_is_dangerous: bool,
        mut set_action: impl FnMut(Option<()>),
    ) {
        let theme = &self.app_context.theme;

        user_interface.allocate_ui(vec2(user_interface.available_width(), 32.0), |user_interface| {
            let button_size = vec2(120.0, 28.0);
            let button_spacing = 12.0;
            let total_button_row_width = button_size.x * 2.0 + button_spacing;
            let side_spacing = ((user_interface.available_width() - total_button_row_width) * 0.5).max(0.0);

            user_interface.horizontal(|user_interface| {
                user_interface.add_space(side_spacing);
                user_interface.spacing_mut().item_spacing.x = button_spacing;

                let button_cancel = user_interface.add_sized(
                    button_size,
                    Button::new(RichText::new("Cancel").color(theme.foreground))
                        .fill(theme.background_control_secondary)
                        .stroke(Stroke::new(1.0, theme.background_control_secondary_dark)),
                );

                if button_cancel.clicked() {
                    set_action(None);
                }

                let (confirm_fill, confirm_stroke) = if confirm_is_dangerous {
                    (theme.background_control_danger, theme.background_control_danger_dark)
                } else {
                    (theme.background_control_secondary, theme.background_control_secondary_dark)
                };
                let button_confirm = user_interface.add_sized(
                    button_size,
                    Button::new(RichText::new(confirm_label).color(theme.foreground))
                        .fill(confirm_fill)
                        .stroke(Stroke::new(1.0, confirm_stroke)),
                );

                if button_confirm.clicked() {
                    set_action(Some(()));
                }
            });
        });
    }
}
