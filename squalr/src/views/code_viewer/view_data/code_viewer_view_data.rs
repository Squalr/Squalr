use squalr_engine_api::{
    commands::{
        memory::query::{memory_query_request::MemoryQueryRequest, memory_query_response::MemoryQueryResponse},
        privileged_command_request::PrivilegedCommandRequest,
    },
    dependency_injection::dependency::Dependency,
    plugins::memory_view::PageRetrievalMode,
    structures::{
        data_types::{built_in_types::u8::data_type_u8::DataTypeU8, data_type_ref::DataTypeRef},
        data_values::container_type::ContainerType,
        memory::{address_display::format_absolute_address, bitness::Bitness, normalized_module::NormalizedModule, normalized_region::NormalizedRegion},
        structs::{symbolic_field_definition::SymbolicFieldDefinition, symbolic_struct_definition::SymbolicStructDefinition},
    },
};
use squalr_engine_session::{
    engine_unprivileged_state::EngineUnprivilegedState,
    virtual_snapshots::{virtual_snapshot::VirtualSnapshot, virtual_snapshot_query::VirtualSnapshotQuery},
};
use squalr_plugin_instructions_x86::{DisassembledInstruction, X64InstructionSet, X86InstructionSet};
use std::{
    collections::{BTreeMap, HashMap, HashSet},
    ops::Range,
    sync::Arc,
    time::{Duration, Instant},
};

#[derive(Clone, Debug, Default)]
struct CodeViewerPageCache {
    cached_chunks: BTreeMap<u64, Vec<u8>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct CodeViewerFocusRequest {
    address: u64,
    module_name: String,
}

#[derive(Clone)]
pub struct CodeViewerViewData {
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
    page_caches_by_base_address: HashMap<u64, CodeViewerPageCache>,
    unreadable_page_base_addresses: HashSet<u64>,
    pending_focus_request: Option<CodeViewerFocusRequest>,
    pending_scroll_address: Option<u64>,
    selected_instruction_address: Option<u64>,
    viewport_start_address: Option<u64>,
    breakpoint_addresses: HashSet<u64>,
}

impl CodeViewerPageCache {
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
        let chunk_offset = byte_offset - (byte_offset % CodeViewerViewData::QUERY_CHUNK_SIZE_IN_BYTES);
        let chunk_bytes = self.cached_chunks.get(&chunk_offset)?;
        let chunk_local_index = byte_offset.saturating_sub(chunk_offset) as usize;

        chunk_bytes.get(chunk_local_index).copied()
    }

    fn collect_contiguous_bytes(
        &self,
        start_byte_offset: u64,
        end_byte_offset_exclusive: u64,
    ) -> Vec<u8> {
        let mut collected_bytes = Vec::new();

        for byte_offset in start_byte_offset..end_byte_offset_exclusive {
            let Some(byte_value) = self.get_cached_byte(byte_offset) else {
                break;
            };

            collected_bytes.push(byte_value);
        }

        collected_bytes
    }
}

