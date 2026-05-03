use crate::state::pane_entry_row::PaneEntryRow;
use crate::views::entry_row_viewport::build_selection_relative_viewport_range;
use crate::views::memory_viewer::summary::build_memory_viewer_summary_lines;
use squalr_engine_api::{
    commands::project_items::create::project_items_create_request::ProjectItemsCreateRequest,
    conversions::storage_size_conversions::StorageSizeConversions,
    plugins::memory_view::PageRetrievalMode,
    structures::{
        data_types::{built_in_types::u8::data_type_u8::DataTypeU8, data_type_ref::DataTypeRef},
        data_values::container_type::ContainerType,
        memory::{
            address_display::{format_absolute_address, format_module_address},
            normalized_module::NormalizedModule,
            normalized_region::NormalizedRegion,
        },
        structs::{symbolic_field_definition::SymbolicFieldDefinition, symbolic_struct_definition::SymbolicStructDefinition},
    },
};
use squalr_engine_session::{
    engine_unprivileged_state::EngineUnprivilegedState,
    virtual_snapshots::{virtual_snapshot::VirtualSnapshot, virtual_snapshot_query::VirtualSnapshotQuery},
};
use std::{
    collections::{BTreeMap, HashMap, HashSet},
    ops::Range,
    path::PathBuf,
    sync::Arc,
    time::Duration,
};

#[derive(Clone, Debug, Default)]
struct MemoryViewerPageCache {
    cached_chunks: BTreeMap<u64, Vec<u8>>,
}

impl MemoryViewerPageCache {
    fn cache_chunk(
        &mut self,
        chunk_offset: u64,
        bytes: Vec<u8>,
    ) {
        self.cached_chunks.insert(chunk_offset, bytes);
    }

