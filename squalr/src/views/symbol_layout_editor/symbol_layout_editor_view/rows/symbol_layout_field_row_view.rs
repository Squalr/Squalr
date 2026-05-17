use super::row_icon_button::render_row_icon_button_at;
use crate::app_context::AppContext;
use crate::ui::converters::data_type_to_icon_converter::DataTypeToIconConverter;
use crate::ui::draw::icon_draw::IconDraw;
use crate::ui::text::text_fitting::{measure_text_width, truncate_text_to_width};
use crate::ui::widgets::controls::state_layer::StateLayer;
use crate::views::symbol_layout_editor::view_data::symbol_layout_editor_view_data::{SymbolLayoutEditorViewData, SymbolLayoutFieldEditDraft};
use crate::views::symbol_layout_editor::view_data::symbol_layout_field_container_edit::SymbolLayoutFieldContainerKind;
use eframe::egui::{Align2, Color32, Rect, Response, Sense, TextureHandle, Ui, Widget, pos2, vec2};
use epaint::{CornerRadius, Stroke, StrokeKind};
use squalr_engine_api::dependency_injection::dependency::Dependency;
use squalr_engine_api::structures::data_values::container_type::ContainerType;
use squalr_engine_api::structures::projects::project_symbol_catalog::ProjectSymbolCatalog;
use squalr_engine_api::structures::structs::symbolic_struct_definition::SymbolicLayoutKind;
use std::sync::Arc;

use super::super::SymbolLayoutEditorView;
use super::symbol_layout_field_row_action::SymbolLayoutFieldRowAction;

pub(in crate::views::symbol_layout_editor::symbol_layout_editor_view) struct SymbolLayoutFieldRowView<'view> {
    app_context: Arc<AppContext>,
    symbol_layout_editor_view_data: Dependency<SymbolLayoutEditorViewData>,
    project_symbol_catalog: &'view ProjectSymbolCatalog,
    layout_kind: SymbolicLayoutKind,
    field_draft: &'view SymbolLayoutFieldEditDraft,
    field_index: usize,
    is_selected: bool,
    can_move_up: bool,
    can_move_down: bool,
    context_layout_id: Option<&'view str>,
    can_show_context_menu: bool,
}

impl<'view> SymbolLayoutFieldRowView<'view> {
    #[allow(clippy::too_many_arguments)]
    pub(in crate::views::symbol_layout_editor::symbol_layout_editor_view) fn new(
        app_context: Arc<AppContext>,
        symbol_layout_editor_view_data: Dependency<SymbolLayoutEditorViewData>,
        project_symbol_catalog: &'view ProjectSymbolCatalog,
        layout_kind: SymbolicLayoutKind,
        field_draft: &'view SymbolLayoutFieldEditDraft,
        field_index: usize,
        is_selected: bool,
        can_move_up: bool,
        can_move_down: bool,
        context_layout_id: Option<&'view str>,
        can_show_context_menu: bool,
    ) -> Self {
        Self {
            app_context,
            symbol_layout_editor_view_data,
            project_symbol_catalog,
            layout_kind,
            field_draft,
            field_index,
            is_selected,
            can_move_up,
            can_move_down,
            context_layout_id,
            can_show_context_menu,
        }
    }

