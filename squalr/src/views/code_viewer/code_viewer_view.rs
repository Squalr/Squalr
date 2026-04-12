use crate::{
    app_context::AppContext,
    ui::{
        draw::icon_draw::IconDraw,
        widgets::controls::{
            button::Button, context_menu::context_menu::ContextMenu, data_value_box::data_value_box_view::DataValueBoxView,
            toolbar_menu::toolbar_menu_item_view::ToolbarMenuItemView,
        },
    },
    views::{
        code_viewer::{
            code_viewer_footer_view::CodeViewerFooterView,
            view_data::code_viewer_view_data::{
                CodeViewerInstructionEditState, CodeViewerInstructionEditStatus, CodeViewerInstructionWritePlan, CodeViewerViewData,
            },
        },
        process_selector::view_data::process_selector_view_data::ProcessSelectorViewData,
        project_explorer::project_hierarchy::view_data::project_hierarchy_view_data::ProjectHierarchyViewData,
    },
};
use eframe::egui::{
    Align, Align2, Color32, CursorIcon, Direction, Key, Layout, Pos2, Rect, Response, RichText, ScrollArea, Sense, Spinner, Stroke, Ui, UiBuilder, Widget,
    pos2, vec2,
};
use epaint::{Color32 as EpaintColor32, CornerRadius};
use squalr_engine_api::{
    commands::privileged_command_request::PrivilegedCommandRequest,
    commands::unprivileged_command_request::UnprivilegedCommandRequest,
    dependency_injection::dependency::Dependency,
    events::process::changed::process_changed_event::ProcessChangedEvent,
    structures::{
        data_types::{built_in_types::u64::data_type_u64::DataTypeU64, data_type_ref::DataTypeRef},
        memory::bitness::Bitness,
    },
};
use squalr_plugin_instructions_x86::DisassembledInstruction;
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

#[derive(Clone)]
pub struct CodeViewerView {
    app_context: Arc<AppContext>,
    code_viewer_view_data: Dependency<CodeViewerViewData>,
    process_selector_view_data: Dependency<ProcessSelectorViewData>,
    project_hierarchy_view_data: Dependency<ProjectHierarchyViewData>,
    code_viewer_footer_view: CodeViewerFooterView,
}

#[derive(Clone, Copy)]
struct CodeViewerColumnLayout {
    breakpoint_gutter_rect: Rect,
    address_rect: Rect,
    bytes_rect: Rect,
    text_rect: Rect,
}

impl CodeViewerView {
    const GO_TO_ADDRESS_INPUT_ID: &'static str = "code_viewer_go_to_address";
    const INSTRUCTION_EDIT_INPUT_ID: &'static str = "code_viewer_instruction_edit";
    pub const WINDOW_ID: &'static str = "window_code_viewer";
    const COLUMN_SEPARATOR_THICKNESS: f32 = 3.0;
    const TOOLBAR_HEIGHT: f32 = 32.0;
    const TOOLBAR_ROW_HEIGHT: f32 = 28.0;
    const ROW_HEIGHT: f32 = 22.0;
    const EDIT_WARNING_ROW_HEIGHT: f32 = 26.0;
    const BREAKPOINT_GUTTER_WIDTH: f32 = 28.0;
    const BRANCH_GUTTER_WIDTH: f32 = 56.0;
    const ADDRESS_COLUMN_WIDTH: f32 = 118.0;
    const MINIMUM_BYTES_COLUMN_WIDTH: f32 = 72.0;
    const MINIMUM_TEXT_COLUMN_WIDTH: f32 = 180.0;
    const TEXT_LEFT_PADDING: f32 = 6.0;
    const ROW_TEXT_TOP_PADDING: f32 = 4.0;
    const BREAKPOINT_RADIUS: f32 = 5.0;
    const BRANCH_LANE_SPACING: f32 = 8.0;
    const BRANCH_LANE_RIGHT_PADDING: f32 = 8.0;
    const MAX_BRANCH_LANES: usize = 5;
    const CONTEXT_MENU_WIDTH: f32 = 220.0;

    pub fn new(app_context: Arc<AppContext>) -> Self {
        let code_viewer_view_data = app_context
            .dependency_container
            .register(CodeViewerViewData::new());
        let process_selector_view_data = app_context
            .dependency_container
            .get_dependency::<ProcessSelectorViewData>();
        let project_hierarchy_view_data = app_context
            .dependency_container
            .get_dependency::<ProjectHierarchyViewData>();
        let instance = Self {
            code_viewer_footer_view: CodeViewerFooterView::new(app_context.clone()),
            process_selector_view_data,
            project_hierarchy_view_data,
            code_viewer_view_data,
            app_context,
        };

        CodeViewerViewData::refresh_memory_pages(instance.code_viewer_view_data.clone(), instance.app_context.engine_unprivileged_state.clone());
        instance.listen_for_process_change();

        instance
    }

    fn listen_for_process_change(&self) {
        let engine_unprivileged_state = self.app_context.engine_unprivileged_state.clone();
        let code_viewer_view_data = self.code_viewer_view_data.clone();

        self.app_context
            .engine_unprivileged_state
            .listen_for_engine_event::<ProcessChangedEvent>(move |_process_changed_event| {
                CodeViewerViewData::clear_for_process_change(code_viewer_view_data.clone(), engine_unprivileged_state.clone());
                CodeViewerViewData::refresh_memory_pages(code_viewer_view_data.clone(), engine_unprivileged_state.clone());
            });
    }

    fn get_process_bitness(&self) -> Option<Bitness> {
        self.process_selector_view_data
            .read("Code viewer process bitness")
            .and_then(|process_selector_view_data| {
                process_selector_view_data
                    .opened_process
                    .as_ref()
                    .map(|opened_process_info| opened_process_info.get_bitness())
            })
    }

