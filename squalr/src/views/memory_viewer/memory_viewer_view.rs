use crate::{
    app_context::AppContext,
    ui::{
        draw::icon_draw::IconDraw,
        widgets::controls::{
            button::Button, context_menu::context_menu::ContextMenu, data_value_box::data_value_box_view::DataValueBoxView,
            toolbar_menu::toolbar_menu_item_view::ToolbarMenuItemView,
        },
    },
    views::memory_viewer::{
        memory_viewer_footer_view::MemoryViewerFooterView, memory_viewer_interpretation_panel_view::MemoryViewerInterpretationPanelView,
        view_data::memory_viewer_view_data::MemoryViewerViewData,
    },
    views::project_explorer::project_hierarchy::view_data::project_hierarchy_view_data::ProjectHierarchyViewData,
};
use eframe::egui::{
    Align, Align2, Color32, Direction, Event, Key, Layout, Pos2, Rect, Response, RichText, ScrollArea, Sense, Spinner, Ui, UiBuilder, Widget, pos2, vec2,
};
use epaint::{Color32 as EpaintColor32, CornerRadius, Stroke};
use squalr_engine_api::{
    commands::{
        memory::write::memory_write_request::MemoryWriteRequest, privileged_command_request::PrivilegedCommandRequest,
        unprivileged_command_request::UnprivilegedCommandRequest,
    },
    dependency_injection::dependency::Dependency,
    events::process::changed::process_changed_event::ProcessChangedEvent,
    structures::data_types::{built_in_types::u64::data_type_u64::DataTypeU64, data_type_ref::DataTypeRef},
};
use std::{sync::Arc, time::Duration};

#[derive(Clone)]
pub struct MemoryViewerView {
    app_context: Arc<AppContext>,
    memory_viewer_view_data: Dependency<MemoryViewerViewData>,
    project_hierarchy_view_data: Dependency<ProjectHierarchyViewData>,
    memory_viewer_footer_view: MemoryViewerFooterView,
    memory_viewer_interpretation_panel_view: MemoryViewerInterpretationPanelView,
}

impl MemoryViewerView {
    const GO_TO_ADDRESS_INPUT_ID: &'static str = "memory_viewer_go_to_address";
    pub const WINDOW_ID: &'static str = "window_memory_viewer";
    const TOOLBAR_HEIGHT: f32 = 32.0;
    const TOOLBAR_ROW_HEIGHT: f32 = 28.0;
    const ROW_HEIGHT: f32 = 20.0;
    const ADDRESS_COLUMN_WIDTH: f32 = 126.0;
    const HEX_CELL_WIDTH: f32 = 22.0;
    const ASCII_CELL_WIDTH: f32 = 10.0;
    const HEX_COLUMN_LEFT_PADDING: f32 = 8.0;
    const ASCII_COLUMN_LEFT_PADDING: f32 = 16.0;
    const ADDRESS_TEXT_LEFT_PADDING: f32 = 8.0;
    const ROW_TEXT_TOP_PADDING: f32 = 3.0;
    const CONTEXT_MENU_WIDTH: f32 = 220.0;
    const HEX_CARET_BLINK_INTERVAL: Duration = Duration::from_millis(500);
    const INTERPRETATION_PANEL_WIDTH: f32 = 320.0;
    const CONTENT_PANEL_SPACING: f32 = 10.0;

    pub fn new(app_context: Arc<AppContext>) -> Self {
        let memory_viewer_view_data = app_context
            .dependency_container
            .register(MemoryViewerViewData::new());
        let project_hierarchy_view_data = app_context
            .dependency_container
            .get_dependency::<ProjectHierarchyViewData>();
        let instance = Self {
            memory_viewer_footer_view: MemoryViewerFooterView::new(app_context.clone()),
            memory_viewer_interpretation_panel_view: MemoryViewerInterpretationPanelView::new(app_context.clone()),
            app_context,
            memory_viewer_view_data,
            project_hierarchy_view_data,
        };

        MemoryViewerViewData::refresh_memory_pages(instance.memory_viewer_view_data.clone(), instance.app_context.engine_unprivileged_state.clone());
        instance.listen_for_process_change();

        instance
    }

