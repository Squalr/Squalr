use arc_swap::Guard;
use squalr_engine_api::{
    commands::{
        memory::query::{memory_query_request::MemoryQueryRequest, memory_query_response::MemoryQueryResponse},
        privileged_command_request::PrivilegedCommandRequest,
    },
    conversions::storage_size_conversions::StorageSizeConversions,
    dependency_injection::dependency::Dependency,
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
    time::{Duration, Instant},
};

#[derive(Clone, Debug, Default)]
pub struct MemoryViewerPageCache {
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
        let chunk_offset = byte_offset - (byte_offset % MemoryViewerViewData::QUERY_CHUNK_SIZE_IN_BYTES);
        let chunk_bytes = self.cached_chunks.get(&chunk_offset)?;
        let chunk_local_index = byte_offset.saturating_sub(chunk_offset) as usize;

        chunk_bytes.get(chunk_local_index).copied()
    }
}

#[derive(Clone)]
pub struct MemoryViewerViewData {
    pub virtual_pages: Vec<NormalizedRegion>,
    pub modules: Vec<NormalizedModule>,
    pub current_page_index: u64,
    pub cached_last_page_index: u64,
    pub page_retrieval_mode: PageRetrievalMode,
    pub stats_string: String,
    pub is_querying_memory_pages: bool,
    memory_pages_request_started_at: Option<Instant>,
    active_memory_pages_request_revision: u64,
    next_memory_pages_request_revision: u64,
    last_applied_snapshot_generation: u64,
    page_caches_by_base_address: HashMap<u64, MemoryViewerPageCache>,
    unreadable_page_base_addresses: HashSet<u64>,
}

impl MemoryViewerViewData {
    pub const WINDOW_VIRTUAL_SNAPSHOT_ID: &'static str = "memory_viewer";
    pub const BYTES_PER_ROW: u64 = 16;
    pub const QUERY_CHUNK_SIZE_IN_BYTES: u64 = 256;
    pub const QUERY_PREFETCH_CHUNK_COUNT: u64 = 1;
    pub const SNAPSHOT_REFRESH_INTERVAL: Duration = Duration::from_millis(500);
    pub const REQUEST_STALE_TIMEOUT_MS: u64 = 10_000;

    pub fn new() -> Self {
        Self {
            virtual_pages: Vec::new(),
            modules: Vec::new(),
            current_page_index: 0,
            cached_last_page_index: 0,
            page_retrieval_mode: PageRetrievalMode::FromUserMode,
            stats_string: String::new(),
            is_querying_memory_pages: false,
            memory_pages_request_started_at: None,
            active_memory_pages_request_revision: 0,
            next_memory_pages_request_revision: 1,
            last_applied_snapshot_generation: 0,
            page_caches_by_base_address: HashMap::new(),
            unreadable_page_base_addresses: HashSet::new(),
        }
    }

    pub fn refresh_memory_pages(
        memory_viewer_view_data: Dependency<Self>,
        engine_unprivileged_state: Arc<EngineUnprivilegedState>,
    ) {
        if memory_viewer_view_data
            .read("Memory viewer refresh pages pending")
            .map(|memory_viewer_view_data| memory_viewer_view_data.is_querying_memory_pages)
            .unwrap_or(false)
        {
            return;
        }

        let page_retrieval_mode = memory_viewer_view_data
            .read("Memory viewer refresh pages retrieval mode")
            .map(|memory_viewer_view_data| memory_viewer_view_data.page_retrieval_mode)
            .unwrap_or(PageRetrievalMode::FromUserMode);
        let selected_page_base_address = memory_viewer_view_data
            .read("Memory viewer refresh pages selected base address")
            .and_then(|memory_viewer_view_data| {
                memory_viewer_view_data
                    .virtual_pages
                    .get(Self::load_current_page_index(&memory_viewer_view_data) as usize)
                    .map(|normalized_region| normalized_region.get_base_address())
            });
        let request_revision = match memory_viewer_view_data.write("Memory viewer refresh pages begin") {
            Some(mut memory_viewer_view_data) => {
                let request_revision = memory_viewer_view_data.begin_memory_pages_request();
                memory_viewer_view_data.is_querying_memory_pages = true;
                memory_viewer_view_data.memory_pages_request_started_at = Some(Instant::now());

                request_revision
            }
            None => return,
        };
        let memory_query_request = MemoryQueryRequest { page_retrieval_mode };
        let memory_viewer_view_data_for_response = memory_viewer_view_data.clone();
        let engine_unprivileged_state_for_response = engine_unprivileged_state.clone();
        let did_dispatch = memory_query_request.send(&engine_unprivileged_state, move |memory_query_response| {
            Self::apply_memory_query_response(
                memory_viewer_view_data_for_response,
                engine_unprivileged_state_for_response,
                request_revision,
                selected_page_base_address,
                memory_query_response,
            );
        });

        if !did_dispatch {
            if let Some(mut memory_viewer_view_data) = memory_viewer_view_data.write("Memory viewer refresh pages dispatch failure") {
                if memory_viewer_view_data.should_apply_memory_pages_request(request_revision) {
                    memory_viewer_view_data.complete_memory_pages_request();
                }
            }
        }
    }

