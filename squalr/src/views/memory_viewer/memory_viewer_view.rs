use crate::{
    app_context::AppContext,
    ui::{draw::icon_draw::IconDraw, widgets::controls::button::Button},
    views::memory_viewer::{memory_viewer_footer_view::MemoryViewerFooterView, view_data::memory_viewer_view_data::MemoryViewerViewData},
};
use eframe::egui::{Align, Align2, Color32, Direction, Layout, Response, RichText, ScrollArea, Sense, Spinner, Ui, UiBuilder, Widget, vec2};
use epaint::CornerRadius;
use squalr_engine_api::{dependency_injection::dependency::Dependency, events::process::changed::process_changed_event::ProcessChangedEvent};
use std::sync::Arc;

#[derive(Clone)]
pub struct MemoryViewerView {
    app_context: Arc<AppContext>,
    memory_viewer_view_data: Dependency<MemoryViewerViewData>,
    memory_viewer_footer_view: MemoryViewerFooterView,
}

impl MemoryViewerView {
    pub const WINDOW_ID: &'static str = "window_memory_viewer";
    const TOOLBAR_HEIGHT: f32 = 28.0;
    const ROW_HEIGHT: f32 = 20.0;

    pub fn new(app_context: Arc<AppContext>) -> Self {
        let memory_viewer_view_data = app_context
            .dependency_container
            .register(MemoryViewerViewData::new());
        let instance = Self {
            memory_viewer_footer_view: MemoryViewerFooterView::new(app_context.clone()),
            app_context,
            memory_viewer_view_data,
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
}

impl Widget for MemoryViewerView {
    fn ui(
        self,
        user_interface: &mut Ui,
    ) -> Response {
        MemoryViewerViewData::clear_stale_request_state_if_needed(self.memory_viewer_view_data.clone());
        user_interface
            .ctx()
            .request_repaint_after(MemoryViewerViewData::SNAPSHOT_REFRESH_INTERVAL);

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
                    vec2(36.0, Self::TOOLBAR_HEIGHT),
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

                toolbar_user_interface.add_space(8.0);
                toolbar_user_interface.label(
                    RichText::new("Memory Viewer")
                        .font(theme.font_library.font_noto_sans.font_header.clone())
                        .color(theme.foreground),
                );

                if is_querying_memory_pages {
                    toolbar_user_interface.add_space(8.0);
                    toolbar_user_interface.add(Spinner::new().color(theme.foreground));
                }

                let footer_height = self.memory_viewer_footer_view.get_height();
                let content_rect = user_interface
                    .available_rect_before_wrap()
                    .with_max_y(user_interface.available_rect_before_wrap().max.y - footer_height);
                let content_response = user_interface.allocate_rect(content_rect, Sense::empty());
                let mut content_user_interface = user_interface.new_child(
                    UiBuilder::new()
                        .max_rect(content_response.rect)
                        .layout(Layout::top_down(Align::Min)),
                );
                let current_page = MemoryViewerViewData::get_current_page(self.memory_viewer_view_data.clone());
                let current_page_is_unreadable = MemoryViewerViewData::get_current_page_is_unreadable(self.memory_viewer_view_data.clone());

                content_user_interface
                    .painter()
                    .rect_filled(content_user_interface.max_rect(), CornerRadius::ZERO, theme.background_panel);

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

                        rows_scroll_area.show_rows(&mut content_user_interface, Self::ROW_HEIGHT, row_count, |user_interface, visible_row_range| {
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
                                let (row_rect, _row_response) =
                                    user_interface.allocate_exact_size(vec2(user_interface.available_width(), Self::ROW_HEIGHT), Sense::hover());
                                let row_offset = (row_index as u64).saturating_mul(MemoryViewerViewData::BYTES_PER_ROW);
                                let row_address = Self::format_row_address(&current_page, row_index);
                                let mut hex_columns = Vec::new();
                                let mut ascii_columns = String::new();

                                for column_index in 0..MemoryViewerViewData::BYTES_PER_ROW {
                                    let byte_offset = row_offset.saturating_add(column_index);
                                    let byte_value = if byte_offset < current_page.get_region_size() {
                                        MemoryViewerViewData::get_cached_byte_for_page(self.memory_viewer_view_data.clone(), page_base_address, byte_offset)
                                    } else {
                                        None
                                    };

                                    hex_columns.push(Self::format_hex_cell(byte_value));

                                    if byte_offset < current_page.get_region_size() {
                                        ascii_columns.push(Self::format_ascii_cell(byte_value));
                                    } else {
                                        ascii_columns.push(' ');
                                    }
                                }

                                let row_text = format!("{:016X}  {}  |{}|", row_address, hex_columns.join(" "), ascii_columns);

                                user_interface.painter().text(
                                    row_rect.min + vec2(8.0, 3.0),
                                    Align2::LEFT_TOP,
                                    row_text,
                                    theme.font_library.font_ubuntu_mono_bold.font_normal.clone(),
                                    theme.foreground,
                                );
                            }
                        });

                        if current_page_is_unreadable {
                            content_user_interface.add_space(8.0);
                            content_user_interface.label(
                                RichText::new("This page is currently unreadable, so visible bytes remain as ??.")
                                    .font(theme.font_library.font_noto_sans.font_normal.clone())
                                    .color(theme.background_control_warning),
                            );
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
                                RichText::new("Attach to a process to browse memory pages.")
                                    .font(theme.font_library.font_noto_sans.font_normal.clone())
                                    .color(theme.foreground_preview),
                            );
                        });
                    }
                }

                user_interface.add(self.memory_viewer_footer_view.clone());
            })
            .response
    }
}