    fn listen_for_process_change(&self) {
        let engine_unprivileged_state = self.app_context.engine_unprivileged_state.clone();
        let memory_viewer_view_data = self.memory_viewer_view_data.clone();

        self.app_context
            .engine_unprivileged_state
            .listen_for_engine_event::<ProcessChangedEvent>(move |_process_changed_event| {
                MemoryViewerViewData::clear_for_process_change(memory_viewer_view_data.clone(), engine_unprivileged_state.clone());
                MemoryViewerViewData::refresh_memory_pages(memory_viewer_view_data.clone(), engine_unprivileged_state.clone());
            });
    }

    fn format_row_address(
        normalized_region: &squalr_engine_api::structures::memory::normalized_region::NormalizedRegion,
        row_index: usize,
    ) -> u64 {
        normalized_region
            .get_base_address()
            .saturating_add((row_index as u64).saturating_mul(MemoryViewerViewData::BYTES_PER_ROW))
    }

    fn format_hex_cell(byte_value: Option<u8>) -> String {
        byte_value
            .map(|byte_value| format!("{:02X}", byte_value))
            .unwrap_or_else(|| String::from("??"))
    }

    fn format_ascii_cell(byte_value: Option<u8>) -> char {
        match byte_value {
            Some(byte_value) if byte_value.is_ascii_graphic() || byte_value == b' ' => byte_value as char,
            Some(_) => '.',
            None => '?',
        }
    }

    fn get_hex_columns_left(row_rect: Rect) -> f32 {
        row_rect.min.x + Self::ADDRESS_COLUMN_WIDTH + Self::HEX_COLUMN_LEFT_PADDING
    }

    fn get_ascii_columns_left(row_rect: Rect) -> f32 {
        Self::get_hex_columns_left(row_rect) + (MemoryViewerViewData::BYTES_PER_ROW as f32 * Self::HEX_CELL_WIDTH) + Self::ASCII_COLUMN_LEFT_PADDING
    }

