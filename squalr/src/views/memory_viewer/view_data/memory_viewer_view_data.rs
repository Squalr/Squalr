use crate::ui::geometry::safe_clamp_ord;
use arc_swap::Guard;
use eframe::egui::Pos2;
use squalr_engine_api::{
    commands::{
        memory::query::{memory_query_request::MemoryQueryRequest, memory_query_response::MemoryQueryResponse},
        privileged_command_request::PrivilegedCommandRequest,
        project_items::create::project_items_create_request::ProjectItemsCreateRequest,
    },
    conversions::storage_size_conversions::StorageSizeConversions,
    dependency_injection::dependency::Dependency,
    plugins::memory_view::PageRetrievalMode,
    structures::{
        data_types::{built_in_types::u8::data_type_u8::DataTypeU8, data_type_ref::DataTypeRef},
        data_values::{anonymous_value_string::AnonymousValueString, anonymous_value_string_format::AnonymousValueStringFormat, container_type::ContainerType},
        memory::{
            address_display::{format_absolute_address, format_module_address},
            normalized_module::NormalizedModule,
            normalized_region::NormalizedRegion,
        },
        projects::project_items::project_item_target::ProjectItemTarget,
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
    time::{Duration, Instant},
};

#[derive(Clone, Debug, Default)]
pub struct MemoryViewerPageCache {
    cached_chunks: BTreeMap<u64, Vec<u8>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct MemoryViewerFocusRequest {
    address: u64,
    module_name: String,
    selection_byte_count: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct MemoryViewerSelectionRange {
    anchor_address: u64,
    active_address: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MemoryViewerEditCursorLane {
    Hexadecimal,
    String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MemoryViewerEditState {
    pub cursor_address: u64,
    pub active_lane: MemoryViewerEditCursorLane,
    pub active_nibble_index: u8,
    pub pending_high_nibble: Option<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MemoryViewerSelectionSummary {
    pub selection_start_address: u64,
    pub selection_end_address: u64,
    pub selection_display_text: String,
    pub selected_bytes: Vec<Option<u8>>,
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
    retry_memory_pages_refresh_on_failure: bool,
    last_applied_snapshot_generation: u64,
    page_caches_by_base_address: HashMap<u64, MemoryViewerPageCache>,
    unreadable_page_base_addresses: HashSet<u64>,
    pending_focus_request: Option<MemoryViewerFocusRequest>,
    pending_scroll_address: Option<u64>,
    selected_byte_range: Option<MemoryViewerSelectionRange>,
    is_drag_selection_active: bool,
    edit_state: Option<MemoryViewerEditState>,
    has_keyboard_focus: bool,
    context_menu_address: Option<u64>,
    context_menu_position: Option<Pos2>,
    pub go_to_address_input: AnonymousValueString,
    pub hex_ascii_splitter_ratio: f32,
}

impl MemoryViewerViewData {
    pub const WINDOW_VIRTUAL_SNAPSHOT_ID: &'static str = "memory_viewer";
    pub const BYTES_PER_ROW: u64 = 16;
    pub const MAX_SELECTION_SIZE_IN_BYTES: u64 = 2 * 1024 * 1024;
    pub const QUERY_CHUNK_SIZE_IN_BYTES: u64 = 256;
    pub const QUERY_PREFETCH_CHUNK_COUNT: u64 = 1;
    pub const DEFAULT_HEX_ASCII_SPLITTER_RATIO: f32 = 0.68;
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
            retry_memory_pages_refresh_on_failure: false,
            last_applied_snapshot_generation: 0,
            page_caches_by_base_address: HashMap::new(),
            unreadable_page_base_addresses: HashSet::new(),
            pending_focus_request: None,
            pending_scroll_address: None,
            selected_byte_range: None,
            is_drag_selection_active: false,
            edit_state: None,
            has_keyboard_focus: false,
            context_menu_address: None,
            context_menu_position: None,
            go_to_address_input: AnonymousValueString::new(String::new(), AnonymousValueStringFormat::Hexadecimal, ContainerType::None),
            hex_ascii_splitter_ratio: Self::DEFAULT_HEX_ASCII_SPLITTER_RATIO,
        }
    }

    pub fn request_focus_address(
        memory_viewer_view_data: Dependency<Self>,
        engine_unprivileged_state: Arc<EngineUnprivilegedState>,
        address: u64,
        module_name: String,
    ) {
        Self::request_focus_address_range(memory_viewer_view_data, engine_unprivileged_state, address, module_name, 1);
    }

    pub fn request_focus_address_range(
        memory_viewer_view_data: Dependency<Self>,
        engine_unprivileged_state: Arc<EngineUnprivilegedState>,
        address: u64,
        module_name: String,
        selection_byte_count: u64,
    ) {
        let should_refresh_memory_pages = match memory_viewer_view_data.write("Memory viewer request focus address") {
            Some(mut memory_viewer_view_data) => {
                memory_viewer_view_data.pending_focus_request = Some(MemoryViewerFocusRequest {
                    address,
                    module_name,
                    selection_byte_count: selection_byte_count.max(1),
                });

                if memory_viewer_view_data.try_apply_pending_focus_request() {
                    false
                } else {
                    memory_viewer_view_data.virtual_pages.is_empty() && !memory_viewer_view_data.is_querying_memory_pages
                }
            }
            None => return,
        };

        if should_refresh_memory_pages {
            Self::refresh_memory_pages(memory_viewer_view_data, engine_unprivileged_state);
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

    fn begin_selection_on_lane(
        memory_viewer_view_data: Dependency<Self>,
        address: u64,
        extend_existing_selection: bool,
        edit_cursor_lane: MemoryViewerEditCursorLane,
    ) {
        if let Some(mut memory_viewer_view_data) = memory_viewer_view_data.write("Memory viewer begin byte selection") {
            memory_viewer_view_data.begin_selection_internal(address, extend_existing_selection, edit_cursor_lane);
        }
    }

    pub fn begin_hex_selection(
        memory_viewer_view_data: Dependency<Self>,
        address: u64,
    ) {
        Self::begin_selection_on_lane(memory_viewer_view_data, address, false, MemoryViewerEditCursorLane::Hexadecimal);
    }

    pub fn begin_string_selection(
        memory_viewer_view_data: Dependency<Self>,
        address: u64,
    ) {
        Self::begin_selection_on_lane(memory_viewer_view_data, address, false, MemoryViewerEditCursorLane::String);
    }

    pub fn extend_hex_selection(
        memory_viewer_view_data: Dependency<Self>,
        address: u64,
    ) {
        Self::begin_selection_on_lane(memory_viewer_view_data, address, true, MemoryViewerEditCursorLane::Hexadecimal);
    }

    pub fn extend_string_selection(
        memory_viewer_view_data: Dependency<Self>,
        address: u64,
    ) {
        Self::begin_selection_on_lane(memory_viewer_view_data, address, true, MemoryViewerEditCursorLane::String);
    }

    pub fn update_byte_selection(
        memory_viewer_view_data: Dependency<Self>,
        address: u64,
    ) {
        if let Some(mut memory_viewer_view_data) = memory_viewer_view_data.write("Memory viewer update byte selection") {
            memory_viewer_view_data.update_selection_internal(address);
        }
    }

    pub fn set_drag_selection_active(
        memory_viewer_view_data: Dependency<Self>,
        is_drag_selection_active: bool,
    ) {
        if let Some(mut memory_viewer_view_data) = memory_viewer_view_data.write("Memory viewer set drag selection active") {
            memory_viewer_view_data.is_drag_selection_active = is_drag_selection_active;
        }
    }

    pub fn is_drag_selection_active(memory_viewer_view_data: Dependency<Self>) -> bool {
        memory_viewer_view_data
            .read("Memory viewer is drag selection active")
            .map(|memory_viewer_view_data| memory_viewer_view_data.is_drag_selection_active)
            .unwrap_or(false)
    }

    pub fn move_cursor_horizontal(
        memory_viewer_view_data: Dependency<Self>,
        column_delta: i64,
        extend_selection: bool,
    ) {
        if let Some(mut memory_viewer_view_data) = memory_viewer_view_data.write("Memory viewer move cursor horizontal") {
            memory_viewer_view_data.move_cursor_internal(column_delta, extend_selection);
        }
    }

    pub fn move_cursor_vertical(
        memory_viewer_view_data: Dependency<Self>,
        row_delta: i64,
        extend_selection: bool,
    ) {
        if let Some(mut memory_viewer_view_data) = memory_viewer_view_data.write("Memory viewer move cursor vertical") {
            memory_viewer_view_data.move_cursor_internal(row_delta.saturating_mul(Self::BYTES_PER_ROW as i64), extend_selection);
        }
    }

    pub fn is_byte_selected(
        memory_viewer_view_data: Dependency<Self>,
        address: u64,
    ) -> bool {
        memory_viewer_view_data
            .read("Memory viewer is byte selected")
            .and_then(|memory_viewer_view_data| memory_viewer_view_data.resolve_selected_address_bounds())
            .map(|(selection_start_address, selection_end_address)| address >= selection_start_address && address <= selection_end_address)
            .unwrap_or(false)
    }

    pub fn get_selected_address_bounds(memory_viewer_view_data: Dependency<Self>) -> Option<(u64, u64)> {
        memory_viewer_view_data
            .read("Memory viewer selected address bounds")
            .and_then(|memory_viewer_view_data| memory_viewer_view_data.resolve_selected_address_bounds())
    }

    pub fn get_selection_summary(memory_viewer_view_data: Dependency<Self>) -> Option<MemoryViewerSelectionSummary> {
        memory_viewer_view_data
            .read("Memory viewer selection summary")
            .and_then(|memory_viewer_view_data| memory_viewer_view_data.build_selection_summary())
    }

    pub fn get_go_to_address_preview_text(memory_viewer_view_data: Dependency<Self>) -> String {
        memory_viewer_view_data
            .read("Memory viewer go to address preview")
            .and_then(|memory_viewer_view_data| {
                memory_viewer_view_data
                    .resolve_selected_address_bounds()
                    .map(|(selection_start_address, _)| selection_start_address)
                    .or_else(|| {
                        memory_viewer_view_data
                            .virtual_pages
                            .get(Self::load_current_page_index(&memory_viewer_view_data) as usize)
                            .map(|normalized_region| normalized_region.get_base_address())
                    })
                    .map(format_absolute_address)
            })
            .unwrap_or_else(|| String::from("Go to address"))
    }

    pub fn seek_to_input_address(memory_viewer_view_data: Dependency<Self>) {
        let Some(mut memory_viewer_view_data) = memory_viewer_view_data.write("Memory viewer seek to input address") else {
            return;
        };
        let Some(target_address) = parse_address_text(
            memory_viewer_view_data
                .go_to_address_input
                .get_anonymous_value_string(),
        ) else {
            return;
        };

        if let Some(resolved_address) = memory_viewer_view_data.seek_to_address_internal(target_address, 1) {
            memory_viewer_view_data
                .go_to_address_input
                .set_anonymous_value_string(format_absolute_address(resolved_address));
        }
    }

    pub fn select_all_bytes_on_current_page(memory_viewer_view_data: Dependency<Self>) {
        if let Some(mut memory_viewer_view_data) = memory_viewer_view_data.write("Memory viewer select all bytes on current page") {
            let Some(current_page_address_range) = memory_viewer_view_data.resolve_current_page_address_range() else {
                return;
            };
            let active_edit_cursor_lane = memory_viewer_view_data.resolve_active_edit_cursor_lane_or_default();

            memory_viewer_view_data.set_selected_byte_range(current_page_address_range.start, current_page_address_range.end.saturating_sub(1));
            memory_viewer_view_data.has_keyboard_focus = true;
            memory_viewer_view_data.set_edit_cursor(current_page_address_range.start, active_edit_cursor_lane);
        }
    }

    pub fn clear_selection(memory_viewer_view_data: Dependency<Self>) {
        if let Some(mut memory_viewer_view_data) = memory_viewer_view_data.write("Memory viewer clear selection") {
            memory_viewer_view_data.selected_byte_range = None;
            memory_viewer_view_data.is_drag_selection_active = false;
            memory_viewer_view_data.edit_state = None;
        }
    }

    pub fn handle_edit_backspace(memory_viewer_view_data: Dependency<Self>) {
        if let Some(mut memory_viewer_view_data) = memory_viewer_view_data.write("Memory viewer edit backspace") {
            memory_viewer_view_data.handle_edit_backspace_internal();
        }
    }

    pub fn append_edit_text(
        memory_viewer_view_data: Dependency<Self>,
        typed_text: &str,
    ) -> Vec<(u64, Vec<u8>)> {
        let Some(mut memory_viewer_view_data) = memory_viewer_view_data.write("Memory viewer append edit text") else {
            return Vec::new();
        };

        memory_viewer_view_data.append_edit_text_internal(typed_text)
    }

    pub fn apply_memory_write(
        memory_viewer_view_data: Dependency<Self>,
        write_start_address: u64,
        written_bytes: &[u8],
    ) {
        let Some(mut memory_viewer_view_data) = memory_viewer_view_data.write("Memory viewer apply memory write") else {
            return;
        };
        let Some(current_page) = memory_viewer_view_data
            .virtual_pages
            .get(memory_viewer_view_data.current_page_index as usize)
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
            let chunk_bytes = memory_viewer_view_data
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

    pub fn set_keyboard_focus(
        memory_viewer_view_data: Dependency<Self>,
        has_keyboard_focus: bool,
    ) {
        if let Some(mut memory_viewer_view_data) = memory_viewer_view_data.write("Memory viewer set keyboard focus") {
            memory_viewer_view_data.has_keyboard_focus = has_keyboard_focus;

            if !has_keyboard_focus {
                if let Some(edit_state) = memory_viewer_view_data.edit_state.as_mut() {
                    edit_state.active_nibble_index = 0;
                    edit_state.pending_high_nibble = None;
                }
            }
        }
    }

    pub fn has_keyboard_focus(memory_viewer_view_data: Dependency<Self>) -> bool {
        memory_viewer_view_data
            .read("Memory viewer has keyboard focus")
            .map(|memory_viewer_view_data| memory_viewer_view_data.has_keyboard_focus)
            .unwrap_or(false)
    }

    pub fn get_edit_state(memory_viewer_view_data: Dependency<Self>) -> Option<MemoryViewerEditState> {
        memory_viewer_view_data
            .read("Memory viewer edit state")
            .and_then(|memory_viewer_view_data| memory_viewer_view_data.edit_state.clone())
    }

    pub fn show_context_menu(
        memory_viewer_view_data: Dependency<Self>,
        address: u64,
        position: Pos2,
    ) {
        if let Some(mut memory_viewer_view_data) = memory_viewer_view_data.write("Memory viewer show context menu") {
            memory_viewer_view_data.context_menu_address = Some(address);
            memory_viewer_view_data.context_menu_position = Some(position);
            memory_viewer_view_data.has_keyboard_focus = true;
        }
    }

    pub fn hide_context_menu(memory_viewer_view_data: Dependency<Self>) {
        if let Some(mut memory_viewer_view_data) = memory_viewer_view_data.write("Memory viewer hide context menu") {
            memory_viewer_view_data.context_menu_address = None;
            memory_viewer_view_data.context_menu_position = None;
        }
    }

    pub fn get_context_menu_state(memory_viewer_view_data: Dependency<Self>) -> Option<(u64, Pos2)> {
        let memory_viewer_view_data = memory_viewer_view_data.read("Memory viewer context menu state")?;

        Some((memory_viewer_view_data.context_menu_address?, memory_viewer_view_data.context_menu_position?))
    }

    pub fn build_address_project_item_create_request(
        memory_viewer_view_data: Dependency<Self>,
        absolute_address: u64,
        target_directory_path: Option<PathBuf>,
    ) -> Option<ProjectItemsCreateRequest> {
        Self::build_address_project_item_create_request_with_data_type(memory_viewer_view_data, absolute_address, target_directory_path, None)
    }

    pub fn build_address_project_item_create_request_with_data_type(
        memory_viewer_view_data: Dependency<Self>,
        absolute_address: u64,
        target_directory_path: Option<PathBuf>,
        explicit_data_type_id: Option<String>,
    ) -> Option<ProjectItemsCreateRequest> {
        let memory_viewer_view_data = memory_viewer_view_data.read("Memory viewer build address project item request")?;
        let (selection_start_address, selection_end_address) = memory_viewer_view_data
            .resolve_selected_address_bounds()
            .filter(|(selection_start_address, selection_end_address)| {
                absolute_address >= *selection_start_address && absolute_address <= *selection_end_address
            })
            .unwrap_or((absolute_address, absolute_address));
        let (project_item_address, project_item_module_name) = memory_viewer_view_data.resolve_project_item_address(selection_start_address);
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
            target: ProjectItemTarget::new_address(project_item_address, project_item_module_name),
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
        retry_memory_pages_refresh_on_failure: bool,
    ) {
        if let Some(mut memory_viewer_view_data) = memory_viewer_view_data.write("Memory viewer clear for process change") {
            memory_viewer_view_data.virtual_pages.clear();
            memory_viewer_view_data.modules.clear();
            memory_viewer_view_data.current_page_index = 0;
            memory_viewer_view_data.cached_last_page_index = 0;
            memory_viewer_view_data.stats_string.clear();
            memory_viewer_view_data.retry_memory_pages_refresh_on_failure = retry_memory_pages_refresh_on_failure;
            memory_viewer_view_data.last_applied_snapshot_generation = 0;
            memory_viewer_view_data.page_caches_by_base_address.clear();
            memory_viewer_view_data.unreadable_page_base_addresses.clear();
            memory_viewer_view_data.pending_scroll_address = None;
            memory_viewer_view_data.selected_byte_range = None;
            memory_viewer_view_data.is_drag_selection_active = false;
            memory_viewer_view_data.edit_state = None;
            memory_viewer_view_data.has_keyboard_focus = false;
            memory_viewer_view_data.context_menu_address = None;
            memory_viewer_view_data.context_menu_position = None;
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

    pub fn take_pending_scroll_row_index(
        memory_viewer_view_data: Dependency<Self>,
        normalized_region: &NormalizedRegion,
    ) -> Option<usize> {
        let mut memory_viewer_view_data = memory_viewer_view_data.write("Memory viewer take pending scroll row index")?;
        let pending_scroll_address = memory_viewer_view_data.pending_scroll_address?;

        if !normalized_region.contains_address(pending_scroll_address) {
            return None;
        }

        let row_index = pending_scroll_address
            .saturating_sub(normalized_region.get_base_address())
            .checked_div(Self::BYTES_PER_ROW)
            .and_then(|row_index| usize::try_from(row_index).ok())?;
        memory_viewer_view_data.pending_scroll_address = None;

        Some(row_index)
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
        let (has_pages, should_retry_refresh_after_failed_response) = match memory_viewer_view_data.write("Memory viewer apply memory query response") {
            Some(mut memory_viewer_view_data) => {
                if !memory_viewer_view_data.should_apply_memory_pages_request(request_revision) {
                    return;
                }

                memory_viewer_view_data.complete_memory_pages_request();

                if !memory_query_response.success {
                    (false, memory_viewer_view_data.consume_retry_memory_pages_refresh_on_failure(false))
                } else {
                    memory_viewer_view_data.virtual_pages = memory_query_response.virtual_pages;
                    memory_viewer_view_data.modules = memory_query_response.modules;
                    memory_viewer_view_data.page_caches_by_base_address.clear();
                    memory_viewer_view_data.unreadable_page_base_addresses.clear();
                    memory_viewer_view_data.consume_retry_memory_pages_refresh_on_failure(true);
                    memory_viewer_view_data.last_applied_snapshot_generation = 0;
                    memory_viewer_view_data.cached_last_page_index = memory_viewer_view_data.virtual_pages.len().saturating_sub(1) as u64;
                    if !memory_viewer_view_data.try_apply_pending_focus_request() {
                        memory_viewer_view_data.current_page_index =
                            Self::resolve_page_index_after_refresh(&memory_viewer_view_data.virtual_pages, selected_page_base_address).unwrap_or_else(|| {
                                Self::resolve_initial_page_index(&memory_viewer_view_data.virtual_pages, &memory_viewer_view_data.modules).unwrap_or(
                                    safe_clamp_ord(memory_viewer_view_data.current_page_index, 0, memory_viewer_view_data.cached_last_page_index),
                                )
                            });
                    }

                    let current_page = memory_viewer_view_data
                        .virtual_pages
                        .get(memory_viewer_view_data.current_page_index as usize)
                        .cloned();
                    memory_viewer_view_data.stats_string = Self::format_stats_for_page_from_modules(
                        &memory_viewer_view_data.modules,
                        &memory_viewer_view_data.unreadable_page_base_addresses,
                        current_page.as_ref(),
                    );

                    (!memory_viewer_view_data.virtual_pages.is_empty(), false)
                }
            }
            None => (false, false),
        };

        if should_retry_refresh_after_failed_response {
            Self::refresh_memory_pages(memory_viewer_view_data, engine_unprivileged_state);
        } else if !has_pages {
            engine_unprivileged_state.set_virtual_snapshot_queries(Self::WINDOW_VIRTUAL_SNAPSHOT_ID, Self::SNAPSHOT_REFRESH_INTERVAL, Vec::new());
        }
    }

    fn load_current_page_index(memory_viewer_view_data: &Guard<Arc<Self>>) -> u64 {
        safe_clamp_ord(memory_viewer_view_data.current_page_index, 0, memory_viewer_view_data.cached_last_page_index)
    }

    fn set_page_index(
        memory_viewer_view_data: Dependency<Self>,
        new_page_index: u64,
    ) {
        if let Some(mut memory_viewer_view_data) = memory_viewer_view_data.write("Memory viewer set page index") {
            let bounded_page_index = safe_clamp_ord(new_page_index, 0, memory_viewer_view_data.cached_last_page_index);

            if bounded_page_index == memory_viewer_view_data.current_page_index {
                return;
            }

            memory_viewer_view_data.current_page_index = bounded_page_index;
            memory_viewer_view_data.pending_scroll_address = None;
            memory_viewer_view_data.selected_byte_range = None;
            memory_viewer_view_data.is_drag_selection_active = false;
            memory_viewer_view_data.edit_state = None;
            memory_viewer_view_data.context_menu_address = None;
            memory_viewer_view_data.context_menu_position = None;

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
            if !self.virtual_pages.is_empty() {
                self.pending_focus_request = None;
            }

            return false;
        };

        self.current_page_index = safe_clamp_ord(page_index, 0, self.cached_last_page_index);
        self.pending_scroll_address = Some(resolved_address);
        self.set_selected_byte_range(
            resolved_address,
            resolved_address.saturating_add(pending_focus_request.selection_byte_count.saturating_sub(1)),
        );
        self.is_drag_selection_active = false;
        self.has_keyboard_focus = true;
        let active_edit_cursor_lane = self.resolve_active_edit_cursor_lane_or_default();
        self.set_edit_cursor(resolved_address, active_edit_cursor_lane);
        self.pending_focus_request = None;
        self.last_applied_snapshot_generation = 0;

        let current_page = self.virtual_pages.get(self.current_page_index as usize);
        self.stats_string = Self::format_stats_for_page_from_modules(&self.modules, &self.unreadable_page_base_addresses, current_page);

        true
    }

    fn seek_to_address_internal(
        &mut self,
        target_address: u64,
        selection_byte_count: u64,
    ) -> Option<u64> {
        let (page_index, resolved_address) = Self::resolve_nearest_page_index_and_address(&self.virtual_pages, target_address)?;
        self.current_page_index = safe_clamp_ord(page_index, 0, self.cached_last_page_index);
        self.pending_scroll_address = Some(resolved_address);
        self.set_selected_byte_range(resolved_address, resolved_address.saturating_add(selection_byte_count.saturating_sub(1)));
        self.is_drag_selection_active = false;
        self.has_keyboard_focus = true;
        let active_edit_cursor_lane = self.resolve_active_edit_cursor_lane_or_default();
        self.set_edit_cursor(resolved_address, active_edit_cursor_lane);
        self.last_applied_snapshot_generation = 0;
        let current_page = self.virtual_pages.get(self.current_page_index as usize);
        self.stats_string = Self::format_stats_for_page_from_modules(&self.modules, &self.unreadable_page_base_addresses, current_page);

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
            .and_then(|normalized_module| normalized_module.get_base_address().checked_add(address))
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

                let clamped_address = safe_clamp_ord(target_address, page_base_address, page_end_address.saturating_sub(1));

                Some((page_index as u64, clamped_address, clamped_address.abs_diff(target_address)))
            })
            .min_by_key(|(page_index, clamped_address, distance)| (*distance, *page_index, *clamped_address))
            .map(|(page_index, clamped_address, _)| (page_index, clamped_address))
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

    fn begin_selection_internal(
        &mut self,
        address: u64,
        extend_existing_selection: bool,
        edit_cursor_lane: MemoryViewerEditCursorLane,
    ) {
        let selection_anchor_address = if extend_existing_selection {
            self.selected_byte_range
                .as_ref()
                .map(|selected_byte_range| selected_byte_range.anchor_address)
                .unwrap_or(address)
        } else {
            address
        };

        self.set_selected_byte_range(selection_anchor_address, address);
        self.has_keyboard_focus = true;
        self.set_edit_cursor(address, edit_cursor_lane);
    }

    fn update_selection_internal(
        &mut self,
        address: u64,
    ) {
        if let Some(selection_anchor_address) = self
            .selected_byte_range
            .as_ref()
            .map(|selected_byte_range| selected_byte_range.anchor_address)
        {
            self.set_selected_byte_range(selection_anchor_address, address);
            let active_edit_cursor_lane = self.resolve_active_edit_cursor_lane_or_default();
            self.set_edit_cursor(address, active_edit_cursor_lane);
        }
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
            safe_clamp_ord(
                selection_address,
                current_page_address_range.start,
                current_page_address_range.end.saturating_sub(1),
            )
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

    fn resolve_active_edit_cursor_lane_or_default(&self) -> MemoryViewerEditCursorLane {
        self.edit_state
            .as_ref()
            .map(|edit_state| edit_state.active_lane)
            .unwrap_or(MemoryViewerEditCursorLane::Hexadecimal)
    }

    fn set_edit_cursor(
        &mut self,
        cursor_address: u64,
        edit_cursor_lane: MemoryViewerEditCursorLane,
    ) {
        self.edit_state = Some(MemoryViewerEditState {
            cursor_address,
            active_lane: edit_cursor_lane,
            active_nibble_index: 0,
            pending_high_nibble: None,
        });
    }

    fn handle_edit_backspace_internal(&mut self) {
        match self.resolve_active_edit_cursor_lane_or_default() {
            MemoryViewerEditCursorLane::Hexadecimal => self.handle_hex_edit_backspace_internal(),
            MemoryViewerEditCursorLane::String => self.handle_string_edit_backspace_internal(),
        }
    }

    fn handle_hex_edit_backspace_internal(&mut self) {
        let current_page_address_range = self.resolve_current_page_address_range();
        let Some(edit_state) = self.edit_state.as_mut() else {
            return;
        };

        if edit_state.pending_high_nibble.take().is_some() || edit_state.active_nibble_index == 1 {
            edit_state.active_nibble_index = 0;

            return;
        }

        let Some(current_page_address_range) = current_page_address_range else {
            self.edit_state = None;

            return;
        };
        let last_page_address = current_page_address_range.end.saturating_sub(1);

        if edit_state.cursor_address >= current_page_address_range.end {
            edit_state.cursor_address = last_page_address;
            edit_state.active_nibble_index = 0;

            return;
        }

        if edit_state.cursor_address > current_page_address_range.start {
            edit_state.cursor_address = edit_state.cursor_address.saturating_sub(1);
        }
    }

    fn handle_string_edit_backspace_internal(&mut self) {
        let current_page_address_range = self.resolve_current_page_address_range();
        let Some(edit_state) = self.edit_state.as_mut() else {
            return;
        };

        let Some(current_page_address_range) = current_page_address_range else {
            self.edit_state = None;

            return;
        };
        let last_page_address = current_page_address_range.end.saturating_sub(1);

        if edit_state.cursor_address >= current_page_address_range.end {
            edit_state.cursor_address = last_page_address;

            return;
        }

        if edit_state.cursor_address > current_page_address_range.start {
            edit_state.cursor_address = edit_state.cursor_address.saturating_sub(1);
        }
    }

    fn append_edit_text_internal(
        &mut self,
        typed_text: &str,
    ) -> Vec<(u64, Vec<u8>)> {
        match self.resolve_active_edit_cursor_lane_or_default() {
            MemoryViewerEditCursorLane::Hexadecimal => typed_text
                .chars()
                .filter_map(|typed_character| self.append_hex_edit_character_internal(typed_character.to_ascii_uppercase()))
                .collect(),
            MemoryViewerEditCursorLane::String => self
                .append_string_edit_text_internal(typed_text)
                .into_iter()
                .collect(),
        }
    }

    fn append_hex_edit_character_internal(
        &mut self,
        character: char,
    ) -> Option<(u64, Vec<u8>)> {
        let nibble_value = character.to_digit(16)? as u8;
        let current_page_address_range = self.resolve_current_page_address_range()?;

        if self.edit_state.is_none() {
            let selection_cursor_address = self
                .selected_byte_range
                .as_ref()
                .map(|selected_byte_range| selected_byte_range.active_address)
                .unwrap_or(current_page_address_range.start);
            self.set_edit_cursor(selection_cursor_address, MemoryViewerEditCursorLane::Hexadecimal);
        }

        let edit_state = self.edit_state.as_mut()?;

        if edit_state.cursor_address >= current_page_address_range.end {
            return None;
        }

        if edit_state.active_nibble_index == 0 {
            edit_state.pending_high_nibble = Some(nibble_value);
            edit_state.active_nibble_index = 1;

            return None;
        }

        let edited_byte = (edit_state.pending_high_nibble.take().unwrap_or(0) << 4) | nibble_value;
        let write_start_address = edit_state.cursor_address;
        let next_cursor_address = write_start_address.saturating_add(1);

        edit_state.active_nibble_index = 0;
        edit_state.cursor_address = if next_cursor_address < current_page_address_range.end {
            next_cursor_address
        } else {
            current_page_address_range.end
        };

        Some((write_start_address, vec![edited_byte]))
    }

    fn append_string_edit_text_internal(
        &mut self,
        typed_text: &str,
    ) -> Option<(u64, Vec<u8>)> {
        let current_page_address_range = self.resolve_current_page_address_range()?;

        if self.edit_state.is_none() {
            let selection_cursor_address = self
                .selected_byte_range
                .as_ref()
                .map(|selected_byte_range| selected_byte_range.active_address)
                .unwrap_or(current_page_address_range.start);
            self.set_edit_cursor(selection_cursor_address, MemoryViewerEditCursorLane::String);
        }

        let edit_state = self.edit_state.as_mut()?;

        if edit_state.cursor_address >= current_page_address_range.end {
            return None;
        }

        let available_byte_count = current_page_address_range
            .end
            .saturating_sub(edit_state.cursor_address) as usize;
        let written_bytes = typed_text
            .chars()
            .filter_map(Self::encode_string_edit_character)
            .take(available_byte_count)
            .collect::<Vec<_>>();

        if written_bytes.is_empty() {
            return None;
        }

        let write_start_address = edit_state.cursor_address;
        edit_state.cursor_address = edit_state
            .cursor_address
            .saturating_add(written_bytes.len() as u64);
        edit_state.active_nibble_index = 0;
        edit_state.pending_high_nibble = None;

        Some((write_start_address, written_bytes))
    }

    fn encode_string_edit_character(typed_character: char) -> Option<u8> {
        typed_character.is_ascii().then_some(typed_character as u8)
    }

    fn resolve_current_page_address_range(&self) -> Option<Range<u64>> {
        let current_page = self.virtual_pages.get(self.current_page_index as usize)?;
        let current_page_base_address = current_page.get_base_address();
        let current_page_end_address = current_page.get_end_address();

        (current_page_base_address < current_page_end_address).then_some(current_page_base_address..current_page_end_address)
    }

    fn build_selection_summary(&self) -> Option<MemoryViewerSelectionSummary> {
        let (selection_start_address, selection_end_address) = self.resolve_selected_address_bounds()?;
        let current_page = self.virtual_pages.get(self.current_page_index as usize)?;
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
            .edit_state
            .as_ref()
            .map(|edit_state| edit_state.cursor_address)
            .or_else(|| {
                self.selected_byte_range
                    .as_ref()
                    .map(|selected_byte_range| selected_byte_range.active_address)
            })
            .unwrap_or(current_page_address_range.start);
        let base_cursor_address = safe_clamp_ord(base_cursor_address, current_page_address_range.start, last_page_address);
        let target_cursor_address = if byte_delta >= 0 {
            base_cursor_address
                .saturating_add(byte_delta as u64)
                .min(last_page_address)
        } else {
            base_cursor_address
                .saturating_sub(byte_delta.unsigned_abs())
                .max(current_page_address_range.start)
        };

        if extend_selection {
            let selection_anchor_address = self
                .selected_byte_range
                .as_ref()
                .map(|selected_byte_range| selected_byte_range.anchor_address)
                .unwrap_or(base_cursor_address);
            self.set_selected_byte_range(selection_anchor_address, target_cursor_address);
        } else {
            self.set_selected_byte_range(target_cursor_address, target_cursor_address);
        }

        self.has_keyboard_focus = true;
        let active_edit_cursor_lane = self.resolve_active_edit_cursor_lane_or_default();
        self.set_edit_cursor(target_cursor_address, active_edit_cursor_lane);
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

    fn consume_retry_memory_pages_refresh_on_failure(
        &mut self,
        query_succeeded: bool,
    ) -> bool {
        if query_succeeded {
            self.retry_memory_pages_refresh_on_failure = false;

            return false;
        }

        let should_retry_memory_pages_refresh = self.retry_memory_pages_refresh_on_failure;
        self.retry_memory_pages_refresh_on_failure = false;

        should_retry_memory_pages_refresh
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
    use super::{MemoryViewerEditCursorLane, MemoryViewerEditState, MemoryViewerViewData};
    use squalr_engine_api::{
        dependency_injection::dependency_container::DependencyContainer,
        structures::{
            memory::{normalized_module::NormalizedModule, normalized_region::NormalizedRegion},
            projects::project_items::project_item_target::ProjectItemTarget,
        },
    };

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

    #[test]
    fn go_to_address_input_defaults_to_hexadecimal_format() {
        let memory_viewer_view_data = MemoryViewerViewData::new();

        assert_eq!(
            memory_viewer_view_data
                .go_to_address_input
                .get_anonymous_value_string_format(),
            squalr_engine_api::structures::data_values::anonymous_value_string_format::AnonymousValueStringFormat::Hexadecimal
        );
    }

    #[test]
    fn resolve_focus_address_uses_absolute_address_without_module() {
        let resolved_address = MemoryViewerViewData::resolve_focus_address(&[], 0x1234, "");

        assert_eq!(resolved_address, Some(0x1234));
    }

    #[test]
    fn resolve_focus_address_adds_module_base_for_module_relative_addresses() {
        let modules = vec![NormalizedModule::new("winmine.exe", 0x4000, 0x1000)];

        let resolved_address = MemoryViewerViewData::resolve_focus_address(&modules, 0x120, "winmine.exe");

        assert_eq!(resolved_address, Some(0x4120));
    }

    #[test]
    fn try_apply_pending_focus_request_selects_requested_byte_range() {
        let mut memory_viewer_view_data = MemoryViewerViewData::new();
        memory_viewer_view_data.virtual_pages = vec![NormalizedRegion::new(0x4000, 0x100)];
        memory_viewer_view_data.cached_last_page_index = 0;
        memory_viewer_view_data.pending_focus_request = Some(super::MemoryViewerFocusRequest {
            address: 0x4010,
            module_name: String::new(),
            selection_byte_count: 4,
        });

        assert!(memory_viewer_view_data.try_apply_pending_focus_request());
        assert_eq!(memory_viewer_view_data.resolve_selected_address_bounds(), Some((0x4010, 0x4013)));
        assert_eq!(
            memory_viewer_view_data.edit_state,
            Some(MemoryViewerEditState {
                cursor_address: 0x4010,
                active_lane: MemoryViewerEditCursorLane::Hexadecimal,
                active_nibble_index: 0,
                pending_high_nibble: None,
            })
        );
    }

    #[test]
    fn seek_to_input_address_clamps_to_nearest_page_when_target_is_missing() {
        let dependency_container = DependencyContainer::new();
        let memory_viewer_view_data = dependency_container.register(MemoryViewerViewData::new());

        if let Some(mut memory_viewer_view_data_guard) = memory_viewer_view_data.write("Test memory viewer seek input") {
            memory_viewer_view_data_guard.virtual_pages = vec![
                NormalizedRegion::new(0x1000, 0x100),
                NormalizedRegion::new(0x4000, 0x100),
            ];
            memory_viewer_view_data_guard.cached_last_page_index = 1;
            memory_viewer_view_data_guard
                .go_to_address_input
                .set_anonymous_value_string(String::from("0x3800"));
        }

        MemoryViewerViewData::seek_to_input_address(memory_viewer_view_data.clone());

        let memory_viewer_view_data_guard = memory_viewer_view_data
            .read("Test memory viewer seek state")
            .expect("Expected memory viewer state.");
        assert_eq!(memory_viewer_view_data_guard.current_page_index, 1);
        assert_eq!(memory_viewer_view_data_guard.resolve_selected_address_bounds(), Some((0x4000, 0x4000)));
        assert_eq!(memory_viewer_view_data_guard.pending_scroll_address, Some(0x4000));
    }

    #[test]
    fn clear_selection_removes_selected_byte_range() {
        let dependency_container = DependencyContainer::new();
        let memory_viewer_view_data = dependency_container.register(MemoryViewerViewData::new());

        if let Some(mut memory_viewer_view_data_guard) = memory_viewer_view_data.write("Test memory viewer clear selection setup") {
            memory_viewer_view_data_guard.virtual_pages = vec![NormalizedRegion::new(0x5000, 0x20)];
            memory_viewer_view_data_guard.cached_last_page_index = 0;
            memory_viewer_view_data_guard.begin_selection_internal(0x5004, false, MemoryViewerEditCursorLane::Hexadecimal);
        }

        MemoryViewerViewData::clear_selection(memory_viewer_view_data.clone());

        assert_eq!(MemoryViewerViewData::get_selected_address_bounds(memory_viewer_view_data.clone()), None);
    }

    #[test]
    fn format_project_item_name_uses_module_relative_text_when_module_exists() {
        let project_item_name = MemoryViewerViewData::format_project_item_name(0x22, "winmine.exe");

        assert_eq!(project_item_name, String::from("winmine.exe+0x22"));
    }

    #[test]
    fn append_hex_edit_character_internal_advances_past_original_selection() {
        let mut memory_viewer_view_data = MemoryViewerViewData::new();
        memory_viewer_view_data.virtual_pages = vec![NormalizedRegion::new(0x1000, 3)];
        memory_viewer_view_data.cached_last_page_index = 0;
        memory_viewer_view_data.begin_selection_internal(0x1001, false, MemoryViewerEditCursorLane::Hexadecimal);

        assert_eq!(memory_viewer_view_data.append_hex_edit_character_internal('A'), None);
        assert_eq!(memory_viewer_view_data.append_hex_edit_character_internal('1'), Some((0x1001, vec![0xA1])));
        assert_eq!(
            memory_viewer_view_data.edit_state,
            Some(MemoryViewerEditState {
                cursor_address: 0x1002,
                active_lane: MemoryViewerEditCursorLane::Hexadecimal,
                active_nibble_index: 0,
                pending_high_nibble: None,
            })
        );

        assert_eq!(memory_viewer_view_data.append_hex_edit_character_internal('B'), None);
        assert_eq!(memory_viewer_view_data.append_hex_edit_character_internal('2'), Some((0x1002, vec![0xB2])));
        assert_eq!(
            memory_viewer_view_data.edit_state,
            Some(MemoryViewerEditState {
                cursor_address: 0x1003,
                active_lane: MemoryViewerEditCursorLane::Hexadecimal,
                active_nibble_index: 0,
                pending_high_nibble: None,
            })
        );
        assert_eq!(memory_viewer_view_data.append_hex_edit_character_internal('C'), None);
    }

    #[test]
    fn append_edit_text_internal_in_string_lane_writes_ascii_bytes_and_advances_cursor() {
        let mut memory_viewer_view_data = MemoryViewerViewData::new();
        memory_viewer_view_data.virtual_pages = vec![NormalizedRegion::new(0x1100, 8)];
        memory_viewer_view_data.cached_last_page_index = 0;
        memory_viewer_view_data.begin_selection_internal(0x1101, false, MemoryViewerEditCursorLane::String);

        assert_eq!(memory_viewer_view_data.append_edit_text_internal("Hi"), vec![(0x1101, vec![b'H', b'i'])]);
        assert_eq!(
            memory_viewer_view_data.edit_state,
            Some(MemoryViewerEditState {
                cursor_address: 0x1103,
                active_lane: MemoryViewerEditCursorLane::String,
                active_nibble_index: 0,
                pending_high_nibble: None,
            })
        );
    }

    #[test]
    fn consume_retry_memory_pages_refresh_on_failure_retries_only_once() {
        let mut memory_viewer_view_data = MemoryViewerViewData::new();
        memory_viewer_view_data.retry_memory_pages_refresh_on_failure = true;

        assert!(memory_viewer_view_data.consume_retry_memory_pages_refresh_on_failure(false));
        assert!(!memory_viewer_view_data.retry_memory_pages_refresh_on_failure);
        assert!(!memory_viewer_view_data.consume_retry_memory_pages_refresh_on_failure(false));
    }

    #[test]
    fn consume_retry_memory_pages_refresh_on_failure_clears_retry_after_success() {
        let mut memory_viewer_view_data = MemoryViewerViewData::new();
        memory_viewer_view_data.retry_memory_pages_refresh_on_failure = true;

        assert!(!memory_viewer_view_data.consume_retry_memory_pages_refresh_on_failure(true));
        assert!(!memory_viewer_view_data.retry_memory_pages_refresh_on_failure);
    }

    #[test]
    fn handle_hex_edit_backspace_internal_clears_pending_nibble_before_moving_cursor() {
        let mut memory_viewer_view_data = MemoryViewerViewData::new();
        memory_viewer_view_data.virtual_pages = vec![NormalizedRegion::new(0x2000, 4)];
        memory_viewer_view_data.cached_last_page_index = 0;
        memory_viewer_view_data.begin_selection_internal(0x2001, false, MemoryViewerEditCursorLane::Hexadecimal);

        assert_eq!(memory_viewer_view_data.append_hex_edit_character_internal('F'), None);

        memory_viewer_view_data.handle_hex_edit_backspace_internal();

        assert_eq!(
            memory_viewer_view_data.edit_state,
            Some(MemoryViewerEditState {
                cursor_address: 0x2001,
                active_lane: MemoryViewerEditCursorLane::Hexadecimal,
                active_nibble_index: 0,
                pending_high_nibble: None,
            })
        );
    }

    #[test]
    fn move_cursor_internal_moves_horizontally_within_page_bounds() {
        let mut memory_viewer_view_data = MemoryViewerViewData::new();
        memory_viewer_view_data.virtual_pages = vec![NormalizedRegion::new(0x3000, 4)];
        memory_viewer_view_data.cached_last_page_index = 0;
        memory_viewer_view_data.begin_selection_internal(0x3001, false, MemoryViewerEditCursorLane::Hexadecimal);

        memory_viewer_view_data.move_cursor_internal(1, false);

        assert_eq!(memory_viewer_view_data.resolve_selected_address_bounds(), Some((0x3002, 0x3002)));
        assert_eq!(
            memory_viewer_view_data.edit_state,
            Some(MemoryViewerEditState {
                cursor_address: 0x3002,
                active_lane: MemoryViewerEditCursorLane::Hexadecimal,
                active_nibble_index: 0,
                pending_high_nibble: None,
            })
        );
    }

    #[test]
    fn move_cursor_internal_extends_selection_from_anchor() {
        let mut memory_viewer_view_data = MemoryViewerViewData::new();
        memory_viewer_view_data.virtual_pages = vec![NormalizedRegion::new(0x4000, 0x20)];
        memory_viewer_view_data.cached_last_page_index = 0;
        memory_viewer_view_data.begin_selection_internal(0x4002, false, MemoryViewerEditCursorLane::Hexadecimal);

        memory_viewer_view_data.move_cursor_internal(MemoryViewerViewData::BYTES_PER_ROW as i64, true);

        assert_eq!(memory_viewer_view_data.resolve_selected_address_bounds(), Some((0x4002, 0x4012)));
        assert_eq!(
            memory_viewer_view_data.edit_state,
            Some(MemoryViewerEditState {
                cursor_address: 0x4012,
                active_lane: MemoryViewerEditCursorLane::Hexadecimal,
                active_nibble_index: 0,
                pending_high_nibble: None,
            })
        );
    }

    #[test]
    fn move_cursor_internal_preserves_string_edit_lane() {
        let mut memory_viewer_view_data = MemoryViewerViewData::new();
        memory_viewer_view_data.virtual_pages = vec![NormalizedRegion::new(0x4100, 0x20)];
        memory_viewer_view_data.cached_last_page_index = 0;
        memory_viewer_view_data.begin_selection_internal(0x4102, false, MemoryViewerEditCursorLane::String);

        memory_viewer_view_data.move_cursor_internal(1, false);

        assert_eq!(
            memory_viewer_view_data.edit_state,
            Some(MemoryViewerEditState {
                cursor_address: 0x4103,
                active_lane: MemoryViewerEditCursorLane::String,
                active_nibble_index: 0,
                pending_high_nibble: None,
            })
        );
    }

    #[test]
    fn update_selection_internal_clamps_selection_size_to_two_megabytes() {
        let mut memory_viewer_view_data = MemoryViewerViewData::new();
        memory_viewer_view_data.virtual_pages = vec![NormalizedRegion::new(
            0x9000,
            MemoryViewerViewData::MAX_SELECTION_SIZE_IN_BYTES + 0x4000,
        )];
        memory_viewer_view_data.cached_last_page_index = 0;
        memory_viewer_view_data.begin_selection_internal(0x9000, false, MemoryViewerEditCursorLane::Hexadecimal);

        memory_viewer_view_data.update_selection_internal(0x9000 + MemoryViewerViewData::MAX_SELECTION_SIZE_IN_BYTES + 0x1234);

        assert_eq!(
            memory_viewer_view_data.resolve_selected_address_bounds(),
            Some((0x9000, 0x9000 + MemoryViewerViewData::MAX_SELECTION_SIZE_IN_BYTES - 1))
        );
    }

    #[test]
    fn build_address_project_item_create_request_uses_u8_array_type_for_multi_byte_selection() {
        let dependency_container = DependencyContainer::new();
        let memory_viewer_view_data = dependency_container.register(MemoryViewerViewData::new());

        if let Some(mut memory_viewer_view_data_guard) = memory_viewer_view_data.write("Test selection create request") {
            memory_viewer_view_data_guard.virtual_pages = vec![NormalizedRegion::new(0x5000, 0x20)];
            memory_viewer_view_data_guard.cached_last_page_index = 0;
            memory_viewer_view_data_guard.begin_selection_internal(0x5004, false, MemoryViewerEditCursorLane::Hexadecimal);
            memory_viewer_view_data_guard.update_selection_internal(0x5006);
        }

        let create_request =
            MemoryViewerViewData::build_address_project_item_create_request(memory_viewer_view_data.clone(), 0x5005, None).expect("Expected create request.");

        assert!(matches!(
            create_request.target,
            ProjectItemTarget::Address {
                address: 0x5004,
                module_name: _
            }
        ));
        assert_eq!(create_request.data_type_id, Some(String::from("u8[3]")));
    }

    #[test]
    fn build_address_project_item_create_request_uses_u8_for_single_byte_selection() {
        let dependency_container = DependencyContainer::new();
        let memory_viewer_view_data = dependency_container.register(MemoryViewerViewData::new());

        if let Some(mut memory_viewer_view_data_guard) = memory_viewer_view_data.write("Test single-byte create request") {
            memory_viewer_view_data_guard.virtual_pages = vec![NormalizedRegion::new(0x6000, 0x20)];
            memory_viewer_view_data_guard.cached_last_page_index = 0;
            memory_viewer_view_data_guard.begin_selection_internal(0x6004, false, MemoryViewerEditCursorLane::Hexadecimal);
        }

        let create_request =
            MemoryViewerViewData::build_address_project_item_create_request(memory_viewer_view_data.clone(), 0x6004, None).expect("Expected create request.");

        assert!(matches!(
            create_request.target,
            ProjectItemTarget::Address {
                address: 0x6004,
                module_name: _
            }
        ));
        assert_eq!(create_request.data_type_id, Some(String::from("u8")));
    }

    #[test]
    fn build_address_project_item_create_request_uses_explicit_data_type_override() {
        let dependency_container = DependencyContainer::new();
        let memory_viewer_view_data = dependency_container.register(MemoryViewerViewData::new());

        if let Some(mut memory_viewer_view_data_guard) = memory_viewer_view_data.write("Test explicit type create request") {
            memory_viewer_view_data_guard.virtual_pages = vec![NormalizedRegion::new(0x7000, 0x20)];
            memory_viewer_view_data_guard.cached_last_page_index = 0;
            memory_viewer_view_data_guard.begin_selection_internal(0x7004, false, MemoryViewerEditCursorLane::Hexadecimal);
            memory_viewer_view_data_guard.update_selection_internal(0x7007);
        }

        let create_request = MemoryViewerViewData::build_address_project_item_create_request_with_data_type(
            memory_viewer_view_data.clone(),
            0x7005,
            None,
            Some(String::from("u16[2]")),
        )
        .expect("Expected create request.");

        assert!(matches!(
            create_request.target,
            ProjectItemTarget::Address {
                address: 0x7004,
                module_name: _
            }
        ));
        assert_eq!(create_request.data_type_id, Some(String::from("u16[2]")));
    }

    #[test]
    fn select_all_bytes_on_current_page_selects_entire_page_range() {
        let dependency_container = DependencyContainer::new();
        let memory_viewer_view_data = dependency_container.register(MemoryViewerViewData::new());

        if let Some(mut memory_viewer_view_data_guard) = memory_viewer_view_data.write("Test select all setup") {
            memory_viewer_view_data_guard.virtual_pages = vec![NormalizedRegion::new(0x8000, 0x20)];
            memory_viewer_view_data_guard.cached_last_page_index = 0;
        }

        MemoryViewerViewData::select_all_bytes_on_current_page(memory_viewer_view_data.clone());

        let selected_address_bounds = MemoryViewerViewData::get_selected_address_bounds(memory_viewer_view_data.clone());
        let edit_state = memory_viewer_view_data
            .read("Test select all cursor")
            .and_then(|memory_viewer_view_data_guard| memory_viewer_view_data_guard.edit_state.clone());

        assert_eq!(selected_address_bounds, Some((0x8000, 0x801F)));
        assert_eq!(
            edit_state,
            Some(MemoryViewerEditState {
                cursor_address: 0x8000,
                active_lane: MemoryViewerEditCursorLane::Hexadecimal,
                active_nibble_index: 0,
                pending_high_nibble: None,
            })
        );
    }

    #[test]
    fn select_all_bytes_on_current_page_preserves_active_string_edit_lane() {
        let dependency_container = DependencyContainer::new();
        let memory_viewer_view_data = dependency_container.register(MemoryViewerViewData::new());

        if let Some(mut memory_viewer_view_data_guard) = memory_viewer_view_data.write("Test select all string lane setup") {
            memory_viewer_view_data_guard.virtual_pages = vec![NormalizedRegion::new(0x8800, 0x20)];
            memory_viewer_view_data_guard.cached_last_page_index = 0;
            memory_viewer_view_data_guard.begin_selection_internal(0x8804, false, MemoryViewerEditCursorLane::String);
        }

        MemoryViewerViewData::select_all_bytes_on_current_page(memory_viewer_view_data.clone());

        let edit_state = memory_viewer_view_data
            .read("Test select all string lane cursor")
            .and_then(|memory_viewer_view_data_guard| memory_viewer_view_data_guard.edit_state.clone());

        assert_eq!(
            edit_state,
            Some(MemoryViewerEditState {
                cursor_address: 0x8800,
                active_lane: MemoryViewerEditCursorLane::String,
                active_nibble_index: 0,
                pending_high_nibble: None,
            })
        );
    }

    #[test]
    fn select_all_bytes_on_current_page_clamps_selection_size_to_two_megabytes() {
        let dependency_container = DependencyContainer::new();
        let memory_viewer_view_data = dependency_container.register(MemoryViewerViewData::new());

        if let Some(mut memory_viewer_view_data_guard) = memory_viewer_view_data.write("Test select all clamp setup") {
            memory_viewer_view_data_guard.virtual_pages = vec![NormalizedRegion::new(
                0xA000,
                MemoryViewerViewData::MAX_SELECTION_SIZE_IN_BYTES + 0x8000,
            )];
            memory_viewer_view_data_guard.cached_last_page_index = 0;
        }

        MemoryViewerViewData::select_all_bytes_on_current_page(memory_viewer_view_data.clone());

        assert_eq!(
            MemoryViewerViewData::get_selected_address_bounds(memory_viewer_view_data),
            Some((0xA000, 0xA000 + MemoryViewerViewData::MAX_SELECTION_SIZE_IN_BYTES - 1))
        );
    }
}
