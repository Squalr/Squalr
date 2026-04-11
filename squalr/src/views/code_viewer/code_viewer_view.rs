use crate::{
    app_context::AppContext,
    ui::{draw::icon_draw::IconDraw, widgets::controls::button::Button},
    views::{
        code_viewer::{code_viewer_footer_view::CodeViewerFooterView, view_data::code_viewer_view_data::CodeViewerViewData},
        process_selector::view_data::process_selector_view_data::ProcessSelectorViewData,
    },
};
use eframe::egui::{
    Align, Align2, Color32, Direction, Layout, Pos2, Rect, Response, RichText, ScrollArea, Sense, Spinner, Stroke, Ui, UiBuilder, Widget, pos2, vec2,
};
use epaint::{Color32 as EpaintColor32, CornerRadius};
use squalr_engine_api::{
    dependency_injection::dependency::Dependency, events::process::changed::process_changed_event::ProcessChangedEvent, structures::memory::bitness::Bitness,
};
use squalr_plugin_instructions_x86::DisassembledInstruction;
use std::{collections::HashMap, sync::Arc};

#[derive(Clone)]
pub struct CodeViewerView {
    app_context: Arc<AppContext>,
    code_viewer_view_data: Dependency<CodeViewerViewData>,
    process_selector_view_data: Dependency<ProcessSelectorViewData>,
    code_viewer_footer_view: CodeViewerFooterView,
}

impl CodeViewerView {
    pub const WINDOW_ID: &'static str = "window_code_viewer";
    const TOOLBAR_HEIGHT: f32 = 28.0;
    const ROW_HEIGHT: f32 = 22.0;
    const BREAKPOINT_GUTTER_WIDTH: f32 = 28.0;
    const BRANCH_GUTTER_WIDTH: f32 = 56.0;
    const ADDRESS_COLUMN_WIDTH: f32 = 118.0;
    const BYTES_COLUMN_WIDTH: f32 = 168.0;
    const TEXT_LEFT_PADDING: f32 = 6.0;
    const ROW_TEXT_TOP_PADDING: f32 = 4.0;
    const BREAKPOINT_RADIUS: f32 = 5.0;
    const BRANCH_LANE_SPACING: f32 = 8.0;
    const BRANCH_LANE_RIGHT_PADDING: f32 = 8.0;
    const MAX_BRANCH_LANES: usize = 5;