    fn resolve_byte_address_for_pointer(
        normalized_region: &squalr_engine_api::structures::memory::normalized_region::NormalizedRegion,
        row_index: usize,
        row_rect: Rect,
        pointer_position: Pos2,
    ) -> Option<u64> {
        if !row_rect.contains(pointer_position) {
            return None;
        }

        let row_offset = (row_index as u64).saturating_mul(MemoryViewerViewData::BYTES_PER_ROW);
        let hex_columns_left = Self::get_hex_columns_left(row_rect);
        let ascii_columns_left = Self::get_ascii_columns_left(row_rect);
        let column_index = if pointer_position.x >= hex_columns_left
            && pointer_position.x < hex_columns_left + (MemoryViewerViewData::BYTES_PER_ROW as f32 * Self::HEX_CELL_WIDTH)
        {
            ((pointer_position.x - hex_columns_left) / Self::HEX_CELL_WIDTH).floor() as u64
        } else if pointer_position.x >= ascii_columns_left
            && pointer_position.x < ascii_columns_left + (MemoryViewerViewData::BYTES_PER_ROW as f32 * Self::ASCII_CELL_WIDTH)
        {
            ((pointer_position.x - ascii_columns_left) / Self::ASCII_CELL_WIDTH).floor() as u64
        } else {
            return None;
        };

        let byte_offset = row_offset.saturating_add(column_index);

        if byte_offset >= normalized_region.get_region_size() {
            return None;
        }

        Some(normalized_region.get_base_address().saturating_add(byte_offset))
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

    fn resolve_row_selection_column_range(
        selected_address_bounds: Option<(u64, u64)>,
        row_address: u64,
        row_byte_count: u64,
    ) -> Option<(u64, u64)> {
        let (selection_start_address, selection_end_address) = selected_address_bounds?;
        let row_end_address_exclusive = row_address.saturating_add(row_byte_count);

        if selection_end_address < row_address || selection_start_address >= row_end_address_exclusive {
            return None;
        }

        let row_selection_start_address = selection_start_address.max(row_address);
        let row_selection_end_address = selection_end_address.min(row_end_address_exclusive.saturating_sub(1));

        Some((
            row_selection_start_address.saturating_sub(row_address),
            row_selection_end_address.saturating_sub(row_address),
        ))
    }

    fn build_selection_band_rect(
        row_rect: Rect,
        column_start_index: u64,
        column_end_index: u64,
        column_width: f32,
        columns_left: f32,
        right_padding: f32,
    ) -> Rect {
        Rect::from_min_max(
            pos2(columns_left + (column_start_index as f32) * column_width, row_rect.min.y + 1.0),
            pos2(
                columns_left + ((column_end_index + 1) as f32) * column_width - right_padding,
                row_rect.max.y - 1.0,
            ),
        )
    }

    fn build_hex_text(
        byte_value: Option<u8>,
        byte_address: u64,
        hex_edit_state: Option<&crate::views::memory_viewer::view_data::memory_viewer_view_data::MemoryViewerHexEditState>,
    ) -> String {
        let Some(hex_edit_state) = hex_edit_state else {
            return Self::format_hex_cell(byte_value);
        };

        if hex_edit_state.cursor_address != byte_address || hex_edit_state.active_nibble_index != 1 {
            return Self::format_hex_cell(byte_value);
        }

        let Some(pending_high_nibble) = hex_edit_state.pending_high_nibble else {
            return Self::format_hex_cell(byte_value);
        };

        format!("{:X}?", pending_high_nibble)
    }

    fn draw_hex_edit_caret(
        user_interface: &Ui,
        hex_rect: Rect,
        active_nibble_index: u8,
        caret_color: EpaintColor32,
    ) {
        let nibble_width = (hex_rect.width() / 2.0).max(1.0);
        let nibble_rect = if active_nibble_index == 0 {
            Rect::from_min_max(hex_rect.min, pos2(hex_rect.min.x + nibble_width, hex_rect.max.y))
        } else {
            Rect::from_min_max(pos2(hex_rect.min.x + nibble_width, hex_rect.min.y), hex_rect.max)
        };

        user_interface.painter().rect_stroke(
            nibble_rect.shrink2(vec2(1.0, 1.5)),
            CornerRadius::same(2),
            Stroke::new(1.0, caret_color),
            epaint::StrokeKind::Inside,
        );
    }

    fn dispatch_memory_write(
        &self,
        write_start_address: u64,
        written_bytes: Vec<u8>,
    ) {
        let memory_viewer_view_data = self.memory_viewer_view_data.clone();
        let engine_unprivileged_state = self.app_context.engine_unprivileged_state.clone();
        let engine_unprivileged_state_for_callback = engine_unprivileged_state.clone();
        let written_bytes_for_refresh = written_bytes.clone();
        let memory_write_request = MemoryWriteRequest {
            address: write_start_address,
            module_name: String::new(),
            value: written_bytes,
        };

        memory_write_request.send(&engine_unprivileged_state, move |memory_write_response| {
            if memory_write_response.success {
                MemoryViewerViewData::apply_memory_write(memory_viewer_view_data.clone(), write_start_address, &written_bytes_for_refresh);
                engine_unprivileged_state_for_callback.request_virtual_snapshot_refresh(MemoryViewerViewData::WINDOW_VIRTUAL_SNAPSHOT_ID);
            } else {
                log::warn!("Memory viewer hex edit memory write command failed.");
            }
        });
    }

    fn dispatch_add_address_to_project(
        &self,
        absolute_address: u64,
    ) {
        let target_directory_path = ProjectHierarchyViewData::get_selected_directory_path(self.project_hierarchy_view_data.clone());
        let Some(project_items_create_request) =
            MemoryViewerViewData::build_address_project_item_create_request(self.memory_viewer_view_data.clone(), absolute_address, target_directory_path)
        else {
            log::warn!("Failed to build memory viewer project item create request.");
            return;
        };

        project_items_create_request.send(&self.app_context.engine_unprivileged_state, |project_items_create_response| {
            if !project_items_create_response.success {
                log::warn!("Memory viewer add-to-project command failed.");
            }
        });
    }

    fn build_context_menu_add_label(
        &self,
        context_menu_address: u64,
    ) -> String {
        let selected_address_bounds = MemoryViewerViewData::get_selected_address_bounds(self.memory_viewer_view_data.clone());

        match selected_address_bounds {
            Some((selection_start_address, selection_end_address))
                if context_menu_address >= selection_start_address && context_menu_address <= selection_end_address =>
            {
                let selected_byte_count = selection_end_address
                    .saturating_sub(selection_start_address)
                    .saturating_add(1);

                if selected_byte_count > 1 {
                    format!("Add Selection As u8[{}]", selected_byte_count)
                } else {
                    String::from("Add Selection As u8")
                }
            }
            _ => String::from("Add Byte As u8"),
        }
    }
}

impl Widget for MemoryViewerView {
    fn ui(
        self,
        user_interface: &mut Ui,
    ) -> Response {
        MemoryViewerViewData::clear_stale_request_state_if_needed(self.memory_viewer_view_data.clone());
        let memory_viewer_has_keyboard_focus = MemoryViewerViewData::has_keyboard_focus(self.memory_viewer_view_data.clone());
        let repaint_interval = if memory_viewer_has_keyboard_focus {
            Self::HEX_CARET_BLINK_INTERVAL
        } else {
            MemoryViewerViewData::SNAPSHOT_REFRESH_INTERVAL
        };
        user_interface.ctx().request_repaint_after(repaint_interval);

        if let Some(virtual_snapshot) = self
            .app_context
            .engine_unprivileged_state
            .get_virtual_snapshot(MemoryViewerViewData::WINDOW_VIRTUAL_SNAPSHOT_ID)
        {
            MemoryViewerViewData::apply_virtual_snapshot_results(self.memory_viewer_view_data.clone(), &virtual_snapshot);
        }

        let theme = &self.app_context.theme;

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
                    MemoryViewerViewData::refresh_memory_pages(self.memory_viewer_view_data.clone(), self.app_context.engine_unprivileged_state.clone());
                }