    fn fixed_columns_width() -> f32 {
        Self::BREAKPOINT_GUTTER_WIDTH + Self::BRANCH_GUTTER_WIDTH + Self::ADDRESS_COLUMN_WIDTH
    }

    fn resolve_bytes_text_splitter_position_x(
        content_min_x: f32,
        content_width: f32,
        bytes_text_splitter_ratio: f32,
    ) -> f32 {
        let minimum_splitter_position_x = content_min_x + Self::fixed_columns_width() + Self::MINIMUM_BYTES_COLUMN_WIDTH;
        let maximum_splitter_position_x = content_min_x + (content_width - Self::MINIMUM_TEXT_COLUMN_WIDTH).max(Self::fixed_columns_width());

        if maximum_splitter_position_x <= minimum_splitter_position_x {
            return minimum_splitter_position_x;
        }

        (content_min_x + content_width * bytes_text_splitter_ratio).clamp(minimum_splitter_position_x, maximum_splitter_position_x)
    }

    fn resolve_column_layout(
        row_rect: Rect,
        bytes_text_splitter_position_x: f32,
    ) -> CodeViewerColumnLayout {
        let breakpoint_gutter_rect = Rect::from_min_max(row_rect.min, pos2(row_rect.min.x + Self::BREAKPOINT_GUTTER_WIDTH, row_rect.max.y));
        let branch_gutter_rect = Rect::from_min_max(
            pos2(breakpoint_gutter_rect.max.x, row_rect.min.y),
            pos2(breakpoint_gutter_rect.max.x + Self::BRANCH_GUTTER_WIDTH, row_rect.max.y),
        );
        let address_rect = Rect::from_min_max(
            pos2(branch_gutter_rect.max.x, row_rect.min.y),
            pos2(branch_gutter_rect.max.x + Self::ADDRESS_COLUMN_WIDTH, row_rect.max.y),
        );
        let clamped_bytes_text_splitter_position_x = bytes_text_splitter_position_x.clamp(address_rect.max.x, row_rect.max.x.max(address_rect.max.x));
        let bytes_rect = Rect::from_min_max(
            pos2(address_rect.max.x, row_rect.min.y),
            pos2(clamped_bytes_text_splitter_position_x, row_rect.max.y),
        );
        let text_rect = Rect::from_min_max(pos2(bytes_rect.max.x, row_rect.min.y), row_rect.max);

        CodeViewerColumnLayout {
            breakpoint_gutter_rect,
            address_rect,
            bytes_rect,
            text_rect,
        }
    }

    fn build_bytes_text(bytes: &[u8]) -> String {
        bytes
            .iter()
            .map(|byte_value| format!("{:02X}", byte_value))
            .collect::<Vec<_>>()
            .join(" ")
    }

    fn get_breakpoint_center(row_rect: Rect) -> Pos2 {
        pos2(row_rect.min.x + Self::BREAKPOINT_GUTTER_WIDTH * 0.5, row_rect.center().y)
    }

    fn get_branch_lane_x(
        row_rect: Rect,
        lane_index: usize,
    ) -> f32 {
        row_rect.min.x + Self::BREAKPOINT_GUTTER_WIDTH + Self::BRANCH_GUTTER_WIDTH
            - Self::BRANCH_LANE_RIGHT_PADDING
            - (lane_index as f32) * Self::BRANCH_LANE_SPACING
    }

    fn draw_selection_background(
        user_interface: &Ui,
        rect: Rect,
        fill_color: EpaintColor32,
        border_color: EpaintColor32,
    ) {
        user_interface
            .painter()
            .rect_filled(rect, CornerRadius::same(3), fill_color);
        user_interface
            .painter()
            .rect_stroke(rect, CornerRadius::same(3), Stroke::new(1.0, border_color), epaint::StrokeKind::Inside);
    }

    fn draw_jump_visuals(
        user_interface: &Ui,
        row_rects_by_address: &HashMap<u64, Rect>,
        instruction_lines: &[DisassembledInstruction],
        theme: &crate::ui::theme::Theme,
    ) {
        #[derive(Clone, Copy)]
        struct VisibleBranchSpan {
            source_address: u64,
            target_address: u64,
            source_row_index: usize,
            target_row_index: usize,
            source_row_rect: Rect,
            target_row_rect: Rect,
        }

        let mut branch_spans = instruction_lines
            .iter()
            .enumerate()
            .filter_map(|(source_row_index, instruction_line)| {
                let target_address = instruction_line.branch_target_address?;
                let target_row_index = instruction_lines
                    .iter()
                    .position(|candidate_instruction| candidate_instruction.address == target_address)?;
                let source_row_rect = row_rects_by_address.get(&instruction_line.address)?;
                let target_row_rect = row_rects_by_address.get(&target_address)?;

                Some(VisibleBranchSpan {
                    source_address: instruction_line.address,
                    target_address,
                    source_row_index,
                    target_row_index,
                    source_row_rect: *source_row_rect,
                    target_row_rect: *target_row_rect,
                })
            })
            .collect::<Vec<_>>();
        branch_spans.sort_by_key(|branch_span| {
            (
                branch_span
                    .source_row_index
                    .abs_diff(branch_span.target_row_index),
                branch_span.source_row_index.min(branch_span.target_row_index),
            )
        });

        let mut lane_last_row_indices = Vec::<usize>::new();

        for branch_span in branch_spans {
            let start_row_index = branch_span.source_row_index.min(branch_span.target_row_index);
            let end_row_index = branch_span.source_row_index.max(branch_span.target_row_index);
            let lane_index = lane_last_row_indices
                .iter()
                .position(|last_row_index| *last_row_index < start_row_index)
                .unwrap_or_else(|| {
                    if lane_last_row_indices.len() >= Self::MAX_BRANCH_LANES {
                        usize::MAX
                    } else {
                        lane_last_row_indices.push(0);
                        lane_last_row_indices.len().saturating_sub(1)
                    }
                });

            if lane_index == usize::MAX {
                continue;
            }

            lane_last_row_indices[lane_index] = end_row_index;

            let lane_x = Self::get_branch_lane_x(branch_span.source_row_rect, lane_index);
            let source_y = branch_span.source_row_rect.center().y;
            let target_y = branch_span.target_row_rect.center().y;
            let stub_x = branch_span.source_row_rect.min.x + Self::BREAKPOINT_GUTTER_WIDTH + Self::BRANCH_GUTTER_WIDTH - 3.0;
            let branch_color = Color32::from_rgba_unmultiplied(
                theme.selected_border.r(),
                theme.selected_border.g(),
                theme.selected_border.b(),
                if branch_span.source_address <= branch_span.target_address { 160 } else { 210 },
            );
            let arrow_dx = 4.0;
            let arrow_dy = 3.0;

            user_interface
                .painter()
                .line_segment([pos2(stub_x, source_y), pos2(lane_x, source_y)], Stroke::new(1.0, branch_color));
            user_interface
                .painter()
                .line_segment([pos2(lane_x, source_y), pos2(lane_x, target_y)], Stroke::new(1.0, branch_color));
            user_interface
                .painter()
                .line_segment([pos2(lane_x, target_y), pos2(stub_x, target_y)], Stroke::new(1.0, branch_color));
            user_interface.painter().line_segment(
                [
                    pos2(stub_x, target_y),
                    pos2(stub_x - arrow_dx, target_y - arrow_dy),
                ],
                Stroke::new(1.0, branch_color),
            );
            user_interface.painter().line_segment(
                [
                    pos2(stub_x, target_y),
                    pos2(stub_x - arrow_dx, target_y + arrow_dy),
                ],
                Stroke::new(1.0, branch_color),
            );
        }
    }

