use crate::{
    app_context::AppContext,
    ui::converters::data_type_to_icon_converter::DataTypeToIconConverter,
    views::project_explorer::project_hierarchy::{
        project_hierarchy_empty_space_context_menu_view::ProjectHierarchyEmptySpaceContextMenuView,
        project_hierarchy_project_item_context_menu_view::ProjectHierarchyProjectItemContextMenuView,
        project_item_entry_view::ProjectItemEntryView,
        project_item_inline_rename_view::ProjectItemInlineRenameView,
        view_data::{
            project_hierarchy_drop_target::ProjectHierarchyDropTarget, project_hierarchy_frame_action::ProjectHierarchyFrameAction,
            project_hierarchy_menu_target::ProjectHierarchyMenuTarget, project_hierarchy_tree_entry::ProjectHierarchyTreeEntry,
            project_hierarchy_view_data::ProjectHierarchyViewData,
        },
    },
};
use eframe::egui::{Id, Pos2, Rect, ScrollArea, TextureHandle, Ui};
use epaint::{CornerRadius, Stroke, StrokeKind};
use squalr_engine_api::dependency_injection::dependency::Dependency;
use squalr_engine_api::structures::projects::{
    project_info::ProjectInfo,
    project_items::{built_in_types::project_item_type_directory::ProjectItemTypeDirectory, details::ProjectItemDetailsProjection, project_item::ProjectItem},
};
use std::{
    collections::HashSet,
    path::{Path, PathBuf},
    sync::Arc,
};

pub struct ProjectHierarchyListView<'lifetime> {
    app_context: Arc<AppContext>,
    project_hierarchy_view_data: Dependency<ProjectHierarchyViewData>,
    opened_project_info: Option<&'lifetime ProjectInfo>,
    tree_entries: &'lifetime [ProjectHierarchyTreeEntry],
    selected_project_item_paths: &'lifetime HashSet<PathBuf>,
    selected_project_item_paths_in_tree_order: &'lifetime [PathBuf],
    dragged_project_item_paths: Option<Vec<PathBuf>>,
    active_struct_viewer_project_item_paths: &'lifetime HashSet<PathBuf>,
    active_inline_rename: Option<(PathBuf, String)>,
    menu_target: Option<&'lifetime ProjectHierarchyMenuTarget>,
    menu_position: Option<Pos2>,
    allow_interaction: bool,
}

#[derive(Default)]
pub struct ProjectHierarchyListResponse {
    pub actions: Vec<ProjectHierarchyListAction>,
    pub visible_preview_project_item_paths: Vec<PathBuf>,
}

pub enum ProjectHierarchyListAction {
    Frame(ProjectHierarchyFrameAction),
    DragStarted(PathBuf),
    HoveredDropTarget(ProjectHierarchyDropTarget),
    RenameSubmitted {
        project_item_path: PathBuf,
        project_item_type_id: String,
        edited_name: String,
    },
    CancelTakeOver,
}

impl<'lifetime> ProjectHierarchyListView<'lifetime> {
    const DROP_INSERTION_BAND_HEIGHT: f32 = 7.0;
    const PROJECT_ITEM_ROW_HEIGHT: f32 = 28.0;

    pub fn new(
        app_context: Arc<AppContext>,
        project_hierarchy_view_data: Dependency<ProjectHierarchyViewData>,
        opened_project_info: Option<&'lifetime ProjectInfo>,
        tree_entries: &'lifetime [ProjectHierarchyTreeEntry],
        selected_project_item_paths: &'lifetime HashSet<PathBuf>,
        selected_project_item_paths_in_tree_order: &'lifetime [PathBuf],
        dragged_project_item_paths: Option<Vec<PathBuf>>,
        active_struct_viewer_project_item_paths: &'lifetime HashSet<PathBuf>,
        active_inline_rename: Option<(PathBuf, String)>,
        menu_target: Option<&'lifetime ProjectHierarchyMenuTarget>,
        menu_position: Option<Pos2>,
    ) -> Self {
        let allow_interaction = active_inline_rename.is_none();

        Self {
            app_context,
            project_hierarchy_view_data,
            opened_project_info,
            tree_entries,
            selected_project_item_paths,
            selected_project_item_paths_in_tree_order,
            dragged_project_item_paths,
            active_struct_viewer_project_item_paths,
            active_inline_rename,
            menu_target,
            menu_position,
            allow_interaction,
        }
    }

