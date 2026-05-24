use crate::state::pane_entry_row::PaneEntryRow;
use crate::views::code_viewer::summary::build_code_viewer_summary_lines;
use crate::views::entry_row_viewport::build_selection_relative_viewport_range;
use squalr_engine_api::{
    conversions::storage_size_conversions::StorageSizeConversions,
    plugins::memory_view::PageRetrievalMode,
    structures::{
        data_types::{built_in_types::u8::data_type_u8::DataTypeU8, data_type_ref::DataTypeRef},
        data_values::container_type::ContainerType,
        memory::{bitness::Bitness, normalized_module::NormalizedModule, normalized_region::NormalizedRegion},
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
    time::Duration,
};

#[derive(Clone, Debug, Default)]
struct CodeViewerPageCache {
    cached_chunks: BTreeMap<u64, Vec<u8>>,
}

impl CodeViewerPageCache {
    fn cache_chunk(
        &mut self,
        chunk_offset: u64,
        bytes: Vec<u8>,
    ) {
        self.cached_chunks.insert(chunk_offset, bytes);
    }

    fn collect_contiguous_bytes(
        &self,
        start_byte_offset: u64,
        end_byte_offset_exclusive: u64,
    ) -> Vec<u8> {
        let mut collected_bytes = Vec::new();

        for byte_offset in start_byte_offset..end_byte_offset_exclusive {
            let chunk_offset = byte_offset - (byte_offset % CodeViewerPaneState::QUERY_CHUNK_SIZE_IN_BYTES);
            let Some(chunk_bytes) = self.cached_chunks.get(&chunk_offset) else {
                break;
            };
            let chunk_local_index = byte_offset.saturating_sub(chunk_offset) as usize;
            let Some(byte_value) = chunk_bytes.get(chunk_local_index).copied() else {
                break;
            };

            collected_bytes.push(byte_value);
        }

        collected_bytes
    }
}

#[derive(Clone, Debug)]
pub struct CodeViewerPaneState {
    pub virtual_pages: Vec<NormalizedRegion>,
    pub modules: Vec<NormalizedModule>,
    pub current_page_index: u64,
    pub cached_last_page_index: u64,
    pub last_visible_row_capacity: usize,
    pub page_retrieval_mode: PageRetrievalMode,
    pub stats_string: String,
    pub status_message: String,
    pub is_querying_memory_pages: bool,
    pub has_loaded_memory_pages_once: bool,
    last_applied_snapshot_generation: u64,
    page_caches_by_base_address: HashMap<u64, CodeViewerPageCache>,
    unreadable_page_base_addresses: HashSet<u64>,
    viewport_start_address: Option<u64>,
    selected_instruction_address: Option<u64>,
}

impl CodeViewerPaneState {
    pub const VIRTUAL_SNAPSHOT_ID: &'static str = "tui_code_viewer";
    pub const QUERY_CHUNK_SIZE_IN_BYTES: u64 = 256;
    pub const QUERY_PREFETCH_CHUNK_COUNT: u64 = 1;
    pub const CODE_WINDOW_SIZE_IN_BYTES: u64 = 0x1000;
    pub const DECODE_BACKTRACK_BYTES: u64 = 0x80;
    pub const SNAPSHOT_REFRESH_INTERVAL: Duration = Duration::from_millis(500);

    pub fn summary_lines(
        &self,
        process_bitness: Option<Bitness>,
    ) -> Vec<String> {
        build_code_viewer_summary_lines(self, process_bitness)
    }

    pub fn set_viewport_row_capacity(
        &mut self,
        viewport_row_capacity: usize,
    ) {
        self.last_visible_row_capacity = viewport_row_capacity;
    }

    pub fn current_page_base_address(&self) -> Option<u64> {
        self.current_page().map(NormalizedRegion::get_base_address)
    }

    pub fn selected_instruction_address(&self) -> Option<u64> {
        self.selected_instruction_address
    }

    pub fn viewport_range_label(&self) -> String {
        let Some(current_page) = self.current_page() else {
            return String::from("none");
        };
        let viewport_window = self.resolve_viewport_window_for_current_page(current_page);

        format!("0x{:X}-0x{:X}", viewport_window.start, viewport_window.end)
    }

    pub fn current_instruction_count(
        &self,
        process_bitness: Option<Bitness>,
    ) -> usize {
        self.current_instruction_lines(process_bitness).len()
    }

    pub fn visible_row_entries(
        &self,
        process_bitness: Option<Bitness>,
    ) -> Vec<PaneEntryRow> {
        let instruction_lines = self.current_instruction_lines(process_bitness);
        if instruction_lines.is_empty() {
            return Vec::new();
        }

        let selected_instruction_index = self
            .resolve_selected_instruction_index(&instruction_lines)
            .unwrap_or(0);
        let visible_instruction_range =
            build_selection_relative_viewport_range(instruction_lines.len(), Some(selected_instruction_index), self.last_visible_row_capacity.max(1));
        let mut entry_rows = Vec::with_capacity(visible_instruction_range.len());

        for instruction_index in visible_instruction_range {
            let Some(disassembled_instruction) = instruction_lines.get(instruction_index) else {
                continue;
            };

            let marker_text = if instruction_index == selected_instruction_index {
                ">".to_string()
            } else {
                String::new()
            };
            let instruction_bytes = disassembled_instruction
                .bytes
                .iter()
                .map(|instruction_byte| format!("{:02X}", instruction_byte))
                .collect::<Vec<_>>()
                .join(" ");
            let primary_text = format!(
                "0x{:016X}  {:<24}  {}",
                disassembled_instruction.address, instruction_bytes, disassembled_instruction.text
            );
            let secondary_text = Some(if let Some(branch_target_address) = disassembled_instruction.branch_target_address {
                format!("branch=0x{:X} | len={}", branch_target_address, disassembled_instruction.length)
            } else {
                format!("len={}", disassembled_instruction.length)
            });

            if instruction_index == selected_instruction_index {
                entry_rows.push(PaneEntryRow::selected(marker_text, primary_text, secondary_text));
            } else {
                entry_rows.push(PaneEntryRow::normal(marker_text, primary_text, secondary_text));
            }
        }

        entry_rows
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
        self.viewport_start_address = self.current_page().map(NormalizedRegion::get_base_address);
        self.selected_instruction_address = self.viewport_start_address;
        self.has_loaded_memory_pages_once = !self.virtual_pages.is_empty();
        self.stats_string = self.format_stats_for_current_page();
    }

    pub fn clear_for_process_change(
        &mut self,
        engine_unprivileged_state: &Arc<EngineUnprivilegedState>,
    ) {
        self.virtual_pages.clear();
        self.modules.clear();
        self.current_page_index = 0;
        self.cached_last_page_index = 0;
        self.last_visible_row_capacity = 0;
        self.stats_string = String::from("No page selected");
        self.status_message = String::from("Process changed. Code viewer cleared");
        self.is_querying_memory_pages = false;
        self.has_loaded_memory_pages_once = false;
        self.last_applied_snapshot_generation = 0;
        self.page_caches_by_base_address.clear();
        self.unreadable_page_base_addresses.clear();
        self.viewport_start_address = None;
        self.selected_instruction_address = None;
        engine_unprivileged_state.set_virtual_snapshot_queries(Self::VIRTUAL_SNAPSHOT_ID, Self::SNAPSHOT_REFRESH_INTERVAL, Vec::new());
    }

    pub fn build_visible_chunk_queries(&self) -> Vec<VirtualSnapshotQuery> {
        let Some(current_page) = self.current_page() else {
            return Vec::new();
        };
        let viewport_start_address = self.resolve_viewport_start_address(current_page);
        let viewport_window = Self::resolve_viewport_window_for_address(current_page, viewport_start_address);
        let query_start_offset = viewport_window
            .start
            .saturating_sub(current_page.get_base_address())
            .saturating_sub(Self::DECODE_BACKTRACK_BYTES);
        let query_end_offset_exclusive = viewport_window
            .end
            .saturating_sub(current_page.get_base_address())
            .min(current_page.get_region_size());
        let first_visible_chunk_index = query_start_offset / Self::QUERY_CHUNK_SIZE_IN_BYTES;
        let last_visible_chunk_index = query_end_offset_exclusive
            .saturating_sub(1)
            .checked_div(Self::QUERY_CHUNK_SIZE_IN_BYTES)
            .unwrap_or(0);
        let first_prefetched_chunk_index = first_visible_chunk_index.saturating_sub(Self::QUERY_PREFETCH_CHUNK_COUNT);
        let max_chunk_index = current_page
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
                let chunk_length = current_page
                    .get_region_size()
                    .saturating_sub(chunk_offset)
                    .min(Self::QUERY_CHUNK_SIZE_IN_BYTES);

                (chunk_length > 0).then_some(VirtualSnapshotQuery::Address {
                    query_id: Self::build_chunk_query_id(current_page.get_base_address(), chunk_offset),
                    address: current_page.get_base_address().saturating_add(chunk_offset),
                    module_name: String::new(),
                    symbolic_struct_definition: Self::build_chunk_symbolic_struct_definition(chunk_length),
                })
            })
            .collect()
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
        self.stats_string = self.format_stats_for_current_page();
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

    pub fn select_previous_instruction(
        &mut self,
        process_bitness: Option<Bitness>,
    ) {
        let instruction_lines = self.current_instruction_lines(process_bitness);
        let Some(selected_instruction_index) = self.resolve_selected_instruction_index(&instruction_lines) else {
            return;
        };
        let previous_instruction_index = selected_instruction_index.saturating_sub(1);
        self.selected_instruction_address = instruction_lines
            .get(previous_instruction_index)
            .map(|disassembled_instruction| disassembled_instruction.address);
    }

    pub fn select_next_instruction(
        &mut self,
        process_bitness: Option<Bitness>,
    ) {
        let instruction_lines = self.current_instruction_lines(process_bitness);
        let Some(selected_instruction_index) = self.resolve_selected_instruction_index(&instruction_lines) else {
            return;
        };
        let next_instruction_index = selected_instruction_index
            .saturating_add(1)
            .min(instruction_lines.len().saturating_sub(1));
        self.selected_instruction_address = instruction_lines
            .get(next_instruction_index)
            .map(|disassembled_instruction| disassembled_instruction.address);
    }

    pub fn page_up_instructions(
        &mut self,
        process_bitness: Option<Bitness>,
    ) {
        let instruction_lines = self.current_instruction_lines(process_bitness);
        let Some(selected_instruction_index) = self.resolve_selected_instruction_index(&instruction_lines) else {
            return;
        };
        let page_jump = self.last_visible_row_capacity.max(1);
        let target_instruction_index = selected_instruction_index.saturating_sub(page_jump);
        self.selected_instruction_address = instruction_lines
            .get(target_instruction_index)
            .map(|disassembled_instruction| disassembled_instruction.address);
    }

    pub fn page_down_instructions(
        &mut self,
        process_bitness: Option<Bitness>,
    ) {
        let instruction_lines = self.current_instruction_lines(process_bitness);
        let Some(selected_instruction_index) = self.resolve_selected_instruction_index(&instruction_lines) else {
            return;
        };
        let page_jump = self.last_visible_row_capacity.max(1);
        let target_instruction_index = selected_instruction_index
            .saturating_add(page_jump)
            .min(instruction_lines.len().saturating_sub(1));
        self.selected_instruction_address = instruction_lines
            .get(target_instruction_index)
            .map(|disassembled_instruction| disassembled_instruction.address);
    }

    pub fn focus_address(
        &mut self,
        address: u64,
        module_name: &str,
    ) -> bool {
        let resolved_address = Self::resolve_focus_address(&self.modules, address, module_name);
        let Some(resolved_address) = resolved_address else {
            return false;
        };
        let Some(page_index) = Self::resolve_page_index_after_refresh(&self.virtual_pages, Some(resolved_address)) else {
            return false;
        };
        let Some(current_page) = self.virtual_pages.get(page_index as usize) else {
            return false;
        };

        self.current_page_index = page_index;
        self.viewport_start_address = Some(Self::derive_viewport_start_for_focus_address(current_page, resolved_address));
        self.selected_instruction_address = Some(resolved_address);
        self.stats_string = self.format_stats_for_current_page();

        true
    }

    fn current_page(&self) -> Option<&NormalizedRegion> {
        self.virtual_pages.get(self.current_page_index as usize)
    }

    fn current_instruction_lines(
        &self,
        process_bitness: Option<Bitness>,
    ) -> Vec<DisassembledInstruction> {
        let Some(current_page) = self.current_page() else {
            return Vec::new();
        };
        let viewport_start_address = self.resolve_viewport_start_address(current_page);
        let viewport_window = Self::resolve_viewport_window_for_address(current_page, viewport_start_address);
        let decode_start_address = viewport_window
            .start
            .saturating_sub(Self::DECODE_BACKTRACK_BYTES)
            .max(current_page.get_base_address());
        let decode_end_address_exclusive = viewport_window.end.min(current_page.get_end_address());
        let page_base_address = current_page.get_base_address();
        let decode_start_offset = decode_start_address.saturating_sub(page_base_address);
        let decode_end_offset_exclusive = decode_end_address_exclusive.saturating_sub(page_base_address);

        let cached_bytes = self
            .page_caches_by_base_address
            .get(&page_base_address)
            .map(|code_viewer_page_cache| code_viewer_page_cache.collect_contiguous_bytes(decode_start_offset, decode_end_offset_exclusive))
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

    fn resolve_selected_instruction_index(
        &self,
        instruction_lines: &[DisassembledInstruction],
    ) -> Option<usize> {
        if instruction_lines.is_empty() {
            return None;
        }

        let selected_instruction_address = self
            .selected_instruction_address
            .unwrap_or_else(|| instruction_lines[0].address);

        instruction_lines
            .iter()
            .position(|disassembled_instruction| disassembled_instruction.address == selected_instruction_address)
            .or_else(|| {
                instruction_lines
                    .iter()
                    .position(|disassembled_instruction| disassembled_instruction.address >= selected_instruction_address)
            })
            .or_else(|| instruction_lines.len().checked_sub(1))
    }

    fn resolve_viewport_start_address(
        &self,
        current_page: &NormalizedRegion,
    ) -> u64 {
        self.viewport_start_address
            .unwrap_or_else(|| current_page.get_base_address())
            .clamp(current_page.get_base_address(), Self::max_viewport_start_address(current_page))
    }

    fn resolve_viewport_window_for_current_page(
        &self,
        current_page: &NormalizedRegion,
    ) -> Range<u64> {
        let viewport_start_address = self.resolve_viewport_start_address(current_page);

        Self::resolve_viewport_window_for_address(current_page, viewport_start_address)
    }

    fn set_page_index(
        &mut self,
        page_index: u64,
    ) {
        self.current_page_index = page_index.clamp(0, self.cached_last_page_index);
        self.viewport_start_address = self.current_page().map(NormalizedRegion::get_base_address);
        self.selected_instruction_address = self.viewport_start_address;
        self.last_applied_snapshot_generation = 0;
        self.stats_string = self.format_stats_for_current_page();
    }

    fn format_stats_for_current_page(&self) -> String {
        let Some(current_page) = self.current_page() else {
            return String::from("No page selected");
        };
        let module_label = self
            .modules
            .iter()
            .find(|normalized_module| normalized_module.contains_address(current_page.get_base_address()))
            .map(|normalized_module| normalized_module.get_module_name().to_string())
            .unwrap_or_else(|| String::from("(No Module)"));
        let viewport_window = self.resolve_viewport_window_for_current_page(current_page);
        let page_size_label = StorageSizeConversions::value_to_metric_size(current_page.get_region_size() as u128);

        format!(
            "Base 0x{:X} | Size {} | View 0x{:X}-0x{:X} | {}{}",
            current_page.get_base_address(),
            page_size_label,
            viewport_window.start,
            viewport_window.end,
            module_label,
            if self
                .unreadable_page_base_addresses
                .contains(&current_page.get_base_address())
            {
                " | Unreadable"
            } else {
                ""
            }
        )
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

    fn derive_viewport_start_for_focus_address(
        normalized_region: &NormalizedRegion,
        focus_address: u64,
    ) -> u64 {
        let desired_start_address = focus_address.saturating_sub(Self::CODE_WINDOW_SIZE_IN_BYTES / 4);

        desired_start_address.clamp(normalized_region.get_base_address(), Self::max_viewport_start_address(normalized_region))
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
        let clamped_viewport_start_address =
            viewport_start_address.clamp(normalized_region.get_base_address(), Self::max_viewport_start_address(normalized_region));
        let viewport_end_address = clamped_viewport_start_address
            .saturating_add(Self::CODE_WINDOW_SIZE_IN_BYTES)
            .min(normalized_region.get_end_address());

        clamped_viewport_start_address..viewport_end_address
    }
}

impl Default for CodeViewerPaneState {
    fn default() -> Self {
        Self {
            virtual_pages: Vec::new(),
            modules: Vec::new(),
            current_page_index: 0,
            cached_last_page_index: 0,
            last_visible_row_capacity: 0,
            page_retrieval_mode: PageRetrievalMode::FromUserMode,
            stats_string: String::from("No page selected"),
            status_message: String::from("Ready"),
            is_querying_memory_pages: false,
            has_loaded_memory_pages_once: false,
            last_applied_snapshot_generation: 0,
            page_caches_by_base_address: HashMap::new(),
            unreadable_page_base_addresses: HashSet::new(),
            viewport_start_address: None,
            selected_instruction_address: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::CodeViewerPaneState;
    use squalr_engine_api::structures::memory::{bitness::Bitness, normalized_module::NormalizedModule, normalized_region::NormalizedRegion};

    #[test]
    fn build_visible_chunk_queries_aligns_viewport_to_chunk_window() {
        let mut code_viewer_pane_state = CodeViewerPaneState::default();
        code_viewer_pane_state.virtual_pages = vec![NormalizedRegion::new(0x1000, 0x3000)];
        code_viewer_pane_state.viewport_start_address = Some(0x1180);

        let query_ids = code_viewer_pane_state
            .build_visible_chunk_queries()
            .into_iter()
            .map(|virtual_snapshot_query| virtual_snapshot_query.get_query_id().to_string())
            .collect::<Vec<_>>();

        assert_eq!(query_ids.first().map(String::as_str), Some("0000000000001000:0000000000000000"));
        assert_eq!(query_ids.last().map(String::as_str), Some("0000000000001000:0000000000001200"));
        assert_eq!(query_ids.len(), 19);
    }

    #[test]
    fn derive_viewport_start_for_focus_address_clamps_to_page_bounds() {
        let normalized_region = NormalizedRegion::new(0x4000, 0x500);
        let derived_viewport_start = CodeViewerPaneState::derive_viewport_start_for_focus_address(&normalized_region, 0x4010);

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

        let resolved_page_index = CodeViewerPaneState::resolve_initial_page_index(&virtual_pages, &modules);

        assert_eq!(resolved_page_index, Some(1));
    }

    #[test]
    fn visible_row_entries_decode_x86_bytes() {
        let mut code_viewer_pane_state = CodeViewerPaneState::default();
        code_viewer_pane_state.virtual_pages = vec![NormalizedRegion::new(0x1000, 0x100)];
        code_viewer_pane_state.cached_last_page_index = 0;
        code_viewer_pane_state.viewport_start_address = Some(0x1000);
        code_viewer_pane_state.selected_instruction_address = Some(0x1000);
        code_viewer_pane_state.last_visible_row_capacity = 4;
        code_viewer_pane_state
            .page_caches_by_base_address
            .entry(0x1000)
            .or_default()
            .cache_chunk(0, vec![0x90, 0xC3, 0x00, 0x00]);

        let visible_row_entries = code_viewer_pane_state.visible_row_entries(Some(Bitness::Bit32));

        assert!(!visible_row_entries.is_empty());
        assert!(visible_row_entries[0].primary_text.contains("nop"));
    }
}