    fn get_cached_byte(
        &self,
        byte_offset: u64,
    ) -> Option<u8> {
        let chunk_offset = byte_offset - (byte_offset % MemoryViewerPaneState::QUERY_CHUNK_SIZE_IN_BYTES);
        let chunk_bytes = self.cached_chunks.get(&chunk_offset)?;
        let chunk_local_index = byte_offset.saturating_sub(chunk_offset) as usize;

        chunk_bytes.get(chunk_local_index).copied()
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum MemoryViewerInputMode {
    #[default]
    Normal,
    SeekInput,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MemoryViewerSelectionRange {
    pub anchor_address: u64,
    pub active_address: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MemoryViewerHexEditState {
    pub cursor_address: u64,
    pub active_nibble_index: u8,
    pub pending_high_nibble: Option<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MemoryViewerSelectionSummary {
    pub selection_start_address: u64,
    pub selection_end_address: u64,
    pub selection_display_text: String,
    pub selected_bytes: Vec<Option<u8>>,
}

#[derive(Clone, Debug)]
pub struct MemoryViewerPaneState {
    pub virtual_pages: Vec<NormalizedRegion>,
    pub modules: Vec<NormalizedModule>,
    pub current_page_index: u64,
    pub cached_last_page_index: u64,
    pub selected_row_index: usize,
    pub last_visible_row_capacity: usize,
    pub page_retrieval_mode: PageRetrievalMode,
    pub stats_string: String,
    pub status_message: String,
    pub input_mode: MemoryViewerInputMode,
    pub pending_seek_input: String,
    pub selected_byte_range: Option<MemoryViewerSelectionRange>,
    pub hex_edit_state: Option<MemoryViewerHexEditState>,
    pub is_querying_memory_pages: bool,
    pub has_loaded_memory_pages_once: bool,
    last_applied_snapshot_generation: u64,
    page_caches_by_base_address: HashMap<u64, MemoryViewerPageCache>,
    unreadable_page_base_addresses: HashSet<u64>,
}

impl MemoryViewerPaneState {
    pub const VIRTUAL_SNAPSHOT_ID: &'static str = "tui_memory_viewer";
    pub const BYTES_PER_ROW: u64 = 16;
    pub const MAX_SELECTION_SIZE_IN_BYTES: u64 = 2 * 1024 * 1024;
    pub const QUERY_CHUNK_SIZE_IN_BYTES: u64 = 256;
    pub const QUERY_PREFETCH_CHUNK_COUNT: u64 = 1;
    pub const SNAPSHOT_REFRESH_INTERVAL: Duration = Duration::from_millis(500);

    pub fn summary_lines(&self) -> Vec<String> {
        build_memory_viewer_summary_lines(self)
    }

    pub fn set_viewport_row_capacity(
        &mut self,
        viewport_row_capacity: usize,
    ) {
        self.last_visible_row_capacity = viewport_row_capacity;
        self.synchronize_selected_row_index_with_active_address();
    }

    pub fn current_page_base_address(&self) -> Option<u64> {
        self.current_page().map(NormalizedRegion::get_base_address)
    }

    pub fn selected_row_address(&self) -> Option<u64> {
        let current_page = self.current_page()?;
        let selected_row_offset = (self.selected_row_index as u64).saturating_mul(Self::BYTES_PER_ROW);

        (selected_row_offset < current_page.get_region_size()).then_some(
            current_page
                .get_base_address()
                .saturating_add(selected_row_offset),
        )
    }

    pub fn selected_cursor_address(&self) -> Option<u64> {
        self.hex_edit_state
            .as_ref()
            .map(|hex_edit_state| hex_edit_state.cursor_address)
            .or_else(|| {
                self.selected_byte_range
                    .as_ref()
                    .map(|selected_byte_range| selected_byte_range.active_address)
            })
    }

    pub fn get_selected_address_bounds(&self) -> Option<(u64, u64)> {
        self.resolve_selected_address_bounds()
    }

    pub fn selection_summary(&self) -> Option<MemoryViewerSelectionSummary> {
        self.build_selection_summary()
    }

    pub fn current_page_row_count(&self) -> usize {
        self.current_page().map(Self::get_page_row_count).unwrap_or(0)
    }

    pub fn visible_row_entries(&self) -> Vec<PaneEntryRow> {
        let Some(current_page) = self.current_page() else {
            return Vec::new();
        };
        let page_base_address = current_page.get_base_address();
        let visible_row_range = self.visible_row_range_for_current_page();
        let selected_address_bounds = self.resolve_selected_address_bounds();
        let mut row_entries = Vec::with_capacity(visible_row_range.len());

        for row_index in visible_row_range {
            let row_offset = (row_index as u64).saturating_mul(Self::BYTES_PER_ROW);
            let row_byte_count = current_page
                .get_region_size()
                .saturating_sub(row_offset)
                .min(Self::BYTES_PER_ROW);
            let row_address = page_base_address.saturating_add(row_offset);
            let row_end_address = row_address.saturating_add(row_byte_count.saturating_sub(1));
            let row_contains_selection = selected_address_bounds
                .map(|(selection_start_address, selection_end_address)| selection_start_address <= row_end_address && selection_end_address >= row_address)
                .unwrap_or(false);
            let marker_text = if self.selected_row_index == row_index {
                ">".to_string()
            } else if row_contains_selection {
                "*".to_string()
            } else {
                String::new()
            };
            let mut rendered_hex_cells = String::new();
            let mut ascii_text = String::with_capacity(row_byte_count as usize);

            for column_offset in 0..row_byte_count {
                let byte_offset = row_offset.saturating_add(column_offset);
                let absolute_address = row_address.saturating_add(column_offset);
                let cached_byte = self.get_cached_byte_for_current_page(byte_offset);
                let is_selected_byte = selected_address_bounds
                    .map(|(selection_start_address, selection_end_address)| {
                        absolute_address >= selection_start_address && absolute_address <= selection_end_address
                    })
                    .unwrap_or(false);
                let is_cursor_byte = self
                    .hex_edit_state
                    .as_ref()
                    .map(|hex_edit_state| hex_edit_state.cursor_address == absolute_address)
                    .unwrap_or(false);
                let pending_high_nibble = self
                    .hex_edit_state
                    .as_ref()
                    .filter(|hex_edit_state| hex_edit_state.cursor_address == absolute_address)
                    .and_then(|hex_edit_state| hex_edit_state.pending_high_nibble);

                rendered_hex_cells.push_str(&Self::render_hex_cell(cached_byte, is_selected_byte, is_cursor_byte, pending_high_nibble));
                ascii_text.push(cached_byte.map(format_ascii_byte).unwrap_or('?'));
            }

            let primary_text = format!("0x{:016X} {}", row_address, rendered_hex_cells);
            let secondary_text = Some(ascii_text);

            if self.selected_row_index == row_index {
                row_entries.push(PaneEntryRow::selected(marker_text, primary_text, secondary_text));
            } else if row_contains_selection {
                row_entries.push(PaneEntryRow::normal(marker_text, primary_text, secondary_text));
            } else {
                row_entries.push(PaneEntryRow::normal(marker_text, primary_text, secondary_text));
            }
        }

        row_entries
    }

    pub fn refresh_pages_from_response(
        &mut self,
        virtual_pages: Vec<NormalizedRegion>,
        modules: Vec<NormalizedModule>,
        selected_page_base_address: Option<u64>,
    ) {
        self.virtual_pages = virtual_pages;
        self.modules = modules;
        self.page_caches_by_base_address.clear();
        self.unreadable_page_base_addresses.clear();
        self.last_applied_snapshot_generation = 0;
        self.cached_last_page_index = self.virtual_pages.len().saturating_sub(1) as u64;
        self.current_page_index = Self::resolve_page_index_after_refresh(&self.virtual_pages, selected_page_base_address)
            .unwrap_or_else(|| Self::resolve_initial_page_index(&self.virtual_pages, &self.modules).unwrap_or(0))
            .clamp(0, self.cached_last_page_index);
        self.has_loaded_memory_pages_once = !self.virtual_pages.is_empty();
        self.selected_byte_range = None;
        self.hex_edit_state = None;
        if let Some(current_page_base_address) = self.current_page_base_address() {
            self.set_selection_to_address(current_page_base_address, false);
        } else {
            self.selected_row_index = 0;
        }
        self.stats_string = Self::format_stats_for_page_from_modules(&self.modules, &self.unreadable_page_base_addresses, self.current_page());
    }

    pub fn clear_for_process_change(
        &mut self,
        engine_unprivileged_state: &Arc<EngineUnprivilegedState>,
    ) {
        self.virtual_pages.clear();
        self.modules.clear();
        self.current_page_index = 0;
        self.cached_last_page_index = 0;
        self.selected_row_index = 0;
        self.stats_string = String::from("No page selected");
        self.status_message = String::from("Process changed. Memory viewer cleared");
        self.input_mode = MemoryViewerInputMode::Normal;
        self.pending_seek_input.clear();
        self.selected_byte_range = None;
        self.hex_edit_state = None;
        self.is_querying_memory_pages = false;
        self.has_loaded_memory_pages_once = false;
        self.last_applied_snapshot_generation = 0;
        self.page_caches_by_base_address.clear();
        self.unreadable_page_base_addresses.clear();
        engine_unprivileged_state.set_virtual_snapshot_queries(Self::VIRTUAL_SNAPSHOT_ID, Self::SNAPSHOT_REFRESH_INTERVAL, Vec::new());
    }

    pub fn build_visible_chunk_queries(&self) -> Vec<VirtualSnapshotQuery> {
        let Some(current_page) = self.current_page() else {
            return Vec::new();
        };

        Self::build_chunk_queries_for_row_range(current_page, self.visible_row_range_for_current_page())
    }

    pub fn apply_virtual_snapshot_results(
        &mut self,
        virtual_snapshot: &VirtualSnapshot,
    ) {
        if self.last_applied_snapshot_generation >= virtual_snapshot.get_generation() {
            return;
        }

        for (query_id, virtual_snapshot_query_result) in virtual_snapshot.get_query_results() {
            let Some((page_base_address, chunk_offset)) = Self::parse_chunk_query_id(query_id) else {
                continue;
            };
            let Some(memory_read_response) = &virtual_snapshot_query_result.memory_read_response else {
                continue;
            };

            if !memory_read_response.success {
                self.unreadable_page_base_addresses.insert(page_base_address);
                continue;
            }

            let Some(chunk_bytes) = memory_read_response
                .valued_struct
                .get_fields()
                .first()
                .and_then(|valued_struct_field| valued_struct_field.get_data_value())
                .map(|data_value| data_value.get_value_bytes().to_vec())
            else {
                continue;
            };

            self.page_caches_by_base_address
                .entry(page_base_address)
                .or_default()
                .cache_chunk(chunk_offset, chunk_bytes);
            self.unreadable_page_base_addresses.remove(&page_base_address);
        }

        self.last_applied_snapshot_generation = virtual_snapshot.get_generation();
        self.stats_string = Self::format_stats_for_page_from_modules(&self.modules, &self.unreadable_page_base_addresses, self.current_page());
    }

    pub fn navigate_first_page(&mut self) {
        self.set_page_index(0, self.current_page_relative_byte_offset());
    }

    pub fn navigate_last_page(&mut self) {
        self.set_page_index(self.cached_last_page_index, self.current_page_relative_byte_offset());
    }

    pub fn navigate_previous_page(&mut self) {
        self.set_page_index(self.current_page_index.saturating_sub(1), self.current_page_relative_byte_offset());
    }

    pub fn navigate_next_page(&mut self) {
        self.set_page_index(self.current_page_index.saturating_add(1), self.current_page_relative_byte_offset());
    }

    pub fn move_cursor_horizontal(
        &mut self,
        column_delta: i64,
        extend_selection: bool,
    ) {
        self.move_cursor_internal(column_delta, extend_selection);
    }

    pub fn move_cursor_vertical(
        &mut self,
        row_delta: i64,
        extend_selection: bool,
    ) {
        self.move_cursor_internal(row_delta.saturating_mul(Self::BYTES_PER_ROW as i64), extend_selection);
    }

    pub fn select_all_bytes_on_current_page(&mut self) {
        let Some(current_page_address_range) = self.resolve_current_page_address_range() else {
            return;
        };

        self.set_selected_byte_range(current_page_address_range.start, current_page_address_range.end.saturating_sub(1));
        self.set_hex_edit_cursor(current_page_address_range.start);
        self.synchronize_selected_row_index_with_active_address();
    }

    pub fn clear_selection(&mut self) {
        self.selected_byte_range = None;
        self.hex_edit_state = None;
        self.synchronize_selected_row_index_with_active_address();
    }

    pub fn begin_seek_input(&mut self) {
        self.input_mode = MemoryViewerInputMode::SeekInput;
        self.pending_seek_input = self
            .selected_cursor_address()
            .map(format_absolute_address)
            .unwrap_or_default();
        self.status_message = String::from("Enter an address and press Enter.");
    }

    pub fn cancel_seek_input(&mut self) {
        self.input_mode = MemoryViewerInputMode::Normal;
        self.pending_seek_input.clear();
        self.status_message = String::from("Canceled address seek.");
    }

    pub fn clear_pending_seek_input(&mut self) {
        self.pending_seek_input.clear();
    }

    pub fn backspace_pending_seek_input(&mut self) {
        self.pending_seek_input.pop();
    }

    pub fn append_pending_seek_character(
        &mut self,
        pending_character: char,
    ) {
        if pending_character.is_ascii_hexdigit() || matches!(pending_character, 'x' | 'X') {
            self.pending_seek_input.push(pending_character);
        }
    }

    pub fn commit_seek_input(&mut self) -> bool {
        let Some(target_address) = parse_address_text(&self.pending_seek_input) else {
            self.status_message = String::from("Address input is not valid hexadecimal or decimal text.");
            return false;
        };
        let did_seek = self.seek_to_address_internal(target_address);
        if did_seek {
            self.input_mode = MemoryViewerInputMode::Normal;
            self.pending_seek_input.clear();
            self.status_message = format!("Focused address {}.", format_absolute_address(target_address));
        } else {
            self.status_message = String::from("No memory pages are available for the requested address.");
        }

        did_seek
    }

    pub fn focus_address(
        &mut self,
        address: u64,
        module_name: &str,
    ) -> bool {
        let resolved_address = if module_name.is_empty() {
            Some(address)
        } else {
            self.modules
                .iter()
                .find(|normalized_module| {
                    normalized_module
                        .get_module_name()
                        .eq_ignore_ascii_case(module_name)
                })
                .and_then(|normalized_module| normalized_module.get_base_address().checked_add(address))
        };
        let Some(resolved_address) = resolved_address else {
            return false;
        };

        self.focus_absolute_address_internal(resolved_address)
    }

    pub fn handle_hex_edit_backspace(&mut self) {
        self.handle_hex_edit_backspace_internal();
        self.synchronize_selected_row_index_with_active_address();
    }

    pub fn append_hex_edit_character(
        &mut self,
        character: char,
    ) -> Option<(u64, Vec<u8>)> {
        let upper_hex_character = character.to_ascii_uppercase();

        if !upper_hex_character.is_ascii_hexdigit() {
            return None;
        }

        let write_result = self.append_hex_edit_character_internal(upper_hex_character);
        self.synchronize_selected_row_index_with_active_address();

        write_result
    }

    pub fn apply_memory_write(
        &mut self,
        write_start_address: u64,
        written_bytes: &[u8],
    ) {
        let Some(current_page) = self.current_page().cloned() else {
            return;
        };
        let current_page_base_address = current_page.get_base_address();
        let current_page_end_address = current_page.get_end_address();

        for (written_byte_index, written_byte) in written_bytes.iter().enumerate() {
            let written_byte_address = write_start_address.saturating_add(written_byte_index as u64);

            if written_byte_address < current_page_base_address || written_byte_address >= current_page_end_address {
                continue;
            }

            let byte_offset = written_byte_address.saturating_sub(current_page_base_address);
            let chunk_offset = byte_offset - (byte_offset % Self::QUERY_CHUNK_SIZE_IN_BYTES);
            let chunk_local_index = byte_offset.saturating_sub(chunk_offset) as usize;
            let chunk_length = current_page
                .get_region_size()
                .saturating_sub(chunk_offset)
                .min(Self::QUERY_CHUNK_SIZE_IN_BYTES) as usize;
            let chunk_bytes = self
                .page_caches_by_base_address
                .entry(current_page_base_address)
                .or_default()
                .cached_chunks
                .entry(chunk_offset)
                .or_insert_with(|| vec![0; chunk_length]);

            if chunk_local_index < chunk_bytes.len() {
                chunk_bytes[chunk_local_index] = *written_byte;
            }
        }
    }

    pub fn build_address_project_item_create_request_with_data_type(
        &self,
        target_directory_path: Option<PathBuf>,
        explicit_data_type_id: Option<String>,
    ) -> Option<ProjectItemsCreateRequest> {
        let (selection_start_address, selection_end_address) = self.resolve_selected_address_bounds().or_else(|| {
            self.selected_cursor_address()
                .map(|selected_cursor_address| (selected_cursor_address, selected_cursor_address))
        })?;
        let (project_item_address, project_item_module_name) = self.resolve_project_item_address(selection_start_address);
        let selected_byte_count = selection_end_address
            .saturating_sub(selection_start_address)
            .saturating_add(1)
            .max(1);
        let resolved_data_type_id = explicit_data_type_id.unwrap_or_else(|| {
            if selected_byte_count > 1 {
                format!("u8[{}]", selected_byte_count)
            } else {
                String::from("u8")
            }
        });

        Some(ProjectItemsCreateRequest {
            parent_directory_path: target_directory_path.unwrap_or_default(),
            project_item_name: Self::format_project_item_name(project_item_address, &project_item_module_name),
            is_directory: false,
            address: Some(project_item_address),
            module_name: Some(project_item_module_name),
            data_type_id: Some(resolved_data_type_id),
        })
    }

    pub fn get_page_row_count(normalized_region: &NormalizedRegion) -> usize {
        let row_count_u64 = normalized_region
            .get_region_size()
            .saturating_add(Self::BYTES_PER_ROW.saturating_sub(1))
            .checked_div(Self::BYTES_PER_ROW)
            .unwrap_or(0);

        usize::try_from(row_count_u64).unwrap_or(usize::MAX)
    }

    fn current_page(&self) -> Option<&NormalizedRegion> {
        self.virtual_pages.get(self.current_page_index as usize)
    }

    fn visible_row_range_for_current_page(&self) -> Range<usize> {
        let row_count = self.current_page_row_count();
        let viewport_row_capacity = self.last_visible_row_capacity.max(1);

        build_selection_relative_viewport_range(row_count, Some(self.selected_row_index), viewport_row_capacity)
    }

    fn get_cached_byte_for_current_page(
        &self,
        byte_offset: u64,
    ) -> Option<u8> {
        let current_page_base_address = self.current_page_base_address()?;

        self.page_caches_by_base_address
            .get(&current_page_base_address)
            .and_then(|memory_viewer_page_cache| memory_viewer_page_cache.get_cached_byte(byte_offset))
    }

    fn render_hex_cell(
        cached_byte: Option<u8>,
        is_selected_byte: bool,
        is_cursor_byte: bool,
        pending_high_nibble: Option<u8>,
    ) -> String {
        let byte_text = if let Some(pending_high_nibble) = pending_high_nibble {
            format!("{:X}?", pending_high_nibble)
        } else {
            cached_byte
                .map(|byte_value| format!("{:02X}", byte_value))
                .unwrap_or_else(|| String::from("??"))
        };

        match (is_selected_byte, is_cursor_byte) {
            (true, true) => format!("{{{}}}", byte_text),
            (true, false) => format!("[{}]", byte_text),
            (false, true) => format!("<{}>", byte_text),
            (false, false) => format!(" {} ", byte_text),
        }
    }

    fn resolve_selected_address_bounds(&self) -> Option<(u64, u64)> {
        let selected_byte_range = self.selected_byte_range.as_ref()?;
        let current_page_address_range = self.resolve_current_page_address_range();
        let selection_anchor_address = Self::clamp_selection_address_to_page_bounds(selected_byte_range.anchor_address, current_page_address_range.as_ref());
        let selection_active_address = Self::clamp_selection_active_address(
            selection_anchor_address,
            selected_byte_range.active_address,
            current_page_address_range.as_ref(),
        );

        Some((
            selection_anchor_address.min(selection_active_address),
            selection_anchor_address.max(selection_active_address),
        ))
    }

    fn resolve_project_item_address(
        &self,
        absolute_address: u64,
    ) -> (u64, String) {
        if let Some(module) = self
            .modules
            .iter()
            .find(|normalized_module| normalized_module.contains_address(absolute_address))
        {
            return (absolute_address.saturating_sub(module.get_base_address()), module.get_module_name().to_string());
        }

        (absolute_address, String::new())
    }

    fn build_selection_summary(&self) -> Option<MemoryViewerSelectionSummary> {
        let (selection_start_address, selection_end_address) = self.resolve_selected_address_bounds()?;
        let current_page = self.current_page()?;
        let current_page_base_address = current_page.get_base_address();
        let current_page_end_address = current_page.get_end_address();

        if selection_start_address < current_page_base_address || selection_end_address >= current_page_end_address {
            return None;
        }

        let selected_bytes = (selection_start_address..=selection_end_address)
            .map(|selected_byte_address| {
                let selected_byte_offset = selected_byte_address.saturating_sub(current_page_base_address);

                self.page_caches_by_base_address
                    .get(&current_page_base_address)
                    .and_then(|memory_viewer_page_cache| memory_viewer_page_cache.get_cached_byte(selected_byte_offset))
            })
            .collect::<Vec<_>>();
        let (selection_display_address, selection_display_module_name) = self.resolve_project_item_address(selection_start_address);
        let selection_display_text = if selection_display_module_name.is_empty() {
            format_absolute_address(selection_start_address)
        } else {
            format_module_address(&selection_display_module_name, selection_display_address)
        };

        Some(MemoryViewerSelectionSummary {
            selection_start_address,
            selection_end_address,
            selection_display_text,
            selected_bytes,
        })
    }

    fn synchronize_selected_row_index_with_active_address(&mut self) {
        let Some(current_page) = self.current_page() else {
            self.selected_row_index = 0;
            return;
        };
        let current_page_base_address = current_page.get_base_address();
        let current_page_end_address = current_page.get_end_address();
        let active_address = self
            .selected_cursor_address()
            .unwrap_or(current_page_base_address)
            .clamp(current_page_base_address, current_page_end_address.saturating_sub(1));
        let row_index = active_address
            .saturating_sub(current_page_base_address)
            .checked_div(Self::BYTES_PER_ROW)
            .and_then(|resolved_row_index| usize::try_from(resolved_row_index).ok())
            .unwrap_or(0);

        self.selected_row_index = row_index.min(Self::get_page_row_count(current_page).saturating_sub(1));
    }

    fn current_page_relative_byte_offset(&self) -> Option<u64> {
        let current_page_base_address = self.current_page_base_address()?;
        let selected_cursor_address = self.selected_cursor_address()?;

        Some(selected_cursor_address.saturating_sub(current_page_base_address))
    }

    fn set_page_index(
        &mut self,
        page_index: u64,
        preferred_byte_offset: Option<u64>,
    ) {
        self.current_page_index = page_index.clamp(0, self.cached_last_page_index);
        self.last_applied_snapshot_generation = 0;
        if let Some(current_page) = self.current_page() {
            let page_focus_address = current_page.get_base_address().saturating_add(
                preferred_byte_offset
                    .unwrap_or(0)
                    .min(current_page.get_region_size().saturating_sub(1)),
            );
            self.set_selection_to_address(page_focus_address, false);
        } else {
            self.selected_byte_range = None;
            self.hex_edit_state = None;
            self.selected_row_index = 0;
        }
        self.stats_string = Self::format_stats_for_page_from_modules(&self.modules, &self.unreadable_page_base_addresses, self.current_page());
    }

    fn set_selection_to_address(
        &mut self,
        address: u64,
        extend_selection: bool,
    ) {
        let selection_anchor_address = if extend_selection {
            self.selected_byte_range
                .as_ref()
                .map(|selected_byte_range| selected_byte_range.anchor_address)
                .unwrap_or(address)
        } else {
            address
        };

        self.set_selected_byte_range(selection_anchor_address, address);
        self.set_hex_edit_cursor(address);
        self.synchronize_selected_row_index_with_active_address();
    }

    fn set_selected_byte_range(
        &mut self,
        selection_anchor_address: u64,
        selection_active_address: u64,
    ) {
        let current_page_address_range = self.resolve_current_page_address_range();
        let clamped_selection_anchor_address = Self::clamp_selection_address_to_page_bounds(selection_anchor_address, current_page_address_range.as_ref());
        let clamped_selection_active_address =
            Self::clamp_selection_active_address(clamped_selection_anchor_address, selection_active_address, current_page_address_range.as_ref());

        self.selected_byte_range = Some(MemoryViewerSelectionRange {
            anchor_address: clamped_selection_anchor_address,
            active_address: clamped_selection_active_address,
        });
    }

    fn clamp_selection_address_to_page_bounds(
        selection_address: u64,
        current_page_address_range: Option<&Range<u64>>,
    ) -> u64 {
        current_page_address_range.map_or(selection_address, |current_page_address_range| {
            selection_address.clamp(current_page_address_range.start, current_page_address_range.end.saturating_sub(1))
        })
    }

    fn clamp_selection_active_address(
        selection_anchor_address: u64,
        requested_selection_active_address: u64,
        current_page_address_range: Option<&Range<u64>>,
    ) -> u64 {
        let max_selection_delta = Self::MAX_SELECTION_SIZE_IN_BYTES.saturating_sub(1);
        let capped_selection_active_address = if requested_selection_active_address >= selection_anchor_address {
            requested_selection_active_address.min(selection_anchor_address.saturating_add(max_selection_delta))
        } else {
            requested_selection_active_address.max(selection_anchor_address.saturating_sub(max_selection_delta))
        };

        Self::clamp_selection_address_to_page_bounds(capped_selection_active_address, current_page_address_range)
    }

    fn set_hex_edit_cursor(
        &mut self,
        cursor_address: u64,
    ) {
        self.hex_edit_state = Some(MemoryViewerHexEditState {
            cursor_address,
            active_nibble_index: 0,
            pending_high_nibble: None,
        });
    }

    fn focus_absolute_address_internal(
        &mut self,
        absolute_address: u64,
    ) -> bool {
        let Some((page_index, clamped_address)) = Self::resolve_nearest_page_index_and_address(&self.virtual_pages, absolute_address) else {
            return false;
        };

        self.current_page_index = page_index;
        self.last_applied_snapshot_generation = 0;
        self.set_selection_to_address(clamped_address, false);
        self.stats_string = Self::format_stats_for_page_from_modules(&self.modules, &self.unreadable_page_base_addresses, self.current_page());

        true
    }

    fn seek_to_address_internal(
        &mut self,
        target_address: u64,
    ) -> bool {
        self.focus_absolute_address_internal(target_address)
    }

    fn move_cursor_internal(
        &mut self,
        byte_delta: i64,
        extend_selection: bool,
    ) {
        let Some(current_page_address_range) = self.resolve_current_page_address_range() else {
            return;
        };
        let last_page_address = current_page_address_range.end.saturating_sub(1);
        let base_cursor_address = self
            .hex_edit_state
            .as_ref()
            .map(|hex_edit_state| hex_edit_state.cursor_address)
            .or_else(|| {
                self.selected_byte_range
                    .as_ref()
                    .map(|selected_byte_range| selected_byte_range.active_address)
            })
            .unwrap_or(current_page_address_range.start)
            .clamp(current_page_address_range.start, last_page_address);
        let target_cursor_address = if byte_delta >= 0 {
            base_cursor_address
                .saturating_add(byte_delta as u64)
                .min(last_page_address)
        } else {
            base_cursor_address
                .saturating_sub(byte_delta.unsigned_abs())
                .max(current_page_address_range.start)
        };

        self.set_selection_to_address(target_cursor_address, extend_selection);
    }

    fn handle_hex_edit_backspace_internal(&mut self) {
        let current_page_address_range = self.resolve_current_page_address_range();
        let Some(hex_edit_state) = self.hex_edit_state.as_mut() else {
            return;
        };

        if hex_edit_state.pending_high_nibble.take().is_some() || hex_edit_state.active_nibble_index == 1 {
            hex_edit_state.active_nibble_index = 0;

            return;
        }

        let Some(current_page_address_range) = current_page_address_range else {
            self.hex_edit_state = None;

            return;
        };
        let last_page_address = current_page_address_range.end.saturating_sub(1);

        if hex_edit_state.cursor_address >= current_page_address_range.end {
            hex_edit_state.cursor_address = last_page_address;
            hex_edit_state.active_nibble_index = 0;

            return;
        }

        if hex_edit_state.cursor_address > current_page_address_range.start {
            hex_edit_state.cursor_address = hex_edit_state.cursor_address.saturating_sub(1);
        }
    }

    fn append_hex_edit_character_internal(
        &mut self,
        character: char,
    ) -> Option<(u64, Vec<u8>)> {
        let nibble_value = character.to_digit(16)? as u8;
        let current_page_address_range = self.resolve_current_page_address_range()?;

        if self.hex_edit_state.is_none() {
            let selection_cursor_address = self
                .selected_byte_range
                .as_ref()
                .map(|selected_byte_range| selected_byte_range.active_address)
                .unwrap_or(current_page_address_range.start);
            self.set_hex_edit_cursor(selection_cursor_address);
        }

        let hex_edit_state = self.hex_edit_state.as_mut()?;

        if hex_edit_state.cursor_address >= current_page_address_range.end {
            return None;
        }

        if hex_edit_state.active_nibble_index == 0 {
            hex_edit_state.pending_high_nibble = Some(nibble_value);
            hex_edit_state.active_nibble_index = 1;

            return None;
        }

        let edited_byte = (hex_edit_state.pending_high_nibble.take().unwrap_or(0) << 4) | nibble_value;
        let write_start_address = hex_edit_state.cursor_address;
        let next_cursor_address = write_start_address.saturating_add(1);

        hex_edit_state.active_nibble_index = 0;
        hex_edit_state.cursor_address = if next_cursor_address < current_page_address_range.end {
            next_cursor_address
        } else {
            current_page_address_range.end
        };

        Some((write_start_address, vec![edited_byte]))
    }

    fn resolve_current_page_address_range(&self) -> Option<Range<u64>> {
        let current_page = self.current_page()?;
        let current_page_base_address = current_page.get_base_address();
        let current_page_end_address = current_page.get_end_address();

        (current_page_base_address < current_page_end_address).then_some(current_page_base_address..current_page_end_address)
    }

    fn build_chunk_queries_for_row_range(
        normalized_region: &NormalizedRegion,
        visible_row_range: Range<usize>,
    ) -> Vec<VirtualSnapshotQuery> {
        if visible_row_range.is_empty() || normalized_region.get_region_size() == 0 {
            return Vec::new();
        }

        let first_visible_byte_offset = (visible_row_range.start as u64).saturating_mul(Self::BYTES_PER_ROW);
        let last_visible_byte_offset_exclusive = ((visible_row_range.end as u64).saturating_mul(Self::BYTES_PER_ROW)).min(normalized_region.get_region_size());

        if first_visible_byte_offset >= last_visible_byte_offset_exclusive {
            return Vec::new();
        }

        let first_visible_chunk_index = first_visible_byte_offset / Self::QUERY_CHUNK_SIZE_IN_BYTES;
        let last_visible_chunk_index = last_visible_byte_offset_exclusive
            .saturating_sub(1)
            .checked_div(Self::QUERY_CHUNK_SIZE_IN_BYTES)
            .unwrap_or(0);
        let first_prefetched_chunk_index = first_visible_chunk_index.saturating_sub(Self::QUERY_PREFETCH_CHUNK_COUNT);
        let max_chunk_index = normalized_region
            .get_region_size()
            .saturating_sub(1)
            .checked_div(Self::QUERY_CHUNK_SIZE_IN_BYTES)
            .unwrap_or(0);
        let last_prefetched_chunk_index = last_visible_chunk_index
            .saturating_add(Self::QUERY_PREFETCH_CHUNK_COUNT)
            .min(max_chunk_index);

        (first_prefetched_chunk_index..=last_prefetched_chunk_index)
            .filter_map(|chunk_index| {
                let chunk_offset = chunk_index.saturating_mul(Self::QUERY_CHUNK_SIZE_IN_BYTES);
                let chunk_length = normalized_region
                    .get_region_size()
                    .saturating_sub(chunk_offset)
                    .min(Self::QUERY_CHUNK_SIZE_IN_BYTES);

                (chunk_length > 0).then_some(VirtualSnapshotQuery::Address {
                    query_id: Self::build_chunk_query_id(normalized_region.get_base_address(), chunk_offset),
                    address: normalized_region
                        .get_base_address()
                        .saturating_add(chunk_offset),
                    module_name: String::new(),
                    symbolic_struct_definition: Self::build_chunk_symbolic_struct_definition(chunk_length),
                })
            })
            .collect()
    }

    fn build_chunk_query_id(
        page_base_address: u64,
        chunk_offset: u64,
    ) -> String {
        format!("{:016X}:{:016X}", page_base_address, chunk_offset)
    }

    fn parse_chunk_query_id(query_id: &str) -> Option<(u64, u64)> {
        let mut query_id_segments = query_id.split(':');
        let page_base_address = u64::from_str_radix(query_id_segments.next()?, 16).ok()?;
        let chunk_offset = u64::from_str_radix(query_id_segments.next()?, 16).ok()?;

        Some((page_base_address, chunk_offset))
    }

    fn build_chunk_symbolic_struct_definition(chunk_length: u64) -> SymbolicStructDefinition {
        SymbolicStructDefinition::new_anonymous(vec![SymbolicFieldDefinition::new(
            DataTypeRef::new(DataTypeU8::DATA_TYPE_ID),
            ContainerType::ArrayFixed(chunk_length.max(1)),
        )])
    }

    fn resolve_page_index_after_refresh(
        virtual_pages: &[NormalizedRegion],
        selected_page_base_address: Option<u64>,
    ) -> Option<u64> {
        let selected_page_base_address = selected_page_base_address?;

        virtual_pages
            .iter()
            .position(|normalized_region| {
                normalized_region.get_base_address() == selected_page_base_address
                    || (selected_page_base_address >= normalized_region.get_base_address() && selected_page_base_address < normalized_region.get_end_address())
            })
            .map(|page_index| page_index as u64)
    }

    fn resolve_nearest_page_index_and_address(
        virtual_pages: &[NormalizedRegion],
        target_address: u64,
    ) -> Option<(u64, u64)> {
        virtual_pages
            .iter()
            .enumerate()
            .filter_map(|(page_index, normalized_region)| {
                let page_base_address = normalized_region.get_base_address();
                let page_end_address = normalized_region.get_end_address();

                if page_base_address >= page_end_address {
                    return None;
                }

                let clamped_address = target_address.clamp(page_base_address, page_end_address.saturating_sub(1));

                Some((page_index as u64, clamped_address, clamped_address.abs_diff(target_address)))
            })
            .min_by_key(|(page_index, clamped_address, distance)| (*distance, *page_index, *clamped_address))
            .map(|(page_index, clamped_address, _)| (page_index, clamped_address))
    }

    fn resolve_initial_page_index(
        virtual_pages: &[NormalizedRegion],
        modules: &[NormalizedModule],
    ) -> Option<u64> {
        let first_module_base_address = modules.first().map(NormalizedModule::get_base_address)?;

        Self::resolve_page_index_after_refresh(virtual_pages, Some(first_module_base_address))
    }

    fn format_stats_for_page_from_modules(
        modules: &[NormalizedModule],
        unreadable_page_base_addresses: &HashSet<u64>,
        normalized_region: Option<&NormalizedRegion>,
    ) -> String {
        let Some(normalized_region) = normalized_region else {
            return String::from("No page selected");
        };
        let module_label = modules
            .iter()
            .find(|normalized_module| normalized_module.contains_address(normalized_region.get_base_address()))
            .map(|normalized_module| normalized_module.get_module_name().to_string())
            .unwrap_or_else(|| String::from("(No Module)"));
        let page_size_label = StorageSizeConversions::value_to_metric_size(normalized_region.get_region_size() as u128);

        format!(
            "Base 0x{:X} | Size {} | {}{}",
            normalized_region.get_base_address(),
            page_size_label,
            module_label,
            if unreadable_page_base_addresses.contains(&normalized_region.get_base_address()) {
                " | Unreadable"
            } else {
                ""
            }
        )
    }

    fn format_project_item_name(
        project_item_address: u64,
        project_item_module_name: &str,
    ) -> String {
        if project_item_module_name.is_empty() {
            format!("0x{:X}", project_item_address)
        } else {
            format_module_address(project_item_module_name, project_item_address)
        }
    }
}

impl Default for MemoryViewerPaneState {
    fn default() -> Self {
        Self {
            virtual_pages: Vec::new(),
            modules: Vec::new(),
            current_page_index: 0,
            cached_last_page_index: 0,
            selected_row_index: 0,
            last_visible_row_capacity: 0,
            page_retrieval_mode: PageRetrievalMode::FromUserMode,
            stats_string: String::from("No page selected"),
            status_message: String::from("Ready"),
            input_mode: MemoryViewerInputMode::Normal,
            pending_seek_input: String::new(),
            selected_byte_range: None,
            hex_edit_state: None,
            is_querying_memory_pages: false,
            has_loaded_memory_pages_once: false,
            last_applied_snapshot_generation: 0,
            page_caches_by_base_address: HashMap::new(),
            unreadable_page_base_addresses: HashSet::new(),
        }
    }
}

fn format_ascii_byte(byte_value: u8) -> char {
    if byte_value.is_ascii_graphic() || byte_value == b' ' {
        byte_value as char
    } else {
        '.'
    }
}

fn parse_address_text(address_text: &str) -> Option<u64> {
    let trimmed_address_text = address_text.trim();

    if let Some(hex_address_text) = trimmed_address_text
        .strip_prefix("0x")
        .or_else(|| trimmed_address_text.strip_prefix("0X"))
    {
        return u64::from_str_radix(hex_address_text, 16).ok();
    }

    if trimmed_address_text
        .chars()
        .all(|address_character| address_character.is_ascii_hexdigit())
    {
        return u64::from_str_radix(trimmed_address_text, 16).ok();
    }

    trimmed_address_text.parse::<u64>().ok()
}

#[cfg(test)]
mod tests {
    use super::{MemoryViewerInputMode, MemoryViewerPaneState, MemoryViewerSelectionRange};
    use squalr_engine_api::structures::memory::{normalized_module::NormalizedModule, normalized_region::NormalizedRegion};

    #[test]
    fn build_visible_chunk_queries_aligns_visible_rows_to_prefetched_chunks() {
        let mut memory_viewer_pane_state = MemoryViewerPaneState::default();
        memory_viewer_pane_state.virtual_pages = vec![NormalizedRegion::new(0x1000, 600)];
        memory_viewer_pane_state.last_visible_row_capacity = 3;
        memory_viewer_pane_state.selected_row_index = 3;

        let query_ids = memory_viewer_pane_state
            .build_visible_chunk_queries()
            .into_iter()
            .map(|virtual_snapshot_query| virtual_snapshot_query.get_query_id().to_string())
            .collect::<Vec<_>>();

        assert_eq!(
            query_ids,
            vec![
                MemoryViewerPaneState::build_chunk_query_id(0x1000, 0),
                MemoryViewerPaneState::build_chunk_query_id(0x1000, 256),
            ]
        );
    }

    #[test]
    fn resolve_initial_page_index_prefers_first_module_page() {
        let virtual_pages = vec![
            NormalizedRegion::new(0x1000, 0x100),
            NormalizedRegion::new(0x4000, 0x100),
            NormalizedRegion::new(0x8000, 0x100),
        ];
        let modules = vec![NormalizedModule::new("winmine.exe", 0x4000, 0x1000)];

        let resolved_page_index = MemoryViewerPaneState::resolve_initial_page_index(&virtual_pages, &modules);

        assert_eq!(resolved_page_index, Some(1));
    }

    #[test]
    fn move_cursor_horizontal_extends_selection_from_anchor() {
        let mut memory_viewer_pane_state = MemoryViewerPaneState::default();
        memory_viewer_pane_state.virtual_pages = vec![NormalizedRegion::new(0x1000, 0x40)];
        memory_viewer_pane_state.focus_address(0x1004, "");

        memory_viewer_pane_state.move_cursor_horizontal(3, true);

        assert_eq!(
            memory_viewer_pane_state.selected_byte_range,
            Some(MemoryViewerSelectionRange {
                anchor_address: 0x1004,
                active_address: 0x1007,
            })
        );
    }

    #[test]
    fn move_cursor_horizontal_clamps_selection_size_to_two_megabytes() {
        let mut memory_viewer_pane_state = MemoryViewerPaneState::default();
        memory_viewer_pane_state.virtual_pages = vec![NormalizedRegion::new(
            0x2000,
            MemoryViewerPaneState::MAX_SELECTION_SIZE_IN_BYTES + 0x8000,
        )];
        memory_viewer_pane_state.focus_address(0x2000, "");

        memory_viewer_pane_state.move_cursor_horizontal((MemoryViewerPaneState::MAX_SELECTION_SIZE_IN_BYTES + 0x1234) as i64, true);

        assert_eq!(
            memory_viewer_pane_state.resolve_selected_address_bounds(),
            Some((0x2000, 0x2000 + MemoryViewerPaneState::MAX_SELECTION_SIZE_IN_BYTES - 1))
        );
    }

    #[test]
    fn append_hex_edit_character_writes_and_advances_cursor() {
        let mut memory_viewer_pane_state = MemoryViewerPaneState::default();
        memory_viewer_pane_state.virtual_pages = vec![NormalizedRegion::new(0x1000, 0x40)];
        memory_viewer_pane_state.focus_address(0x1001, "");

        assert_eq!(memory_viewer_pane_state.append_hex_edit_character('A'), None);
        assert_eq!(memory_viewer_pane_state.append_hex_edit_character('1'), Some((0x1001, vec![0xA1])));
        assert_eq!(
            memory_viewer_pane_state
                .hex_edit_state
                .as_ref()
                .map(|hex_edit_state| hex_edit_state.cursor_address),
            Some(0x1002)
        );
    }

    #[test]
    fn commit_seek_input_clamps_to_nearest_page() {
        let mut memory_viewer_pane_state = MemoryViewerPaneState::default();
        memory_viewer_pane_state.virtual_pages = vec![
            NormalizedRegion::new(0x1000, 0x100),
            NormalizedRegion::new(0x3000, 0x100),
        ];
        memory_viewer_pane_state.begin_seek_input();
        memory_viewer_pane_state.pending_seek_input = String::from("0x2800");

        assert!(memory_viewer_pane_state.commit_seek_input());
        assert_eq!(memory_viewer_pane_state.input_mode, MemoryViewerInputMode::Normal);
        assert_eq!(memory_viewer_pane_state.current_page_index, 1);
        assert_eq!(memory_viewer_pane_state.selected_cursor_address(), Some(0x3000));
    }

    #[test]
    fn build_address_project_item_create_request_uses_selection_width() {
        let mut memory_viewer_pane_state = MemoryViewerPaneState::default();
        memory_viewer_pane_state.virtual_pages = vec![NormalizedRegion::new(0x1000, 0x100)];
        memory_viewer_pane_state.modules = vec![NormalizedModule::new("game.exe", 0x1000, 0x1000)];
        memory_viewer_pane_state.selected_byte_range = Some(MemoryViewerSelectionRange {
            anchor_address: 0x1004,
            active_address: 0x1007,
        });

        let create_request = memory_viewer_pane_state
            .build_address_project_item_create_request_with_data_type(None, None)
            .expect("Expected create request.");

        assert_eq!(create_request.project_item_name, String::from("game.exe+0x4"));
        assert_eq!(create_request.address, Some(0x4));
        assert_eq!(create_request.module_name, Some(String::from("game.exe")));
        assert_eq!(create_request.data_type_id, Some(String::from("u8[4]")));
    }

    #[test]
    fn select_all_bytes_on_current_page_clamps_selection_size_to_two_megabytes() {
        let mut memory_viewer_pane_state = MemoryViewerPaneState::default();
        memory_viewer_pane_state.virtual_pages = vec![NormalizedRegion::new(
            0x4000,
            MemoryViewerPaneState::MAX_SELECTION_SIZE_IN_BYTES + 0x9000,
        )];

        memory_viewer_pane_state.select_all_bytes_on_current_page();

        assert_eq!(
            memory_viewer_pane_state.resolve_selected_address_bounds(),
            Some((0x4000, 0x4000 + MemoryViewerPaneState::MAX_SELECTION_SIZE_IN_BYTES - 1))
        );
        assert_eq!(
            memory_viewer_pane_state
                .hex_edit_state
                .as_ref()
                .map(|hex_edit_state| hex_edit_state.cursor_address),
            Some(0x4000)
        );
    }
}