    pub fn show(
        self,
        user_interface: &mut Ui,
    ) -> ProjectHierarchyListResponse {
        let mut list_response = ProjectHierarchyListResponse::default();
        let mut drag_started_project_item_path: Option<PathBuf> = None;
        let mut visible_row_rects = Vec::new();

        let scroll_area_output = ScrollArea::vertical()
            .id_salt("project_hierarchy")
            .auto_shrink([false, false])
            .show_rows(
                user_interface,
                Self::PROJECT_ITEM_ROW_HEIGHT,
                self.tree_entries.len(),
                |user_interface, visible_row_range| {
                    list_response.visible_preview_project_item_paths.extend(
                        self.tree_entries[visible_row_range.clone()]
                            .iter()
                            .map(|tree_entry| tree_entry.project_item_path.clone()),
                    );

                    for tree_entry in &self.tree_entries[visible_row_range] {
                        let icon = Self::resolve_tree_entry_icon(self.app_context.clone(), self.opened_project_info, &tree_entry.project_item);
                        let is_selected = self
                            .selected_project_item_paths
                            .contains(&tree_entry.project_item_path)
                            && self
                                .active_struct_viewer_project_item_paths
                                .contains(&tree_entry.project_item_path);
                        let is_inline_rename_row = self
                            .active_inline_rename
                            .as_ref()
                            .is_some_and(|(project_item_path, _)| project_item_path == &tree_entry.project_item_path);
                        let (row_response, should_request_rename, should_request_value_edit) = if is_inline_rename_row {
                            let Some((_, project_item_type_id)) = self.active_inline_rename.as_ref() else {
                                continue;
                            };

                            let inline_rename_response = Self::render_inline_rename_row(
                                &self.app_context,
                                user_interface,
                                tree_entry,
                                icon,
                                is_selected,
                                project_item_type_id,
                                &mut list_response,
                            );

                            (inline_rename_response, false, false)
                        } else {
                            let mut row_frame_action = ProjectHierarchyFrameAction::None;
                            let project_item_entry_view_response = ProjectItemEntryView::new(
                                self.app_context.clone(),
                                &tree_entry.project_item_path,
                                &tree_entry.display_name,
                                &tree_entry.preview_path,
                                &tree_entry.preview_value,
                                tree_entry.is_activated,
                                tree_entry.depth,
                                icon,
                                is_selected,
                                ProjectHierarchyViewData::is_cut_project_item_path(self.project_hierarchy_view_data.clone(), &tree_entry.project_item_path),
                                tree_entry.is_directory,
                                tree_entry.has_children,
                                tree_entry.is_expanded,
                                &mut row_frame_action,
                            )
                            .show(user_interface);

                            if row_frame_action != ProjectHierarchyFrameAction::None {
                                list_response
                                    .actions
                                    .push(ProjectHierarchyListAction::Frame(row_frame_action));
                            }

                            (
                                project_item_entry_view_response.row_response,
                                project_item_entry_view_response.should_request_rename,
                                project_item_entry_view_response.should_request_value_edit,
                            )
                        };
                        visible_row_rects.push(row_response.rect);

                        if !self.allow_interaction {
                            continue;
                        }

                        if should_request_rename {
                            list_response
                                .actions
                                .push(ProjectHierarchyListAction::Frame(ProjectHierarchyFrameAction::RequestRename(
                                    tree_entry.project_item_path.clone(),
                                )));
                        } else if should_request_value_edit {
                            list_response
                                .actions
                                .push(ProjectHierarchyListAction::Frame(ProjectHierarchyFrameAction::RequestValueEdit(
                                    tree_entry.project_item_path.clone(),
                                )));
                        }

                        if row_response.drag_started() {
                            drag_started_project_item_path = Some(tree_entry.project_item_path.clone());
                            list_response
                                .actions
                                .push(ProjectHierarchyListAction::DragStarted(tree_entry.project_item_path.clone()));
                        }

                        for frame_action in ProjectHierarchyProjectItemContextMenuView::new(
                            self.app_context.clone(),
                            self.project_hierarchy_view_data.clone(),
                            self.opened_project_info,
                            tree_entry,
                            &row_response,
                            self.selected_project_item_paths_in_tree_order,
                            self.menu_target,
                            self.menu_position,
                        )
                        .show(user_interface)
                        {
                            list_response
                                .actions
                                .push(ProjectHierarchyListAction::Frame(frame_action));
                        }
                        self.handle_project_item_drop_target(
                            user_interface,
                            tree_entry,
                            row_response.rect,
                            drag_started_project_item_path.as_ref(),
                            &mut list_response,
                        );
                    }
                },
            );

        if self.allow_interaction {
            self.handle_empty_space_interactions(
                user_interface,
                scroll_area_output.inner_rect,
                &visible_row_rects,
                drag_started_project_item_path.as_ref(),
                &mut list_response,
            );
        }

        list_response
    }