    fn dispatch_add_instructions_to_project(
        &self,
        absolute_address: u64,
        instruction_lines: &[DisassembledInstruction],
    ) {
        let target_directory_path = ProjectHierarchyViewData::get_selected_directory_path(self.project_hierarchy_view_data.clone());
        let Some(project_items_create_request) = CodeViewerViewData::build_instruction_project_item_create_request(
            self.code_viewer_view_data.clone(),
            absolute_address,
            target_directory_path,
            self.get_process_bitness(),
            instruction_lines,
        ) else {
            log::warn!("Failed to build code viewer instruction project item create request.");
            return;
        };

        project_items_create_request.send(&self.app_context.engine_unprivileged_state, |project_items_create_response| {
            if !project_items_create_response.success {
                log::warn!("Code viewer add-instructions-to-project command failed.");
            }
        });
    }

    fn build_context_menu_add_label(
        &self,
        context_menu_address: u64,
        selected_instruction_addresses: &HashSet<u64>,
    ) -> String {
        if selected_instruction_addresses.contains(&context_menu_address) && selected_instruction_addresses.len() > 1 {
            String::from("Add Instructions to Project")
        } else {
            String::from("Add Instruction to Project")
        }
    }

    fn dispatch_instruction_write(
        &self,
        instruction_write_plan: CodeViewerInstructionWritePlan,
    ) {
        let code_viewer_view_data = self.code_viewer_view_data.clone();
        let engine_unprivileged_state = self.app_context.engine_unprivileged_state.clone();
        let engine_unprivileged_state_for_callback = engine_unprivileged_state.clone();
        let written_bytes_for_refresh = instruction_write_plan.written_bytes.clone();
        let write_start_address = instruction_write_plan.start_address;
        let memory_write_request = squalr_engine_api::commands::memory::write::memory_write_request::MemoryWriteRequest {
            address: write_start_address,
            module_name: String::new(),
            value: instruction_write_plan.written_bytes,
        };

        memory_write_request.send(&engine_unprivileged_state, move |memory_write_response| {
            if memory_write_response.success {
                CodeViewerViewData::apply_memory_write(code_viewer_view_data.clone(), write_start_address, &written_bytes_for_refresh);
                CodeViewerViewData::finish_instruction_write(code_viewer_view_data.clone(), write_start_address);
                engine_unprivileged_state_for_callback.request_virtual_snapshot_refresh(CodeViewerViewData::WINDOW_VIRTUAL_SNAPSHOT_ID);
            } else {
                CodeViewerViewData::set_instruction_edit_error(code_viewer_view_data.clone(), String::from("Instruction write failed."));
                log::warn!("Code viewer instruction write command failed.");
            }
        });
    }

    fn instruction_edit_data_type_ref(&self) -> DataTypeRef {
        DataTypeRef::new(match self.get_process_bitness().unwrap_or(Bitness::Bit64) {
            Bitness::Bit32 => "i_x86",
            Bitness::Bit64 => "i_x64",
        })
    }

    fn build_context_menu_edit_label(
        &self,
        context_menu_address: u64,
        selected_instruction_addresses: &HashSet<u64>,
    ) -> String {
        if selected_instruction_addresses.contains(&context_menu_address) && selected_instruction_addresses.len() > 1 {
            String::from("Edit Instructions")
        } else {
            String::from("Edit Instruction")
        }
    }