    pub fn new(app_context: Arc<AppContext>) -> Self {
        let code_viewer_view_data = app_context
            .dependency_container
            .register(CodeViewerViewData::new());
        let process_selector_view_data = app_context
            .dependency_container
            .get_dependency::<ProcessSelectorViewData>();
        let instance = Self {
            code_viewer_footer_view: CodeViewerFooterView::new(app_context.clone()),
            process_selector_view_data,
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

    fn render_instruction_row(
        &self,
        user_interface: &mut Ui,
        instruction_line: &DisassembledInstruction,
        pending_scroll_address: Option<u64>,
    ) -> Rect {
        let theme = &self.app_context.theme;
        let (row_rect, row_response) = user_interface.allocate_exact_size(vec2(user_interface.available_width(), Self::ROW_HEIGHT), Sense::click());
        let is_selected = CodeViewerViewData::get_selected_instruction_address(self.code_viewer_view_data.clone())
            .map(|selected_instruction_address| selected_instruction_address == instruction_line.address)
            .unwrap_or(false);
        let breakpoint_gutter_rect = Rect::from_min_max(row_rect.min, pos2(row_rect.min.x + Self::BREAKPOINT_GUTTER_WIDTH, row_rect.max.y));
        let branch_gutter_rect = Rect::from_min_max(
            pos2(breakpoint_gutter_rect.max.x, row_rect.min.y),
            pos2(breakpoint_gutter_rect.max.x + Self::BRANCH_GUTTER_WIDTH, row_rect.max.y),
        );
        let address_rect = Rect::from_min_max(
            pos2(branch_gutter_rect.max.x, row_rect.min.y),
            pos2(branch_gutter_rect.max.x + Self::ADDRESS_COLUMN_WIDTH, row_rect.max.y),
        );
        let bytes_rect = Rect::from_min_max(
            pos2(address_rect.max.x, row_rect.min.y),
            pos2(address_rect.max.x + Self::BYTES_COLUMN_WIDTH, row_rect.max.y),
        );
        let text_rect = Rect::from_min_max(pos2(bytes_rect.max.x, row_rect.min.y), row_rect.max);

        if is_selected {
            Self::draw_selection_background(
                user_interface,
                row_rect.shrink2(vec2(1.0, 1.0)),
                theme.selected_background,
                theme.selected_border,
            );
        }

        let breakpoint_response = user_interface.interact(
            breakpoint_gutter_rect,
            user_interface.make_persistent_id(("code_viewer_breakpoint", instruction_line.address)),
            Sense::click(),
        );
        if breakpoint_response.clicked() {
            CodeViewerViewData::toggle_breakpoint_address(self.code_viewer_view_data.clone(), instruction_line.address);
        }

        if row_response.clicked() {
            CodeViewerViewData::select_instruction_address(self.code_viewer_view_data.clone(), instruction_line.address);
        }

        if pending_scroll_address
            .map(|pending_scroll_address| pending_scroll_address == instruction_line.address)
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
            .with_clip_rect(address_rect.intersect(user_interface.clip_rect()))
            .text(
                pos2(address_rect.min.x + Self::TEXT_LEFT_PADDING, row_rect.min.y + Self::ROW_TEXT_TOP_PADDING),
                Align2::LEFT_TOP,
                format!("{:016X}", instruction_line.address),
                theme.font_library.font_ubuntu_mono_bold.font_normal.clone(),
                theme.foreground,
            );
        user_interface
            .painter()
            .with_clip_rect(bytes_rect.intersect(user_interface.clip_rect()))
            .text(
                pos2(bytes_rect.min.x + Self::TEXT_LEFT_PADDING, row_rect.min.y + Self::ROW_TEXT_TOP_PADDING),
                Align2::LEFT_TOP,
                Self::build_bytes_text(&instruction_line.bytes),
                theme.font_library.font_ubuntu_mono_bold.font_normal.clone(),
                theme.hexadecimal_green,
            );
        user_interface
            .painter()
            .with_clip_rect(text_rect.intersect(user_interface.clip_rect()))
            .text(
                pos2(text_rect.min.x + Self::TEXT_LEFT_PADDING, row_rect.min.y + Self::ROW_TEXT_TOP_PADDING),
                Align2::LEFT_TOP,
                &instruction_line.text,
                theme.font_library.font_ubuntu_mono_bold.font_normal.clone(),
                if instruction_line.is_control_flow {
                    theme.background_control_info
                } else {
                    theme.foreground
                },
            );

        row_rect
    }
}

impl Widget for CodeViewerView {
    fn ui(
        self,
        user_interface: &mut Ui,
    ) -> Response {
        let theme = &self.app_context.theme;
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
                    vec2(36.0, Self::TOOLBAR_HEIGHT),
                    Button::new_from_theme(theme)
                        .background_color(Color32::TRANSPARENT)
                        .with_tooltip_text("Refresh memory pages."),
                );
                IconDraw::draw(&toolbar_user_interface, refresh_button.rect, &theme.icon_library.icon_handle_navigation_refresh);

                if refresh_button.clicked() {
                    CodeViewerViewData::refresh_memory_pages(self.code_viewer_view_data.clone(), self.app_context.engine_unprivileged_state.clone());
                }

                let home_button = toolbar_user_interface.add_sized(
                    vec2(36.0, Self::TOOLBAR_HEIGHT),
                    Button::new_from_theme(theme)
                        .background_color(Color32::TRANSPARENT)
                        .with_tooltip_text("Reset the code window to the start of the current page."),
                );
                IconDraw::draw(&toolbar_user_interface, home_button.rect, &theme.icon_library.icon_handle_navigation_home);

                if home_button.clicked() {
                    CodeViewerViewData::reset_viewport_to_page_start(self.code_viewer_view_data.clone());
                }

                let previous_window_button = toolbar_user_interface.add_sized(
                    vec2(36.0, Self::TOOLBAR_HEIGHT),
                    Button::new_from_theme(theme)
                        .background_color(Color32::TRANSPARENT)
                        .with_tooltip_text("Shift the current code window backward."),
                );
                IconDraw::draw(
                    &toolbar_user_interface,
                    previous_window_button.rect,
                    &theme.icon_library.icon_handle_navigation_left_arrow_small,
                );

                if previous_window_button.clicked() {
                    CodeViewerViewData::shift_viewport_window(
                        self.code_viewer_view_data.clone(),
                        -((CodeViewerViewData::CODE_WINDOW_SIZE_IN_BYTES / 2) as i64),
                    );
                }

                let next_window_button = toolbar_user_interface.add_sized(
                    vec2(36.0, Self::TOOLBAR_HEIGHT),
                    Button::new_from_theme(theme)
                        .background_color(Color32::TRANSPARENT)
                        .with_tooltip_text("Shift the current code window forward."),
                );
                IconDraw::draw(
                    &toolbar_user_interface,
                    next_window_button.rect,
                    &theme.icon_library.icon_handle_navigation_right_arrow_small,
                );

                if next_window_button.clicked() {
                    CodeViewerViewData::shift_viewport_window(self.code_viewer_view_data.clone(), (CodeViewerViewData::CODE_WINDOW_SIZE_IN_BYTES / 2) as i64);
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
                content_user_interface
                    .painter()
                    .rect_filled(content_user_interface.max_rect(), CornerRadius::ZERO, theme.background_panel);

                let process_bitness = self.get_process_bitness();
                let current_page = CodeViewerViewData::get_current_page(self.code_viewer_view_data.clone());

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

                        let instruction_lines = CodeViewerViewData::build_instruction_lines(self.code_viewer_view_data.clone(), process_bitness);
                        let pending_scroll_address = CodeViewerViewData::take_pending_scroll_address(self.code_viewer_view_data.clone());
                        let current_page_is_unreadable = CodeViewerViewData::is_current_page_unreadable(self.code_viewer_view_data.clone(), &current_page);

                        if instruction_lines.is_empty() && current_page_is_unreadable {
                            content_user_interface.centered_and_justified(|user_interface| {
                                user_interface.label(
                                    RichText::new("This page is currently unreadable, so no code rows could be decoded.")
                                        .font(theme.font_library.font_noto_sans.font_normal.clone())
                                        .color(theme.background_control_warning),
                                );
                            });
                        } else if instruction_lines.is_empty() {
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

                                    for instruction_line in &instruction_lines {
                                        let row_rect = self.render_instruction_row(user_interface, instruction_line, pending_scroll_address);
                                        row_rects_by_address.insert(instruction_line.address, row_rect);
                                    }

                                    Self::draw_jump_visuals(user_interface, &row_rects_by_address, &instruction_lines, theme);
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

                user_interface.add(self.code_viewer_footer_view.clone());
            })
            .response
    }
}