    pub fn project_item_rename_text_storage_id(project_item_path: &Path) -> Id {
        Id::new(("project_hierarchy_rename_text", project_item_path.to_path_buf()))
    }

    pub fn project_item_rename_highlight_storage_id(project_item_path: &Path) -> Id {
        Id::new(("project_hierarchy_rename_highlight", project_item_path.to_path_buf()))
    }

    pub fn clear_project_item_rename_state(
        user_interface: &Ui,
        project_item_path: &Path,
    ) {
        let rename_text_storage_id = Self::project_item_rename_text_storage_id(project_item_path);
        let rename_highlight_storage_id = Self::project_item_rename_highlight_storage_id(project_item_path);

        user_interface.ctx().data_mut(|data| {
            data.remove::<String>(rename_text_storage_id);
            data.remove::<bool>(rename_highlight_storage_id);
        });
    }

    pub fn clear_active_project_item_rename_state(
        user_interface: &Ui,
        project_hierarchy_view_data: Dependency<ProjectHierarchyViewData>,
    ) {
        let active_project_item_path = project_hierarchy_view_data
            .read("Project hierarchy resolve active inline rename state")
            .and_then(|project_hierarchy_view_data| match &project_hierarchy_view_data.take_over_state {
                crate::views::project_explorer::project_hierarchy::view_data::project_hierarchy_take_over_state::ProjectHierarchyTakeOverState::RenameProjectItem {
                    project_item_path,
                    ..
                } => Some(project_item_path.clone()),
                _ => None,
            });

        if let Some(active_project_item_path) = active_project_item_path {
            Self::clear_project_item_rename_state(user_interface, &active_project_item_path);
        }
    }

    fn render_inline_rename_row(
        app_context: &Arc<AppContext>,
        user_interface: &mut Ui,
        tree_entry: &ProjectHierarchyTreeEntry,
        icon: Option<TextureHandle>,
        is_selected: bool,
        project_item_type_id: &str,
        list_response: &mut ProjectHierarchyListResponse,
    ) -> eframe::egui::Response {
        let rename_text_storage_id = Self::project_item_rename_text_storage_id(&tree_entry.project_item_path);
        let rename_highlight_storage_id = Self::project_item_rename_highlight_storage_id(&tree_entry.project_item_path);
        let mut rename_text = user_interface
            .ctx()
            .data_mut(|data| data.get_temp::<String>(rename_text_storage_id))
            .unwrap_or_else(|| tree_entry.display_name.clone());
        let mut should_highlight_text = user_interface
            .ctx()
            .data_mut(|data| data.get_temp::<bool>(rename_highlight_storage_id))
            .unwrap_or(true);
        let inline_rename_response = ProjectItemInlineRenameView::new(
            app_context.clone(),
            &tree_entry.project_item_path,
            &mut rename_text,
            &mut should_highlight_text,
            tree_entry.is_activated,
            tree_entry.depth,
            icon,
            is_selected,
            tree_entry.is_directory,
            tree_entry.has_children,
            tree_entry.is_expanded,
        )
        .show(user_interface);

        if inline_rename_response.should_commit {
            list_response
                .actions
                .push(ProjectHierarchyListAction::RenameSubmitted {
                    project_item_path: tree_entry.project_item_path.clone(),
                    project_item_type_id: project_item_type_id.to_string(),
                    edited_name: rename_text.clone(),
                });
        }

        if inline_rename_response.should_cancel {
            list_response
                .actions
                .push(ProjectHierarchyListAction::CancelTakeOver);
        }

        user_interface.ctx().data_mut(|data| {
            data.insert_temp(rename_text_storage_id, rename_text);
            data.insert_temp(rename_highlight_storage_id, should_highlight_text);
        });

        inline_rename_response.row_response
    }