    fn render_instruction_row(
        &self,
        user_interface: &mut Ui,
        instruction_line: &DisassembledInstruction,
        selected_instruction_addresses: &HashSet<u64>,
        bytes_text_splitter_position_x: f32,
        scroll_target_address: Option<u64>,
        instruction_lines: &[DisassembledInstruction],
        instruction_edit_state: Option<&CodeViewerInstructionEditState>,
    ) -> Rect {
        let theme = &self.app_context.theme;
        let (row_rect, row_response) = user_interface.allocate_exact_size(vec2(user_interface.available_width(), Self::ROW_HEIGHT), Sense::click());
        let is_selected = selected_instruction_addresses.contains(&instruction_line.address);
        let is_instruction_edit_row = instruction_edit_state
            .map(|instruction_edit_state| instruction_edit_state.start_address == instruction_line.address)
            .unwrap_or(false);
        let column_layout = Self::resolve_column_layout(row_rect, bytes_text_splitter_position_x);

        if is_selected {
            Self::draw_selection_background(
                user_interface,
                row_rect.shrink2(vec2(1.0, 1.0)),
                theme.selected_background,
                theme.selected_border,
            );
        }

        let breakpoint_response = user_interface.interact(
            column_layout.breakpoint_gutter_rect,
            user_interface.make_persistent_id(("code_viewer_breakpoint", instruction_line.address)),
            Sense::click(),
        );
        if breakpoint_response.clicked() {
            CodeViewerViewData::toggle_breakpoint_address(self.code_viewer_view_data.clone(), instruction_line.address);
        }

        if row_response.clicked() {
            let should_extend_selection = user_interface.input(|input_state| input_state.modifiers.shift);

            CodeViewerViewData::set_keyboard_focus(self.code_viewer_view_data.clone(), true);

            if should_extend_selection {
                CodeViewerViewData::extend_instruction_selection(self.code_viewer_view_data.clone(), instruction_line.address);
            } else {
                CodeViewerViewData::select_instruction_address(self.code_viewer_view_data.clone(), instruction_line.address);
            }
        }

        if row_response.double_clicked() {
            CodeViewerViewData::request_instruction_edit(self.code_viewer_view_data.clone(), instruction_line.address, instruction_lines);
        }

        if row_response.secondary_clicked() {
            if !selected_instruction_addresses.contains(&instruction_line.address) {
                CodeViewerViewData::select_instruction_address(self.code_viewer_view_data.clone(), instruction_line.address);
            }

            CodeViewerViewData::show_context_menu(
                self.code_viewer_view_data.clone(),
                instruction_line.address,
                row_response.hover_pos().unwrap_or(row_rect.left_bottom()),
            );
        }

        if scroll_target_address
            .map(|scroll_target_address| scroll_target_address == instruction_line.address)
            .unwrap_or(false)
        {
            user_interface.scroll_to_rect(row_rect, Some(Align::Center));
        }

        if CodeViewerViewData::has_breakpoint_address(self.code_viewer_view_data.clone(), instruction_line.address) {
            user_interface
                .painter()
                .circle_filled(Self::get_breakpoint_center(row_rect), Self::BREAKPOINT_RADIUS, theme.error_red);
        }

        user_interface
            .painter()
            .with_clip_rect(column_layout.address_rect.intersect(user_interface.clip_rect()))
            .text(
                pos2(
                    column_layout.address_rect.min.x + Self::TEXT_LEFT_PADDING,
                    row_rect.min.y + Self::ROW_TEXT_TOP_PADDING,
                ),
                Align2::LEFT_TOP,
                format!("{:016X}", instruction_line.address),
                theme.font_library.font_ubuntu_mono_bold.font_normal.clone(),
                theme.hexadecimal_green,
            );
        user_interface
            .painter()
            .with_clip_rect(column_layout.bytes_rect.intersect(user_interface.clip_rect()))
            .text(
                pos2(
                    column_layout.bytes_rect.min.x + Self::TEXT_LEFT_PADDING,
                    row_rect.min.y + Self::ROW_TEXT_TOP_PADDING,
                ),
                Align2::LEFT_TOP,
                Self::build_bytes_text(&instruction_line.bytes),
                theme.font_library.font_ubuntu_mono_bold.font_normal.clone(),
                theme.hexadecimal_green,
            );
        if is_instruction_edit_row {
            if let Some(instruction_edit_state) = instruction_edit_state {
                self.render_instruction_text_edit_contents(user_interface, column_layout.text_rect, instruction_edit_state);
            }
        } else {
            user_interface
                .painter()
                .with_clip_rect(column_layout.text_rect.intersect(user_interface.clip_rect()))
                .text(
                    pos2(
                        column_layout.text_rect.min.x + Self::TEXT_LEFT_PADDING,
                        row_rect.min.y + Self::ROW_TEXT_TOP_PADDING,
                    ),
                    Align2::LEFT_TOP,
                    &instruction_line.text,
                    theme.font_library.font_ubuntu_mono_bold.font_normal.clone(),
                    if instruction_line.is_control_flow {
                        theme.background_control_info
                    } else {
                        theme.foreground
                    },
                );
        }

        row_rect
    }