    pub fn clear_stale_request_state_if_needed(memory_viewer_view_data: Dependency<Self>) {
        let now = Instant::now();

        if let Some(mut memory_viewer_view_data) = memory_viewer_view_data.write("Memory viewer clear stale request state") {
            if memory_viewer_view_data.is_querying_memory_pages
                && memory_viewer_view_data
                    .memory_pages_request_started_at
                    .map(|request_started_at| now.duration_since(request_started_at) >= Duration::from_millis(Self::REQUEST_STALE_TIMEOUT_MS))
                    .unwrap_or(true)
            {
                memory_viewer_view_data.complete_memory_pages_request();
            }
        }
    }

    pub fn navigate_first_page(memory_viewer_view_data: Dependency<Self>) {
        Self::set_page_index(memory_viewer_view_data, 0);
    }

    pub fn navigate_last_page(memory_viewer_view_data: Dependency<Self>) {
        let new_page_index = memory_viewer_view_data
            .read("Memory viewer navigate last")
            .map(|memory_viewer_view_data| memory_viewer_view_data.cached_last_page_index)
            .unwrap_or(0);

        Self::set_page_index(memory_viewer_view_data, new_page_index);
    }

    pub fn navigate_previous_page(memory_viewer_view_data: Dependency<Self>) {
        let new_page_index = memory_viewer_view_data
            .read("Memory viewer navigate previous")
            .map(|memory_viewer_view_data| Self::load_current_page_index(&memory_viewer_view_data).saturating_sub(1))
            .unwrap_or(0);

        Self::set_page_index(memory_viewer_view_data, new_page_index);
    }

    pub fn navigate_next_page(memory_viewer_view_data: Dependency<Self>) {
        let new_page_index = memory_viewer_view_data
            .read("Memory viewer navigate next")
            .map(|memory_viewer_view_data| Self::load_current_page_index(&memory_viewer_view_data).saturating_add(1))
            .unwrap_or(0);

        Self::set_page_index(memory_viewer_view_data, new_page_index);
    }

    pub fn set_page_index_string(
        memory_viewer_view_data: Dependency<Self>,
        new_page_index_text: &str,
    ) {
        let new_page_index = new_page_index_text
            .chars()
            .take_while(|character| character.is_ascii_digit())
            .collect::<String>()
            .parse::<u64>()
            .unwrap_or(0);

        Self::set_page_index(memory_viewer_view_data, new_page_index);
    }

    pub fn get_current_page_index_string(memory_viewer_view_data: Dependency<Self>) -> String {
        memory_viewer_view_data
            .read("Memory viewer current page index string")
            .map(|memory_viewer_view_data| Self::load_current_page_index(&memory_viewer_view_data).to_string())
            .unwrap_or_else(|| String::from("0"))
    }