    fn handle_project_item_drop_target(
        &self,
        user_interface: &mut Ui,
        tree_entry: &ProjectHierarchyTreeEntry,
        row_rect: Rect,
        drag_started_project_item_path: Option<&PathBuf>,
        list_response: &mut ProjectHierarchyListResponse,
    ) {
        let active_dragged_project_item_paths = drag_started_project_item_path
            .map(|drag_started_project_item_path| vec![drag_started_project_item_path.clone()])
            .or_else(|| self.dragged_project_item_paths.clone());

        let Some(active_dragged_project_item_paths) = active_dragged_project_item_paths else {
            return;
        };

        let Some(pointer_position) = user_interface.input(|input_state| input_state.pointer.hover_pos()) else {
            return;
        };

        if !row_rect.contains(pointer_position) {
            return;
        }

        let Some(hovered_drop_target) = Self::resolve_drop_target(&active_dragged_project_item_paths, tree_entry, row_rect, pointer_position) else {
            return;
        };

        list_response
            .actions
            .push(ProjectHierarchyListAction::HoveredDropTarget(hovered_drop_target.clone()));
        Self::paint_drop_target_indicator(&self.app_context, user_interface, row_rect, &hovered_drop_target);
    }

    fn handle_empty_space_interactions(
        &self,
        user_interface: &mut Ui,
        scroll_area_rect: Rect,
        visible_row_rects: &[Rect],
        drag_started_project_item_path: Option<&PathBuf>,
        list_response: &mut ProjectHierarchyListResponse,
    ) {
        let pointer_position = user_interface.input(|input_state| input_state.pointer.hover_pos());
        let is_pointer_in_empty_space = pointer_position
            .filter(|pointer_position| scroll_area_rect.contains(*pointer_position))
            .map(|pointer_position| {
                !visible_row_rects
                    .iter()
                    .any(|row_rect| row_rect.contains(pointer_position))
            })
            .unwrap_or(false);

        if is_pointer_in_empty_space {
            self.handle_empty_space_drop_target(
                user_interface,
                scroll_area_rect,
                visible_row_rects,
                drag_started_project_item_path,
                list_response,
            );
        }

        let secondary_clicked_position = user_interface.input(|input_state| {
            if input_state.pointer.secondary_clicked() {
                input_state.pointer.hover_pos()
            } else {
                None
            }
        });

        if let Some(context_menu_position) = secondary_clicked_position.filter(|context_menu_position| scroll_area_rect.contains(*context_menu_position)) {
            let is_context_menu_position_empty = !visible_row_rects
                .iter()
                .any(|row_rect| row_rect.contains(context_menu_position));

            if is_context_menu_position_empty {
                if let Some(project_root_directory_path) = ProjectHierarchyViewData::get_project_root_directory_path(self.project_hierarchy_view_data.clone()) {
                    ProjectHierarchyViewData::show_empty_space_menu(
                        self.project_hierarchy_view_data.clone(),
                        project_root_directory_path,
                        context_menu_position,
                    );
                }
            }
        }

        for frame_action in ProjectHierarchyEmptySpaceContextMenuView::new(
            self.app_context.clone(),
            self.project_hierarchy_view_data.clone(),
            self.menu_target,
            self.menu_position,
        )
        .show(user_interface)
        {
            list_response
                .actions
                .push(ProjectHierarchyListAction::Frame(frame_action));
        }
    }