    fn render_instruction_text_edit_contents(
        &self,
        user_interface: &mut Ui,
        text_rect: Rect,
        instruction_edit_state: &CodeViewerInstructionEditState,
    ) {
        let theme = &self.app_context.theme;
        let mut edit_row_user_interface = user_interface.new_child(
            UiBuilder::new()
                .max_rect(text_rect)
                .layout(Layout::left_to_right(Align::Center)),
        );
        let inner_text_rect = text_rect.shrink2(vec2(2.0, 2.0));
        edit_row_user_interface.set_clip_rect(inner_text_rect);

        let validation_data_type = self.instruction_edit_data_type_ref();
        let mut edit_value = instruction_edit_state.edit_value.clone();
        let original_edit_value = edit_value.clone();
        let did_commit_on_enter = DataValueBoxView::consume_commit_on_enter(user_interface, Self::INSTRUCTION_EDIT_INPUT_ID);
        let button_width = 32.0;
        let button_spacing = 4.0;
        let total_button_width = button_width * 2.0 + button_spacing;
        edit_row_user_interface.add(
            DataValueBoxView::new(
                self.app_context.clone(),
                &mut edit_value,
                &validation_data_type,
                false,
                true,
                "Type assembly here. Press Enter to write.",
                Self::INSTRUCTION_EDIT_INPUT_ID,
            )
            .width((inner_text_rect.width() - total_button_width - 8.0).max(120.0))
            .height(Self::TOOLBAR_ROW_HEIGHT)
            .use_format_text_colors(false),
        );

        if edit_value != original_edit_value {
            CodeViewerViewData::set_instruction_edit_value(self.code_viewer_view_data.clone(), edit_value);
        }

        let should_commit_edit = did_commit_on_enter;
        let should_cancel_edit = edit_row_user_interface.input(|input_state| input_state.key_pressed(Key::Escape));

        edit_row_user_interface.add_space(button_spacing);
        let cancel_button = edit_row_user_interface.add_sized(
            vec2(32.0, Self::TOOLBAR_ROW_HEIGHT),
            Button::new_from_theme(theme)
                .background_color(Color32::TRANSPARENT)
                .with_tooltip_text("Cancel instruction edit."),
        );
        IconDraw::draw(&edit_row_user_interface, cancel_button.rect, &theme.icon_library.icon_handle_navigation_cancel);

        let commit_button = edit_row_user_interface.add_sized(
            vec2(32.0, Self::TOOLBAR_ROW_HEIGHT),
            Button::new_from_theme(theme)
                .background_color(Color32::TRANSPARENT)
                .with_tooltip_text("Assemble and commit this instruction edit."),
        );
        IconDraw::draw(&edit_row_user_interface, commit_button.rect, &theme.icon_library.icon_handle_common_check_mark);

        if cancel_button.clicked() || should_cancel_edit {
            CodeViewerViewData::cancel_instruction_edit(self.code_viewer_view_data.clone());
            return;
        }

        if commit_button.clicked() || should_commit_edit {
            if let Some(instruction_write_plan) =
                CodeViewerViewData::evaluate_instruction_edit_commit(self.code_viewer_view_data.clone(), self.get_process_bitness())
            {
                self.dispatch_instruction_write(instruction_write_plan);
            }
        }
    }

    fn render_instruction_edit_warning(
        &self,
        user_interface: &mut Ui,
        instruction_edit_state: &CodeViewerInstructionEditState,
        bytes_text_splitter_position_x: f32,
    ) {
        let Some(instruction_edit_status) = instruction_edit_state.status.as_ref() else {
            return;
        };
        let theme = &self.app_context.theme;
        let (warning_rect, _) = user_interface.allocate_exact_size(vec2(user_interface.available_width(), Self::EDIT_WARNING_ROW_HEIGHT), Sense::hover());
        let mut warning_user_interface = user_interface.new_child(
            UiBuilder::new()
                .max_rect(warning_rect)
                .layout(Layout::left_to_right(Align::Center)),
        );
        let warning_color = match instruction_edit_status {
            CodeViewerInstructionEditStatus::Invalid(_) => theme.error_red,
            CodeViewerInstructionEditStatus::PendingFillWithNops { .. } | CodeViewerInstructionEditStatus::PendingOverwrite { .. } => {
                theme.background_control_warning
            }
        };

        warning_user_interface.painter().rect_stroke(
            warning_rect.shrink2(vec2(1.0, 1.0)),
            CornerRadius::same(3),
            Stroke::new(1.0, warning_color),
            epaint::StrokeKind::Inside,
        );
        warning_user_interface.add_space((bytes_text_splitter_position_x - warning_rect.min.x + 6.0).max(0.0));

        match instruction_edit_status {
            CodeViewerInstructionEditStatus::Invalid(error) => {
                warning_user_interface.label(
                    RichText::new(error)
                        .font(theme.font_library.font_noto_sans.font_normal.clone())
                        .color(theme.error_red),
                );
            }
            CodeViewerInstructionEditStatus::PendingFillWithNops { remaining_byte_count, .. } => {
                warning_user_interface.label(
                    RichText::new(format!(
                        "Replacement is {} byte(s) shorter. Fill the remainder with NOPs?",
                        remaining_byte_count
                    ))
                    .font(theme.font_library.font_noto_sans.font_normal.clone())
                    .color(theme.background_control_warning),
                );
                warning_user_interface.add_space(8.0);
                let fill_button = warning_user_interface.add_sized(
                    vec2(96.0, Self::TOOLBAR_ROW_HEIGHT - 2.0),
                    Button::new_from_theme(theme)
                        .background_color(Color32::TRANSPARENT)
                        .with_tooltip_text("Write the replacement and pad the remaining bytes with no-operations."),
                );
                warning_user_interface.painter().text(
                    fill_button.rect.center(),
                    Align2::CENTER_CENTER,
                    "Fill + Write",
                    theme.font_library.font_noto_sans.font_normal.clone(),
                    theme.foreground,
                );

                if fill_button.clicked() {
                    if let Some(instruction_write_plan) =
                        CodeViewerViewData::accept_instruction_edit_pending_fill_with_nops(self.code_viewer_view_data.clone(), self.get_process_bitness())
                    {
                        self.dispatch_instruction_write(instruction_write_plan);
                    }
                }
            }
            CodeViewerInstructionEditStatus::PendingOverwrite { overwritten_byte_count, .. } => {
                warning_user_interface.label(
                    RichText::new(format!(
                        "Replacement is {} byte(s) longer and will overwrite the next instruction(s).",
                        overwritten_byte_count
                    ))
                    .font(theme.font_library.font_noto_sans.font_normal.clone())
                    .color(theme.background_control_warning),
                );
                warning_user_interface.add_space(8.0);
                let overwrite_button = warning_user_interface.add_sized(
                    vec2(92.0, Self::TOOLBAR_ROW_HEIGHT - 2.0),
                    Button::new_from_theme(theme)
                        .background_color(Color32::TRANSPARENT)
                        .with_tooltip_text("Write the longer replacement and allow it to overwrite the following bytes."),
                );
                warning_user_interface.painter().text(
                    overwrite_button.rect.center(),
                    Align2::CENTER_CENTER,
                    "Write Anyway",
                    theme.font_library.font_noto_sans.font_normal.clone(),
                    theme.foreground,
                );

                if overwrite_button.clicked() {
                    if let Some(instruction_write_plan) = CodeViewerViewData::accept_instruction_edit_pending_overwrite(self.code_viewer_view_data.clone()) {
                        self.dispatch_instruction_write(instruction_write_plan);
                    }
                }
            }
        }
    }
}