                let is_querying_memory_pages = self
                    .memory_viewer_view_data
                    .read("Memory viewer toolbar state")
                    .map(|memory_viewer_view_data| memory_viewer_view_data.is_querying_memory_pages)
                    .unwrap_or(false);

                if is_querying_memory_pages {
                    toolbar_user_interface.add_space(8.0);
                    toolbar_user_interface.add(Spinner::new().color(theme.foreground));
                }

                let go_to_preview_text = MemoryViewerViewData::get_go_to_address_preview_text(self.memory_viewer_view_data.clone());
                let address_data_type = DataTypeRef::new(DataTypeU64::DATA_TYPE_ID);
                let mut should_seek_to_address = DataValueBoxView::consume_commit_on_enter(user_interface, Self::GO_TO_ADDRESS_INPUT_ID);
                toolbar_user_interface.add_space(12.0);
                if let Some(mut memory_viewer_view_data) = self
                    .memory_viewer_view_data
                    .write("Memory viewer toolbar go to address input")
                {
                    toolbar_user_interface.add(
                        DataValueBoxView::new(
                            self.app_context.clone(),
                            &mut memory_viewer_view_data.go_to_address_input,
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
                        .with_tooltip_text("Seek the memory viewer to the requested address."),
                );
                IconDraw::draw(
                    &toolbar_user_interface,
                    apply_go_to_button.rect,
                    &theme.icon_library.icon_handle_navigation_right_arrow,
                );
                should_seek_to_address |= apply_go_to_button.clicked();

                if should_seek_to_address {
                    MemoryViewerViewData::seek_to_input_address(self.memory_viewer_view_data.clone());
                }

                let footer_height = self.memory_viewer_footer_view.get_height();
                let content_rect = user_interface
                    .available_rect_before_wrap()
                    .with_max_y(user_interface.available_rect_before_wrap().max.y - footer_height);
                let content_response = user_interface.allocate_rect(content_rect, Sense::click_and_drag());
                let mut content_user_interface = user_interface.new_child(
                    UiBuilder::new()
                        .max_rect(content_response.rect)
                        .layout(Layout::top_down(Align::Min)),
                );
                content_user_interface.set_clip_rect(content_response.rect);
                if content_response.clicked() || content_response.drag_started() {
                    MemoryViewerViewData::set_keyboard_focus(self.memory_viewer_view_data.clone(), true);
                }

                if user_interface.input(|input_state| input_state.pointer.primary_released()) {
                    MemoryViewerViewData::set_drag_selection_active(self.memory_viewer_view_data.clone(), false);
                }

                if user_interface.input(|input_state| input_state.pointer.any_pressed())
                    && user_interface
                        .input(|input_state| input_state.pointer.interact_pos())
                        .map(|pointer_position| !content_response.rect.contains(pointer_position))
                        .unwrap_or(false)
                {
                    MemoryViewerViewData::set_keyboard_focus(self.memory_viewer_view_data.clone(), false);
                    MemoryViewerViewData::hide_context_menu(self.memory_viewer_view_data.clone());
                    MemoryViewerViewData::set_drag_selection_active(self.memory_viewer_view_data.clone(), false);
                }

                if memory_viewer_has_keyboard_focus && user_interface.input(|input_state| input_state.key_pressed(Key::Backspace)) {
                    MemoryViewerViewData::handle_hex_edit_backspace(self.memory_viewer_view_data.clone());
                }

                if memory_viewer_has_keyboard_focus {
                    if user_interface.input(|input_state| input_state.key_pressed(Key::Escape)) {
                        MemoryViewerViewData::clear_selection(self.memory_viewer_view_data.clone());
                        MemoryViewerViewData::hide_context_menu(self.memory_viewer_view_data.clone());
                    }

                    let is_select_all_shortcut_pressed =
                        user_interface.input(|input_state| (input_state.modifiers.command || input_state.modifiers.ctrl) && input_state.key_pressed(Key::A));
                    let is_shift_modifier_active = user_interface.input(|input_state| input_state.modifiers.shift);

                    if is_select_all_shortcut_pressed {
                        MemoryViewerViewData::select_all_bytes_on_current_page(self.memory_viewer_view_data.clone());
                    }

                    if user_interface.input(|input_state| input_state.key_pressed(Key::ArrowLeft)) {
                        MemoryViewerViewData::move_cursor_horizontal(self.memory_viewer_view_data.clone(), -1, is_shift_modifier_active);
                    }

                    if user_interface.input(|input_state| input_state.key_pressed(Key::ArrowRight)) {
                        MemoryViewerViewData::move_cursor_horizontal(self.memory_viewer_view_data.clone(), 1, is_shift_modifier_active);
                    }

                    if user_interface.input(|input_state| input_state.key_pressed(Key::ArrowUp)) {
                        MemoryViewerViewData::move_cursor_vertical(self.memory_viewer_view_data.clone(), -1, is_shift_modifier_active);
                    }

                    if user_interface.input(|input_state| input_state.key_pressed(Key::ArrowDown)) {
                        MemoryViewerViewData::move_cursor_vertical(self.memory_viewer_view_data.clone(), 1, is_shift_modifier_active);
                    }
                }

                if memory_viewer_has_keyboard_focus {
                    let typed_hex_characters = user_interface.input(|input_state| {
                        input_state
                            .events
                            .iter()
                            .filter_map(|event| match event {
                                Event::Text(text) => Some(text.clone()),
                                _ => None,
                            })
                            .collect::<Vec<String>>()
                    });

                    for typed_text in typed_hex_characters {
                        for typed_character in typed_text.chars() {
                            if let Some((write_start_address, written_bytes)) =
                                MemoryViewerViewData::append_hex_edit_character(self.memory_viewer_view_data.clone(), typed_character)
                            {
                                self.dispatch_memory_write(write_start_address, written_bytes);
                            }
                        }
                    }
                }
                let current_page = MemoryViewerViewData::get_current_page(self.memory_viewer_view_data.clone());
                let current_page_is_unreadable = MemoryViewerViewData::get_current_page_is_unreadable(self.memory_viewer_view_data.clone());

                content_user_interface
                    .painter()
                    .rect_filled(content_user_interface.max_rect(), CornerRadius::ZERO, theme.background_panel);

                let interpretation_panel_width = Self::INTERPRETATION_PANEL_WIDTH
                    .min((content_response.rect.width() * 0.42).max(220.0))
                    .max(220.0);
                let content_area_rect = content_response.rect;
                let left_content_rect = Rect::from_min_max(
                    content_area_rect.min,
                    pos2(
                        (content_area_rect.max.x - interpretation_panel_width - Self::CONTENT_PANEL_SPACING).max(content_area_rect.min.x),
                        content_area_rect.max.y,
                    ),
                );
                let right_panel_rect = Rect::from_min_max(
                    pos2(
                        (left_content_rect.max.x + Self::CONTENT_PANEL_SPACING).min(content_area_rect.max.x),
                        content_area_rect.min.y,
                    ),
                    content_area_rect.max,
                );
                let mut left_content_user_interface = user_interface.new_child(
                    UiBuilder::new()
                        .max_rect(left_content_rect)
                        .layout(Layout::top_down(Align::Min)),
                );
                left_content_user_interface.set_clip_rect(left_content_rect);
                let mut right_panel_user_interface = user_interface.new_child(
                    UiBuilder::new()
                        .max_rect(right_panel_rect)
                        .layout(Layout::top_down(Align::Min)),
                );
                right_panel_user_interface.set_clip_rect(right_panel_rect);

                match current_page {
                    Some(current_page) => {
                        let row_count = MemoryViewerViewData::get_page_row_count(&current_page);
                        let page_base_address = current_page.get_base_address();
                        let pending_scroll_row_index = MemoryViewerViewData::take_pending_scroll_row_index(self.memory_viewer_view_data.clone(), &current_page);
                        let mut rows_scroll_area = ScrollArea::vertical()
                            .id_salt("memory_viewer_rows")
                            .auto_shrink([false, false]);

                        if let Some(pending_scroll_row_index) = pending_scroll_row_index {
                            rows_scroll_area = rows_scroll_area.vertical_scroll_offset((pending_scroll_row_index as f32) * Self::ROW_HEIGHT);
                        }

                        rows_scroll_area.show_rows(
                            &mut left_content_user_interface,
                            Self::ROW_HEIGHT,
                            row_count,
                            |user_interface, visible_row_range| {
                                let pointer_interaction_position = user_interface.input(|input_state| input_state.pointer.interact_pos());
                                let is_shift_modifier_active = user_interface.input(|input_state| input_state.modifiers.shift);
                                let is_drag_selection_active = MemoryViewerViewData::is_drag_selection_active(self.memory_viewer_view_data.clone());
                                let caret_is_visible = memory_viewer_has_keyboard_focus
                                    && ((user_interface.input(|input_state| input_state.time) / Self::HEX_CARET_BLINK_INTERVAL.as_secs_f64()).floor() as u64)
                                        .is_multiple_of(2);
                                let visible_chunk_queries = MemoryViewerViewData::build_visible_chunk_queries(&current_page, visible_row_range.clone());

                                self.app_context
                                    .engine_unprivileged_state
                                    .set_virtual_snapshot_queries(
                                        MemoryViewerViewData::WINDOW_VIRTUAL_SNAPSHOT_ID,
                                        MemoryViewerViewData::SNAPSHOT_REFRESH_INTERVAL,
                                        visible_chunk_queries,
                                    );
                                self.app_context
                                    .engine_unprivileged_state
                                    .request_virtual_snapshot_refresh(MemoryViewerViewData::WINDOW_VIRTUAL_SNAPSHOT_ID);

                                for row_index in visible_row_range {
                                    let (row_rect, row_response) =
                                        user_interface.allocate_exact_size(vec2(user_interface.available_width(), Self::ROW_HEIGHT), Sense::click_and_drag());
                                    let row_offset = (row_index as u64).saturating_mul(MemoryViewerViewData::BYTES_PER_ROW);
                                    let row_address = Self::format_row_address(&current_page, row_index);
                                    let hovered_byte_address = row_response.hover_pos().and_then(|pointer_position| {
                                        Self::resolve_byte_address_for_pointer(&current_page, row_index, row_rect, pointer_position)
                                    });
                                    let pointer_byte_address = pointer_interaction_position.and_then(|pointer_position| {
                                        Self::resolve_byte_address_for_pointer(&current_page, row_index, row_rect, pointer_position)
                                    });
                                    let selected_address_bounds = MemoryViewerViewData::get_selected_address_bounds(self.memory_viewer_view_data.clone());
                                    let hex_edit_state = MemoryViewerViewData::get_hex_edit_state(self.memory_viewer_view_data.clone());

                                    if row_response.clicked() {
                                        MemoryViewerViewData::set_keyboard_focus(self.memory_viewer_view_data.clone(), true);
                                        MemoryViewerViewData::set_drag_selection_active(self.memory_viewer_view_data.clone(), false);

                                        if let Some(hovered_byte_address) = hovered_byte_address {
                                            if is_shift_modifier_active {
                                                MemoryViewerViewData::extend_byte_selection(self.memory_viewer_view_data.clone(), hovered_byte_address);
                                            } else {
                                                MemoryViewerViewData::begin_byte_selection(self.memory_viewer_view_data.clone(), hovered_byte_address);
                                            }
                                        }
                                    }

                                    if row_response.drag_started() {
                                        MemoryViewerViewData::set_keyboard_focus(self.memory_viewer_view_data.clone(), true);
                                        MemoryViewerViewData::set_drag_selection_active(self.memory_viewer_view_data.clone(), true);

                                        if let Some(hovered_byte_address) = hovered_byte_address {
                                            if is_shift_modifier_active {
                                                MemoryViewerViewData::extend_byte_selection(self.memory_viewer_view_data.clone(), hovered_byte_address);
                                            } else {
                                                MemoryViewerViewData::begin_byte_selection(self.memory_viewer_view_data.clone(), hovered_byte_address);
                                            }
                                        }
                                    }

                                    if is_drag_selection_active {
                                        if let Some(pointer_byte_address) = pointer_byte_address.filter(|_| selected_address_bounds.is_some()) {
                                            MemoryViewerViewData::update_byte_selection(self.memory_viewer_view_data.clone(), pointer_byte_address);
                                        }
                                    }

                                    if row_response.secondary_clicked() {
                                        MemoryViewerViewData::show_context_menu(
                                            self.memory_viewer_view_data.clone(),
                                            hovered_byte_address.unwrap_or(row_address),
                                            row_response.hover_pos().unwrap_or(row_rect.left_bottom()),
                                        );
                                    }

                                    user_interface
                                        .painter()
                                        .with_clip_rect(row_rect.intersect(user_interface.clip_rect()))
                                        .text(
                                            pos2(row_rect.min.x + Self::ADDRESS_TEXT_LEFT_PADDING, row_rect.min.y + Self::ROW_TEXT_TOP_PADDING),
                                            Align2::LEFT_TOP,
                                            format!("{:016X}", row_address),
                                            theme.font_library.font_ubuntu_mono_bold.font_normal.clone(),
                                            theme.hexadecimal_green,
                                        );

                                    let hex_columns_left = Self::get_hex_columns_left(row_rect);
                                    let ascii_columns_left = Self::get_ascii_columns_left(row_rect);
                                    let row_byte_count = (current_page.get_region_size().saturating_sub(row_offset)).min(MemoryViewerViewData::BYTES_PER_ROW);

                                    if let Some((selection_start_column, selection_end_column)) =
                                        Self::resolve_row_selection_column_range(selected_address_bounds, row_address, row_byte_count)
                                    {
                                        let hex_selection_rect = Self::build_selection_band_rect(
                                            row_rect,
                                            selection_start_column,
                                            selection_end_column,
                                            Self::HEX_CELL_WIDTH,
                                            hex_columns_left,
                                            2.0,
                                        );
                                        let ascii_selection_rect = Self::build_selection_band_rect(
                                            row_rect,
                                            selection_start_column,
                                            selection_end_column,
                                            Self::ASCII_CELL_WIDTH,
                                            ascii_columns_left,
                                            0.0,
                                        );

                                        Self::draw_selection_background(user_interface, hex_selection_rect, theme.selected_background, theme.selected_border);
                                        Self::draw_selection_background(user_interface, ascii_selection_rect, theme.selected_background, theme.selected_border);
                                    }

                                    for column_index in 0..MemoryViewerViewData::BYTES_PER_ROW {
                                        let byte_offset = row_offset.saturating_add(column_index);

                                        if byte_offset >= current_page.get_region_size() {
                                            continue;
                                        }

                                        let byte_address = page_base_address.saturating_add(byte_offset);
                                        let byte_value = MemoryViewerViewData::get_cached_byte_for_page(
                                            self.memory_viewer_view_data.clone(),
                                            page_base_address,
                                            byte_offset,
                                        );
                                        let hex_text = Self::build_hex_text(byte_value, byte_address, hex_edit_state.as_ref());
                                        let ascii_text = Self::format_ascii_cell(byte_value).to_string();
                                        let hex_rect = Rect::from_min_max(
                                            pos2(hex_columns_left + (column_index as f32) * Self::HEX_CELL_WIDTH, row_rect.min.y + 1.0),
                                            pos2(
                                                hex_columns_left + ((column_index + 1) as f32) * Self::HEX_CELL_WIDTH - 2.0,
                                                row_rect.max.y - 1.0,
                                            ),
                                        );
                                        let ascii_rect = Rect::from_min_max(
                                            pos2(ascii_columns_left + (column_index as f32) * Self::ASCII_CELL_WIDTH, row_rect.min.y + 1.0),
                                            pos2(ascii_columns_left + ((column_index + 1) as f32) * Self::ASCII_CELL_WIDTH, row_rect.max.y - 1.0),
                                        );

                                        if caret_is_visible
                                            && hex_edit_state
                                                .as_ref()
                                                .map(|hex_edit_state| hex_edit_state.cursor_address == byte_address)
                                                .unwrap_or(false)
                                        {
                                            Self::draw_hex_edit_caret(
                                                user_interface,
                                                hex_rect,
                                                hex_edit_state
                                                    .as_ref()
                                                    .map(|hex_edit_state| hex_edit_state.active_nibble_index)
                                                    .unwrap_or(0),
                                                theme.hexadecimal_green,
                                            );
                                        }

                                        user_interface
                                            .painter()
                                            .with_clip_rect(hex_rect.intersect(user_interface.clip_rect()))
                                            .text(
                                                pos2(hex_rect.min.x + 1.0, row_rect.min.y + Self::ROW_TEXT_TOP_PADDING),
                                                Align2::LEFT_TOP,
                                                hex_text,
                                                theme.font_library.font_ubuntu_mono_bold.font_normal.clone(),
                                                theme.hexadecimal_green,
                                            );
                                        user_interface
                                            .painter()
                                            .with_clip_rect(ascii_rect.intersect(user_interface.clip_rect()))
                                            .text(
                                                pos2(ascii_rect.min.x + 1.0, row_rect.min.y + Self::ROW_TEXT_TOP_PADDING),
                                                Align2::LEFT_TOP,
                                                ascii_text,
                                                theme.font_library.font_ubuntu_mono_bold.font_normal.clone(),
                                                theme.foreground,
                                            );
                                    }
                                }
                            },
                        );

                        if current_page_is_unreadable {
                            left_content_user_interface.add_space(8.0);
                            left_content_user_interface.label(
                                RichText::new("This page is currently unreadable, so visible bytes remain as ??.")
                                    .font(theme.font_library.font_noto_sans.font_normal.clone())
                                    .color(theme.background_control_warning),
                            );
                        }
                    }
                    None if is_querying_memory_pages => {
                        left_content_user_interface.allocate_ui_with_layout(
                            vec2(left_content_user_interface.available_width(), 32.0),
                            Layout::centered_and_justified(Direction::LeftToRight),
                            |user_interface| {
                                user_interface.add(Spinner::new().color(theme.foreground));
                            },
                        );
                    }
                    None => {
                        left_content_user_interface.centered_and_justified(|user_interface| {
                            user_interface.label(
                                RichText::new("Attach to a process to browse memory pages.")
                                    .font(theme.font_library.font_noto_sans.font_normal.clone())
                                    .color(theme.foreground_preview),
                            );
                        });
                    }
                }

                right_panel_user_interface.add(self.memory_viewer_interpretation_panel_view.clone());

                if let Some((context_menu_address, context_menu_position)) = MemoryViewerViewData::get_context_menu_state(self.memory_viewer_view_data.clone())
                {
                    let mut open = true;
                    let add_action_label = self.build_context_menu_add_label(context_menu_address);

                    ContextMenu::new(
                        self.app_context.clone(),
                        "memory_viewer_context_menu",
                        context_menu_position,
                        |user_interface, should_close| {
                            if user_interface
                                .add(ToolbarMenuItemView::new(
                                    self.app_context.clone(),
                                    &add_action_label,
                                    "memory_viewer_ctx_add_to_project",
                                    &None,
                                    Self::CONTEXT_MENU_WIDTH,
                                ))
                                .clicked()
                            {
                                self.dispatch_add_address_to_project(context_menu_address);
                                *should_close = true;
                            }
                        },
                    )
                    .width(Self::CONTEXT_MENU_WIDTH)
                    .corner_radius(8)
                    .show(user_interface, &mut open);

                    if !open {
                        MemoryViewerViewData::hide_context_menu(self.memory_viewer_view_data.clone());
                    }
                }

                user_interface.add(self.memory_viewer_footer_view.clone());
            })
            .response
    }
}