    fn handle_empty_space_drop_target(
        &self,
        user_interface: &mut Ui,
        scroll_area_rect: Rect,
        visible_row_rects: &[Rect],
        drag_started_project_item_path: Option<&PathBuf>,
        list_response: &mut ProjectHierarchyListResponse,
    ) {
        let active_dragged_project_item_paths = drag_started_project_item_path
            .map(|drag_started_project_item_path| vec![drag_started_project_item_path.clone()])
            .or_else(|| self.dragged_project_item_paths.clone());
        let Some(active_dragged_project_item_paths) = active_dragged_project_item_paths else {
            return;
        };
        let Some(drop_target) = self.resolve_empty_space_drop_target(&active_dragged_project_item_paths) else {
            return;
        };
        list_response
            .actions
            .push(ProjectHierarchyListAction::HoveredDropTarget(drop_target));

        Self::paint_empty_space_drop_target_indicator(&self.app_context, user_interface, scroll_area_rect, visible_row_rects);
    }

    fn resolve_empty_space_drop_target(
        &self,
        active_dragged_project_item_paths: &[PathBuf],
    ) -> Option<ProjectHierarchyDropTarget> {
        let project_root_directory_path = ProjectHierarchyViewData::get_project_root_directory_path(self.project_hierarchy_view_data.clone())?;

        if !Self::can_render_into_directory_drop_target(active_dragged_project_item_paths, &project_root_directory_path) {
            return None;
        }

        self.tree_entries
            .iter()
            .rev()
            .map(|tree_entry| &tree_entry.project_item_path)
            .find(|project_item_path| {
                project_item_path.parent() == Some(project_root_directory_path.as_path()) && !active_dragged_project_item_paths.contains(project_item_path)
            })
            .cloned()
            .map(ProjectHierarchyDropTarget::After)
            .or_else(|| Some(ProjectHierarchyDropTarget::Into(project_root_directory_path)))
    }

    fn resolve_drop_target(
        active_dragged_project_item_paths: &[PathBuf],
        tree_entry: &ProjectHierarchyTreeEntry,
        row_rect: Rect,
        pointer_position: Pos2,
    ) -> Option<ProjectHierarchyDropTarget> {
        if active_dragged_project_item_paths.contains(&tree_entry.project_item_path) {
            return None;
        }

        let insertion_band_height = Self::DROP_INSERTION_BAND_HEIGHT.min(row_rect.height() / 2.0);

        if pointer_position.y <= row_rect.top() + insertion_band_height
            && Self::can_render_insertion_drop_target(active_dragged_project_item_paths, &tree_entry.project_item_path)
        {
            return Some(ProjectHierarchyDropTarget::Before(tree_entry.project_item_path.clone()));
        }

        if pointer_position.y >= row_rect.bottom() - insertion_band_height
            && Self::can_render_insertion_drop_target(active_dragged_project_item_paths, &tree_entry.project_item_path)
        {
            return Some(ProjectHierarchyDropTarget::After(tree_entry.project_item_path.clone()));
        }

        if tree_entry.is_directory && Self::can_render_into_directory_drop_target(active_dragged_project_item_paths, &tree_entry.project_item_path) {
            return Some(ProjectHierarchyDropTarget::Into(tree_entry.project_item_path.clone()));
        }

        None
    }

    fn can_render_insertion_drop_target(
        active_dragged_project_item_paths: &[PathBuf],
        target_project_item_path: &Path,
    ) -> bool {
        let Some(target_directory_path) = target_project_item_path.parent() else {
            return false;
        };

        !active_dragged_project_item_paths.contains(&target_project_item_path.to_path_buf())
            && active_dragged_project_item_paths
                .iter()
                .all(|dragged_project_item_path| !target_directory_path.starts_with(dragged_project_item_path))
    }

    fn can_render_into_directory_drop_target(
        active_dragged_project_item_paths: &[PathBuf],
        target_project_item_path: &Path,
    ) -> bool {
        !active_dragged_project_item_paths
            .iter()
            .any(|dragged_project_item_path| target_project_item_path.starts_with(dragged_project_item_path))
    }