    pub(in crate::views::symbol_layout_editor::symbol_layout_editor_view) fn format_data_type_preview(field_draft: &SymbolLayoutFieldEditDraft) -> String {
        let data_type_id = field_draft
            .data_type_selection
            .visible_data_type()
            .get_data_type_id()
            .trim();
        let container_suffix = match field_draft.container_edit.kind {
            SymbolLayoutFieldContainerKind::Element => String::new(),
            SymbolLayoutFieldContainerKind::Array => ContainerType::Array.to_string(),
            SymbolLayoutFieldContainerKind::FixedArray => {
                let fixed_array_length = field_draft.container_edit.fixed_array_length.trim();

                if fixed_array_length.is_empty() {
                    String::from("[?]")
                } else if !field_draft
                    .container_edit
                    .display_count_resolver_id
                    .trim()
                    .is_empty()
                {
                    format!(
                        "[{}] display resolver({})",
                        fixed_array_length,
                        field_draft.container_edit.display_count_resolver_id.trim()
                    )
                } else {
                    format!("[{}]", fixed_array_length)
                }
            }
            SymbolLayoutFieldContainerKind::DynamicArray => {
                let resolver_id = field_draft
                    .container_edit
                    .dynamic_array_count_resolver_id
                    .trim();

                if resolver_id.is_empty() {
                    ContainerType::Array.to_string()
                } else {
                    format!("[resolver({})]", resolver_id)
                }
            }
            SymbolLayoutFieldContainerKind::Pointer => ContainerType::Pointer(field_draft.container_edit.pointer_size).to_string(),
            SymbolLayoutFieldContainerKind::FixedPointerArray => {
                let fixed_array_length = field_draft.container_edit.fixed_array_length.trim();

                if fixed_array_length.is_empty() {
                    format!("*({})[?]", field_draft.container_edit.pointer_size)
                } else if !field_draft
                    .container_edit
                    .display_count_resolver_id
                    .trim()
                    .is_empty()
                {
                    format!(
                        "*({})[{}] display resolver({})",
                        field_draft.container_edit.pointer_size,
                        fixed_array_length,
                        field_draft.container_edit.display_count_resolver_id.trim()
                    )
                } else {
                    format!("*({})[{}]", field_draft.container_edit.pointer_size, fixed_array_length)
                }
            }
            SymbolLayoutFieldContainerKind::DynamicPointerArray => {
                let resolver_id = field_draft
                    .container_edit
                    .dynamic_array_count_resolver_id
                    .trim();

                if resolver_id.is_empty() {
                    format!("*({})[]", field_draft.container_edit.pointer_size)
                } else {
                    format!("*({})[resolver({})]", field_draft.container_edit.pointer_size, resolver_id)
                }
            }
        };

        format!("{}{}", data_type_id, container_suffix)
    }

