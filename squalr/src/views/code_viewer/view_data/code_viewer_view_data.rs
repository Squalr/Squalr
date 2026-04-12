use eframe::egui::Pos2;
use squalr_engine_api::{
    commands::{
        memory::query::{memory_query_request::MemoryQueryRequest, memory_query_response::MemoryQueryResponse},
        privileged_command_request::PrivilegedCommandRequest,
        project_items::create::project_items_create_request::ProjectItemsCreateRequest,
    },
    dependency_injection::dependency::Dependency,
    plugins::instruction_set::InstructionSet,
    plugins::memory_view::PageRetrievalMode,
    structures::{
        data_types::{built_in_types::u8::data_type_u8::DataTypeU8, data_type_ref::DataTypeRef},
        data_values::{anonymous_value_string::AnonymousValueString, anonymous_value_string_format::AnonymousValueStringFormat, container_type::ContainerType},
        memory::{
            address_display::{format_absolute_address, format_module_address},
            bitness::Bitness,
            normalized_module::NormalizedModule,
            normalized_region::NormalizedRegion,
        },
        projects::project_items::built_in_types::project_item_type_address::ProjectItemTypeAddress,
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
    path::PathBuf,
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

#[derive(Clone, Debug, PartialEq, Eq)]
struct CodeViewerInstructionSelectionRange {
    anchor_instruction_address: u64,
    active_instruction_address: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct CodeViewerInstructionWritePlan {
    pub start_address: u64,
    pub written_bytes: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum CodeViewerInstructionEditStatus {
    Invalid(String),
    PendingFillWithNops { assembled_bytes: Vec<u8>, remaining_byte_count: usize },
    PendingOverwrite { assembled_bytes: Vec<u8>, overwritten_byte_count: usize },
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct CodeViewerInstructionEditState {
    pub start_address: u64,
    pub end_address_exclusive: u64,
    pub edit_value: AnonymousValueString,
    pub status: Option<CodeViewerInstructionEditStatus>,
}

impl CodeViewerInstructionEditState {
    pub(crate) fn original_byte_count(&self) -> usize {
        self.end_address_exclusive.saturating_sub(self.start_address) as usize
    }
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
    selected_instruction_range: Option<CodeViewerInstructionSelectionRange>,
    viewport_start_address: Option<u64>,
    breakpoint_addresses: HashSet<u64>,
    context_menu_address: Option<u64>,
    context_menu_position: Option<Pos2>,
    instruction_edit_state: Option<CodeViewerInstructionEditState>,
    pub go_to_address_input: AnonymousValueString,
    pub bytes_text_splitter_ratio: f32,
    has_keyboard_focus: bool,
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
    pub const DEFAULT_BYTES_TEXT_SPLITTER_RATIO: f32 = 0.42;
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
            selected_instruction_range: None,
            viewport_start_address: None,
            breakpoint_addresses: HashSet::new(),
            context_menu_address: None,
            context_menu_position: None,
            instruction_edit_state: None,
            go_to_address_input: AnonymousValueString::new(String::new(), AnonymousValueStringFormat::Hexadecimal, ContainerType::None),
            bytes_text_splitter_ratio: Self::DEFAULT_BYTES_TEXT_SPLITTER_RATIO,
            has_keyboard_focus: false,
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
            code_viewer_view_data.selected_instruction_range = None;
            code_viewer_view_data.viewport_start_address = None;
            code_viewer_view_data.context_menu_address = None;
            code_viewer_view_data.context_menu_position = None;
            code_viewer_view_data.has_keyboard_focus = false;
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

    pub fn get_go_to_address_preview_text(code_viewer_view_data: Dependency<Self>) -> String {
        code_viewer_view_data
            .read("Code viewer go to address preview")
            .and_then(|code_viewer_view_data| {
                code_viewer_view_data
                    .resolve_selected_instruction_address()
                    .or_else(|| code_viewer_view_data.resolve_viewport_start_address())
                    .or_else(|| {
                        code_viewer_view_data
                            .virtual_pages
                            .get(code_viewer_view_data.current_page_index as usize)
                            .map(|normalized_region| normalized_region.get_base_address())
                    })
                    .map(format_absolute_address)
            })
            .unwrap_or_else(|| String::from("Go to address"))
    }

    pub fn seek_to_input_address(code_viewer_view_data: Dependency<Self>) {
        let Some(mut code_viewer_view_data) = code_viewer_view_data.write("Code viewer seek to input address") else {
            return;
        };
        let Some(target_address) = parse_address_text(
            code_viewer_view_data
                .go_to_address_input
                .get_anonymous_value_string(),
        ) else {
            return;
        };

        if let Some(resolved_address) = code_viewer_view_data.seek_to_address_internal(target_address) {
            code_viewer_view_data
                .go_to_address_input
                .set_anonymous_value_string(format_absolute_address(resolved_address));
        }
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

    pub fn resolve_scroll_target_address(
        pending_scroll_address: Option<u64>,
        instruction_lines: &[DisassembledInstruction],
    ) -> Option<u64> {
        let pending_scroll_address = pending_scroll_address?;

        instruction_lines
            .iter()
            .min_by_key(|instruction_line| instruction_line.address.abs_diff(pending_scroll_address))
            .map(|instruction_line| instruction_line.address)
    }

    pub fn select_instruction_address(
        code_viewer_view_data: Dependency<Self>,
        address: u64,
    ) {
        if let Some(mut code_viewer_view_data) = code_viewer_view_data.write("Code viewer select instruction address") {
            code_viewer_view_data.selected_instruction_range = Some(CodeViewerInstructionSelectionRange {
                anchor_instruction_address: address,
                active_instruction_address: address,
            });
        }
    }

    pub fn extend_instruction_selection(
        code_viewer_view_data: Dependency<Self>,
        address: u64,
    ) {
        if let Some(mut code_viewer_view_data) = code_viewer_view_data.write("Code viewer extend instruction selection") {
            let selection_anchor_address = code_viewer_view_data
                .selected_instruction_range
                .as_ref()
                .map(|selected_instruction_range| selected_instruction_range.anchor_instruction_address)
                .unwrap_or(address);

            code_viewer_view_data.selected_instruction_range = Some(CodeViewerInstructionSelectionRange {
                anchor_instruction_address: selection_anchor_address,
                active_instruction_address: address,
            });
        }
    }

    pub fn get_selected_instruction_address(code_viewer_view_data: Dependency<Self>) -> Option<u64> {
        code_viewer_view_data
            .read("Code viewer selected instruction address")
            .and_then(|code_viewer_view_data| code_viewer_view_data.resolve_selected_instruction_address())
    }

    pub fn get_selected_instruction_addresses(
        code_viewer_view_data: Dependency<Self>,
        instruction_lines: &[DisassembledInstruction],
    ) -> HashSet<u64> {
        code_viewer_view_data
            .read("Code viewer selected instruction addresses")
            .map(|code_viewer_view_data| code_viewer_view_data.resolve_selected_instruction_addresses(instruction_lines))
            .unwrap_or_default()
    }

    pub fn show_context_menu(
        code_viewer_view_data: Dependency<Self>,
        address: u64,
        position: Pos2,
    ) {
        if let Some(mut code_viewer_view_data) = code_viewer_view_data.write("Code viewer show context menu") {
            code_viewer_view_data.context_menu_address = Some(address);
            code_viewer_view_data.context_menu_position = Some(position);
            code_viewer_view_data.has_keyboard_focus = true;
        }
    }

    pub fn hide_context_menu(code_viewer_view_data: Dependency<Self>) {
        if let Some(mut code_viewer_view_data) = code_viewer_view_data.write("Code viewer hide context menu") {
            code_viewer_view_data.context_menu_address = None;
            code_viewer_view_data.context_menu_position = None;
        }
    }

    pub fn get_context_menu_state(code_viewer_view_data: Dependency<Self>) -> Option<(u64, Pos2)> {
        let code_viewer_view_data = code_viewer_view_data.read("Code viewer context menu state")?;

        Some((code_viewer_view_data.context_menu_address?, code_viewer_view_data.context_menu_position?))
    }

    pub fn request_instruction_edit(
        code_viewer_view_data: Dependency<Self>,
        instruction_address: u64,
        instruction_lines: &[DisassembledInstruction],
    ) {
        let Some(mut code_viewer_view_data) = code_viewer_view_data.write("Code viewer request instruction edit") else {
            return;
        };
        let Some((selection_start_index, selection_end_index)) =
            code_viewer_view_data.resolve_instruction_edit_index_range(instruction_address, instruction_lines)
        else {
            return;
        };
        let instruction_slice = &instruction_lines[selection_start_index..=selection_end_index];
        let Some(first_instruction) = instruction_slice.first() else {
            return;
        };
        let Some(last_instruction) = instruction_slice.last() else {
            return;
        };
        let edit_text = instruction_slice
            .iter()
            .map(|instruction_line| instruction_line.text.clone())
            .collect::<Vec<_>>()
            .join("; ");

        code_viewer_view_data.selected_instruction_range = Some(CodeViewerInstructionSelectionRange {
            anchor_instruction_address: first_instruction.address,
            active_instruction_address: last_instruction.address,
        });
        code_viewer_view_data.instruction_edit_state = Some(CodeViewerInstructionEditState {
            start_address: first_instruction.address,
            end_address_exclusive: last_instruction
                .address
                .saturating_add((last_instruction.length as u64).max(1)),
            edit_value: AnonymousValueString::new(edit_text, AnonymousValueStringFormat::String, ContainerType::None),
            status: None,
        });
        code_viewer_view_data.context_menu_address = None;
        code_viewer_view_data.context_menu_position = None;
        code_viewer_view_data.has_keyboard_focus = true;
    }

    pub(crate) fn get_instruction_edit_state(code_viewer_view_data: Dependency<Self>) -> Option<CodeViewerInstructionEditState> {
        code_viewer_view_data
            .read("Code viewer instruction edit state")
            .and_then(|code_viewer_view_data| code_viewer_view_data.instruction_edit_state.clone())
    }

    pub fn set_instruction_edit_value(
        code_viewer_view_data: Dependency<Self>,
        edit_value: AnonymousValueString,
    ) {
        if let Some(mut code_viewer_view_data) = code_viewer_view_data.write("Code viewer set instruction edit value") {
            if let Some(instruction_edit_state) = code_viewer_view_data.instruction_edit_state.as_mut() {
                instruction_edit_state.edit_value = edit_value;
                instruction_edit_state.status = None;
            }
        }
    }

    pub fn cancel_instruction_edit(code_viewer_view_data: Dependency<Self>) {
        if let Some(mut code_viewer_view_data) = code_viewer_view_data.write("Code viewer cancel instruction edit") {
            code_viewer_view_data.instruction_edit_state = None;
        }
    }

    pub(crate) fn evaluate_instruction_edit_commit(
        code_viewer_view_data: Dependency<Self>,
        process_bitness: Option<Bitness>,
    ) -> Option<CodeViewerInstructionWritePlan> {
        let Some(mut code_viewer_view_data) = code_viewer_view_data.write("Code viewer evaluate instruction edit commit") else {
            return None;
        };
        let Some(instruction_edit_state) = code_viewer_view_data.instruction_edit_state.as_mut() else {
            return None;
        };
        let instruction_set = Self::create_instruction_set_for_process_bitness(process_bitness);
        let assembled_bytes = match instruction_set.assemble(
            instruction_edit_state
                .edit_value
                .get_anonymous_value_string()
                .trim(),
        ) {
            Ok(assembled_bytes) => assembled_bytes,
            Err(error) => {
                instruction_edit_state.status = Some(CodeViewerInstructionEditStatus::Invalid(error));
                return None;
            }
        };
        let original_byte_count = instruction_edit_state.original_byte_count();

        if assembled_bytes.len() == original_byte_count {
            instruction_edit_state.status = None;

            return Some(CodeViewerInstructionWritePlan {
                start_address: instruction_edit_state.start_address,
                written_bytes: assembled_bytes,
            });
        }

        if assembled_bytes.len() < original_byte_count {
            let remaining_byte_count = original_byte_count.saturating_sub(assembled_bytes.len());
            instruction_edit_state.status = Some(CodeViewerInstructionEditStatus::PendingFillWithNops {
                assembled_bytes,
                remaining_byte_count,
            });

            return None;
        }

        instruction_edit_state.status = Some(CodeViewerInstructionEditStatus::PendingOverwrite {
            overwritten_byte_count: assembled_bytes.len().saturating_sub(original_byte_count),
            assembled_bytes,
        });

        None
    }

    pub(crate) fn accept_instruction_edit_pending_fill_with_nops(
        code_viewer_view_data: Dependency<Self>,
        process_bitness: Option<Bitness>,
    ) -> Option<CodeViewerInstructionWritePlan> {
        let Some(mut code_viewer_view_data) = code_viewer_view_data.write("Code viewer accept instruction fill with nops") else {
            return None;
        };
        let Some(instruction_edit_state) = code_viewer_view_data.instruction_edit_state.as_mut() else {
            return None;
        };
        let Some(CodeViewerInstructionEditStatus::PendingFillWithNops {
            assembled_bytes,
            remaining_byte_count,
        }) = instruction_edit_state.status.clone()
        else {
            return None;
        };
        let instruction_set = Self::create_instruction_set_for_process_bitness(process_bitness);
        let nop_fill_bytes = match instruction_set.build_no_operation_fill(remaining_byte_count) {
            Ok(nop_fill_bytes) => nop_fill_bytes,
            Err(error) => {
                instruction_edit_state.status = Some(CodeViewerInstructionEditStatus::Invalid(error));
                return None;
            }
        };
        let mut written_bytes = assembled_bytes;
        written_bytes.extend_from_slice(&nop_fill_bytes);
        instruction_edit_state.status = None;

        Some(CodeViewerInstructionWritePlan {
            start_address: instruction_edit_state.start_address,
            written_bytes,
        })
    }

    pub(crate) fn accept_instruction_edit_pending_overwrite(code_viewer_view_data: Dependency<Self>) -> Option<CodeViewerInstructionWritePlan> {
        let Some(mut code_viewer_view_data) = code_viewer_view_data.write("Code viewer accept instruction overwrite") else {
            return None;
        };
        let Some(instruction_edit_state) = code_viewer_view_data.instruction_edit_state.as_mut() else {
            return None;
        };
        let Some(CodeViewerInstructionEditStatus::PendingOverwrite { assembled_bytes, .. }) = instruction_edit_state.status.clone() else {
            return None;
        };
        instruction_edit_state.status = None;

        Some(CodeViewerInstructionWritePlan {
            start_address: instruction_edit_state.start_address,
            written_bytes: assembled_bytes,
        })
    }

    pub fn finish_instruction_write(
        code_viewer_view_data: Dependency<Self>,
        write_start_address: u64,
    ) {
        if let Some(mut code_viewer_view_data) = code_viewer_view_data.write("Code viewer finish instruction write") {
            code_viewer_view_data.instruction_edit_state = None;
            code_viewer_view_data.selected_instruction_range = Some(CodeViewerInstructionSelectionRange {
                anchor_instruction_address: write_start_address,
                active_instruction_address: write_start_address,
            });
            code_viewer_view_data.pending_scroll_address = Some(write_start_address);
        }
    }

    pub fn set_instruction_edit_error(
        code_viewer_view_data: Dependency<Self>,
        error: String,
    ) {
        if let Some(mut code_viewer_view_data) = code_viewer_view_data.write("Code viewer set instruction edit error") {
            if let Some(instruction_edit_state) = code_viewer_view_data.instruction_edit_state.as_mut() {
                instruction_edit_state.status = Some(CodeViewerInstructionEditStatus::Invalid(error));
            }
        }
    }

    pub fn build_instruction_project_item_create_request(
        code_viewer_view_data: Dependency<Self>,
        absolute_address: u64,
        target_directory_path: Option<PathBuf>,
        process_bitness: Option<Bitness>,
        instruction_lines: &[DisassembledInstruction],
    ) -> Option<ProjectItemsCreateRequest> {
        let code_viewer_view_data = code_viewer_view_data.read("Code viewer build instruction project item request")?;
        let (selection_start_address, selected_byte_count) = code_viewer_view_data
            .resolve_selected_instruction_project_item_span(absolute_address, instruction_lines)
            .or_else(|| {
                instruction_lines
                    .iter()
                    .find(|instruction_line| instruction_line.address == absolute_address)
                    .map(|instruction_line| (instruction_line.address, (instruction_line.length as u64).max(1)))
            })?;
        let (project_item_address, project_item_module_name) = code_viewer_view_data.resolve_project_item_address(selection_start_address);
        let instruction_data_type_id = Self::instruction_data_type_id_for_process_bitness(process_bitness);
        let resolved_data_type_id = if selected_byte_count > 1 {
            format!("{}[{}]", instruction_data_type_id, selected_byte_count)
        } else {
            instruction_data_type_id.to_string()
        };

        Some(ProjectItemsCreateRequest {
            parent_directory_path: target_directory_path.unwrap_or_default(),
            project_item_name: Self::format_project_item_name(project_item_address, &project_item_module_name),
            project_item_type: ProjectItemTypeAddress::PROJECT_ITEM_TYPE_ID.to_string(),
            pointer: None,
            address: Some(project_item_address),
            module_name: Some(project_item_module_name),
            data_type_id: Some(resolved_data_type_id),
        })
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

    pub fn clear_selection(code_viewer_view_data: Dependency<Self>) {
        if let Some(mut code_viewer_view_data) = code_viewer_view_data.write("Code viewer clear selection") {
            code_viewer_view_data.selected_instruction_range = None;
            code_viewer_view_data.pending_scroll_address = None;
            code_viewer_view_data.instruction_edit_state = None;
        }
    }

    pub fn set_keyboard_focus(
        code_viewer_view_data: Dependency<Self>,
        has_keyboard_focus: bool,
    ) {
        if let Some(mut code_viewer_view_data) = code_viewer_view_data.write("Code viewer set keyboard focus") {
            code_viewer_view_data.has_keyboard_focus = has_keyboard_focus;
        }
    }

    pub fn has_keyboard_focus(code_viewer_view_data: Dependency<Self>) -> bool {
        code_viewer_view_data
            .read("Code viewer has keyboard focus")
            .map(|code_viewer_view_data| code_viewer_view_data.has_keyboard_focus)
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

    fn resolve_selected_instruction_address(&self) -> Option<u64> {
        self.selected_instruction_range
            .as_ref()
            .map(|selected_instruction_range| selected_instruction_range.active_instruction_address)
    }

    fn resolve_selected_instruction_index_range(
        &self,
        instruction_lines: &[DisassembledInstruction],
    ) -> Option<(usize, usize)> {
        let selected_instruction_range = self.selected_instruction_range.as_ref()?;
        let anchor_instruction_index = instruction_lines
            .iter()
            .position(|instruction_line| instruction_line.address == selected_instruction_range.anchor_instruction_address)?;
        let active_instruction_index = instruction_lines
            .iter()
            .position(|instruction_line| instruction_line.address == selected_instruction_range.active_instruction_address)?;

        Some((
            anchor_instruction_index.min(active_instruction_index),
            anchor_instruction_index.max(active_instruction_index),
        ))
    }

    fn resolve_selected_instruction_addresses(
        &self,
        instruction_lines: &[DisassembledInstruction],
    ) -> HashSet<u64> {
        let Some((selection_start_index, selection_end_index)) = self.resolve_selected_instruction_index_range(instruction_lines) else {
            return HashSet::new();
        };

        instruction_lines[selection_start_index..=selection_end_index]
            .iter()
            .map(|instruction_line| instruction_line.address)
            .collect()
    }

    fn resolve_selected_instruction_project_item_span(
        &self,
        context_menu_address: u64,
        instruction_lines: &[DisassembledInstruction],
    ) -> Option<(u64, u64)> {
        let (selection_start_index, selection_end_index) = self.resolve_selected_instruction_index_range(instruction_lines)?;
        let selected_instruction_addresses = self.resolve_selected_instruction_addresses(instruction_lines);

        if !selected_instruction_addresses.contains(&context_menu_address) {
            return None;
        }

        let first_selected_instruction = instruction_lines.get(selection_start_index)?;
        let last_selected_instruction = instruction_lines.get(selection_end_index)?;
        let selection_end_address_exclusive = last_selected_instruction
            .address
            .saturating_add((last_selected_instruction.length as u64).max(1));
        let selected_byte_count = selection_end_address_exclusive
            .saturating_sub(first_selected_instruction.address)
            .max(1);

        Some((first_selected_instruction.address, selected_byte_count))
    }

    fn resolve_instruction_edit_index_range(
        &self,
        instruction_address: u64,
        instruction_lines: &[DisassembledInstruction],
    ) -> Option<(usize, usize)> {
        let target_instruction_index = instruction_lines
            .iter()
            .position(|instruction_line| instruction_line.address == instruction_address)?;

        let Some((selection_start_index, selection_end_index)) = self.resolve_selected_instruction_index_range(instruction_lines) else {
            return Some((target_instruction_index, target_instruction_index));
        };
        let selected_instruction_addresses = self.resolve_selected_instruction_addresses(instruction_lines);

        if selected_instruction_addresses.contains(&instruction_address) {
            Some((selection_start_index, selection_end_index))
        } else {
            Some((target_instruction_index, target_instruction_index))
        }
    }

    fn instruction_data_type_id_for_process_bitness(process_bitness: Option<Bitness>) -> &'static str {
        match process_bitness.unwrap_or(Bitness::Bit64) {
            Bitness::Bit32 => "i_x86",
            Bitness::Bit64 => "i_x64",
        }
    }

    fn create_instruction_set_for_process_bitness(process_bitness: Option<Bitness>) -> Box<dyn InstructionSet> {
        match process_bitness.unwrap_or(Bitness::Bit64) {
            Bitness::Bit32 => Box::new(X86InstructionSet::new()),
            Bitness::Bit64 => Box::new(X64InstructionSet::new()),
        }
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
                code_viewer_view_data.selected_instruction_range = None;
                code_viewer_view_data.instruction_edit_state = None;
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

    pub fn apply_memory_write(
        code_viewer_view_data: Dependency<Self>,
        write_start_address: u64,
        written_bytes: &[u8],
    ) {
        let Some(mut code_viewer_view_data) = code_viewer_view_data.write("Code viewer apply memory write") else {
            return;
        };
        let Some(current_page) = code_viewer_view_data
            .virtual_pages
            .get(code_viewer_view_data.current_page_index as usize)
            .cloned()
        else {
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
            let chunk_bytes = code_viewer_view_data
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

        let Some((page_index, resolved_address)) = Self::resolve_nearest_page_index_and_address(&self.virtual_pages, focus_address) else {
            self.pending_focus_request = None;

            return false;
        };

        self.current_page_index = page_index;
        if let Some(current_page) = self.virtual_pages.get(page_index as usize) {
            self.viewport_start_address = Some(Self::derive_viewport_start_for_focus_address(current_page, resolved_address));
            self.stats_string = Self::format_stats_for_page_from_modules(
                &self.modules,
                &self.unreadable_page_base_addresses,
                Some(current_page),
                self.viewport_start_address,
            );
        }
        self.selected_instruction_range = Some(CodeViewerInstructionSelectionRange {
            anchor_instruction_address: resolved_address,
            active_instruction_address: resolved_address,
        });
        self.instruction_edit_state = None;
        self.pending_scroll_address = Some(resolved_address);
        self.pending_focus_request = None;
        self.has_keyboard_focus = true;

        true
    }

    fn seek_to_address_internal(
        &mut self,
        target_address: u64,
    ) -> Option<u64> {
        let (page_index, resolved_address) = Self::resolve_nearest_page_index_and_address(&self.virtual_pages, target_address)?;
        self.current_page_index = page_index.min(self.cached_last_page_index);

        if let Some(current_page) = self.virtual_pages.get(self.current_page_index as usize) {
            self.viewport_start_address = Some(Self::derive_viewport_start_for_focus_address(current_page, resolved_address));
            self.stats_string = Self::format_stats_for_page_from_modules(
                &self.modules,
                &self.unreadable_page_base_addresses,
                Some(current_page),
                self.viewport_start_address,
            );
        }

        self.selected_instruction_range = Some(CodeViewerInstructionSelectionRange {
            anchor_instruction_address: resolved_address,
            active_instruction_address: resolved_address,
        });
        self.instruction_edit_state = None;
        self.pending_scroll_address = Some(resolved_address);
        self.has_keyboard_focus = true;

        Some(resolved_address)
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
    use super::{CodeViewerInstructionEditState, CodeViewerInstructionEditStatus, CodeViewerViewData};
    use squalr_engine_api::{
        dependency_injection::dependency_container::DependencyContainer,
        structures::{
            data_values::{
                anonymous_value_string::AnonymousValueString, anonymous_value_string_format::AnonymousValueStringFormat, container_type::ContainerType,
            },
            memory::{bitness::Bitness, normalized_module::NormalizedModule, normalized_region::NormalizedRegion},
        },
    };

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

    #[test]
    fn select_instruction_address_does_not_queue_auto_scroll() {
        let dependency_container = DependencyContainer::new();
        let code_viewer_view_data = dependency_container.register(CodeViewerViewData::new());

        CodeViewerViewData::select_instruction_address(code_viewer_view_data.clone(), 0x4010);

        assert_eq!(
            CodeViewerViewData::get_selected_instruction_address(code_viewer_view_data.clone()),
            Some(0x4010)
        );
        assert_eq!(CodeViewerViewData::take_pending_scroll_address(code_viewer_view_data.clone()), None);
    }

    #[test]
    fn go_to_address_input_defaults_to_hexadecimal_format() {
        let code_viewer_view_data = CodeViewerViewData::new();

        assert_eq!(
            code_viewer_view_data
                .go_to_address_input
                .get_anonymous_value_string_format(),
            squalr_engine_api::structures::data_values::anonymous_value_string_format::AnonymousValueStringFormat::Hexadecimal
        );
    }

    #[test]
    fn extend_instruction_selection_selects_contiguous_instruction_rows() {
        let dependency_container = DependencyContainer::new();
        let code_viewer_view_data = dependency_container.register(CodeViewerViewData::new());
        let instruction_lines = vec![
            squalr_plugin_instructions_x86::DisassembledInstruction {
                address: 0x4010,
                length: 2,
                bytes: vec![0x31, 0xC0],
                text: String::from("xor eax, eax"),
                branch_target_address: None,
                is_control_flow: false,
            },
            squalr_plugin_instructions_x86::DisassembledInstruction {
                address: 0x4012,
                length: 5,
                bytes: vec![0xB8, 0x01, 0x00, 0x00, 0x00],
                text: String::from("mov eax, 1"),
                branch_target_address: None,
                is_control_flow: false,
            },
            squalr_plugin_instructions_x86::DisassembledInstruction {
                address: 0x4017,
                length: 1,
                bytes: vec![0xC3],
                text: String::from("ret"),
                branch_target_address: None,
                is_control_flow: true,
            },
        ];

        CodeViewerViewData::select_instruction_address(code_viewer_view_data.clone(), 0x4010);
        CodeViewerViewData::extend_instruction_selection(code_viewer_view_data.clone(), 0x4017);

        let selected_instruction_addresses = CodeViewerViewData::get_selected_instruction_addresses(code_viewer_view_data.clone(), &instruction_lines);

        assert_eq!(
            selected_instruction_addresses,
            std::collections::HashSet::from([0x4010_u64, 0x4012_u64, 0x4017_u64])
        );
    }

    #[test]
    fn resolve_scroll_target_address_chooses_nearest_instruction_address() {
        let instruction_lines = vec![
            squalr_plugin_instructions_x86::DisassembledInstruction {
                address: 0x1000,
                length: 1,
                bytes: vec![0x90],
                text: String::from("nop"),
                branch_target_address: None,
                is_control_flow: false,
            },
            squalr_plugin_instructions_x86::DisassembledInstruction {
                address: 0x1010,
                length: 1,
                bytes: vec![0x90],
                text: String::from("nop"),
                branch_target_address: None,
                is_control_flow: false,
            },
        ];

        assert_eq!(
            CodeViewerViewData::resolve_scroll_target_address(Some(0x100E), &instruction_lines),
            Some(0x1010)
        );
    }

    #[test]
    fn seek_to_input_address_clamps_to_nearest_page_when_target_is_missing() {
        let dependency_container = DependencyContainer::new();
        let code_viewer_view_data = dependency_container.register(CodeViewerViewData::new());

        if let Some(mut code_viewer_view_data_guard) = code_viewer_view_data.write("Test code viewer seek input") {
            code_viewer_view_data_guard.virtual_pages = vec![
                NormalizedRegion::new(0x1000, 0x100),
                NormalizedRegion::new(0x4000, 0x100),
            ];
            code_viewer_view_data_guard.cached_last_page_index = 1;
            code_viewer_view_data_guard
                .go_to_address_input
                .set_anonymous_value_string(String::from("0x3800"));
        }

        CodeViewerViewData::seek_to_input_address(code_viewer_view_data.clone());

        let code_viewer_view_data_guard = code_viewer_view_data
            .read("Test code viewer seek state")
            .expect("Expected code viewer state.");
        assert_eq!(code_viewer_view_data_guard.current_page_index, 1);
        assert_eq!(
            code_viewer_view_data_guard
                .selected_instruction_range
                .as_ref()
                .map(|selected_instruction_range| selected_instruction_range.active_instruction_address),
            Some(0x4000)
        );
        assert_eq!(code_viewer_view_data_guard.pending_scroll_address, Some(0x4000));
    }

    #[test]
    fn build_instruction_project_item_create_request_uses_selected_instruction_byte_span() {
        let dependency_container = DependencyContainer::new();
        let code_viewer_view_data = dependency_container.register(CodeViewerViewData::new());
        let instruction_lines = vec![
            squalr_plugin_instructions_x86::DisassembledInstruction {
                address: 0x5004,
                length: 5,
                bytes: vec![0xB8, 0x01, 0x00, 0x00, 0x00],
                text: String::from("mov eax, 1"),
                branch_target_address: None,
                is_control_flow: false,
            },
            squalr_plugin_instructions_x86::DisassembledInstruction {
                address: 0x5009,
                length: 2,
                bytes: vec![0xFF, 0xC0],
                text: String::from("inc eax"),
                branch_target_address: None,
                is_control_flow: false,
            },
        ];

        if let Some(mut code_viewer_view_data_guard) = code_viewer_view_data.write("Test code viewer create request setup") {
            code_viewer_view_data_guard.modules = vec![NormalizedModule::new("winmine.exe", 0x5000, 0x100)];
        }

        CodeViewerViewData::select_instruction_address(code_viewer_view_data.clone(), 0x5004);
        CodeViewerViewData::extend_instruction_selection(code_viewer_view_data.clone(), 0x5009);

        let create_request = CodeViewerViewData::build_instruction_project_item_create_request(
            code_viewer_view_data.clone(),
            0x5009,
            None,
            Some(squalr_engine_api::structures::memory::bitness::Bitness::Bit32),
            &instruction_lines,
        )
        .expect("Expected instruction project item request.");

        assert_eq!(create_request.address, Some(0x4));
        assert_eq!(create_request.module_name.as_deref(), Some("winmine.exe"));
        assert_eq!(create_request.data_type_id.as_deref(), Some("i_x86[7]"));
    }

    #[test]
    fn request_instruction_edit_uses_selected_instruction_span() {
        let dependency_container = DependencyContainer::new();
        let code_viewer_view_data = dependency_container.register(CodeViewerViewData::new());
        let instruction_lines = vec![
            squalr_plugin_instructions_x86::DisassembledInstruction {
                address: 0x4010,
                length: 1,
                bytes: vec![0x90],
                text: String::from("nop"),
                branch_target_address: None,
                is_control_flow: false,
            },
            squalr_plugin_instructions_x86::DisassembledInstruction {
                address: 0x4011,
                length: 1,
                bytes: vec![0xC3],
                text: String::from("ret"),
                branch_target_address: None,
                is_control_flow: true,
            },
        ];

        CodeViewerViewData::select_instruction_address(code_viewer_view_data.clone(), 0x4010);
        CodeViewerViewData::extend_instruction_selection(code_viewer_view_data.clone(), 0x4011);
        CodeViewerViewData::request_instruction_edit(code_viewer_view_data.clone(), 0x4011, &instruction_lines);

        let instruction_edit_state = CodeViewerViewData::get_instruction_edit_state(code_viewer_view_data.clone()).expect("Expected instruction edit state.");
        assert_eq!(instruction_edit_state.start_address, 0x4010);
        assert_eq!(instruction_edit_state.end_address_exclusive, 0x4012);
        assert_eq!(instruction_edit_state.edit_value.get_anonymous_value_string(), "nop; ret");
        assert_eq!(
            instruction_edit_state
                .edit_value
                .get_anonymous_value_string_format(),
            AnonymousValueStringFormat::String
        );
    }

    #[test]
    fn evaluate_instruction_edit_commit_returns_exact_write_plan() {
        let dependency_container = DependencyContainer::new();
        let code_viewer_view_data = dependency_container.register(CodeViewerViewData::new());

        if let Some(mut code_viewer_view_data_guard) = code_viewer_view_data.write("Test code viewer exact instruction edit") {
            code_viewer_view_data_guard.instruction_edit_state = Some(CodeViewerInstructionEditState {
                start_address: 0x5000,
                end_address_exclusive: 0x5001,
                edit_value: AnonymousValueString::new(String::from("push eax"), AnonymousValueStringFormat::String, ContainerType::None),
                status: None,
            });
        }

        let instruction_write_plan = CodeViewerViewData::evaluate_instruction_edit_commit(code_viewer_view_data.clone(), Some(Bitness::Bit32))
            .expect("Expected exact instruction write plan.");

        assert_eq!(instruction_write_plan.start_address, 0x5000);
        assert_eq!(instruction_write_plan.written_bytes, vec![0x50]);
        assert_eq!(
            CodeViewerViewData::get_instruction_edit_state(code_viewer_view_data.clone()).and_then(|instruction_edit_state| instruction_edit_state.status),
            None
        );
    }

    #[test]
    fn evaluate_instruction_edit_commit_sets_pending_fill_with_nops() {
        let dependency_container = DependencyContainer::new();
        let code_viewer_view_data = dependency_container.register(CodeViewerViewData::new());

        if let Some(mut code_viewer_view_data_guard) = code_viewer_view_data.write("Test code viewer underfill instruction edit") {
            code_viewer_view_data_guard.instruction_edit_state = Some(CodeViewerInstructionEditState {
                start_address: 0x5000,
                end_address_exclusive: 0x5002,
                edit_value: AnonymousValueString::new(String::from("push eax"), AnonymousValueStringFormat::String, ContainerType::None),
                status: None,
            });
        }

        assert!(CodeViewerViewData::evaluate_instruction_edit_commit(code_viewer_view_data.clone(), Some(Bitness::Bit32)).is_none());

        let instruction_edit_state = CodeViewerViewData::get_instruction_edit_state(code_viewer_view_data.clone()).expect("Expected instruction edit state.");

        assert_eq!(
            instruction_edit_state.status,
            Some(CodeViewerInstructionEditStatus::PendingFillWithNops {
                assembled_bytes: vec![0x50],
                remaining_byte_count: 1,
            })
        );

        let instruction_write_plan = CodeViewerViewData::accept_instruction_edit_pending_fill_with_nops(code_viewer_view_data.clone(), Some(Bitness::Bit32))
            .expect("Expected NOP-filled instruction write plan.");

        assert_eq!(instruction_write_plan.written_bytes, vec![0x50, 0x90]);
    }

    #[test]
    fn evaluate_instruction_edit_commit_sets_pending_overwrite() {
        let dependency_container = DependencyContainer::new();
        let code_viewer_view_data = dependency_container.register(CodeViewerViewData::new());

        if let Some(mut code_viewer_view_data_guard) = code_viewer_view_data.write("Test code viewer overwrite instruction edit") {
            code_viewer_view_data_guard.instruction_edit_state = Some(CodeViewerInstructionEditState {
                start_address: 0x5000,
                end_address_exclusive: 0x5001,
                edit_value: AnonymousValueString::new(String::from("push eax; push eax"), AnonymousValueStringFormat::String, ContainerType::None),
                status: None,
            });
        }

        assert!(CodeViewerViewData::evaluate_instruction_edit_commit(code_viewer_view_data.clone(), Some(Bitness::Bit32)).is_none());

        let instruction_edit_state = CodeViewerViewData::get_instruction_edit_state(code_viewer_view_data.clone()).expect("Expected instruction edit state.");

        assert_eq!(
            instruction_edit_state.status,
            Some(CodeViewerInstructionEditStatus::PendingOverwrite {
                assembled_bytes: vec![0x50, 0x50],
                overwritten_byte_count: 1,
            })
        );

        let instruction_write_plan =
            CodeViewerViewData::accept_instruction_edit_pending_overwrite(code_viewer_view_data.clone()).expect("Expected overwrite instruction write plan.");

        assert_eq!(instruction_write_plan.written_bytes, vec![0x50, 0x50]);
    }
}