    fn paint_drop_target_indicator(
        app_context: &Arc<AppContext>,
        user_interface: &mut Ui,
        row_rect: Rect,
        drop_target: &ProjectHierarchyDropTarget,
    ) {
        let theme = &app_context.theme;

        match drop_target {
            ProjectHierarchyDropTarget::Into(_) => {
                user_interface
                    .painter()
                    .rect_filled(row_rect, CornerRadius::ZERO, theme.selected_background);
                user_interface
                    .painter()
                    .rect_stroke(row_rect, CornerRadius::ZERO, Stroke::new(1.0, theme.selected_border), StrokeKind::Inside);
            }
            ProjectHierarchyDropTarget::Before(_) | ProjectHierarchyDropTarget::After(_) => {
                let indicator_y = match drop_target {
                    ProjectHierarchyDropTarget::Before(_) => row_rect.top() + 0.5,
                    ProjectHierarchyDropTarget::After(_) => row_rect.bottom() - 0.5,
                    ProjectHierarchyDropTarget::Into(_) => row_rect.center().y,
                };
                let indicator_left = row_rect.left() + 8.0;
                let indicator_right = row_rect.right() - 8.0;
                let indicator_cap_half_height = 5.0;

                user_interface.painter().line_segment(
                    [
                        Pos2::new(indicator_left, indicator_y),
                        Pos2::new(indicator_right, indicator_y),
                    ],
                    Stroke::new(3.0, theme.selected_border),
                );
                user_interface.painter().line_segment(
                    [
                        Pos2::new(indicator_left, indicator_y - indicator_cap_half_height),
                        Pos2::new(indicator_left, indicator_y + indicator_cap_half_height),
                    ],
                    Stroke::new(3.0, theme.selected_border),
                );
                user_interface.painter().line_segment(
                    [
                        Pos2::new(indicator_right, indicator_y - indicator_cap_half_height),
                        Pos2::new(indicator_right, indicator_y + indicator_cap_half_height),
                    ],
                    Stroke::new(3.0, theme.selected_border),
                );
            }
        }
    }

    fn paint_empty_space_drop_target_indicator(
        app_context: &Arc<AppContext>,
        user_interface: &mut Ui,
        scroll_area_rect: Rect,
        visible_row_rects: &[Rect],
    ) {
        let theme = &app_context.theme;
        let empty_space_top = visible_row_rects
            .iter()
            .map(Rect::bottom)
            .fold(scroll_area_rect.top(), f32::max)
            .min(scroll_area_rect.bottom());
        let empty_space_rect = Rect::from_min_max(
            Pos2::new(scroll_area_rect.left(), empty_space_top),
            Pos2::new(scroll_area_rect.right(), scroll_area_rect.bottom()),
        );

        if empty_space_rect.is_positive() {
            user_interface
                .painter()
                .rect_filled(empty_space_rect, CornerRadius::ZERO, theme.selected_background);
            user_interface.painter().rect_stroke(
                empty_space_rect,
                CornerRadius::ZERO,
                Stroke::new(1.0, theme.selected_border),
                StrokeKind::Inside,
            );
        }
    }

    fn resolve_tree_entry_icon(
        app_context: Arc<AppContext>,
        opened_project_info: Option<&ProjectInfo>,
        project_item: &ProjectItem,
    ) -> Option<TextureHandle> {
        let icon_library = &app_context.theme.icon_library;
        let project_item_type_id = project_item.get_item_type().get_project_item_type_id();

        if project_item_type_id == ProjectItemTypeDirectory::PROJECT_ITEM_TYPE_ID {
            Some(icon_library.icon_handle_file_system_open_folder.clone())
        } else {
            let icon_data_type_id = ProjectItemDetailsProjection::resolve_project_item_icon_data_type_id(project_item).unwrap_or_default();
            let is_symbol_layout = opened_project_info.is_some_and(|project_info| {
                project_info
                    .get_project_symbol_catalog()
                    .contains_struct_layout_id(&icon_data_type_id)
            });

            Some(DataTypeToIconConverter::convert_data_type_or_symbol_layout_to_icon(
                &icon_data_type_id,
                is_symbol_layout,
                icon_library,
            ))
        }
    }
}