    pub(in crate::views::symbol_layout_editor::symbol_layout_editor_view) fn show(
        self,
        user_interface: &mut Ui,
    ) -> Option<SymbolLayoutFieldRowAction> {
        let theme = &self.app_context.theme;
        let mut pending_field_row_action = None;

        let (row_rect, row_response) = user_interface.allocate_exact_size(
            vec2(user_interface.available_width().max(1.0), SymbolLayoutEditorView::LIST_ROW_HEIGHT),
            Sense::click(),
        );
        if self.is_selected {
            user_interface
                .painter()
                .rect_filled(row_rect, CornerRadius::ZERO, theme.selected_background);
            user_interface
                .painter()
                .rect_stroke(row_rect, CornerRadius::ZERO, Stroke::new(1.0, theme.selected_border), StrokeKind::Inside);
        }

        StateLayer {
            bounds_min: row_rect.min,
            bounds_max: row_rect.max,
            enabled: true,
            pressed: row_response.is_pointer_button_down_on(),
            has_hover: row_response.hovered(),
            has_focus: row_response.has_focus(),
            corner_radius: CornerRadius::ZERO,
            border_width: 0.0,
            hover_color: theme.hover_tint,
            pressed_color: theme.pressed_tint,
            border_color: theme.background_control_secondary_dark,
            border_color_focused: theme.background_control_secondary_dark,
        }
        .ui(user_interface);

        let button_area_width = SymbolLayoutEditorView::ICON_BUTTON_WIDTH * 2.0;
        let button_area_left = (row_rect.max.x - button_area_width).max(row_rect.min.x);
        let mut button_min_x = button_area_left;
        let mut render_next_button = |icon_handle: &TextureHandle, tooltip_text: &str, is_disabled: bool| -> Response {
            let button_rect = Rect::from_min_size(
                pos2(button_min_x, row_rect.min.y),
                vec2(SymbolLayoutEditorView::ICON_BUTTON_WIDTH, SymbolLayoutEditorView::LIST_ROW_HEIGHT),
            );
            button_min_x += SymbolLayoutEditorView::ICON_BUTTON_WIDTH;

            render_row_icon_button_at(&self.app_context, user_interface, button_rect, icon_handle, tooltip_text, is_disabled)
        };

        let entry_name = if self.layout_kind.is_union() { "variant" } else { "field" };
        let move_up_response = render_next_button(
            &theme.icon_library.icon_handle_navigation_up_arrow_small,
            &format!("Move this {} up.", entry_name),
            !self.can_move_up,
        );
        if move_up_response.clicked() {
            pending_field_row_action = Some(SymbolLayoutFieldRowAction::MoveUp);
        }

        let move_down_response = render_next_button(
            &theme.icon_library.icon_handle_navigation_down_arrow_small,
            &format!("Move this {} down.", entry_name),
            !self.can_move_down,
        );
        if move_down_response.clicked() {
            pending_field_row_action = Some(SymbolLayoutFieldRowAction::MoveDown);
        }

        let row_was_secondary_clicked = self.can_show_context_menu && row_response.secondary_clicked();
        if row_was_secondary_clicked {
            let context_menu_position = row_response
                .interact_pointer_pos()
                .unwrap_or_else(|| row_rect.left_bottom());
            SymbolLayoutEditorViewData::show_field_context_menu_for_layout(
                self.symbol_layout_editor_view_data.clone(),
                self.context_layout_id.map(str::to_string),
                self.field_index,
                context_menu_position,
            );
        } else if row_response.clicked() {
            SymbolLayoutEditorViewData::hide_field_context_menu(self.symbol_layout_editor_view_data.clone());
        }

        let trimmed_field_name = self.field_draft.field_name.trim();
        let field_name = if trimmed_field_name.is_empty() {
            String::from("<unnamed>")
        } else {
            trimmed_field_name.to_string()
        };
        let data_type_ref = self.field_draft.data_type_selection.visible_data_type();
        let is_symbol_layout = self
            .project_symbol_catalog
            .contains_struct_layout_id(data_type_ref.get_data_type_id());
        let data_type_icon =
            DataTypeToIconConverter::convert_data_type_or_symbol_layout_to_icon(data_type_ref.get_data_type_id(), is_symbol_layout, &theme.icon_library);
        let icon_size = vec2(SymbolLayoutEditorView::FIELD_ROW_ICON_SIZE, SymbolLayoutEditorView::FIELD_ROW_ICON_SIZE);
        let icon_rect = Rect::from_min_size(
            pos2(
                row_rect.min.x + SymbolLayoutEditorView::FIELD_ROW_LEFT_PADDING,
                row_rect.center().y - icon_size.y * 0.5,
            ),
            icon_size,
        );
        IconDraw::draw_sized_tinted(user_interface, icon_rect.center(), icon_size, &data_type_icon, Color32::WHITE);

        let preview_text = Self::format_data_type_preview(self.field_draft);
        let preview_right = button_area_left - SymbolLayoutEditorView::FIELD_ROW_LEFT_PADDING;
        let label_position = pos2(icon_rect.max.x + SymbolLayoutEditorView::FIELD_ROW_ICON_GAP, row_rect.center().y);
        let label_max_width = (preview_right - label_position.x).max(0.0);
        let label_text = truncate_text_to_width(
            user_interface,
            &field_name,
            &theme.font_library.font_noto_sans.font_normal,
            theme.foreground,
            label_max_width,
        );
        let label_width = measure_text_width(user_interface, &label_text, &theme.font_library.font_noto_sans.font_normal, theme.foreground);
        let preview_max_width = (preview_right - label_position.x - label_width - SymbolLayoutEditorView::FIELD_ROW_PREVIEW_GAP).max(0.0);
        let preview_text = truncate_text_to_width(
            user_interface,
            &preview_text,
            &theme.font_library.font_noto_sans.font_small,
            theme.foreground_preview,
            preview_max_width,
        );
        user_interface.painter().text(
            label_position,
            Align2::LEFT_CENTER,
            label_text,
            theme.font_library.font_noto_sans.font_normal.clone(),
            theme.foreground,
        );

        if !preview_text.is_empty() {
            user_interface.painter().text(
                pos2(preview_right, row_rect.center().y),
                Align2::RIGHT_CENTER,
                preview_text,
                theme.font_library.font_noto_sans.font_small.clone(),
                theme.foreground_preview,
            );
        }

        if row_response.clicked() && !row_was_secondary_clicked && pending_field_row_action.is_none() {
            pending_field_row_action = Some(SymbolLayoutFieldRowAction::SelectField);
        }

        pending_field_row_action
    }
}