impl CodeViewerViewData {
    pub const WINDOW_VIRTUAL_SNAPSHOT_ID: &'static str = "code_viewer";
    pub const QUERY_CHUNK_SIZE_IN_BYTES: u64 = 256;
    pub const QUERY_PREFETCH_CHUNK_COUNT: u64 = 1;
    pub const CODE_WINDOW_SIZE_IN_BYTES: u64 = 0x1000;
    pub const DECODE_BACKTRACK_BYTES: u64 = 0x80;
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
            pending_focus_request: None,
            pending_scroll_address: None,
            selected_instruction_address: None,
            viewport_start_address: None,
            breakpoint_addresses: HashSet::new(),
        }
    }

    pub fn request_focus_address(
        code_viewer_view_data: Dependency<Self>,
        engine_unprivileged_state: Arc<EngineUnprivilegedState>,
        address: u64,
        module_name: String,
    ) {
        let should_refresh_memory_pages = match code_viewer_view_data.write("Code viewer request focus address") {
            Some(mut code_viewer_view_data) => {
                code_viewer_view_data.pending_focus_request = Some(CodeViewerFocusRequest { address, module_name });

                if code_viewer_view_data.try_apply_pending_focus_request() {
                    false
                } else {
                    code_viewer_view_data.virtual_pages.is_empty() && !code_viewer_view_data.is_querying_memory_pages
                }
            }
            None => return,
        };

        if should_refresh_memory_pages {
            Self::refresh_memory_pages(code_viewer_view_data, engine_unprivileged_state);
        }
    }

    pub fn refresh_memory_pages(
        code_viewer_view_data: Dependency<Self>,
        engine_unprivileged_state: Arc<EngineUnprivilegedState>,
    ) {
        if code_viewer_view_data
            .read("Code viewer refresh pages pending")
            .map(|code_viewer_view_data| code_viewer_view_data.is_querying_memory_pages)
            .unwrap_or(false)
        {
            return;
        }

        let page_retrieval_mode = code_viewer_view_data
            .read("Code viewer refresh pages retrieval mode")
            .map(|code_viewer_view_data| code_viewer_view_data.page_retrieval_mode)
            .unwrap_or(PageRetrievalMode::FromUserMode);
        let selected_page_base_address = code_viewer_view_data
            .read("Code viewer refresh pages selected base address")
            .and_then(|code_viewer_view_data| {
                code_viewer_view_data
                    .virtual_pages
                    .get(Self::load_current_page_index(&code_viewer_view_data) as usize)
                    .map(|normalized_region| normalized_region.get_base_address())
            });
        let request_revision = match code_viewer_view_data.write("Code viewer refresh pages begin") {
            Some(mut code_viewer_view_data) => {
                let request_revision = code_viewer_view_data.begin_memory_pages_request();
                code_viewer_view_data.is_querying_memory_pages = true;
                code_viewer_view_data.memory_pages_request_started_at = Some(Instant::now());

                request_revision
            }
            None => return,
        };
        let memory_query_request = MemoryQueryRequest { page_retrieval_mode };
        let code_viewer_view_data_for_response = code_viewer_view_data.clone();
        let engine_unprivileged_state_for_response = engine_unprivileged_state.clone();
        let did_dispatch = memory_query_request.send(&engine_unprivileged_state, move |memory_query_response| {
            Self::apply_memory_query_response(
                code_viewer_view_data_for_response,
                engine_unprivileged_state_for_response,
                request_revision,
                selected_page_base_address,
                memory_query_response,
            );
        });

        if !did_dispatch {
            if let Some(mut code_viewer_view_data) = code_viewer_view_data.write("Code viewer refresh pages dispatch failure") {
                if code_viewer_view_data.should_apply_memory_pages_request(request_revision) {
                    code_viewer_view_data.complete_memory_pages_request();
                }
            }
        }
    }

    pub fn clear_stale_request_state_if_needed(code_viewer_view_data: Dependency<Self>) {
        let now = Instant::now();

        if let Some(mut code_viewer_view_data) = code_viewer_view_data.write("Code viewer clear stale request state") {
            if code_viewer_view_data.is_querying_memory_pages
                && code_viewer_view_data
                    .memory_pages_request_started_at
                    .map(|request_started_at| now.duration_since(request_started_at) >= Duration::from_millis(Self::REQUEST_STALE_TIMEOUT_MS))
                    .unwrap_or(true)
            {
                code_viewer_view_data.complete_memory_pages_request();
            }
        }
    }

    pub fn clear_for_process_change(
        code_viewer_view_data: Dependency<Self>,
        engine_unprivileged_state: Arc<EngineUnprivilegedState>,
    ) {
        if let Some(mut code_viewer_view_data) = code_viewer_view_data.write("Code viewer clear for process change") {
            code_viewer_view_data.virtual_pages.clear();
            code_viewer_view_data.modules.clear();
            code_viewer_view_data.current_page_index = 0;
            code_viewer_view_data.cached_last_page_index = 0;
            code_viewer_view_data.stats_string.clear();
            code_viewer_view_data.last_applied_snapshot_generation = 0;
            code_viewer_view_data.page_caches_by_base_address.clear();
            code_viewer_view_data.unreadable_page_base_addresses.clear();
            code_viewer_view_data.pending_focus_request = None;
            code_viewer_view_data.pending_scroll_address = None;
            code_viewer_view_data.selected_instruction_address = None;
            code_viewer_view_data.viewport_start_address = None;
            code_viewer_view_data.complete_memory_pages_request();
        }

        engine_unprivileged_state.set_virtual_snapshot_queries(Self::WINDOW_VIRTUAL_SNAPSHOT_ID, Self::SNAPSHOT_REFRESH_INTERVAL, Vec::new());
    }

    pub fn navigate_first_page(code_viewer_view_data: Dependency<Self>) {
        Self::set_page_index(code_viewer_view_data, 0);
    }

    pub fn navigate_last_page(code_viewer_view_data: Dependency<Self>) {
        let new_page_index = code_viewer_view_data
            .read("Code viewer navigate last")
            .map(|code_viewer_view_data| code_viewer_view_data.cached_last_page_index)
            .unwrap_or(0);

        Self::set_page_index(code_viewer_view_data, new_page_index);
    }

    pub fn navigate_previous_page(code_viewer_view_data: Dependency<Self>) {
        let new_page_index = code_viewer_view_data
            .read("Code viewer navigate previous")
            .map(|code_viewer_view_data| Self::load_current_page_index(&code_viewer_view_data).saturating_sub(1))
            .unwrap_or(0);

        Self::set_page_index(code_viewer_view_data, new_page_index);
    }

    pub fn navigate_next_page(code_viewer_view_data: Dependency<Self>) {
        let new_page_index = code_viewer_view_data
            .read("Code viewer navigate next")
            .map(|code_viewer_view_data| Self::load_current_page_index(&code_viewer_view_data).saturating_add(1))
            .unwrap_or(0);

        Self::set_page_index(code_viewer_view_data, new_page_index);
    }

    pub fn set_page_index_string(
        code_viewer_view_data: Dependency<Self>,
        new_page_index_text: &str,
    ) {
        let new_page_index = new_page_index_text
            .chars()
            .take_while(|character| character.is_ascii_digit())
            .collect::<String>()
            .parse::<u64>()
            .unwrap_or(0);

        Self::set_page_index(code_viewer_view_data, new_page_index);
    }

    pub fn get_current_page_index_string(code_viewer_view_data: Dependency<Self>) -> String {
        code_viewer_view_data
            .read("Code viewer current page index string")
            .map(|code_viewer_view_data| Self::load_current_page_index(&code_viewer_view_data).to_string())
            .unwrap_or_else(|| String::from("0"))
    }

    pub fn get_current_page(code_viewer_view_data: Dependency<Self>) -> Option<NormalizedRegion> {
        let code_viewer_view_data = code_viewer_view_data.read("Code viewer current page")?;
        let current_page_index = Self::load_current_page_index(&code_viewer_view_data) as usize;

        code_viewer_view_data
            .virtual_pages
            .get(current_page_index)
            .cloned()
    }

    pub fn get_current_viewport_start_string(code_viewer_view_data: Dependency<Self>) -> String {
        code_viewer_view_data
            .read("Code viewer current viewport start string")
            .and_then(|code_viewer_view_data| {
                code_viewer_view_data
                    .viewport_start_address
                    .or_else(|| {
                        code_viewer_view_data
                            .virtual_pages
                            .get(code_viewer_view_data.current_page_index as usize)
                            .map(|normalized_region| normalized_region.get_base_address())
                    })
                    .map(format_absolute_address)
            })
            .unwrap_or_else(|| String::from("0x0"))
    }

    pub fn set_viewport_start_string(
        code_viewer_view_data: Dependency<Self>,
        viewport_start_text: &str,
    ) {
        let Some(viewport_start_address) = parse_address_text(viewport_start_text) else {
            return;
        };

        Self::set_viewport_start_address(code_viewer_view_data, viewport_start_address);
    }

    pub fn shift_viewport_window(
        code_viewer_view_data: Dependency<Self>,
        byte_delta: i64,
    ) {
        let Some(current_page) = Self::get_current_page(code_viewer_view_data.clone()) else {
            return;
        };
        let current_viewport_start = Self::get_viewport_start_address(code_viewer_view_data.clone()).unwrap_or(current_page.get_base_address());
        let next_viewport_start = if byte_delta >= 0 {
            current_viewport_start.saturating_add(byte_delta as u64)
        } else {
            current_viewport_start.saturating_sub(byte_delta.unsigned_abs())
        };

        Self::set_viewport_start_address(code_viewer_view_data, next_viewport_start);
    }

    pub fn reset_viewport_to_page_start(code_viewer_view_data: Dependency<Self>) {
        let Some(current_page) = Self::get_current_page(code_viewer_view_data.clone()) else {
            return;
        };

        Self::set_viewport_start_address(code_viewer_view_data, current_page.get_base_address());
    }

    pub fn get_viewport_start_address(code_viewer_view_data: Dependency<Self>) -> Option<u64> {
        code_viewer_view_data
            .read("Code viewer viewport start")
            .and_then(|code_viewer_view_data| code_viewer_view_data.resolve_viewport_start_address())
    }

    pub fn build_visible_chunk_queries(
        normalized_region: &NormalizedRegion,
        viewport_start_address: u64,
    ) -> Vec<VirtualSnapshotQuery> {
        if normalized_region.get_region_size() == 0 {
            return Vec::new();
        }

        let viewport_window = Self::resolve_viewport_window_for_address(normalized_region, viewport_start_address);
        let query_start_offset = viewport_window
            .start
            .saturating_sub(normalized_region.get_base_address())
            .saturating_sub(Self::DECODE_BACKTRACK_BYTES);
        let query_end_offset_exclusive = viewport_window
            .end
            .saturating_sub(normalized_region.get_base_address())
            .min(normalized_region.get_region_size());
        let first_visible_chunk_index = query_start_offset / Self::QUERY_CHUNK_SIZE_IN_BYTES;
        let last_visible_chunk_index = query_end_offset_exclusive
            .saturating_sub(1)
            .checked_div(Self::QUERY_CHUNK_SIZE_IN_BYTES)
            .unwrap_or(0);
        let first_prefetched_chunk_index = first_visible_chunk_index.saturating_sub(Self::QUERY_PREFETCH_CHUNK_COUNT);
        let last_prefetched_chunk_index = last_visible_chunk_index.saturating_add(Self::QUERY_PREFETCH_CHUNK_COUNT);

        (first_prefetched_chunk_index..=last_prefetched_chunk_index)
            .filter_map(|chunk_index| {
                let chunk_offset = chunk_index.saturating_mul(Self::QUERY_CHUNK_SIZE_IN_BYTES);

                if chunk_offset >= normalized_region.get_region_size() {
                    return None;
                }

                let chunk_length = Self::QUERY_CHUNK_SIZE_IN_BYTES.min(normalized_region.get_region_size().saturating_sub(chunk_offset));

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
        code_viewer_view_data: Dependency<Self>,
        virtual_snapshot: &VirtualSnapshot,
    ) {
        if let Some(mut code_viewer_view_data) = code_viewer_view_data.write("Code viewer apply virtual snapshot") {
            if code_viewer_view_data.last_applied_snapshot_generation >= virtual_snapshot.get_generation() {
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
                    code_viewer_view_data
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

                code_viewer_view_data
                    .page_caches_by_base_address
                    .entry(page_base_address)
                    .or_default()
                    .cache_chunk(chunk_offset, chunk_bytes);
                code_viewer_view_data
                    .unreadable_page_base_addresses
                    .remove(&page_base_address);
            }

            code_viewer_view_data.last_applied_snapshot_generation = virtual_snapshot.get_generation();
        }
    }

    pub fn build_instruction_lines(
        code_viewer_view_data: Dependency<Self>,
        process_bitness: Option<Bitness>,
    ) -> Vec<DisassembledInstruction> {
        let Some(current_page) = Self::get_current_page(code_viewer_view_data.clone()) else {
            return Vec::new();
        };
        let Some(viewport_start_address) = Self::get_viewport_start_address(code_viewer_view_data.clone()) else {
            return Vec::new();
        };
        let viewport_window = Self::resolve_viewport_window_for_address(&current_page, viewport_start_address);
        let decode_start_address = viewport_window
            .start
            .saturating_sub(Self::DECODE_BACKTRACK_BYTES);
        let decode_start_address = decode_start_address.max(current_page.get_base_address());
        let decode_end_address_exclusive = viewport_window.end.min(current_page.get_end_address());
        let page_base_address = current_page.get_base_address();
        let decode_start_offset = decode_start_address.saturating_sub(page_base_address);
        let decode_end_offset_exclusive = decode_end_address_exclusive.saturating_sub(page_base_address);

        let cached_bytes = code_viewer_view_data
            .read("Code viewer build instruction lines")
            .and_then(|code_viewer_view_data| {
                code_viewer_view_data
                    .page_caches_by_base_address
                    .get(&page_base_address)
                    .map(|code_viewer_page_cache| code_viewer_page_cache.collect_contiguous_bytes(decode_start_offset, decode_end_offset_exclusive))
            })
            .unwrap_or_default();

        if cached_bytes.is_empty() {
            return Vec::new();
        }

        let decode_result = match process_bitness.unwrap_or(Bitness::Bit64) {
            Bitness::Bit32 => X86InstructionSet::new().disassemble_block(&cached_bytes, decode_start_address),
            Bitness::Bit64 => X64InstructionSet::new().disassemble_block(&cached_bytes, decode_start_address),
        };

        let Ok(decoded_instructions) = decode_result else {
            return Vec::new();
        };

        decoded_instructions
            .into_iter()
            .filter(|decoded_instruction| decoded_instruction.address >= viewport_window.start && decoded_instruction.address < viewport_window.end)
            .collect()
    }

    pub fn take_pending_scroll_address(code_viewer_view_data: Dependency<Self>) -> Option<u64> {
        let mut code_viewer_view_data = code_viewer_view_data.write("Code viewer take pending scroll address")?;
        code_viewer_view_data.pending_scroll_address.take()
    }

    pub fn select_instruction_address(
        code_viewer_view_data: Dependency<Self>,
        address: u64,
    ) {
        if let Some(mut code_viewer_view_data) = code_viewer_view_data.write("Code viewer select instruction address") {
            code_viewer_view_data.selected_instruction_address = Some(address);
            code_viewer_view_data.pending_scroll_address = Some(address);
        }
    }

    pub fn get_selected_instruction_address(code_viewer_view_data: Dependency<Self>) -> Option<u64> {
        code_viewer_view_data
            .read("Code viewer selected instruction address")
            .and_then(|code_viewer_view_data| code_viewer_view_data.selected_instruction_address)
    }

    pub fn toggle_breakpoint_address(
        code_viewer_view_data: Dependency<Self>,
        address: u64,
    ) {
        if let Some(mut code_viewer_view_data) = code_viewer_view_data.write("Code viewer toggle breakpoint") {
            if !code_viewer_view_data.breakpoint_addresses.insert(address) {
                code_viewer_view_data.breakpoint_addresses.remove(&address);
            }
        }
    }

    pub fn has_breakpoint_address(
        code_viewer_view_data: Dependency<Self>,
        address: u64,
    ) -> bool {
        code_viewer_view_data
            .read("Code viewer breakpoint lookup")
            .map(|code_viewer_view_data| code_viewer_view_data.breakpoint_addresses.contains(&address))
            .unwrap_or(false)
    }

    pub fn is_current_page_unreadable(
        code_viewer_view_data: Dependency<Self>,
        normalized_region: &NormalizedRegion,
    ) -> bool {
        code_viewer_view_data
            .read("Code viewer unreadable page lookup")
            .map(|code_viewer_view_data| {
                code_viewer_view_data
                    .unreadable_page_base_addresses
                    .contains(&normalized_region.get_base_address())
            })
            .unwrap_or(false)
    }

    fn set_page_index(
        code_viewer_view_data: Dependency<Self>,
        new_page_index: u64,
    ) {
        if let Some(mut code_viewer_view_data) = code_viewer_view_data.write("Code viewer set page index") {
            code_viewer_view_data.current_page_index = new_page_index.min(code_viewer_view_data.cached_last_page_index);

            let current_page = code_viewer_view_data
                .virtual_pages
                .get(code_viewer_view_data.current_page_index as usize)
                .cloned();
            if let Some(current_page) = current_page {
                code_viewer_view_data.viewport_start_address = Some(current_page.get_base_address());
                code_viewer_view_data.selected_instruction_address = None;
                code_viewer_view_data.pending_scroll_address = Some(current_page.get_base_address());
                code_viewer_view_data.stats_string = Self::format_stats_for_page_from_modules(
                    &code_viewer_view_data.modules,
                    &code_viewer_view_data.unreadable_page_base_addresses,
                    Some(&current_page),
                    code_viewer_view_data.viewport_start_address,
                );
            }
        }
    }

    fn set_viewport_start_address(
        code_viewer_view_data: Dependency<Self>,
        viewport_start_address: u64,
    ) {
        if let Some(mut code_viewer_view_data) = code_viewer_view_data.write("Code viewer set viewport start address") {
            let Some(current_page) = code_viewer_view_data
                .virtual_pages
                .get(code_viewer_view_data.current_page_index as usize)
                .cloned()
            else {
                return;
            };
            let clamped_viewport_start_address = Self::clamp_viewport_start_address(&current_page, viewport_start_address);

            code_viewer_view_data.viewport_start_address = Some(clamped_viewport_start_address);
            code_viewer_view_data.pending_scroll_address = Some(clamped_viewport_start_address);
            code_viewer_view_data.stats_string = Self::format_stats_for_page_from_modules(
                &code_viewer_view_data.modules,
                &code_viewer_view_data.unreadable_page_base_addresses,
                Some(&current_page),
                code_viewer_view_data.viewport_start_address,
            );
        }
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

    fn apply_memory_query_response(
        code_viewer_view_data: Dependency<Self>,
        engine_unprivileged_state: Arc<EngineUnprivilegedState>,
        request_revision: u64,
        selected_page_base_address: Option<u64>,
        memory_query_response: MemoryQueryResponse,
    ) {
        let has_pages = match code_viewer_view_data.write("Code viewer apply memory query response") {
            Some(mut code_viewer_view_data) => {
                if !code_viewer_view_data.should_apply_memory_pages_request(request_revision) {
                    return;
                }

                code_viewer_view_data.complete_memory_pages_request();

                if !memory_query_response.success {
                    return;
                }

                code_viewer_view_data.virtual_pages = memory_query_response.virtual_pages;
                code_viewer_view_data.modules = memory_query_response.modules;
                code_viewer_view_data.cached_last_page_index = code_viewer_view_data.virtual_pages.len().saturating_sub(1) as u64;

                if !code_viewer_view_data.try_apply_pending_focus_request() {
                    code_viewer_view_data.current_page_index =
                        Self::resolve_page_index_after_refresh(&code_viewer_view_data.virtual_pages, selected_page_base_address).unwrap_or_else(|| {
                            Self::resolve_initial_page_index(&code_viewer_view_data.virtual_pages, &code_viewer_view_data.modules).unwrap_or(
                                code_viewer_view_data
                                    .current_page_index
                                    .clamp(0, code_viewer_view_data.cached_last_page_index),
                            )
                        });

                    if let Some(current_page) = code_viewer_view_data
                        .virtual_pages
                        .get(code_viewer_view_data.current_page_index as usize)
                    {
                        code_viewer_view_data.viewport_start_address = Some(current_page.get_base_address());
                    }
                }

                let current_page = code_viewer_view_data
                    .virtual_pages
                    .get(code_viewer_view_data.current_page_index as usize)
                    .cloned();
                code_viewer_view_data.stats_string = Self::format_stats_for_page_from_modules(
                    &code_viewer_view_data.modules,
                    &code_viewer_view_data.unreadable_page_base_addresses,
                    current_page.as_ref(),
                    code_viewer_view_data.viewport_start_address,
                );

                !code_viewer_view_data.virtual_pages.is_empty()
            }
            None => false,
        };

        if has_pages {
            engine_unprivileged_state.request_virtual_snapshot_refresh(Self::WINDOW_VIRTUAL_SNAPSHOT_ID);
        } else {
            engine_unprivileged_state.set_virtual_snapshot_queries(Self::WINDOW_VIRTUAL_SNAPSHOT_ID, Self::SNAPSHOT_REFRESH_INTERVAL, Vec::new());
        }
    }

    fn load_current_page_index(code_viewer_view_data: &Self) -> u64 {
        code_viewer_view_data
            .current_page_index
            .min(code_viewer_view_data.cached_last_page_index)
    }

    fn resolve_page_index_after_refresh(
        virtual_pages: &[NormalizedRegion],
        selected_page_base_address: Option<u64>,
    ) -> Option<u64> {
        let selected_page_base_address = selected_page_base_address?;

        virtual_pages
            .iter()
            .position(|normalized_region| normalized_region.get_base_address() == selected_page_base_address)
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

    fn try_apply_pending_focus_request(&mut self) -> bool {
        let Some(pending_focus_request) = self.pending_focus_request.clone() else {
            return false;
        };
        let Some(focus_address) = Self::resolve_focus_address(&self.modules, pending_focus_request.address, &pending_focus_request.module_name) else {
            if !self.modules.is_empty() || pending_focus_request.module_name.is_empty() {
                self.pending_focus_request = None;
            }

            return false;
        };

        let Some(page_index) = self
            .virtual_pages
            .iter()
            .position(|normalized_region| normalized_region.contains_address(focus_address))
            .map(|page_index| page_index as u64)
        else {
            self.pending_focus_request = None;

            return false;
        };

        self.current_page_index = page_index;
        if let Some(current_page) = self.virtual_pages.get(page_index as usize) {
            self.viewport_start_address = Some(Self::derive_viewport_start_for_focus_address(current_page, focus_address));
            self.stats_string = Self::format_stats_for_page_from_modules(
                &self.modules,
                &self.unreadable_page_base_addresses,
                Some(current_page),
                self.viewport_start_address,
            );
        }
        self.selected_instruction_address = Some(focus_address);
        self.pending_scroll_address = Some(focus_address);
        self.pending_focus_request = None;

        true
    }

    fn resolve_focus_address(
        modules: &[NormalizedModule],
        address: u64,
        module_name: &str,
    ) -> Option<u64> {
        if module_name.is_empty() {
            return Some(address);
        }

        modules
            .iter()
            .find(|normalized_module| {
                normalized_module
                    .get_module_name()
                    .eq_ignore_ascii_case(module_name)
            })
            .map(|normalized_module| normalized_module.get_base_address().saturating_add(address))
    }

    fn resolve_viewport_start_address(&self) -> Option<u64> {
        let current_page = self.virtual_pages.get(self.current_page_index as usize)?;

        Some(
            self.viewport_start_address
                .unwrap_or_else(|| current_page.get_base_address())
                .clamp(current_page.get_base_address(), Self::max_viewport_start_address(current_page)),
        )
    }

    fn derive_viewport_start_for_focus_address(
        normalized_region: &NormalizedRegion,
        focus_address: u64,
    ) -> u64 {
        let desired_start_address = focus_address.saturating_sub(Self::CODE_WINDOW_SIZE_IN_BYTES / 4);

        Self::clamp_viewport_start_address(normalized_region, desired_start_address)
    }

    fn clamp_viewport_start_address(
        normalized_region: &NormalizedRegion,
        viewport_start_address: u64,
    ) -> u64 {
        viewport_start_address.clamp(normalized_region.get_base_address(), Self::max_viewport_start_address(normalized_region))
    }

    fn max_viewport_start_address(normalized_region: &NormalizedRegion) -> u64 {
        normalized_region
            .get_end_address()
            .saturating_sub(Self::CODE_WINDOW_SIZE_IN_BYTES.max(1))
            .max(normalized_region.get_base_address())
    }

    fn resolve_viewport_window_for_address(
        normalized_region: &NormalizedRegion,
        viewport_start_address: u64,
    ) -> Range<u64> {
        let clamped_viewport_start = Self::clamp_viewport_start_address(normalized_region, viewport_start_address);
        let viewport_end_address = clamped_viewport_start
            .saturating_add(Self::CODE_WINDOW_SIZE_IN_BYTES)
            .min(normalized_region.get_end_address());

        clamped_viewport_start..viewport_end_address
    }

    fn format_stats_for_page_from_modules(
        modules: &[NormalizedModule],
        unreadable_page_base_addresses: &HashSet<u64>,
        current_page: Option<&NormalizedRegion>,
        viewport_start_address: Option<u64>,
    ) -> String {
        let Some(current_page) = current_page else {
            return String::from("No page selected.");
        };
        let page_base_address = current_page.get_base_address();
        let page_end_address = current_page.get_end_address();
        let module_label = modules
            .iter()
            .find(|normalized_module| normalized_module.contains_address(page_base_address))
            .map(|normalized_module| normalized_module.get_module_name().to_string())
            .unwrap_or_else(|| String::from("(No Module)"));
        let viewport_window = viewport_start_address
            .map(|viewport_start_address| Self::resolve_viewport_window_for_address(current_page, viewport_start_address))
            .unwrap_or_else(|| page_base_address..page_base_address);
        let unreadable_suffix = if unreadable_page_base_addresses.contains(&page_base_address) {
            " | Unreadable"
        } else {
            ""
        };

        format!(
            "{} | Page {} - {} | View {} - {}{}",
            module_label,
            format_absolute_address(page_base_address),
            format_absolute_address(page_end_address),
            format_absolute_address(viewport_window.start),
            format_absolute_address(viewport_window.end),
            unreadable_suffix
        )
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
        .all(|character| character.is_ascii_hexdigit())
    {
        return u64::from_str_radix(trimmed_address_text, 16).ok();
    }

    trimmed_address_text.parse::<u64>().ok()
}

#[cfg(test)]
mod tests {
    use super::CodeViewerViewData;
    use squalr_engine_api::structures::memory::{normalized_module::NormalizedModule, normalized_region::NormalizedRegion};

    #[test]
    fn build_visible_chunk_queries_aligns_viewport_to_chunk_window() {
        let normalized_region = NormalizedRegion::new(0x1000, 0x3000);
        let queries = CodeViewerViewData::build_visible_chunk_queries(&normalized_region, 0x1180);
        let query_ids = queries
            .iter()
            .map(|virtual_snapshot_query| virtual_snapshot_query.get_query_id().to_string())
            .collect::<Vec<_>>();

        assert_eq!(query_ids.first().map(String::as_str), Some("0000000000001000:0000000000000000"));
        assert_eq!(query_ids.last().map(String::as_str), Some("0000000000001000:0000000000001200"));
        assert_eq!(query_ids.len(), 19);
    }

    #[test]
    fn derive_viewport_start_for_focus_address_clamps_to_page_bounds() {
        let normalized_region = NormalizedRegion::new(0x4000, 0x500);
        let derived_viewport_start = CodeViewerViewData::derive_viewport_start_for_focus_address(&normalized_region, 0x4010);

        assert_eq!(derived_viewport_start, 0x4000);
    }

    #[test]
    fn resolve_initial_page_index_prefers_first_module_page() {
        let virtual_pages = vec![
            NormalizedRegion::new(0x1000, 0x100),
            NormalizedRegion::new(0x4000, 0x100),
            NormalizedRegion::new(0x8000, 0x100),
        ];
        let modules = vec![NormalizedModule::new("winmine.exe", 0x4000, 0x1000)];

        let resolved_page_index = CodeViewerViewData::resolve_initial_page_index(&virtual_pages, &modules);

        assert_eq!(resolved_page_index, Some(1));
    }
}