impl Widget for CodeViewerView {
    fn ui(
        self,
        user_interface: &mut Ui,
    ) -> Response {
        let theme = &self.app_context.theme;
        let code_viewer_has_keyboard_focus = CodeViewerViewData::has_keyboard_focus(self.code_viewer_view_data.clone());
        CodeViewerViewData::clear_stale_request_state_if_needed(self.code_viewer_view_data.clone());
        user_interface
            .ctx()
            .request_repaint_after(CodeViewerViewData::SNAPSHOT_REFRESH_INTERVAL);

        if let Some(virtual_snapshot) = self
            .app_context
            .engine_unprivileged_state
            .get_virtual_snapshot(CodeViewerViewData::WINDOW_VIRTUAL_SNAPSHOT_ID)
        {
            CodeViewerViewData::apply_virtual_snapshot_results(self.code_viewer_view_data.clone(), &virtual_snapshot);
        }

        user_interface
            .allocate_ui_with_layout(user_interface.available_size(), Layout::top_down(Align::Min), |user_interface| {
                let (toolbar_rect, _) =
                    user_interface.allocate_exact_size(vec2(user_interface.available_width().max(1.0), Self::TOOLBAR_HEIGHT), Sense::empty());

                user_interface
                    .painter()
                    .rect_filled(toolbar_rect, CornerRadius::ZERO, theme.background_primary);

                let mut toolbar_user_interface = user_interface.new_child(
                    UiBuilder::new()
                        .max_rect(toolbar_rect)
                        .layout(Layout::left_to_right(Align::Center)),
                );
                let refresh_button = toolbar_user_interface.add_sized(
                    vec2(36.0, Self::TOOLBAR_ROW_HEIGHT),
                    Button::new_from_theme(theme)
                        .background_color(Color32::TRANSPARENT)
                        .with_tooltip_text("Refresh memory pages."),
                );
                IconDraw::draw(&toolbar_user_interface, refresh_button.rect, &theme.icon_library.icon_handle_navigation_refresh);

                if refresh_button.clicked() {
                    CodeViewerViewData::refresh_memory_pages(self.code_viewer_view_data.clone(), self.app_context.engine_unprivileged_state.clone());
                }

                let is_querying_memory_pages = self
                    .code_viewer_view_data
                    .read("Code viewer toolbar state")
                    .map(|code_viewer_view_data| code_viewer_view_data.is_querying_memory_pages)
                    .unwrap_or(false);

                if is_querying_memory_pages {
                    toolbar_user_interface.add_space(8.0);
                    toolbar_user_interface.add(Spinner::new().color(theme.foreground));
                }

                toolbar_user_interface.add_space(12.0);
                toolbar_user_interface.label(
                    RichText::new(match self.get_process_bitness().unwrap_or(Bitness::Bit64) {
                        Bitness::Bit32 => "x86 code",
                        Bitness::Bit64 => "x64 code",
                    })
                    .font(theme.font_library.font_noto_sans.font_normal.clone())
                    .color(theme.foreground_preview),
                );

                let go_to_preview_text = CodeViewerViewData::get_go_to_address_preview_text(self.code_viewer_view_data.clone());
                let address_data_type = DataTypeRef::new(DataTypeU64::DATA_TYPE_ID);
                let mut should_seek_to_address = DataValueBoxView::consume_commit_on_enter(user_interface, Self::GO_TO_ADDRESS_INPUT_ID);
                toolbar_user_interface.add_space(12.0);
                if let Some(mut code_viewer_view_data) = self
                    .code_viewer_view_data
                    .write("Code viewer toolbar go to address input")
                {
                    toolbar_user_interface.add(
                        DataValueBoxView::new(
                            self.app_context.clone(),
                            &mut code_viewer_view_data.go_to_address_input,
                            &address_data_type,
                            false,
                            true,
                            &go_to_preview_text,
                            Self::GO_TO_ADDRESS_INPUT_ID,
                        )
                        .width(236.0)
                        .height(Self::TOOLBAR_ROW_HEIGHT)
                        .use_preview_foreground(true)
                        .use_format_text_colors(false),
                    );
                }
                toolbar_user_interface.add_space(6.0);
                let apply_go_to_button = toolbar_user_interface.add_sized(
                    vec2(36.0, Self::TOOLBAR_ROW_HEIGHT),
                    Button::new_from_theme(theme)
                        .background_color(Color32::TRANSPARENT)
                        .with_tooltip_text("Seek the code viewer to the requested address."),
                );
                IconDraw::draw(
                    &toolbar_user_interface,
                    apply_go_to_button.rect,
                    &theme.icon_library.icon_handle_navigation_right_arrow,
                );
                should_seek_to_address |= apply_go_to_button.clicked();

                if should_seek_to_address {
                    CodeViewerViewData::seek_to_input_address(self.code_viewer_view_data.clone());
                }

                let footer_height = self.code_viewer_footer_view.get_height();
                let content_rect = user_interface
                    .available_rect_before_wrap()
                    .with_max_y(user_interface.available_rect_before_wrap().max.y - footer_height);
                let content_response = user_interface.allocate_rect(content_rect, Sense::click());
                let mut content_user_interface = user_interface.new_child(
                    UiBuilder::new()
                        .max_rect(content_response.rect)
                        .layout(Layout::top_down(Align::Min)),
                );
                content_user_interface.set_clip_rect(content_response.rect);
                if content_response.clicked() {
                    CodeViewerViewData::set_keyboard_focus(self.code_viewer_view_data.clone(), true);
                }

                if user_interface.input(|input_state| input_state.pointer.any_pressed())
                    && user_interface
                        .input(|input_state| input_state.pointer.interact_pos())
                        .map(|pointer_position| !content_response.rect.contains(pointer_position))
                        .unwrap_or(false)
                {
                    CodeViewerViewData::set_keyboard_focus(self.code_viewer_view_data.clone(), false);
                }

                if code_viewer_has_keyboard_focus
                    && CodeViewerViewData::get_instruction_edit_state(self.code_viewer_view_data.clone()).is_none()
                    && user_interface.input(|input_state| input_state.key_pressed(Key::Escape))
                {
                    CodeViewerViewData::clear_selection(self.code_viewer_view_data.clone());
                }
                let mut new_bytes_text_splitter_ratio = None;
                let mut bytes_text_splitter_ratio = self
                    .code_viewer_view_data
                    .read("Code viewer column ratios")
                    .map(|code_viewer_view_data| code_viewer_view_data.bytes_text_splitter_ratio)
                    .unwrap_or(CodeViewerViewData::DEFAULT_BYTES_TEXT_SPLITTER_RATIO);
                let content_width = content_response.rect.width();
                let content_min_x = content_response.rect.min.x;

                if content_width > 0.0 {
                    let resolved_bytes_text_splitter_position_x =
                        Self::resolve_bytes_text_splitter_position_x(content_min_x, content_width, bytes_text_splitter_ratio);
                    let resolved_bytes_text_splitter_ratio = (resolved_bytes_text_splitter_position_x - content_min_x) / content_width;

                    if (resolved_bytes_text_splitter_ratio - bytes_text_splitter_ratio).abs() > f32::EPSILON {
                        bytes_text_splitter_ratio = resolved_bytes_text_splitter_ratio;
                        new_bytes_text_splitter_ratio = Some(resolved_bytes_text_splitter_ratio);
                    }
                }

                let bytes_text_splitter_position_x =
                    Self::resolve_bytes_text_splitter_position_x(content_min_x, content_width.max(1.0), bytes_text_splitter_ratio);
                let address_separator_position_x = content_min_x + Self::fixed_columns_width();

                content_user_interface
                    .painter()
                    .rect_filled(content_user_interface.max_rect(), CornerRadius::ZERO, theme.background_panel);

                let process_bitness = self.get_process_bitness();
                let current_page = CodeViewerViewData::get_current_page(self.code_viewer_view_data.clone());
                let mut visible_instruction_lines = Vec::new();

                match current_page {
                    Some(current_page) => {
                        let viewport_start_address =
                            CodeViewerViewData::get_viewport_start_address(self.code_viewer_view_data.clone()).unwrap_or(current_page.get_base_address());
                        let visible_chunk_queries = CodeViewerViewData::build_visible_chunk_queries(&current_page, viewport_start_address);
                        self.app_context
                            .engine_unprivileged_state
                            .set_virtual_snapshot_queries(
                                CodeViewerViewData::WINDOW_VIRTUAL_SNAPSHOT_ID,
                                CodeViewerViewData::SNAPSHOT_REFRESH_INTERVAL,
                                visible_chunk_queries,
                            );
                        self.app_context
                            .engine_unprivileged_state
                            .request_virtual_snapshot_refresh(CodeViewerViewData::WINDOW_VIRTUAL_SNAPSHOT_ID);

                        visible_instruction_lines = CodeViewerViewData::build_instruction_lines(self.code_viewer_view_data.clone(), process_bitness);
                        let pending_scroll_address = CodeViewerViewData::take_pending_scroll_address(self.code_viewer_view_data.clone());
                        let scroll_target_address = CodeViewerViewData::resolve_scroll_target_address(pending_scroll_address, &visible_instruction_lines);
                        let current_page_is_unreadable = CodeViewerViewData::is_current_page_unreadable(self.code_viewer_view_data.clone(), &current_page);

                        if visible_instruction_lines.is_empty() && current_page_is_unreadable {
                            content_user_interface.centered_and_justified(|user_interface| {
                                user_interface.label(
                                    RichText::new("This page is currently unreadable, so no code rows could be decoded.")
                                        .font(theme.font_library.font_noto_sans.font_normal.clone())
                                        .color(theme.background_control_warning),
                                );
                            });
                        } else if visible_instruction_lines.is_empty() {
                            content_user_interface.centered_and_justified(|user_interface| {
                                user_interface.label(
                                    RichText::new("The current code window has no decoded instructions yet. Scroll or refresh to materialize more bytes.")
                                        .font(theme.font_library.font_noto_sans.font_normal.clone())
                                        .color(theme.foreground_preview),
                                );
                            });
                        } else {
                            ScrollArea::vertical()
                                .id_salt("code_viewer_rows")
                                .auto_shrink([false, false])
                                .show(&mut content_user_interface, |user_interface| {
                                    let mut row_rects_by_address = HashMap::new();
                                    let selected_instruction_addresses =
                                        CodeViewerViewData::get_selected_instruction_addresses(self.code_viewer_view_data.clone(), &visible_instruction_lines);
                                    let instruction_edit_state = CodeViewerViewData::get_instruction_edit_state(self.code_viewer_view_data.clone());

                                    for instruction_line in &visible_instruction_lines {
                                        let row_rect = self.render_instruction_row(
                                            user_interface,
                                            instruction_line,
                                            &selected_instruction_addresses,
                                            bytes_text_splitter_position_x,
                                            scroll_target_address,
                                            &visible_instruction_lines,
                                            instruction_edit_state.as_ref(),
                                        );
                                        row_rects_by_address.insert(instruction_line.address, row_rect);

                                        if instruction_edit_state
                                            .as_ref()
                                            .map(|instruction_edit_state| instruction_edit_state.start_address == instruction_line.address)
                                            .unwrap_or(false)
                                        {
                                            if let Some(instruction_edit_state) = instruction_edit_state.as_ref() {
                                                self.render_instruction_edit_warning(user_interface, instruction_edit_state, bytes_text_splitter_position_x);
                                            }
                                        }
                                    }

                                    Self::draw_jump_visuals(user_interface, &row_rects_by_address, &visible_instruction_lines, theme);
                                });
                        }
                    }
                    None if is_querying_memory_pages => {
                        content_user_interface.allocate_ui_with_layout(
                            vec2(content_user_interface.available_width(), 32.0),
                            Layout::centered_and_justified(Direction::LeftToRight),
                            |user_interface| {
                                user_interface.add(Spinner::new().color(theme.foreground));
                            },
                        );
                    }
                    None => {
                        content_user_interface.centered_and_justified(|user_interface| {
                            user_interface.label(
                                RichText::new("Attach to a process to browse code pages.")
                                    .font(theme.font_library.font_noto_sans.font_normal.clone())
                                    .color(theme.foreground_preview),
                            );
                        });
                    }
                }

                let splitter_min_y = content_response.rect.min.y;
                let splitter_max_y = content_response.rect.max.y;
                let address_separator_rect = Rect::from_min_max(
                    pos2(address_separator_position_x - Self::COLUMN_SEPARATOR_THICKNESS * 0.5, splitter_min_y),
                    pos2(address_separator_position_x + Self::COLUMN_SEPARATOR_THICKNESS * 0.5, splitter_max_y),
                );
                let bytes_text_splitter_rect = Rect::from_min_max(
                    pos2(bytes_text_splitter_position_x - Self::COLUMN_SEPARATOR_THICKNESS * 0.5, splitter_min_y),
                    pos2(bytes_text_splitter_position_x + Self::COLUMN_SEPARATOR_THICKNESS * 0.5, splitter_max_y),
                );

                user_interface
                    .painter()
                    .rect_filled(address_separator_rect, CornerRadius::ZERO, theme.background_control);

                let bytes_text_splitter_response = user_interface
                    .interact(
                        bytes_text_splitter_rect,
                        user_interface.id().with("code_viewer_bytes_text_splitter"),
                        Sense::drag(),
                    )
                    .on_hover_cursor(CursorIcon::ResizeHorizontal);
                user_interface
                    .painter()
                    .rect_filled(bytes_text_splitter_rect, CornerRadius::ZERO, theme.background_control);

                if bytes_text_splitter_response.dragged() && content_width > 0.0 {
                    let new_bytes_text_splitter_position_x =
                        Self::resolve_bytes_text_splitter_position_x(content_min_x, content_width, bytes_text_splitter_ratio)
                            + bytes_text_splitter_response.drag_delta().x;
                    let bounded_bytes_text_splitter_position_x = Self::resolve_bytes_text_splitter_position_x(
                        content_min_x,
                        content_width,
                        (new_bytes_text_splitter_position_x - content_min_x) / content_width,
                    );

                    new_bytes_text_splitter_ratio = Some((bounded_bytes_text_splitter_position_x - content_min_x) / content_width);
                }

                if let Some(new_bytes_text_splitter_ratio) = new_bytes_text_splitter_ratio {
                    if let Some(mut code_viewer_view_data) = self
                        .code_viewer_view_data
                        .write("Code viewer update column ratios")
                    {
                        code_viewer_view_data.bytes_text_splitter_ratio = new_bytes_text_splitter_ratio;
                    }
                }

                if let Some((context_menu_address, context_menu_position)) = CodeViewerViewData::get_context_menu_state(self.code_viewer_view_data.clone()) {
                    let mut open = true;
                    let selected_instruction_addresses =
                        CodeViewerViewData::get_selected_instruction_addresses(self.code_viewer_view_data.clone(), &visible_instruction_lines);
                    let add_action_label = self.build_context_menu_add_label(context_menu_address, &selected_instruction_addresses);
                    let edit_action_label = self.build_context_menu_edit_label(context_menu_address, &selected_instruction_addresses);

                    ContextMenu::new(
                        self.app_context.clone(),
                        "code_viewer_context_menu",
                        context_menu_position,
                        |user_interface, should_close| {
                            if user_interface
                                .add(ToolbarMenuItemView::new(
                                    self.app_context.clone(),
                                    &edit_action_label,
                                    "code_viewer_ctx_edit_instruction",
                                    &None,
                                    Self::CONTEXT_MENU_WIDTH,
                                ))
                                .clicked()
                            {
                                CodeViewerViewData::request_instruction_edit(
                                    self.code_viewer_view_data.clone(),
                                    context_menu_address,
                                    &visible_instruction_lines,
                                );
                                *should_close = true;
                            }

                            if user_interface
                                .add(ToolbarMenuItemView::new(
                                    self.app_context.clone(),
                                    &add_action_label,
                                    "code_viewer_ctx_add_to_project",
                                    &None,
                                    Self::CONTEXT_MENU_WIDTH,
                                ))
                                .clicked()
                            {
                                self.dispatch_add_instructions_to_project(context_menu_address, &visible_instruction_lines);
                                *should_close = true;
                            }
                        },
                    )
                    .width(Self::CONTEXT_MENU_WIDTH)
                    .corner_radius(8)
                    .show(user_interface, &mut open);

                    if !open {
                        CodeViewerViewData::hide_context_menu(self.code_viewer_view_data.clone());
                    }
                }

                user_interface.add(self.code_viewer_footer_view.clone());
            })
            .response
    }
}

#[cfg(test)]
mod tests {
    use super::CodeViewerView;

    #[test]
    fn resolve_bytes_text_splitter_position_x_clamps_to_minimum_visible_columns() {
        let resolved_splitter_position_x = CodeViewerView::resolve_bytes_text_splitter_position_x(0.0, 320.0, 0.1);

        assert_eq!(
            resolved_splitter_position_x,
            CodeViewerView::fixed_columns_width() + CodeViewerView::MINIMUM_BYTES_COLUMN_WIDTH
        );
    }
}