    pub fn get_current_page(memory_viewer_view_data: Dependency<Self>) -> Option<NormalizedRegion> {
        let memory_viewer_view_data = memory_viewer_view_data.read("Memory viewer current page")?;
        let current_page_index = Self::load_current_page_index(&memory_viewer_view_data) as usize;

        memory_viewer_view_data
            .virtual_pages
            .get(current_page_index)
            .cloned()
    }

    pub fn get_cached_byte_for_page(
        memory_viewer_view_data: Dependency<Self>,
        page_base_address: u64,
        byte_offset: u64,
    ) -> Option<u8> {
        memory_viewer_view_data
            .read("Memory viewer cached byte")
            .and_then(|memory_viewer_view_data| {
                memory_viewer_view_data
                    .page_caches_by_base_address
                    .get(&page_base_address)
                    .and_then(|memory_viewer_page_cache| memory_viewer_page_cache.get_cached_byte(byte_offset))
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

    pub fn build_visible_chunk_queries(
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

                if chunk_length == 0 {
                    return None;
                }

                Some(VirtualSnapshotQuery::Address {
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

    pub fn apply_virtual_snapshot_results(
        memory_viewer_view_data: Dependency<Self>,
        virtual_snapshot: &VirtualSnapshot,
    ) {
        if let Some(mut memory_viewer_view_data) = memory_viewer_view_data.write("Memory viewer apply virtual snapshot") {
            if memory_viewer_view_data.last_applied_snapshot_generation >= virtual_snapshot.get_generation() {
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
                    memory_viewer_view_data
                        .unreadable_page_base_addresses
                        .insert(page_base_address);
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

                memory_viewer_view_data
                    .page_caches_by_base_address
                    .entry(page_base_address)
                    .or_default()
                    .cache_chunk(chunk_offset, chunk_bytes);
                memory_viewer_view_data
                    .unreadable_page_base_addresses
                    .remove(&page_base_address);
            }

            memory_viewer_view_data.last_applied_snapshot_generation = virtual_snapshot.get_generation();
            let current_page = memory_viewer_view_data
                .virtual_pages
                .get(memory_viewer_view_data.current_page_index as usize)
                .cloned();
            memory_viewer_view_data.stats_string = Self::format_stats_for_page_from_modules(
                &memory_viewer_view_data.modules,
                &memory_viewer_view_data.unreadable_page_base_addresses,
                current_page.as_ref(),
            );
        }
    }

    pub fn clear_for_process_change(
        memory_viewer_view_data: Dependency<Self>,
        engine_unprivileged_state: Arc<EngineUnprivilegedState>,
    ) {
        if let Some(mut memory_viewer_view_data) = memory_viewer_view_data.write("Memory viewer clear for process change") {
            memory_viewer_view_data.virtual_pages.clear();
            memory_viewer_view_data.modules.clear();
            memory_viewer_view_data.current_page_index = 0;
            memory_viewer_view_data.cached_last_page_index = 0;
            memory_viewer_view_data.stats_string.clear();
            memory_viewer_view_data.last_applied_snapshot_generation = 0;
            memory_viewer_view_data.page_caches_by_base_address.clear();
            memory_viewer_view_data.unreadable_page_base_addresses.clear();
            memory_viewer_view_data.complete_memory_pages_request();
        }

        engine_unprivileged_state.set_virtual_snapshot_queries(Self::WINDOW_VIRTUAL_SNAPSHOT_ID, Self::SNAPSHOT_REFRESH_INTERVAL, Vec::new());
    }

    pub fn build_chunk_query_id(
        page_base_address: u64,
        chunk_offset: u64,
    ) -> String {
        format!("{:016X}:{:016X}", page_base_address, chunk_offset)
    }

    pub fn get_module_label_for_page(
        memory_viewer_view_data: Dependency<Self>,
        normalized_region: &NormalizedRegion,
    ) -> Option<String> {
        let memory_viewer_view_data = memory_viewer_view_data.read("Memory viewer module label")?;
        let page_base_address = normalized_region.get_base_address();

        memory_viewer_view_data
            .modules
            .iter()
            .find(|normalized_module| normalized_module.contains_address(page_base_address))
            .map(|normalized_module| normalized_module.get_module_name().to_string())
    }

    pub fn get_current_page_is_unreadable(memory_viewer_view_data: Dependency<Self>) -> bool {
        memory_viewer_view_data
            .read("Memory viewer current page unreadable")
            .map(|memory_viewer_view_data| {
                memory_viewer_view_data
                    .virtual_pages
                    .get(Self::load_current_page_index(&memory_viewer_view_data) as usize)
                    .map(|normalized_region| {
                        memory_viewer_view_data
                            .unreadable_page_base_addresses
                            .contains(&normalized_region.get_base_address())
                    })
                    .unwrap_or(false)
            })
            .unwrap_or(false)
    }

    fn format_stats_for_page_from_modules(
        modules: &[NormalizedModule],
        unreadable_page_base_addresses: &HashSet<u64>,
        normalized_region: Option<&NormalizedRegion>,
    ) -> String {
        let Some(normalized_region) = normalized_region else {
            return String::from("No page selected.");
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

    fn apply_memory_query_response(
        memory_viewer_view_data: Dependency<Self>,
        engine_unprivileged_state: Arc<EngineUnprivilegedState>,
        request_revision: u64,
        selected_page_base_address: Option<u64>,
        memory_query_response: MemoryQueryResponse,
    ) {
        let has_pages = match memory_viewer_view_data.write("Memory viewer apply memory query response") {
            Some(mut memory_viewer_view_data) => {
                if !memory_viewer_view_data.should_apply_memory_pages_request(request_revision) {
                    return;
                }

                memory_viewer_view_data.complete_memory_pages_request();
                memory_viewer_view_data.virtual_pages = memory_query_response.virtual_pages;
                memory_viewer_view_data.modules = memory_query_response.modules;
                memory_viewer_view_data.page_caches_by_base_address.clear();
                memory_viewer_view_data.unreadable_page_base_addresses.clear();
                memory_viewer_view_data.last_applied_snapshot_generation = 0;
                memory_viewer_view_data.cached_last_page_index = memory_viewer_view_data.virtual_pages.len().saturating_sub(1) as u64;
                memory_viewer_view_data.current_page_index =
                    Self::resolve_page_index_after_refresh(&memory_viewer_view_data.virtual_pages, selected_page_base_address).unwrap_or_else(|| {
                        Self::resolve_initial_page_index(&memory_viewer_view_data.virtual_pages, &memory_viewer_view_data.modules).unwrap_or(
                            memory_viewer_view_data
                                .current_page_index
                                .clamp(0, memory_viewer_view_data.cached_last_page_index),
                        )
                    });

                let current_page = memory_viewer_view_data
                    .virtual_pages
                    .get(memory_viewer_view_data.current_page_index as usize)
                    .cloned();
                memory_viewer_view_data.stats_string = Self::format_stats_for_page_from_modules(
                    &memory_viewer_view_data.modules,
                    &memory_viewer_view_data.unreadable_page_base_addresses,
                    current_page.as_ref(),
                );

                !memory_viewer_view_data.virtual_pages.is_empty()
            }
            None => false,
        };

        if !has_pages {
            engine_unprivileged_state.set_virtual_snapshot_queries(Self::WINDOW_VIRTUAL_SNAPSHOT_ID, Self::SNAPSHOT_REFRESH_INTERVAL, Vec::new());
        }
    }

    fn load_current_page_index(memory_viewer_view_data: &Guard<Arc<Self>>) -> u64 {
        memory_viewer_view_data
            .current_page_index
            .clamp(0, memory_viewer_view_data.cached_last_page_index)
    }

    fn set_page_index(
        memory_viewer_view_data: Dependency<Self>,
        new_page_index: u64,
    ) {
        if let Some(mut memory_viewer_view_data) = memory_viewer_view_data.write("Memory viewer set page index") {
            let bounded_page_index = new_page_index.clamp(0, memory_viewer_view_data.cached_last_page_index);

            if bounded_page_index == memory_viewer_view_data.current_page_index {
                return;
            }

            memory_viewer_view_data.current_page_index = bounded_page_index;

            let current_page = memory_viewer_view_data
                .virtual_pages
                .get(bounded_page_index as usize)
                .cloned();
            memory_viewer_view_data.stats_string = Self::format_stats_for_page_from_modules(
                &memory_viewer_view_data.modules,
                &memory_viewer_view_data.unreadable_page_base_addresses,
                current_page.as_ref(),
            );
            memory_viewer_view_data.last_applied_snapshot_generation = 0;
        }
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
        let first_module_base_address = modules
            .first()
            .map(|normalized_module| normalized_module.get_base_address())?;

        Self::resolve_page_index_after_refresh(virtual_pages, Some(first_module_base_address))
    }

    fn begin_memory_pages_request(&mut self) -> u64 {
        let request_revision = self.next_memory_pages_request_revision;
        self.next_memory_pages_request_revision = self.next_memory_pages_request_revision.saturating_add(1);
        self.active_memory_pages_request_revision = request_revision;

        request_revision
    }

    fn should_apply_memory_pages_request(
        &self,
        request_revision: u64,
    ) -> bool {
        self.active_memory_pages_request_revision == request_revision
    }

    fn complete_memory_pages_request(&mut self) {
        self.is_querying_memory_pages = false;
        self.memory_pages_request_started_at = None;
    }
}

#[cfg(test)]
mod tests {
    use super::MemoryViewerViewData;
    use squalr_engine_api::structures::memory::{normalized_module::NormalizedModule, normalized_region::NormalizedRegion};

    #[test]
    fn build_visible_chunk_queries_aligns_visible_rows_to_prefetched_chunk_window() {
        let normalized_region = NormalizedRegion::new(0x1000, 600);
        let queries = MemoryViewerViewData::build_visible_chunk_queries(&normalized_region, 2..5);
        let query_ids = queries
            .iter()
            .map(|virtual_snapshot_query| virtual_snapshot_query.get_query_id().to_string())
            .collect::<Vec<_>>();

        assert_eq!(
            query_ids,
            vec![
                MemoryViewerViewData::build_chunk_query_id(0x1000, 0),
                MemoryViewerViewData::build_chunk_query_id(0x1000, 256),
            ]
        );
    }

    #[test]
    fn build_visible_chunk_queries_clamps_prefetch_to_page_bounds() {
        let normalized_region = NormalizedRegion::new(0x2000, 128);
        let queries = MemoryViewerViewData::build_visible_chunk_queries(&normalized_region, 0..1);
        let query_ids = queries
            .iter()
            .map(|virtual_snapshot_query| virtual_snapshot_query.get_query_id().to_string())
            .collect::<Vec<_>>();

        assert_eq!(query_ids, vec![MemoryViewerViewData::build_chunk_query_id(0x2000, 0)]);
    }

    #[test]
    fn page_row_count_rounds_up_partial_rows() {
        let normalized_region = NormalizedRegion::new(0x3000, 17);

        assert_eq!(MemoryViewerViewData::get_page_row_count(&normalized_region), 2);
    }

    #[test]
    fn resolve_page_index_after_refresh_matches_page_identity_by_base_address() {
        let virtual_pages = vec![
            NormalizedRegion::new(0x1000, 0x100),
            NormalizedRegion::new(0x2000, 0x100),
            NormalizedRegion::new(0x3000, 0x100),
        ];

        let resolved_page_index = MemoryViewerViewData::resolve_page_index_after_refresh(&virtual_pages, Some(0x2000));

        assert_eq!(resolved_page_index, Some(1));
    }

    #[test]
    fn resolve_initial_page_index_prefers_first_module_page() {
        let virtual_pages = vec![
            NormalizedRegion::new(0x1000, 0x100),
            NormalizedRegion::new(0x4000, 0x100),
            NormalizedRegion::new(0x8000, 0x100),
        ];
        let modules = vec![NormalizedModule::new("winmine.exe", 0x4000, 0x1000)];

        let resolved_page_index = MemoryViewerViewData::resolve_initial_page_index(&virtual_pages, &modules);

        assert_eq!(resolved_page_index, Some(1));
    }
}
