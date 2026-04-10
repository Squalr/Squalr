use crate::state::pane_entry_row::PaneEntryRow;
use crate::views::entry_row_viewport::build_selection_relative_viewport_range;
use crate::views::memory_viewer::summary::build_memory_viewer_summary_lines;
use squalr_engine_api::{
    conversions::storage_size_conversions::StorageSizeConversions,
    plugins::memory_view::PageRetrievalMode,
    structures::{
        data_types::{built_in_types::u8::data_type_u8::DataTypeU8, data_type_ref::DataTypeRef},
        data_values::container_type::ContainerType,
        memory::{normalized_module::NormalizedModule, normalized_region::NormalizedRegion},
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
    pub is_querying_memory_pages: bool,
    pub has_loaded_memory_pages_once: bool,
    last_applied_snapshot_generation: u64,
    page_caches_by_base_address: HashMap<u64, MemoryViewerPageCache>,
    unreadable_page_base_addresses: HashSet<u64>,
}

impl MemoryViewerPaneState {
    pub const VIRTUAL_SNAPSHOT_ID: &'static str = "tui_memory_viewer";
    pub const BYTES_PER_ROW: u64 = 16;
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
        self.clamp_selected_row_to_current_page();
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

    pub fn current_page_row_count(&self) -> usize {
        self.current_page().map(Self::get_page_row_count).unwrap_or(0)
    }

    pub fn visible_row_entries(&self) -> Vec<PaneEntryRow> {
        let Some(current_page) = self.current_page() else {
            return Vec::new();
        };
        let page_base_address = current_page.get_base_address();
        let visible_row_range = self.visible_row_range_for_current_page();
        let mut row_entries = Vec::with_capacity(visible_row_range.len());

        for row_index in visible_row_range {
            let row_offset = (row_index as u64).saturating_mul(Self::BYTES_PER_ROW);
            let row_byte_count = current_page
                .get_region_size()
                .saturating_sub(row_offset)
                .min(Self::BYTES_PER_ROW);
            let row_address = page_base_address.saturating_add(row_offset);
            let hex_text = (0..row_byte_count)
                .map(|column_offset| {
                    self.get_cached_byte_for_current_page(row_offset.saturating_add(column_offset))
                        .map(|byte_value| format!("{:02X}", byte_value))
                        .unwrap_or_else(|| String::from("??"))
                })
                .collect::<Vec<_>>()
                .join(" ");
            let ascii_text = (0..row_byte_count)
                .map(|column_offset| {
                    self.get_cached_byte_for_current_page(row_offset.saturating_add(column_offset))
                        .map(format_ascii_byte)
                        .unwrap_or('?')
                })
                .collect::<String>();
            let marker_text = if self.selected_row_index == row_index {
                ">".to_string()
            } else {
                String::new()
            };
            let primary_text = format!("0x{:016X}  {}", row_address, hex_text);
            let secondary_text = Some(ascii_text);

            if self.selected_row_index == row_index {
                row_entries.push(PaneEntryRow::selected(marker_text, primary_text, secondary_text));
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
        self.selected_row_index = 0;
        self.has_loaded_memory_pages_once = !self.virtual_pages.is_empty();
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
        self.set_page_index(0);
    }

    pub fn navigate_last_page(&mut self) {
        self.set_page_index(self.cached_last_page_index);
    }

    pub fn navigate_previous_page(&mut self) {
        self.set_page_index(self.current_page_index.saturating_sub(1));
    }

    pub fn navigate_next_page(&mut self) {
        self.set_page_index(self.current_page_index.saturating_add(1));
    }

    pub fn select_previous_row(&mut self) {
        self.selected_row_index = self.selected_row_index.saturating_sub(1);
    }

    pub fn select_next_row(&mut self) {
        let last_row_index = self.current_page_row_count().saturating_sub(1);
        self.selected_row_index = self.selected_row_index.saturating_add(1).min(last_row_index);
    }

    pub fn page_up_rows(&mut self) {
        let page_jump = self.last_visible_row_capacity.max(1);
        self.selected_row_index = self.selected_row_index.saturating_sub(page_jump);
    }

    pub fn page_down_rows(&mut self) {
        let page_jump = self.last_visible_row_capacity.max(1);
        let last_row_index = self.current_page_row_count().saturating_sub(1);
        self.selected_row_index = self
            .selected_row_index
            .saturating_add(page_jump)
            .min(last_row_index);
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
        let Some(page_index) = Self::resolve_page_index_after_refresh(&self.virtual_pages, Some(resolved_address)) else {
            return false;
        };
        let Some(current_page) = self.virtual_pages.get(page_index as usize) else {
            return false;
        };
        let row_index = resolved_address
            .saturating_sub(current_page.get_base_address())
            .checked_div(Self::BYTES_PER_ROW)
            .and_then(|row_index| usize::try_from(row_index).ok())
            .unwrap_or(0);

        self.current_page_index = page_index;
        self.selected_row_index = row_index.min(Self::get_page_row_count(current_page).saturating_sub(1));
        self.stats_string = Self::format_stats_for_page_from_modules(&self.modules, &self.unreadable_page_base_addresses, self.current_page());

        true
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

    fn clamp_selected_row_to_current_page(&mut self) {
        let last_row_index = self.current_page_row_count().saturating_sub(1);
        self.selected_row_index = self.selected_row_index.min(last_row_index);
    }

    fn set_page_index(
        &mut self,
        page_index: u64,
    ) {
        self.current_page_index = page_index.clamp(0, self.cached_last_page_index);
        self.selected_row_index = 0;
        self.last_applied_snapshot_generation = 0;
        self.stats_string = Self::format_stats_for_page_from_modules(&self.modules, &self.unreadable_page_base_addresses, self.current_page());
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

#[cfg(test)]
mod tests {
    use super::MemoryViewerPaneState;
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
}
