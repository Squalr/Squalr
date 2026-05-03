use crate::app_context::AppContext;
use crate::ui::converters::{data_type_to_icon_converter::DataTypeToIconConverter, data_type_to_string_converter::DataTypeToStringConverter};
use crate::ui::widgets::controls::{
    button::Button as ThemeButton, combo_box::combo_box_item_view::ComboBoxItemView, combo_box::combo_box_view::ComboBoxView,
    context_menu::context_menu::ContextMenu, data_value_box::data_value_box_view::DataValueBoxView, groupbox::GroupBox,
    toolbar_menu::toolbar_menu_item_view::ToolbarMenuItemView,
};
use crate::views::{
    code_viewer::{code_viewer_view::CodeViewerView, view_data::code_viewer_view_data::CodeViewerViewData},
    memory_viewer::{memory_viewer_view::MemoryViewerView, view_data::memory_viewer_view_data::MemoryViewerViewData},
    struct_viewer::view_data::struct_viewer_focus_target::StructViewerFocusTarget,
    struct_viewer::view_data::struct_viewer_view_data::StructViewerViewData,
    symbol_explorer::symbol_explorer_toolbar_view::{SymbolExplorerToolbarAction, SymbolExplorerToolbarView},
    symbol_explorer::symbol_tree_entry_view::SymbolTreeEntryView,
    symbol_explorer::symbol_tree_inline_rename_view::SymbolTreeInlineRenameView,
    symbol_explorer::view_data::{
        symbol_explorer_view_data::{
            DefineFieldDraft, ModuleRootCreateDraft, SymbolExplorerContextMenuTarget, SymbolExplorerSelection, SymbolExplorerTakeOverState,
            SymbolExplorerViewData,
        },
        symbol_tree_entry::{ResolvedPointerTarget, SymbolTreeEntry, SymbolTreeEntryKind, build_symbol_tree_entries, resolve_symbol_tree_entry_size_in_bytes},
    },
};
use eframe::egui::{Align, Color32, Direction, Id, Key, Layout, Response, RichText, ScrollArea, TextEdit, Ui, UiBuilder, Widget, vec2};
use epaint::{Stroke, pos2};
use squalr_engine_api::commands::{
    memory::{
        read::{memory_read_request::MemoryReadRequest, memory_read_response::MemoryReadResponse},
        write::memory_write_request::MemoryWriteRequest,
    },
    privileged_command_request::PrivilegedCommandRequest,
    privileged_command_response::TypedPrivilegedCommandResponse,
    project_symbols::{
        create::project_symbols_create_request::ProjectSymbolsCreateRequest,
        create_module::project_symbols_create_module_request::ProjectSymbolsCreateModuleRequest,
        delete::project_symbols_delete_request::{ProjectSymbolsDeleteModuleRange, ProjectSymbolsDeleteModuleRangeMode, ProjectSymbolsDeleteRequest},
        rename::project_symbols_rename_request::ProjectSymbolsRenameRequest,
        rename_module::project_symbols_rename_module_request::ProjectSymbolsRenameModuleRequest,
    },
    unprivileged_command_request::UnprivilegedCommandRequest,
};
use squalr_engine_api::dependency_injection::dependency::Dependency;
use squalr_engine_api::engine::engine_execution_context::EngineExecutionContext;
use squalr_engine_api::structures::data_types::built_in_types::{string::utf8::data_type_string_utf8::DataTypeStringUtf8, u64::data_type_u64::DataTypeU64};
use squalr_engine_api::structures::data_types::data_type_ref::DataTypeRef;
use squalr_engine_api::structures::data_values::{
    anonymous_value_string::AnonymousValueString, anonymous_value_string_format::AnonymousValueStringFormat, container_type::ContainerType,
};
use squalr_engine_api::structures::memory::pointer::Pointer;
use squalr_engine_api::structures::pointer_scans::pointer_scan_pointer_size::PointerScanPointerSize;
use squalr_engine_api::structures::projects::{
    project_items::built_in_types::project_item_type_address::ProjectItemTypeAddress, project_symbol_catalog::ProjectSymbolCatalog,
    project_symbol_locator::ProjectSymbolLocator,
};
use squalr_engine_api::structures::structs::{
    symbolic_field_definition::SymbolicFieldDefinition, symbolic_struct_definition::SymbolicStructDefinition, valued_struct::ValuedStruct,
    valued_struct_field::ValuedStructField,
};
use squalr_engine_session::virtual_snapshots::virtual_snapshot_query::VirtualSnapshotQuery;
use squalr_engine_session::virtual_snapshots::virtual_snapshot_query_result::VirtualSnapshotQueryResult;
use std::collections::{HashMap, HashSet};
use std::str::FromStr;
use std::sync::Arc;
use std::sync::mpsc;
use std::time::Duration;

#[derive(Clone)]
pub struct SymbolExplorerView {
    app_context: Arc<AppContext>,
    symbol_explorer_view_data: Dependency<SymbolExplorerViewData>,
    struct_viewer_view_data: Dependency<StructViewerViewData>,
    memory_viewer_view_data: Dependency<MemoryViewerViewData>,
    code_viewer_view_data: Dependency<CodeViewerViewData>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ModuleChildRangeTarget {
    module_name: String,
    offset: u64,
    length: u64,
    display_name: String,
    delete_mode: ProjectSymbolsDeleteModuleRangeMode,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct DeleteConfirmationDescription {
    text: String,
    is_warning: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ModuleFieldTypeOptionKind {
    BuiltIn,
    StructLayout,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ModuleFieldTypeOption {
    data_type_ref: DataTypeRef,
    label: String,
    kind: ModuleFieldTypeOptionKind,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct U8SpanEditTarget {
    module_name: String,
    offset: u64,
    length: u64,
}

#[derive(Clone, Debug)]
struct DefineFieldPlan {
    project_symbols_create_request: ProjectSymbolsCreateRequest,
}

impl SymbolExplorerView {
    pub const WINDOW_ID: &'static str = "window_symbol_explorer";
    const POINTER_CHILDREN_VIRTUAL_SNAPSHOT_ID: &'static str = "symbol_explorer_pointer_children";
    const PREVIEW_VALUES_VIRTUAL_SNAPSHOT_ID: &'static str = "symbol_explorer_preview_values";
    const POINTER_CHILDREN_REFRESH_INTERVAL: Duration = Duration::from_millis(250);
    const PREVIEW_VALUES_REFRESH_INTERVAL: Duration = Duration::from_millis(250);
    const TOOLBAR_HEIGHT: f32 = 28.0;
    const CREATE_DISPLAY_NAME_DATA_VALUE_BOX_ID: &'static str = "symbol_explorer_create_display_name";
    const STRUCT_VIEWER_SYMBOL_NAME_FIELD: &'static str = "display_name";
    const STRUCT_VIEWER_SYMBOL_SIZE_FIELD: &'static str = "size";
    const STRUCT_VIEWER_SYMBOL_PATH_FIELD: &'static str = "path";
    const STRING_DATA_TYPE_ID: &'static str = "string_utf8";
    const INLINE_RENAME_TEXT_STORAGE_ID_PREFIX: &'static str = "symbol_explorer_inline_rename_text";
    const INLINE_RENAME_HIGHLIGHT_STORAGE_ID_PREFIX: &'static str = "symbol_explorer_inline_rename_highlight";
    const MAX_SYMBOL_PREVIEW_ELEMENT_COUNT: u64 = 4;
    const MAX_SYMBOL_PREVIEW_DISPLAY_ELEMENT_COUNT: usize = 3;
    const MAX_SYMBOL_PREVIEW_ARRAY_CHARACTER_COUNT: usize = 24;
    const MODULE_FIELD_BUILT_IN_TYPE_IDS: [&'static str; 18] = [
        "u8", "i8", "i16", "i16be", "i32", "i32be", "i64", "i64be", "u16", "u16be", "u32", "u32be", "u64", "u64be", "f32", "f32be", "f64", "f64be",
    ];
    const DEFINE_FIELD_CONTAINER_SELECTOR_WIDTH: f32 = 118.0;

    pub fn new(app_context: Arc<AppContext>) -> Self {
        let symbol_explorer_view_data = app_context
            .dependency_container
            .register(SymbolExplorerViewData::new());
        let struct_viewer_view_data = app_context
            .dependency_container
            .get_dependency::<StructViewerViewData>();
        let memory_viewer_view_data = app_context
            .dependency_container
            .get_dependency::<MemoryViewerViewData>();
        let code_viewer_view_data = app_context
            .dependency_container
            .get_dependency::<CodeViewerViewData>();

        Self {
            app_context,
            symbol_explorer_view_data,
            struct_viewer_view_data,
            memory_viewer_view_data,
            code_viewer_view_data,
        }
    }

    fn get_opened_project_symbol_catalog(&self) -> Option<ProjectSymbolCatalog> {
        let opened_project = self
            .app_context
            .engine_unprivileged_state
            .get_project_manager()
            .get_opened_project();
        let opened_project = opened_project.read().ok()?;

        opened_project.as_ref().map(|opened_project| {
            opened_project
                .get_project_info()
                .get_project_symbol_catalog()
                .clone()
        })
    }

    fn focus_memory_viewer_for_locator(
        &self,
        locator: &ProjectSymbolLocator,
    ) {
        MemoryViewerViewData::request_focus_address(
            self.memory_viewer_view_data.clone(),
            self.app_context.engine_unprivileged_state.clone(),
            locator.get_focus_address(),
            locator.get_focus_module_name().to_string(),
        );

        match self.app_context.docking_manager.write() {
            Ok(mut docking_manager) => {
                docking_manager.set_window_visibility(MemoryViewerView::WINDOW_ID, true);
                docking_manager.select_tab_by_window_id(MemoryViewerView::WINDOW_ID);
            }
            Err(error) => {
                log::error!("Failed to acquire docking manager while opening memory viewer from Symbol Explorer: {}", error);
            }
        }
    }

    fn focus_code_viewer_for_locator(
        &self,
        locator: &ProjectSymbolLocator,
    ) {
        CodeViewerViewData::request_focus_address(
            self.code_viewer_view_data.clone(),
            self.app_context.engine_unprivileged_state.clone(),
            locator.get_focus_address(),
            locator.get_focus_module_name().to_string(),
        );

        match self.app_context.docking_manager.write() {
            Ok(mut docking_manager) => {
                docking_manager.set_window_visibility(CodeViewerView::WINDOW_ID, true);
                docking_manager.select_tab_by_window_id(CodeViewerView::WINDOW_ID);
            }
            Err(error) => {
                log::error!("Failed to acquire docking manager while opening code viewer from Symbol Explorer: {}", error);
            }
        }
    }

    fn rename_symbol_claim(
        &self,
        symbol_locator_key: &str,
        display_name: String,
    ) {
        let project_symbols_rename_request = ProjectSymbolsRenameRequest {
            symbol_locator_key: symbol_locator_key.to_string(),
            display_name,
        };

        project_symbols_rename_request.send(&self.app_context.engine_unprivileged_state, |_project_symbols_rename_response| {});
    }

    fn rename_module_root(
        &self,
        module_name: &str,
        new_module_name: String,
    ) {
        let symbol_explorer_view_data = self.symbol_explorer_view_data.clone();
        let project_symbols_rename_module_request = ProjectSymbolsRenameModuleRequest {
            module_name: module_name.to_string(),
            new_module_name,
        };

        project_symbols_rename_module_request.send(&self.app_context.engine_unprivileged_state, move |project_symbols_rename_module_response| {
            if project_symbols_rename_module_response.success {
                let module_name = project_symbols_rename_module_response.module_name;

                SymbolExplorerViewData::set_selected_entry(
                    symbol_explorer_view_data.clone(),
                    Some(SymbolExplorerSelection::ModuleRoot(module_name.clone())),
                );
                SymbolExplorerViewData::expand_tree_node(symbol_explorer_view_data, &format!("module:{}", module_name));
            }
        });
    }

    fn rename_u8_segment(
        &self,
        module_name: &str,
        offset: u64,
        length: u64,
        display_name: String,
    ) {
        let symbol_explorer_view_data = self.symbol_explorer_view_data.clone();
        let module_name = module_name.to_string();
        let project_symbols_create_request = ProjectSymbolsCreateRequest {
            display_name,
            struct_layout_id: Self::u8_array_type_id(length),
            address: None,
            module_name: Some(module_name.clone()),
            offset: Some(offset),
            metadata: Default::default(),
        };

        project_symbols_create_request.send(&self.app_context.engine_unprivileged_state, move |project_symbols_create_response| {
            if project_symbols_create_response.success {
                SymbolExplorerViewData::set_selected_entry(
                    symbol_explorer_view_data.clone(),
                    Some(SymbolExplorerSelection::SymbolClaim(project_symbols_create_response.created_symbol_locator_key)),
                );
                SymbolExplorerViewData::expand_tree_node(symbol_explorer_view_data, &format!("module:{}", module_name));
            }
        });
    }

    fn delete_symbol_claim(
        &self,
        symbol_locator_key: &str,
    ) {
        SymbolExplorerViewData::cancel_take_over_state(self.symbol_explorer_view_data.clone());
        let project_symbols_delete_request = ProjectSymbolsDeleteRequest {
            symbol_locator_keys: vec![symbol_locator_key.to_string()],
            module_names: Vec::new(),
            module_ranges: Vec::new(),
        };

        project_symbols_delete_request.send(&self.app_context.engine_unprivileged_state, |_project_symbols_delete_response| {});
    }

    fn delete_module_range(
        &self,
        module_name: &str,
        offset: u64,
        length: u64,
        mode: ProjectSymbolsDeleteModuleRangeMode,
    ) {
        SymbolExplorerViewData::cancel_take_over_state(self.symbol_explorer_view_data.clone());
        let project_symbols_delete_request = ProjectSymbolsDeleteRequest {
            symbol_locator_keys: Vec::new(),
            module_names: Vec::new(),
            module_ranges: vec![ProjectSymbolsDeleteModuleRange {
                module_name: module_name.to_string(),
                offset,
                length,
                mode,
            }],
        };

        project_symbols_delete_request.send(&self.app_context.engine_unprivileged_state, |_project_symbols_delete_response| {});
    }

    fn build_delete_module_range_confirmation_description(
        module_name: &str,
        length: u64,
        mode: ProjectSymbolsDeleteModuleRangeMode,
    ) -> DeleteConfirmationDescription {
        match mode {
            ProjectSymbolsDeleteModuleRangeMode::ShiftLeft => DeleteConfirmationDescription {
                text: format!(
                    "WARNING: {} will be {} byte(s) smaller. Proceeding fields will be shifted left.",
                    module_name, length
                ),
                is_warning: true,
            },
            ProjectSymbolsDeleteModuleRangeMode::ReplaceWithU8 => DeleteConfirmationDescription {
                text: String::from("This removes the field definition and preserves the module bytes as u8[]."),
                is_warning: false,
            },
        }
    }

    fn build_module_field_type_options(project_symbol_catalog: &ProjectSymbolCatalog) -> Vec<ModuleFieldTypeOption> {
        let mut type_options = Self::MODULE_FIELD_BUILT_IN_TYPE_IDS
            .iter()
            .map(|data_type_id| ModuleFieldTypeOption {
                data_type_ref: DataTypeRef::new(data_type_id),
                label: DataTypeToStringConverter::convert_data_type_to_string(data_type_id),
                kind: ModuleFieldTypeOptionKind::BuiltIn,
            })
            .collect::<Vec<_>>();

        for struct_layout_descriptor in project_symbol_catalog.get_struct_layout_descriptors() {
            let struct_layout_id = struct_layout_descriptor.get_struct_layout_id();
            let struct_data_type_ref = DataTypeRef::new(struct_layout_id);

            if !type_options
                .iter()
                .any(|type_option| type_option.data_type_ref == struct_data_type_ref)
            {
                type_options.push(ModuleFieldTypeOption {
                    data_type_ref: struct_data_type_ref,
                    label: struct_layout_id.to_string(),
                    kind: ModuleFieldTypeOptionKind::StructLayout,
                });
            }
        }

        type_options
    }

    fn filter_module_field_type_options(
        type_options: &[ModuleFieldTypeOption],
        search_text: &str,
    ) -> Vec<ModuleFieldTypeOption> {
        let normalized_search_text = search_text.trim().to_lowercase();

        if normalized_search_text.is_empty() {
            return type_options.to_vec();
        }

        type_options
            .iter()
            .filter(|type_option| {
                type_option
                    .label
                    .to_lowercase()
                    .contains(&normalized_search_text)
                    || type_option
                        .data_type_ref
                        .get_data_type_id()
                        .to_lowercase()
                        .contains(&normalized_search_text)
            })
            .cloned()
            .collect()
    }

    fn module_field_type_option_uses_icon(type_option_kind: ModuleFieldTypeOptionKind) -> bool {
        matches!(type_option_kind, ModuleFieldTypeOptionKind::BuiltIn)
    }

    fn module_field_type_search_storage_id(menu_id: &str) -> Id {
        Id::new(("symbol_explorer_module_field_type_search", menu_id))
    }

    fn filter_registered_pointer_sizes(registered_data_type_refs: &[DataTypeRef]) -> Vec<PointerScanPointerSize> {
        let registered_data_type_ids = registered_data_type_refs
            .iter()
            .map(|data_type_ref| data_type_ref.get_data_type_id().to_string())
            .collect::<HashSet<_>>();

        PointerScanPointerSize::ALL
            .iter()
            .copied()
            .filter(|pointer_size| registered_data_type_ids.contains(pointer_size.to_data_type_ref().get_data_type_id()))
            .collect()
    }

    fn delete_module_root(
        &self,
        module_name: &str,
    ) {
        SymbolExplorerViewData::cancel_take_over_state(self.symbol_explorer_view_data.clone());
        let project_symbols_delete_request = ProjectSymbolsDeleteRequest {
            symbol_locator_keys: Vec::new(),
            module_names: vec![module_name.to_string()],
            module_ranges: Vec::new(),
        };

        project_symbols_delete_request.send(&self.app_context.engine_unprivileged_state, |_project_symbols_delete_response| {});
    }

    fn create_module_root(
        &self,
        project_symbols_create_module_request: ProjectSymbolsCreateModuleRequest,
    ) {
        let symbol_explorer_view_data = self.symbol_explorer_view_data.clone();

        project_symbols_create_module_request.send(&self.app_context.engine_unprivileged_state, move |project_symbols_create_module_response| {
            if project_symbols_create_module_response.success {
                let module_name = project_symbols_create_module_response.module_name;

                SymbolExplorerViewData::set_selected_entry(
                    symbol_explorer_view_data.clone(),
                    Some(SymbolExplorerSelection::ModuleRoot(module_name.clone())),
                );
                SymbolExplorerViewData::expand_tree_node(symbol_explorer_view_data, &format!("module:{}", module_name));
            }
        });
    }

    fn build_u8_module_claim_create_request(
        module_name: &str,
        offset: u64,
        length: u64,
    ) -> Option<ProjectSymbolsCreateRequest> {
        (length > 0).then(|| ProjectSymbolsCreateRequest {
            display_name: format!("u8_{:08X}", offset),
            struct_layout_id: Self::u8_array_type_id(length),
            address: None,
            module_name: Some(module_name.to_string()),
            offset: Some(offset),
            metadata: Default::default(),
        })
    }

    fn send_project_symbols_create_requests_sequential<ExecutionContext>(
        engine_unprivileged_state: Arc<ExecutionContext>,
        symbol_explorer_view_data: Dependency<SymbolExplorerViewData>,
        mut project_symbols_create_requests: Vec<ProjectSymbolsCreateRequest>,
        selection_module_name: Option<String>,
        selected_create_request_position: Option<usize>,
        create_request_position: usize,
    ) where
        ExecutionContext: EngineExecutionContext + 'static,
    {
        if project_symbols_create_requests.is_empty() {
            return;
        }

        let project_symbols_create_request = project_symbols_create_requests.remove(0);
        let should_select_created_symbol = selected_create_request_position.is_some_and(|selected_position| selected_position == create_request_position);
        let next_engine_unprivileged_state = engine_unprivileged_state.clone();
        let next_symbol_explorer_view_data = symbol_explorer_view_data.clone();
        let next_selection_module_name = selection_module_name.clone();
        let next_selected_create_request_position = selected_create_request_position;

        project_symbols_create_request.send(&engine_unprivileged_state, move |project_symbols_create_response| {
            if !project_symbols_create_response.success {
                log::warn!("Stopping sequential symbol creation after a project-symbols create request failed.");
                return;
            }

            if should_select_created_symbol {
                SymbolExplorerViewData::set_selected_entry(
                    symbol_explorer_view_data.clone(),
                    Some(SymbolExplorerSelection::SymbolClaim(project_symbols_create_response.created_symbol_locator_key)),
                );

                if let Some(module_name) = selection_module_name {
                    SymbolExplorerViewData::expand_tree_node(symbol_explorer_view_data.clone(), &format!("module:{}", module_name));
                }
            }

            Self::send_project_symbols_create_requests_sequential(
                next_engine_unprivileged_state,
                next_symbol_explorer_view_data,
                project_symbols_create_requests,
                next_selection_module_name,
                next_selected_create_request_position,
                create_request_position.saturating_add(1),
            );
        });
    }

    fn split_u8_span_edit_target_in_half(
        &self,
        u8_span_edit_target: &U8SpanEditTarget,
    ) {
        let length = u8_span_edit_target.length;
        if length < 2 {
            log::warn!("Cannot split a u8[] segment smaller than 2 byte(s).");
            return;
        }

        let first_length = length / 2;
        let second_length = length.saturating_sub(first_length);
        let Some(second_offset) = u8_span_edit_target.offset.checked_add(first_length) else {
            log::warn!("Cannot split u8[] segment because the second half offset overflowed.");
            return;
        };
        let Some(first_create_request) = Self::build_u8_module_claim_create_request(&u8_span_edit_target.module_name, u8_span_edit_target.offset, first_length)
        else {
            return;
        };
        let Some(second_create_request) = Self::build_u8_module_claim_create_request(&u8_span_edit_target.module_name, second_offset, second_length) else {
            return;
        };

        Self::send_project_symbols_create_requests_sequential(
            self.app_context.engine_unprivileged_state.clone(),
            self.symbol_explorer_view_data.clone(),
            vec![first_create_request, second_create_request],
            Some(u8_span_edit_target.module_name.clone()),
            Some(1),
            0,
        );
    }

    fn u8_array_type_id(length: u64) -> String {
        format!("u8[{}]", length)
    }

    fn parse_define_field_relative_offset(
        relative_offset_text: &str,
        relative_offset_format: AnonymousValueStringFormat,
    ) -> Result<u64, String> {
        let trimmed_relative_offset_text = relative_offset_text.trim();

        if trimmed_relative_offset_text.is_empty() {
            return Err(String::from("Offset is required."));
        }

        let normalized_binary_text = trimmed_relative_offset_text
            .strip_prefix("0b")
            .or_else(|| trimmed_relative_offset_text.strip_prefix("0B"));

        if let Some(binary_text) = normalized_binary_text {
            if binary_text.is_empty() {
                return Err(String::from("Binary offset is missing digits."));
            }

            return u64::from_str_radix(binary_text, 2).map_err(|_| format!("Invalid binary offset: {}.", trimmed_relative_offset_text));
        }

        let normalized_hex_text = trimmed_relative_offset_text
            .strip_prefix("0x")
            .or_else(|| trimmed_relative_offset_text.strip_prefix("0X"));

        if let Some(hex_text) = normalized_hex_text {
            if hex_text.is_empty() {
                return Err(String::from("Hex offset is missing digits."));
            }

            return u64::from_str_radix(hex_text, 16).map_err(|_| format!("Invalid hex offset: {}.", trimmed_relative_offset_text));
        }

        match relative_offset_format {
            AnonymousValueStringFormat::Binary => {
                u64::from_str_radix(trimmed_relative_offset_text, 2).map_err(|_| format!("Invalid binary offset: {}.", trimmed_relative_offset_text))
            }
            AnonymousValueStringFormat::Hexadecimal | AnonymousValueStringFormat::Address => {
                u64::from_str_radix(trimmed_relative_offset_text, 16).map_err(|_| format!("Invalid hex offset: {}.", trimmed_relative_offset_text))
            }
            _ => trimmed_relative_offset_text
                .parse::<u64>()
                .map_err(|_| format!("Invalid decimal offset: {}.", trimmed_relative_offset_text)),
        }
    }

    fn resolve_define_field_struct_layout_id_size(
        &self,
        project_symbol_catalog: &ProjectSymbolCatalog,
        struct_layout_id: &str,
    ) -> Option<u64> {
        let symbolic_field_definition = SymbolicFieldDefinition::from_str(struct_layout_id).ok()?;

        self.resolve_define_field_symbolic_size(project_symbol_catalog, &symbolic_field_definition, &mut HashSet::new())
    }

    fn resolve_define_field_symbolic_size(
        &self,
        project_symbol_catalog: &ProjectSymbolCatalog,
        symbolic_field_definition: &SymbolicFieldDefinition,
        visited_type_ids: &mut HashSet<String>,
    ) -> Option<u64> {
        if let Some(pointer_size) = symbolic_field_definition
            .get_container_type()
            .get_pointer_size()
        {
            return Some(pointer_size.get_size_in_bytes());
        }

        let data_type_id = symbolic_field_definition
            .get_data_type_ref()
            .get_data_type_id()
            .to_string();
        let unit_size_in_bytes = if let Some(symbolic_struct_definition) = project_symbol_catalog
            .get_struct_layout_descriptors()
            .iter()
            .find(|struct_layout_descriptor| struct_layout_descriptor.get_struct_layout_id() == data_type_id)
            .map(|struct_layout_descriptor| struct_layout_descriptor.get_struct_layout_definition().clone())
        {
            if !visited_type_ids.insert(data_type_id.clone()) {
                return None;
            }

            let struct_size_in_bytes = symbolic_struct_definition
                .get_fields()
                .iter()
                .try_fold(0_u64, |accumulated_size, field_definition| {
                    accumulated_size.checked_add(self.resolve_define_field_symbolic_size(project_symbol_catalog, field_definition, visited_type_ids)?)
                })?;

            visited_type_ids.remove(&data_type_id);
            struct_size_in_bytes
        } else if let Some(default_value) = self
            .app_context
            .engine_unprivileged_state
            .get_default_value(symbolic_field_definition.get_data_type_ref())
        {
            default_value.get_size_in_bytes()
        } else {
            return None;
        };

        Some(
            symbolic_field_definition
                .get_container_type()
                .get_total_size_in_bytes(unit_size_in_bytes),
        )
    }

    fn build_define_field_struct_layout_id(define_field_draft: &DefineFieldDraft) -> String {
        let data_type_ref = define_field_draft.data_type_selection.visible_data_type();
        let symbolic_field_definition = SymbolicFieldDefinition::new(data_type_ref.clone(), define_field_draft.container_type);

        symbolic_field_definition.to_string()
    }

    fn build_define_field_plan(
        define_field_draft: &DefineFieldDraft,
        module_name: &str,
        segment_offset: u64,
        segment_length: u64,
        resolve_type_size: impl Fn(&str) -> Option<u64>,
    ) -> Result<DefineFieldPlan, String> {
        let display_name = define_field_draft.display_name.trim();

        if display_name.is_empty() {
            return Err(String::from("Field name is required."));
        }

        let relative_offset = Self::parse_define_field_relative_offset(&define_field_draft.relative_offset_text, define_field_draft.relative_offset_format)?;
        let struct_layout_id = Self::build_define_field_struct_layout_id(define_field_draft);
        let Some(field_size) = resolve_type_size(&struct_layout_id) else {
            return Err(format!("Cannot resolve byte size for `{}`.", struct_layout_id));
        };

        if field_size == 0 {
            return Err(format!("`{}` has no byte size.", struct_layout_id));
        }

        let Some(relative_field_end) = relative_offset.checked_add(field_size) else {
            return Err(String::from("Field range is too large."));
        };

        if relative_field_end > segment_length {
            return Err(format!(
                "`{}` is {} byte(s), which does not fit inside this u8[] segment at offset 0x{:X}.",
                struct_layout_id, field_size, relative_offset
            ));
        }

        let Some(absolute_offset) = segment_offset.checked_add(relative_offset) else {
            return Err(String::from("Module offset is too large."));
        };

        Ok(DefineFieldPlan {
            project_symbols_create_request: ProjectSymbolsCreateRequest {
                display_name: display_name.to_string(),
                struct_layout_id,
                address: None,
                module_name: Some(module_name.to_string()),
                offset: Some(absolute_offset),
                metadata: Default::default(),
            },
        })
    }

    fn inline_rename_text_storage_id(symbol_locator_key: &str) -> Id {
        Id::new((Self::INLINE_RENAME_TEXT_STORAGE_ID_PREFIX, symbol_locator_key))
    }

    fn inline_rename_highlight_storage_id(symbol_locator_key: &str) -> Id {
        Id::new((Self::INLINE_RENAME_HIGHLIGHT_STORAGE_ID_PREFIX, symbol_locator_key))
    }

    fn clear_inline_rename_state(
        &self,
        user_interface: &mut Ui,
        symbol_locator_key: &str,
    ) {
        let rename_text_storage_id = Self::inline_rename_text_storage_id(symbol_locator_key);
        let rename_highlight_storage_id = Self::inline_rename_highlight_storage_id(symbol_locator_key);

        user_interface.ctx().data_mut(|data| {
            data.remove::<String>(rename_text_storage_id);
            data.remove::<bool>(rename_highlight_storage_id);
        });
        SymbolExplorerViewData::cancel_inline_rename(self.symbol_explorer_view_data.clone());
    }

    fn build_selected_symbol_tree_entry<'entry>(
        symbol_tree_entries: &'entry [SymbolTreeEntry],
        selected_entry: Option<&SymbolExplorerSelection>,
    ) -> Option<&'entry SymbolTreeEntry> {
        match selected_entry {
            Some(SymbolExplorerSelection::ModuleRoot(selected_module_name)) => symbol_tree_entries.iter().find(|symbol_tree_entry| {
                matches!(
                    symbol_tree_entry.get_kind(),
                    SymbolTreeEntryKind::ModuleSpace { module_name, .. } if module_name == selected_module_name
                )
            }),
            Some(SymbolExplorerSelection::SymbolClaim(selected_symbol_locator_key)) => symbol_tree_entries.iter().find(|symbol_tree_entry| {
                matches!(
                    symbol_tree_entry.get_kind(),
                    SymbolTreeEntryKind::SymbolClaim { symbol_locator_key } if symbol_locator_key == selected_symbol_locator_key
                )
            }),
            Some(SymbolExplorerSelection::DerivedNode(selected_node_key)) => symbol_tree_entries
                .iter()
                .find(|symbol_tree_entry| symbol_tree_entry.get_node_key() == selected_node_key),
            _ => None,
        }
    }

    fn build_module_child_range_target(
        project_symbol_catalog: &ProjectSymbolCatalog,
        symbol_tree_entry: &SymbolTreeEntry,
        resolve_primitive_size_in_bytes: impl Fn(&DataTypeRef) -> Option<u64> + Copy,
    ) -> Option<ModuleChildRangeTarget> {
        match symbol_tree_entry.get_kind() {
            SymbolTreeEntryKind::U8Segment { module_name, offset, length } => Some(ModuleChildRangeTarget {
                module_name: module_name.to_string(),
                offset: *offset,
                length: *length,
                display_name: symbol_tree_entry.get_display_name().to_string(),
                delete_mode: ProjectSymbolsDeleteModuleRangeMode::ShiftLeft,
            }),
            SymbolTreeEntryKind::SymbolClaim { .. } if symbol_tree_entry.get_depth() == 1 => {
                let ProjectSymbolLocator::ModuleOffset { module_name, offset } = symbol_tree_entry.get_locator() else {
                    return None;
                };
                let length = resolve_symbol_tree_entry_size_in_bytes(project_symbol_catalog, symbol_tree_entry, resolve_primitive_size_in_bytes);

                (length > 0).then(|| ModuleChildRangeTarget {
                    module_name: module_name.to_string(),
                    offset: *offset,
                    length,
                    display_name: symbol_tree_entry.get_display_name().to_string(),
                    delete_mode: if symbol_tree_entry.get_symbol_type_id() == "u8"
                        && matches!(symbol_tree_entry.get_container_type(), ContainerType::ArrayFixed(_))
                    {
                        ProjectSymbolsDeleteModuleRangeMode::ShiftLeft
                    } else {
                        ProjectSymbolsDeleteModuleRangeMode::ReplaceWithU8
                    },
                })
            }
            _ => None,
        }
    }

    fn build_u8_span_edit_target(symbol_tree_entry: &SymbolTreeEntry) -> Option<U8SpanEditTarget> {
        match symbol_tree_entry.get_kind() {
            SymbolTreeEntryKind::U8Segment { module_name, offset, length } => (*length > 0).then(|| U8SpanEditTarget {
                module_name: module_name.to_string(),
                offset: *offset,
                length: *length,
            }),
            SymbolTreeEntryKind::SymbolClaim { .. } => {
                if symbol_tree_entry.get_depth() != 1 || symbol_tree_entry.get_symbol_type_id() != "u8" {
                    return None;
                }

                let ContainerType::ArrayFixed(length) = symbol_tree_entry.get_container_type() else {
                    return None;
                };

                if length == 0 {
                    return None;
                }

                let ProjectSymbolLocator::ModuleOffset { module_name, offset } = symbol_tree_entry.get_locator() else {
                    return None;
                };

                Some(U8SpanEditTarget {
                    module_name: module_name.to_string(),
                    offset: *offset,
                    length,
                })
            }
            _ => None,
        }
    }

    fn create_define_field_from_u8_span_edit_target(
        &self,
        module_name: &str,
        define_field_plan: DefineFieldPlan,
    ) {
        Self::send_project_symbols_create_requests_sequential(
            self.app_context.engine_unprivileged_state.clone(),
            self.symbol_explorer_view_data.clone(),
            vec![define_field_plan.project_symbols_create_request],
            Some(module_name.to_string()),
            Some(0),
            0,
        );
    }

    fn request_delete_for_selection(
        &self,
        selected_symbol_claim: Option<&squalr_engine_api::structures::projects::project_symbol_claim::ProjectSymbolClaim>,
        selected_module_name: Option<&str>,
        selected_module_child_range_target: Option<&ModuleChildRangeTarget>,
    ) {
        if let Some(module_child_range_target) = selected_module_child_range_target {
            SymbolExplorerViewData::request_delete_module_range_confirmation(
                self.symbol_explorer_view_data.clone(),
                module_child_range_target.module_name.clone(),
                module_child_range_target.offset,
                module_child_range_target.length,
                module_child_range_target.display_name.clone(),
                module_child_range_target.delete_mode,
            );
        } else if let Some(symbol_claim) = selected_symbol_claim {
            SymbolExplorerViewData::request_delete_symbol_claim_confirmation(
                self.symbol_explorer_view_data.clone(),
                symbol_claim.get_symbol_locator_key().to_string(),
                symbol_claim.get_display_name().to_string(),
            );
        } else if let Some(module_name) = selected_module_name {
            SymbolExplorerViewData::request_delete_module_root_confirmation(self.symbol_explorer_view_data.clone(), module_name.to_string());
        }
    }

    fn build_struct_viewer_focus_target_key(selected_symbol_tree_entry: Option<&SymbolTreeEntry>) -> Option<String> {
        selected_symbol_tree_entry.map(|symbol_tree_entry| {
            format!(
                "{}|{}|{}",
                symbol_tree_entry.get_node_key(),
                symbol_tree_entry.get_display_name(),
                symbol_tree_entry.get_display_type_id()
            )
        })
    }

    fn build_struct_viewer_focus_target(selected_symbol_tree_entry: Option<&SymbolTreeEntry>) -> Option<StructViewerFocusTarget> {
        Self::build_struct_viewer_focus_target_key(selected_symbol_tree_entry).map(|selection_key| StructViewerFocusTarget::SymbolExplorer { selection_key })
    }

    fn is_symbol_tree_entry_struct_viewer_focused(
        symbol_tree_entry: &SymbolTreeEntry,
        shared_struct_viewer_focus_target: Option<&StructViewerFocusTarget>,
    ) -> bool {
        let Some(StructViewerFocusTarget::SymbolExplorer { selection_key }) = shared_struct_viewer_focus_target else {
            return false;
        };

        Self::build_struct_viewer_focus_target_key(Some(symbol_tree_entry))
            .as_ref()
            .is_some_and(|row_selection_key| row_selection_key == selection_key)
    }

    fn focus_symbol_tree_entry_in_struct_viewer(
        &self,
        project_symbol_catalog: &ProjectSymbolCatalog,
        selected_symbol_tree_entry: &SymbolTreeEntry,
    ) {
        let symbol_struct = self.build_symbol_struct_for_tree_entry(project_symbol_catalog, selected_symbol_tree_entry);
        let struct_viewer_edit_callback = self.build_struct_viewer_edit_callback(project_symbol_catalog, selected_symbol_tree_entry);
        let focus_target = Self::build_struct_viewer_focus_target(Some(selected_symbol_tree_entry));

        StructViewerViewData::focus_valued_struct_with_focus_target(
            self.struct_viewer_view_data.clone(),
            self.app_context.engine_unprivileged_state.clone(),
            symbol_struct,
            struct_viewer_edit_callback,
            focus_target,
        );
    }

    fn focus_symbol_tree_entry_for_edit(
        &self,
        project_symbol_catalog: &ProjectSymbolCatalog,
        selected_symbol_tree_entry: &SymbolTreeEntry,
    ) {
        let symbol_struct = self.build_symbol_struct_for_tree_entry(project_symbol_catalog, selected_symbol_tree_entry);
        let selected_field_name = Self::resolve_first_editable_struct_viewer_field_name(&symbol_struct);
        let struct_viewer_edit_callback = self.build_struct_viewer_edit_callback(project_symbol_catalog, selected_symbol_tree_entry);
        let focus_target = Self::build_struct_viewer_focus_target(Some(selected_symbol_tree_entry));

        StructViewerViewData::focus_valued_struct_with_focus_target(
            self.struct_viewer_view_data.clone(),
            self.app_context.engine_unprivileged_state.clone(),
            symbol_struct,
            struct_viewer_edit_callback,
            focus_target,
        );

        if let Some(selected_field_name) = selected_field_name {
            StructViewerViewData::set_selected_field(self.struct_viewer_view_data.clone(), selected_field_name);
        }
    }

    fn resolve_first_editable_struct_viewer_field_name(symbol_struct: &ValuedStruct) -> Option<String> {
        symbol_struct
            .get_fields()
            .iter()
            .find(|valued_struct_field| !valued_struct_field.get_is_read_only())
            .map(|valued_struct_field| valued_struct_field.get_name().to_string())
    }

    fn sync_selected_symbol_into_struct_viewer(
        &self,
        project_symbol_catalog: &ProjectSymbolCatalog,
        selected_symbol_tree_entry: Option<&SymbolTreeEntry>,
    ) {
        let current_focus_target = self
            .struct_viewer_view_data
            .read("Symbol explorer current struct viewer focus target")
            .and_then(|struct_viewer_view_data| struct_viewer_view_data.get_focus_target().cloned());
        let selected_symbol_tree_entry =
            selected_symbol_tree_entry.filter(|symbol_tree_entry| !matches!(symbol_tree_entry.get_kind(), SymbolTreeEntryKind::ModuleSpace { .. }));
        let desired_focus_target = Self::build_struct_viewer_focus_target(selected_symbol_tree_entry);

        if current_focus_target == desired_focus_target {
            return;
        }

        let Some(selected_symbol_tree_entry) = selected_symbol_tree_entry else {
            if matches!(current_focus_target, Some(StructViewerFocusTarget::SymbolExplorer { .. })) {
                StructViewerViewData::clear_focus(self.struct_viewer_view_data.clone());
            }
            return;
        };

        if matches!(
            current_focus_target,
            Some(StructViewerFocusTarget::ProjectHierarchy { .. }) | Some(StructViewerFocusTarget::SymbolTable { .. })
        ) {
            return;
        }

        self.focus_symbol_tree_entry_in_struct_viewer(project_symbol_catalog, selected_symbol_tree_entry);
    }

    fn build_struct_viewer_edit_callback(
        &self,
        project_symbol_catalog: &ProjectSymbolCatalog,
        selected_symbol_tree_entry: &SymbolTreeEntry,
    ) -> Arc<dyn Fn(ValuedStructField) + Send + Sync> {
        let symbol_claim_locator_key = match selected_symbol_tree_entry.get_kind() {
            SymbolTreeEntryKind::SymbolClaim { symbol_locator_key } => Some(symbol_locator_key.to_string()),
            _ => None,
        };
        let selected_symbol_tree_entry = selected_symbol_tree_entry.clone();
        let project_symbol_catalog = project_symbol_catalog.clone();
        let engine_unprivileged_state = self.app_context.engine_unprivileged_state.clone();
        let engine_execution_context: Arc<dyn EngineExecutionContext> = engine_unprivileged_state.clone();

        Arc::new(move |edited_field: ValuedStructField| {
            if edited_field.get_name() == Self::STRUCT_VIEWER_SYMBOL_NAME_FIELD {
                let Some(symbol_locator_key) = symbol_claim_locator_key.as_ref() else {
                    return;
                };
                let next_display_name = StructViewerViewData::read_utf8_field_text(&edited_field)
                    .trim()
                    .to_string();
                if next_display_name.is_empty() || next_display_name == selected_symbol_tree_entry.get_display_name() {
                    return;
                }

                ProjectSymbolsRenameRequest {
                    symbol_locator_key: symbol_locator_key.clone(),
                    display_name: next_display_name,
                }
                .send(&engine_unprivileged_state, |_project_symbols_rename_response| {});
                return;
            }

            let Some(memory_write_request) = Self::build_memory_write_request_for_symbol_value_edit(
                &engine_execution_context,
                &project_symbol_catalog,
                &selected_symbol_tree_entry,
                &edited_field,
            ) else {
                return;
            };

            memory_write_request.send(&engine_unprivileged_state, |memory_write_response| {
                if !memory_write_response.success {
                    log::warn!("Symbol Explorer struct-viewer memory write command failed.");
                }
            });
        })
    }

    fn build_memory_write_request_for_symbol_value_edit(
        engine_execution_context: &Arc<dyn EngineExecutionContext>,
        project_symbol_catalog: &ProjectSymbolCatalog,
        selected_symbol_tree_entry: &SymbolTreeEntry,
        edited_field: &ValuedStructField,
    ) -> Option<MemoryWriteRequest> {
        let edited_data_value = edited_field.get_data_value()?;
        let symbolic_struct_definition =
            Self::build_named_symbolic_struct_definition_for_value_edit(engine_execution_context, project_symbol_catalog, selected_symbol_tree_entry)?;
        let field_offset = Self::resolve_symbol_struct_field_offset(engine_execution_context, &symbolic_struct_definition, edited_field.get_name())?;
        let address = selected_symbol_tree_entry
            .get_locator()
            .get_focus_address()
            .checked_add(field_offset)?;

        Some(MemoryWriteRequest {
            address,
            module_name: selected_symbol_tree_entry
                .get_locator()
                .get_focus_module_name()
                .to_string(),
            value: edited_data_value.get_value_bytes().clone(),
        })
    }

    fn build_named_symbolic_struct_definition_for_value_edit(
        engine_execution_context: &Arc<dyn EngineExecutionContext>,
        project_symbol_catalog: &ProjectSymbolCatalog,
        symbol_tree_entry: &SymbolTreeEntry,
    ) -> Option<SymbolicStructDefinition> {
        let symbolic_struct_definition = Self::build_symbolic_struct_definition_for_symbol_type_for_context(
            engine_execution_context,
            project_symbol_catalog,
            symbol_tree_entry.get_symbol_type_id(),
        )?;

        if !symbolic_struct_definition.get_fields().is_empty() {
            return Some(symbolic_struct_definition);
        }

        Some(SymbolicStructDefinition::new_anonymous(vec![SymbolicFieldDefinition::new(
            DataTypeRef::new(symbol_tree_entry.get_symbol_type_id()),
            symbol_tree_entry.get_container_type(),
        )]))
    }

    fn resolve_symbol_struct_field_offset(
        engine_execution_context: &Arc<dyn EngineExecutionContext>,
        symbolic_struct_definition: &SymbolicStructDefinition,
        edited_field_name: &str,
    ) -> Option<u64> {
        let mut cumulative_field_offset = 0_u64;

        for (field_index, symbolic_field_definition) in symbolic_struct_definition.get_fields().iter().enumerate() {
            if Self::normalize_symbol_value_field_name(symbolic_field_definition.get_field_name(), field_index) == edited_field_name {
                return Some(cumulative_field_offset);
            }

            cumulative_field_offset = cumulative_field_offset.checked_add(Self::resolve_symbolic_field_size_in_bytes(
                engine_execution_context,
                symbolic_field_definition,
                &mut HashSet::new(),
            )?)?;
        }

        None
    }

    fn resolve_symbolic_field_size_in_bytes(
        engine_execution_context: &Arc<dyn EngineExecutionContext>,
        symbolic_field_definition: &SymbolicFieldDefinition,
        visited_type_ids: &mut HashSet<String>,
    ) -> Option<u64> {
        if let Some(pointer_size) = symbolic_field_definition
            .get_container_type()
            .get_pointer_size()
        {
            return Some(pointer_size.get_size_in_bytes());
        }

        let data_type_id = symbolic_field_definition
            .get_data_type_ref()
            .get_data_type_id()
            .to_string();
        let unit_size_in_bytes = if let Some(nested_symbolic_struct_definition) = engine_execution_context.resolve_struct_layout_definition(&data_type_id) {
            if !visited_type_ids.insert(data_type_id.clone()) {
                return None;
            }

            let nested_size_in_bytes =
                Self::resolve_symbolic_struct_size_in_bytes(engine_execution_context, &nested_symbolic_struct_definition, visited_type_ids)?;

            visited_type_ids.remove(&data_type_id);

            nested_size_in_bytes
        } else if let Some(default_value) = engine_execution_context.get_default_value(symbolic_field_definition.get_data_type_ref()) {
            default_value.get_size_in_bytes()
        } else {
            return None;
        };

        Some(
            symbolic_field_definition
                .get_container_type()
                .get_total_size_in_bytes(unit_size_in_bytes),
        )
    }

    fn resolve_symbolic_struct_size_in_bytes(
        engine_execution_context: &Arc<dyn EngineExecutionContext>,
        symbolic_struct_definition: &SymbolicStructDefinition,
        visited_type_ids: &mut HashSet<String>,
    ) -> Option<u64> {
        symbolic_struct_definition
            .get_fields()
            .iter()
            .try_fold(0_u64, |accumulated_size, symbolic_field_definition| {
                accumulated_size.checked_add(Self::resolve_symbolic_field_size_in_bytes(
                    engine_execution_context,
                    symbolic_field_definition,
                    visited_type_ids,
                )?)
            })
    }

    fn normalize_symbol_value_field_name(
        field_name: &str,
        field_index: usize,
    ) -> String {
        if field_name.trim().is_empty() {
            if field_index == 0 {
                String::from("value")
            } else {
                format!("value_{}", field_index)
            }
        } else {
            field_name.to_string()
        }
    }

    fn build_named_symbolic_struct_definition_for_preview(
        &self,
        project_symbol_catalog: &ProjectSymbolCatalog,
        symbol_tree_entry: &SymbolTreeEntry,
        truncate_preview_arrays: bool,
    ) -> Option<SymbolicStructDefinition> {
        let entry_field_definition = SymbolicFieldDefinition::from_str(&symbol_tree_entry.get_display_type_id()).ok()?;
        let preview_container_type = if truncate_preview_arrays {
            match entry_field_definition.get_container_type() {
                ContainerType::ArrayFixed(length) if length > Self::MAX_SYMBOL_PREVIEW_ELEMENT_COUNT => {
                    ContainerType::ArrayFixed(Self::MAX_SYMBOL_PREVIEW_ELEMENT_COUNT)
                }
                container_type => container_type,
            }
        } else {
            entry_field_definition.get_container_type()
        };

        let resolved_symbolic_struct_definition =
            self.build_symbolic_struct_definition_for_symbol_type(project_symbol_catalog, entry_field_definition.get_data_type_ref().get_data_type_id())?;

        if resolved_symbolic_struct_definition.get_fields().len() > 1 {
            return None;
        }

        if resolved_symbolic_struct_definition.get_fields().is_empty() || preview_container_type != ContainerType::None {
            return Some(SymbolicStructDefinition::new_anonymous(vec![SymbolicFieldDefinition::new(
                entry_field_definition.get_data_type_ref().clone(),
                preview_container_type,
            )]));
        }

        Some(resolved_symbolic_struct_definition)
    }

    fn build_symbolic_struct_definition_for_symbol_type_static(
        project_symbol_catalog: &ProjectSymbolCatalog,
        symbol_type_id: &str,
    ) -> Option<SymbolicStructDefinition> {
        if let Some(project_struct_layout_descriptor) = project_symbol_catalog
            .get_struct_layout_descriptors()
            .iter()
            .find(|struct_layout_descriptor| struct_layout_descriptor.get_struct_layout_id() == symbol_type_id)
        {
            return Some(
                project_struct_layout_descriptor
                    .get_struct_layout_definition()
                    .clone(),
            );
        }

        if let Ok(symbolic_struct_definition) = SymbolicStructDefinition::from_str(symbol_type_id) {
            return Some(symbolic_struct_definition);
        }

        if let Ok(symbolic_field_definition) = SymbolicFieldDefinition::from_str(symbol_type_id) {
            return Some(SymbolicStructDefinition::new_anonymous(vec![symbolic_field_definition]));
        }

        Some(SymbolicStructDefinition::new_anonymous(vec![SymbolicFieldDefinition::new(
            DataTypeRef::new(symbol_type_id),
            Default::default(),
        )]))
    }

    fn build_symbol_struct_for_tree_entry(
        &self,
        project_symbol_catalog: &ProjectSymbolCatalog,
        symbol_tree_entry: &SymbolTreeEntry,
    ) -> ValuedStruct {
        let include_symbol_claim_metadata = matches!(symbol_tree_entry.get_kind(), SymbolTreeEntryKind::SymbolClaim { .. });
        let engine_execution_context: Arc<dyn EngineExecutionContext> = self.app_context.engine_unprivileged_state.clone();
        let symbol_size_in_bytes = Self::resolve_symbol_tree_entry_size_for_struct_viewer(&engine_execution_context, symbol_tree_entry);

        if Self::symbol_tree_entry_should_use_external_value_viewer(symbol_tree_entry) {
            return Self::build_external_value_symbol_struct(symbol_tree_entry, include_symbol_claim_metadata, symbol_size_in_bytes);
        }

        let Some(symbolic_struct_definition) = self.build_named_symbolic_struct_definition_for_symbol_tree_entry(project_symbol_catalog, symbol_tree_entry)
        else {
            return self.build_symbol_struct_fallback(
                symbol_tree_entry,
                "Unable to resolve a struct definition for the selected symbol.",
                include_symbol_claim_metadata,
                symbol_size_in_bytes,
            );
        };

        let memory_read_response = Self::dispatch_memory_read_request(
            &engine_execution_context,
            symbol_tree_entry.get_locator().get_focus_address(),
            symbol_tree_entry.get_locator().get_focus_module_name(),
            &symbolic_struct_definition,
        );
        let Some(memory_read_response) = memory_read_response else {
            return self.build_symbol_struct_fallback(
                symbol_tree_entry,
                "Timed out while reading the selected symbol from memory.",
                include_symbol_claim_metadata,
                symbol_size_in_bytes,
            );
        };

        if !memory_read_response.success {
            return self.build_symbol_struct_fallback(
                symbol_tree_entry,
                "The selected symbol could not be read from memory.",
                include_symbol_claim_metadata,
                symbol_size_in_bytes,
            );
        }

        Self::normalize_symbol_memory_struct(
            memory_read_response.valued_struct,
            symbol_tree_entry,
            include_symbol_claim_metadata,
            symbol_size_in_bytes,
        )
    }

    fn build_named_symbolic_struct_definition_for_symbol_tree_entry(
        &self,
        project_symbol_catalog: &ProjectSymbolCatalog,
        symbol_tree_entry: &SymbolTreeEntry,
    ) -> Option<SymbolicStructDefinition> {
        self.build_symbolic_struct_definition_for_symbol_type(project_symbol_catalog, symbol_tree_entry.get_symbol_type_id())
            .map(|symbolic_struct_definition| {
                if !symbolic_struct_definition.get_fields().is_empty() {
                    return symbolic_struct_definition;
                }

                SymbolicStructDefinition::new_anonymous(vec![SymbolicFieldDefinition::new(
                    DataTypeRef::new(symbol_tree_entry.get_symbol_type_id()),
                    symbol_tree_entry.get_container_type(),
                )])
            })
    }

    fn normalize_symbol_memory_struct(
        valued_struct: ValuedStruct,
        symbol_tree_entry: &SymbolTreeEntry,
        include_symbol_claim_metadata: bool,
        symbol_size_in_bytes: Option<u64>,
    ) -> ValuedStruct {
        let mut normalized_fields = Self::build_symbol_struct_metadata_fields(symbol_tree_entry, include_symbol_claim_metadata, symbol_size_in_bytes);

        normalized_fields.extend(
            valued_struct
                .get_fields()
                .iter()
                .enumerate()
                .map(|(field_index, valued_struct_field)| {
                    let resolved_field_name = Self::normalize_symbol_value_field_name(valued_struct_field.get_name(), field_index);

                    ValuedStructField::new(resolved_field_name, valued_struct_field.get_field_data().clone(), false)
                })
                .collect::<Vec<_>>(),
        );

        ValuedStruct::new_anonymous(normalized_fields)
    }

    fn build_symbol_struct_fallback(
        &self,
        symbol_tree_entry: &SymbolTreeEntry,
        status_text: &str,
        include_symbol_claim_metadata: bool,
        symbol_size_in_bytes: Option<u64>,
    ) -> ValuedStruct {
        let mut fallback_fields = Self::build_symbol_struct_metadata_fields(symbol_tree_entry, include_symbol_claim_metadata, symbol_size_in_bytes);

        fallback_fields.extend([
            DataTypeStringUtf8::get_value_from_primitive_string(&symbol_tree_entry.get_locator().to_string())
                .to_named_valued_struct_field(String::from("locator"), true),
            DataTypeStringUtf8::get_value_from_primitive_string(status_text).to_named_valued_struct_field(String::from("status"), true),
        ]);

        ValuedStruct::new_anonymous(fallback_fields)
    }

    fn build_external_value_symbol_struct(
        symbol_tree_entry: &SymbolTreeEntry,
        include_symbol_claim_metadata: bool,
        symbol_size_in_bytes: Option<u64>,
    ) -> ValuedStruct {
        let mut fields = Self::build_symbol_struct_metadata_fields(symbol_tree_entry, include_symbol_claim_metadata, symbol_size_in_bytes);

        fields.push(
            DataTypeStringUtf8::get_value_from_primitive_string("")
                .to_named_valued_struct_field(ProjectItemTypeAddress::PROPERTY_FREEZE_DISPLAY_VALUE.to_string(), true),
        );

        ValuedStruct::new_anonymous(fields)
    }

    fn build_symbol_struct_metadata_fields(
        symbol_tree_entry: &SymbolTreeEntry,
        include_symbol_claim_metadata: bool,
        symbol_size_in_bytes: Option<u64>,
    ) -> Vec<ValuedStructField> {
        let mut metadata_fields = Vec::new();

        if include_symbol_claim_metadata {
            metadata_fields.push(
                DataTypeStringUtf8::get_value_from_primitive_string(symbol_tree_entry.get_display_name())
                    .to_named_valued_struct_field(Self::STRUCT_VIEWER_SYMBOL_NAME_FIELD.to_string(), false),
            );
        }

        metadata_fields.push(
            DataTypeStringUtf8::get_value_from_primitive_string(&symbol_tree_entry.get_display_type_id())
                .to_named_valued_struct_field(ProjectItemTypeAddress::PROPERTY_SYMBOLIC_STRUCT_DEFINITION_REFERENCE.to_string(), true),
        );

        metadata_fields.extend(Self::build_symbol_struct_location_fields(symbol_tree_entry, symbol_size_in_bytes));

        metadata_fields
    }

    fn build_symbol_struct_location_fields(
        symbol_tree_entry: &SymbolTreeEntry,
        symbol_size_in_bytes: Option<u64>,
    ) -> Vec<ValuedStructField> {
        let mut location_fields = Vec::new();
        let locator = symbol_tree_entry.get_locator();

        location_fields.push(
            DataTypeU64::get_value_from_primitive(locator.get_focus_address())
                .to_named_valued_struct_field(ProjectItemTypeAddress::PROPERTY_ADDRESS.to_string(), true),
        );

        location_fields.push(
            DataTypeStringUtf8::get_value_from_primitive_string(locator.get_focus_module_name())
                .to_named_valued_struct_field(ProjectItemTypeAddress::PROPERTY_MODULE.to_string(), true),
        );

        if let Some(symbol_size_in_bytes) = symbol_size_in_bytes {
            location_fields.push(
                DataTypeU64::get_value_from_primitive(symbol_size_in_bytes)
                    .to_named_valued_struct_field(Self::STRUCT_VIEWER_SYMBOL_SIZE_FIELD.to_string(), true),
            );
        }

        if !symbol_tree_entry.get_full_path().is_empty() {
            location_fields.push(
                DataTypeStringUtf8::get_value_from_primitive_string(symbol_tree_entry.get_full_path())
                    .to_named_valued_struct_field(Self::STRUCT_VIEWER_SYMBOL_PATH_FIELD.to_string(), true),
            );
        }

        location_fields
    }

    fn resolve_symbol_tree_entry_size_for_struct_viewer(
        engine_execution_context: &Arc<dyn EngineExecutionContext>,
        symbol_tree_entry: &SymbolTreeEntry,
    ) -> Option<u64> {
        let symbolic_field_definition = SymbolicFieldDefinition::from_str(&symbol_tree_entry.get_display_type_id()).ok()?;

        Self::resolve_symbolic_field_size_in_bytes(engine_execution_context, &symbolic_field_definition, &mut HashSet::new())
    }

    fn symbol_tree_entry_should_use_external_value_viewer(symbol_tree_entry: &SymbolTreeEntry) -> bool {
        matches!(symbol_tree_entry.get_container_type(), ContainerType::Array | ContainerType::ArrayFixed(_))
    }

    fn dispatch_memory_read_request(
        engine_execution_context: &Arc<dyn EngineExecutionContext>,
        address: u64,
        module_name: &str,
        symbolic_struct_definition: &SymbolicStructDefinition,
    ) -> Option<MemoryReadResponse> {
        let memory_read_request = MemoryReadRequest {
            address,
            module_name: module_name.to_string(),
            symbolic_struct_definition: symbolic_struct_definition.clone(),
            suppress_logging: true,
        };
        let memory_read_command = memory_read_request.to_engine_command();
        let (memory_read_response_sender, memory_read_response_receiver) = mpsc::channel();

        let dispatch_result = match engine_execution_context.get_bindings().read() {
            Ok(engine_bindings) => engine_bindings.dispatch_privileged_command(
                memory_read_command,
                Box::new(move |engine_response| {
                    let conversion_result = match MemoryReadResponse::from_engine_response(engine_response) {
                        Ok(memory_read_response) => Ok(memory_read_response),
                        Err(unexpected_response) => Err(format!(
                            "Unexpected response variant for Symbol Explorer memory read request: {:?}",
                            unexpected_response
                        )),
                    };
                    let _ = memory_read_response_sender.send(conversion_result);
                }),
            ),
            Err(error) => {
                log::error!("Failed to acquire engine bindings lock for Symbol Explorer memory read request: {}", error);
                return None;
            }
        };

        if let Err(error) = dispatch_result {
            log::error!("Failed to dispatch Symbol Explorer memory read request: {}", error);
            return None;
        }

        match memory_read_response_receiver.recv_timeout(Duration::from_secs(2)) {
            Ok(Ok(memory_read_response)) => Some(memory_read_response),
            Ok(Err(error)) => {
                log::error!("Failed to convert Symbol Explorer memory read response: {}", error);
                None
            }
            Err(error) => {
                log::error!("Timed out waiting for Symbol Explorer memory read response: {}", error);
                None
            }
        }
    }

    fn sync_pointer_child_virtual_snapshot(
        &self,
        project_symbol_catalog: &ProjectSymbolCatalog,
        symbol_tree_entries: &[SymbolTreeEntry],
    ) {
        let pointer_snapshot_queries = self.build_pointer_snapshot_queries(project_symbol_catalog, symbol_tree_entries);

        self.app_context
            .engine_unprivileged_state
            .set_virtual_snapshot_queries(
                Self::POINTER_CHILDREN_VIRTUAL_SNAPSHOT_ID,
                Self::POINTER_CHILDREN_REFRESH_INTERVAL,
                pointer_snapshot_queries,
            );
        self.app_context
            .engine_unprivileged_state
            .request_virtual_snapshot_refresh(Self::POINTER_CHILDREN_VIRTUAL_SNAPSHOT_ID);
    }

    fn build_pointer_snapshot_queries(
        &self,
        project_symbol_catalog: &ProjectSymbolCatalog,
        symbol_tree_entries: &[SymbolTreeEntry],
    ) -> Vec<VirtualSnapshotQuery> {
        symbol_tree_entries
            .iter()
            .filter(|symbol_tree_entry| {
                symbol_tree_entry.is_expanded()
                    && !matches!(symbol_tree_entry.get_kind(), SymbolTreeEntryKind::PointerTarget)
                    && symbol_tree_entry
                        .get_container_type()
                        .get_pointer_size()
                        .is_some()
            })
            .filter_map(|symbol_tree_entry| self.build_pointer_virtual_snapshot_query(project_symbol_catalog, symbol_tree_entry))
            .collect()
    }

    fn build_pointer_virtual_snapshot_query(
        &self,
        project_symbol_catalog: &ProjectSymbolCatalog,
        symbol_tree_entry: &SymbolTreeEntry,
    ) -> Option<VirtualSnapshotQuery> {
        let pointer_size = symbol_tree_entry.get_container_type().get_pointer_size()?;
        let symbolic_struct_definition =
            self.build_symbolic_struct_definition_for_symbol_type(project_symbol_catalog, symbol_tree_entry.get_symbol_type_id())?;

        Some(VirtualSnapshotQuery::Pointer {
            query_id: symbol_tree_entry.get_node_key().to_string(),
            pointer: Pointer::new_with_size(
                symbol_tree_entry.get_locator().get_focus_address(),
                vec![0],
                symbol_tree_entry
                    .get_locator()
                    .get_focus_module_name()
                    .to_string(),
                pointer_size,
            ),
            symbolic_struct_definition,
        })
    }

    fn build_symbolic_struct_definition_for_symbol_type(
        &self,
        project_symbol_catalog: &ProjectSymbolCatalog,
        symbol_type_id: &str,
    ) -> Option<SymbolicStructDefinition> {
        let engine_execution_context: Arc<dyn EngineExecutionContext> = self.app_context.engine_unprivileged_state.clone();

        Self::build_symbolic_struct_definition_for_symbol_type_for_context(&engine_execution_context, project_symbol_catalog, symbol_type_id)
    }

    fn build_symbolic_struct_definition_for_symbol_type_for_context(
        engine_execution_context: &Arc<dyn EngineExecutionContext>,
        project_symbol_catalog: &ProjectSymbolCatalog,
        symbol_type_id: &str,
    ) -> Option<SymbolicStructDefinition> {
        if let Some(project_struct_layout_descriptor) = project_symbol_catalog
            .get_struct_layout_descriptors()
            .iter()
            .find(|struct_layout_descriptor| struct_layout_descriptor.get_struct_layout_id() == symbol_type_id)
        {
            return Some(
                project_struct_layout_descriptor
                    .get_struct_layout_definition()
                    .clone(),
            );
        }

        if let Ok(symbolic_struct_definition) = SymbolicStructDefinition::from_str(symbol_type_id) {
            return Some(symbolic_struct_definition);
        }

        if let Some(symbolic_struct_definition) = engine_execution_context.resolve_struct_layout_definition(symbol_type_id) {
            return Some(symbolic_struct_definition);
        }

        Self::build_symbolic_struct_definition_for_symbol_type_static(project_symbol_catalog, symbol_type_id)
    }

    fn collect_resolved_pointer_targets_by_node_key(&self) -> HashMap<String, ResolvedPointerTarget> {
        let Some(virtual_snapshot) = self
            .app_context
            .engine_unprivileged_state
            .get_virtual_snapshot(Self::POINTER_CHILDREN_VIRTUAL_SNAPSHOT_ID)
        else {
            return HashMap::new();
        };

        virtual_snapshot
            .get_query_results()
            .iter()
            .filter_map(|(query_id, virtual_snapshot_query_result)| {
                let resolved_address = virtual_snapshot_query_result.resolved_address?;
                let target_locator = if virtual_snapshot_query_result.resolved_module_name.is_empty() {
                    ProjectSymbolLocator::new_absolute_address(resolved_address)
                } else {
                    ProjectSymbolLocator::new_module_offset(virtual_snapshot_query_result.resolved_module_name.clone(), resolved_address)
                };

                Some((
                    query_id.clone(),
                    ResolvedPointerTarget::new(target_locator, virtual_snapshot_query_result.evaluated_pointer_path.clone()),
                ))
            })
            .collect()
    }

    fn sync_symbol_preview_virtual_snapshot(
        &self,
        project_symbol_catalog: &ProjectSymbolCatalog,
        symbol_tree_entries: &[SymbolTreeEntry],
    ) {
        let preview_snapshot_queries = self.build_symbol_preview_snapshot_queries(project_symbol_catalog, symbol_tree_entries);

        self.app_context
            .engine_unprivileged_state
            .set_virtual_snapshot_queries(
                Self::PREVIEW_VALUES_VIRTUAL_SNAPSHOT_ID,
                Self::PREVIEW_VALUES_REFRESH_INTERVAL,
                preview_snapshot_queries,
            );
        self.app_context
            .engine_unprivileged_state
            .request_virtual_snapshot_refresh(Self::PREVIEW_VALUES_VIRTUAL_SNAPSHOT_ID);
    }

    fn build_symbol_preview_snapshot_queries(
        &self,
        project_symbol_catalog: &ProjectSymbolCatalog,
        symbol_tree_entries: &[SymbolTreeEntry],
    ) -> Vec<VirtualSnapshotQuery> {
        symbol_tree_entries
            .iter()
            .filter(|symbol_tree_entry| Self::symbol_tree_entry_should_query_preview(symbol_tree_entry))
            .filter_map(|symbol_tree_entry| self.build_symbol_preview_virtual_snapshot_query(project_symbol_catalog, symbol_tree_entry))
            .collect()
    }

    fn symbol_tree_entry_should_query_preview(symbol_tree_entry: &SymbolTreeEntry) -> bool {
        !matches!(symbol_tree_entry.get_kind(), SymbolTreeEntryKind::ModuleSpace { .. })
    }

    fn build_symbol_preview_virtual_snapshot_query(
        &self,
        project_symbol_catalog: &ProjectSymbolCatalog,
        symbol_tree_entry: &SymbolTreeEntry,
    ) -> Option<VirtualSnapshotQuery> {
        let symbolic_struct_definition = self.build_named_symbolic_struct_definition_for_preview(project_symbol_catalog, symbol_tree_entry, true)?;

        Some(VirtualSnapshotQuery::Address {
            query_id: symbol_tree_entry.get_node_key().to_string(),
            address: symbol_tree_entry.get_locator().get_focus_address(),
            module_name: symbol_tree_entry
                .get_locator()
                .get_focus_module_name()
                .to_string(),
            symbolic_struct_definition,
        })
    }

    fn collect_preview_values_by_node_key(
        &self,
        symbol_tree_entries: &[SymbolTreeEntry],
    ) -> HashMap<String, String> {
        let Some(virtual_snapshot) = self
            .app_context
            .engine_unprivileged_state
            .get_virtual_snapshot(Self::PREVIEW_VALUES_VIRTUAL_SNAPSHOT_ID)
        else {
            return HashMap::new();
        };

        symbol_tree_entries
            .iter()
            .filter_map(|symbol_tree_entry| {
                let virtual_snapshot_query_result = virtual_snapshot
                    .get_query_results()
                    .get(symbol_tree_entry.get_node_key())?;
                let preview_value = self.build_symbol_preview_value(symbol_tree_entry, virtual_snapshot_query_result);

                (!preview_value.is_empty()).then(|| (symbol_tree_entry.get_node_key().to_string(), preview_value))
            })
            .collect()
    }

    fn build_symbol_preview_value(
        &self,
        symbol_tree_entry: &SymbolTreeEntry,
        virtual_snapshot_query_result: &VirtualSnapshotQueryResult,
    ) -> String {
        let Some(memory_read_response) = virtual_snapshot_query_result.memory_read_response.as_ref() else {
            return String::new();
        };

        if !memory_read_response.success {
            return String::new();
        }

        let Some(first_read_field_data_value) = memory_read_response
            .valued_struct
            .get_fields()
            .first()
            .and_then(|valued_struct_field| valued_struct_field.get_data_value())
        else {
            return String::new();
        };

        let default_anonymous_value_string_format = self
            .app_context
            .engine_unprivileged_state
            .get_default_anonymous_value_string_format(first_read_field_data_value.get_data_type_ref());

        self.app_context
            .engine_unprivileged_state
            .anonymize_value(first_read_field_data_value, default_anonymous_value_string_format)
            .map(|anonymous_value_string| {
                Self::format_symbol_preview_value(
                    &anonymous_value_string,
                    symbol_tree_entry.get_container_type(),
                    Self::symbol_preview_was_truncated(symbol_tree_entry),
                )
            })
            .unwrap_or_default()
    }

    fn symbol_preview_was_truncated(symbol_tree_entry: &SymbolTreeEntry) -> bool {
        matches!(
            symbol_tree_entry.get_container_type(),
            ContainerType::ArrayFixed(length) if length > Self::MAX_SYMBOL_PREVIEW_ELEMENT_COUNT
        )
    }

    fn format_symbol_preview_value(
        anonymous_value_string: &AnonymousValueString,
        symbolic_field_container_type: ContainerType,
        preview_was_truncated: bool,
    ) -> String {
        let effective_container_type = if matches!(anonymous_value_string.get_container_type(), ContainerType::Array | ContainerType::ArrayFixed(_)) {
            anonymous_value_string.get_container_type()
        } else {
            symbolic_field_container_type
        };
        let display_value = anonymous_value_string.get_anonymous_value_string();

        if matches!(effective_container_type, ContainerType::Array | ContainerType::ArrayFixed(_)) && !display_value.is_empty() {
            let preview_value = if preview_was_truncated {
                Self::append_symbol_preview_ellipsis(display_value)
            } else {
                Self::truncate_symbol_preview_value(display_value)
            };

            format!("[{}]", preview_value)
        } else {
            display_value.to_string()
        }
    }

    fn append_symbol_preview_ellipsis(display_value: &str) -> String {
        if let Some(truncated_array_preview) = Self::format_symbol_preview_from_elements(display_value, true) {
            return truncated_array_preview;
        }

        let trimmed_display_value = display_value.trim_end_matches(|character: char| character.is_ascii_whitespace() || matches!(character, ',' | ';'));

        if trimmed_display_value.is_empty() {
            String::from("...")
        } else {
            format!("{}...", trimmed_display_value)
        }
    }

    fn truncate_symbol_preview_value(display_value: &str) -> String {
        if let Some(truncated_array_preview) = Self::format_symbol_preview_from_elements(display_value, false) {
            return truncated_array_preview;
        }

        let display_value_character_count = display_value.chars().count();

        if display_value_character_count <= Self::MAX_SYMBOL_PREVIEW_ARRAY_CHARACTER_COUNT {
            return display_value.to_string();
        }

        let truncated_prefix: String = display_value
            .chars()
            .take(Self::MAX_SYMBOL_PREVIEW_ARRAY_CHARACTER_COUNT)
            .collect::<String>()
            .trim_end_matches(|character: char| character.is_ascii_whitespace() || matches!(character, ',' | ';'))
            .to_string();

        format!("{}...", truncated_prefix)
    }

    fn format_symbol_preview_from_elements(
        display_value: &str,
        force_ellipsis: bool,
    ) -> Option<String> {
        let array_elements = Self::split_symbol_preview_elements(display_value);

        if array_elements.len() <= 1 {
            return None;
        }

        let visible_element_count = array_elements
            .len()
            .min(Self::MAX_SYMBOL_PREVIEW_DISPLAY_ELEMENT_COUNT);
        let mut preview_elements = array_elements
            .iter()
            .take(visible_element_count)
            .map(|array_element| (*array_element).to_string())
            .collect::<Vec<_>>();
        let has_hidden_elements = force_ellipsis || array_elements.len() > visible_element_count;

        if has_hidden_elements {
            preview_elements.push(String::from("..."));
        }

        Some(preview_elements.join(", "))
    }

    fn split_symbol_preview_elements(display_value: &str) -> Vec<&str> {
        display_value
            .split([',', ';'])
            .map(str::trim)
            .filter(|array_element| !array_element.is_empty())
            .collect::<Vec<_>>()
    }

    fn format_symbol_tree_size_preview(size_in_bytes: u64) -> String {
        const KIB: f64 = 1024.0;
        const SIZE_UNITS: [&str; 6] = ["B", "KB", "MB", "GB", "TB", "PB"];

        if size_in_bytes < 1024 {
            return format!("{} B", size_in_bytes);
        }

        let mut unit_index = 0_usize;
        let mut size_value = size_in_bytes as f64;

        while size_value >= KIB && unit_index + 1 < SIZE_UNITS.len() {
            size_value /= KIB;
            unit_index += 1;
        }

        let formatted_value = if size_value >= 100.0 {
            format!("{:.0}", size_value)
        } else if size_value >= 10.0 {
            format!("{:.1}", size_value)
        } else {
            format!("{:.2}", size_value)
        };
        let formatted_value = formatted_value
            .trim_end_matches('0')
            .trim_end_matches('.')
            .to_string();

        format!("{} {}", formatted_value, SIZE_UNITS[unit_index])
    }

    fn format_symbol_tree_size_tooltip(size_in_bytes: u64) -> String {
        if size_in_bytes < 1024 {
            String::new()
        } else {
            format!("{} bytes", size_in_bytes)
        }
    }

    fn draw_text_button(
        &self,
        user_interface: &mut Ui,
        label: &str,
        fill_color: Color32,
        enabled: bool,
        minimum_width: f32,
    ) -> Response {
        let theme = &self.app_context.theme;
        let text_color = if enabled { theme.foreground } else { theme.foreground_preview };
        let label_galley = user_interface
            .painter()
            .layout_no_wrap(label.to_string(), theme.font_library.font_noto_sans.font_normal.clone(), text_color);
        let desired_width = (label_galley.size().x + 18.0).max(minimum_width);
        let button_response = user_interface.add_sized(
            [desired_width, 28.0],
            ThemeButton::new_from_theme(theme)
                .disabled(!enabled)
                .background_color(fill_color),
        );
        let text_position = pos2(
            button_response.rect.center().x - label_galley.size().x * 0.5,
            button_response.rect.center().y - label_galley.size().y * 0.5,
        );

        user_interface
            .painter()
            .galley(text_position, label_galley, text_color);

        button_response
    }

    fn draw_sized_action_button(
        &self,
        user_interface: &mut Ui,
        label: &str,
        button_size: eframe::egui::Vec2,
        fill_color: Color32,
        border_color: Color32,
        click_enabled: bool,
    ) -> Response {
        let theme = &self.app_context.theme;
        let text_color = theme.foreground;
        let label_galley = user_interface
            .painter()
            .layout_no_wrap(label.to_string(), theme.font_library.font_noto_sans.font_normal.clone(), text_color);
        let button_response = user_interface.add_sized(
            button_size,
            ThemeButton::new_from_theme(theme)
                .disabled(!click_enabled)
                .background_color(fill_color)
                .border_width(1.0)
                .border_color(border_color),
        );
        let text_position = pos2(
            button_response.rect.center().x - label_galley.size().x * 0.5,
            button_response.rect.center().y - label_galley.size().y * 0.5,
        );

        user_interface
            .painter()
            .galley(text_position, label_galley, text_color);

        button_response
    }

    fn render_string_data_value_box(
        &self,
        user_interface: &mut Ui,
        value_text: &mut String,
        preview_text: &str,
        id: &str,
        width: f32,
    ) {
        let mut anonymous_value_string = AnonymousValueString::new(value_text.clone(), AnonymousValueStringFormat::String, ContainerType::None);
        let string_data_type = DataTypeRef::new(Self::STRING_DATA_TYPE_ID);

        user_interface.add(
            DataValueBoxView::new(
                self.app_context.clone(),
                &mut anonymous_value_string,
                &string_data_type,
                false,
                true,
                preview_text,
                id,
            )
            .width(width.max(1.0))
            .height(Self::TOOLBAR_HEIGHT)
            .show_format_button(false)
            .use_format_text_colors(false),
        );

        let next_value_text = anonymous_value_string.get_anonymous_value_string().to_string();
        if *value_text != next_value_text {
            *value_text = next_value_text;
        }
    }

    fn render_offset_data_value_box(
        &self,
        user_interface: &mut Ui,
        value_text: &mut String,
        value_format: &mut AnonymousValueStringFormat,
        preview_text: &str,
        id: &str,
        width: f32,
    ) {
        let mut anonymous_value_string = AnonymousValueString::new(value_text.clone(), *value_format, ContainerType::None);
        let unsigned_integer_data_type = DataTypeRef::new(DataTypeU64::DATA_TYPE_ID);

        user_interface.add(
            DataValueBoxView::new(
                self.app_context.clone(),
                &mut anonymous_value_string,
                &unsigned_integer_data_type,
                false,
                true,
                preview_text,
                id,
            )
            .width(width.max(1.0))
            .height(Self::TOOLBAR_HEIGHT)
            .allowed_anonymous_value_string_formats(vec![
                AnonymousValueStringFormat::Binary,
                AnonymousValueStringFormat::Decimal,
                AnonymousValueStringFormat::Hexadecimal,
            ])
            .use_format_text_colors(true),
        );

        let next_value_text = anonymous_value_string.get_anonymous_value_string().to_string();
        if *value_text != next_value_text {
            *value_text = next_value_text;
        }

        let next_value_format = anonymous_value_string.get_anonymous_value_string_format();
        if *value_format != next_value_format {
            *value_format = next_value_format;
        }
    }

    fn build_define_field_offset_warning(
        define_field_draft: &DefineFieldDraft,
        segment_length: u64,
        resolve_type_size: impl Fn(&str) -> Option<u64>,
    ) -> Option<String> {
        let relative_offset =
            match Self::parse_define_field_relative_offset(&define_field_draft.relative_offset_text, define_field_draft.relative_offset_format) {
                Ok(relative_offset) => relative_offset,
                Err(parse_error) => return Some(parse_error),
            };
        let struct_layout_id = Self::build_define_field_struct_layout_id(define_field_draft);
        let Some(field_size) = resolve_type_size(&struct_layout_id) else {
            return Some(format!("Cannot resolve byte size for `{}`.", struct_layout_id));
        };

        if field_size == 0 {
            return Some(format!("`{}` has no byte size.", struct_layout_id));
        }

        let relative_field_end = match relative_offset.checked_add(field_size) {
            Some(relative_field_end) => relative_field_end,
            None => return Some(String::from("Field range is too large.")),
        };

        if relative_field_end > segment_length {
            if field_size > segment_length {
                return Some(format!(
                    "`{}` uses {} byte(s); selected span has {}.",
                    struct_layout_id, field_size, segment_length
                ));
            }

            return Some(format!(
                "`{}` uses {} byte(s); choose 0 to {}.",
                struct_layout_id,
                field_size,
                segment_length.saturating_sub(field_size)
            ));
        }

        None
    }

    fn define_field_container_label(container_type: ContainerType) -> String {
        match container_type {
            ContainerType::None => String::from("Value"),
            _ => container_type
                .get_pointer_size()
                .map(|pointer_size| format!("Ptr {}", pointer_size))
                .unwrap_or_else(|| String::from("Value")),
        }
    }

    fn render_define_field_container_selector(
        &self,
        user_interface: &mut Ui,
        container_type: &mut ContainerType,
        pointer_sizes: &[PointerScanPointerSize],
        menu_id: &str,
        width: f32,
    ) {
        let mut selected_container_type = None;
        if let Some(selected_pointer_size) = container_type.get_pointer_size() {
            if !pointer_sizes.contains(&selected_pointer_size) {
                *container_type = ContainerType::None;
            }
        }
        let current_label = Self::define_field_container_label(*container_type);

        user_interface.add(
            ComboBoxView::new(
                self.app_context.clone(),
                current_label,
                menu_id,
                None,
                |popup_user_interface: &mut Ui, should_close: &mut bool| {
                    let value_response = popup_user_interface.add(ComboBoxItemView::new(self.app_context.clone(), "Value", None, width));

                    if value_response.clicked() {
                        selected_container_type = Some(ContainerType::None);
                        *should_close = true;
                    }

                    popup_user_interface.separator();

                    for pointer_size in pointer_sizes {
                        let pointer_label = format!("Ptr {}", pointer_size);
                        let pointer_response = popup_user_interface.add(ComboBoxItemView::new(self.app_context.clone(), &pointer_label, None, width));

                        if pointer_response.clicked() {
                            selected_container_type = Some(ContainerType::Pointer(*pointer_size));
                            *should_close = true;
                        }
                    }
                },
            )
            .width(width)
            .height(Self::TOOLBAR_HEIGHT),
        );

        if let Some(selected_container_type) = selected_container_type {
            *container_type = selected_container_type;
        }
    }

    fn render_module_field_type_combo(
        &self,
        user_interface: &mut Ui,
        project_symbol_catalog: &ProjectSymbolCatalog,
        data_type_selection: &mut crate::ui::widgets::controls::data_type_selector::data_type_selection::DataTypeSelection,
        menu_id: &str,
        width: f32,
    ) {
        let type_options = Self::build_module_field_type_options(project_symbol_catalog);
        let selected_data_type_id = data_type_selection.visible_data_type().get_data_type_id();
        let selected_type_option = type_options
            .iter()
            .find(|type_option| type_option.data_type_ref.get_data_type_id() == selected_data_type_id);
        let combo_label = selected_type_option
            .map(|type_option| type_option.label.clone())
            .unwrap_or_else(|| DataTypeToStringConverter::convert_data_type_to_string(selected_data_type_id));
        let combo_icon = selected_type_option.and_then(|type_option| {
            Self::module_field_type_option_uses_icon(type_option.kind)
                .then(|| DataTypeToIconConverter::convert_data_type_to_icon(type_option.data_type_ref.get_data_type_id(), &self.app_context.theme.icon_library))
        });
        let search_storage_id = Self::module_field_type_search_storage_id(menu_id);

        user_interface.add(
            ComboBoxView::new(
                self.app_context.clone(),
                combo_label,
                menu_id,
                combo_icon,
                |popup_user_interface: &mut Ui, should_close: &mut bool| {
                    let mut search_text = popup_user_interface
                        .ctx()
                        .data_mut(|data| data.get_temp::<String>(search_storage_id).unwrap_or_default());

                    popup_user_interface.add_space(4.0);
                    self.render_string_data_value_box(
                        popup_user_interface,
                        &mut search_text,
                        "Search types",
                        &format!("symbol_explorer_module_field_type_search_{}", menu_id),
                        (popup_user_interface.available_width() - 8.0).max(1.0),
                    );
                    popup_user_interface.add_space(4.0);

                    popup_user_interface
                        .ctx()
                        .data_mut(|data| data.insert_temp(search_storage_id, search_text.clone()));

                    let filtered_type_options = Self::filter_module_field_type_options(&type_options, &search_text);

                    if filtered_type_options.is_empty() {
                        popup_user_interface.label(RichText::new("No matching types").color(self.app_context.theme.foreground_preview));
                        return;
                    }

                    ScrollArea::vertical()
                        .max_height(240.0)
                        .auto_shrink([false, false])
                        .show(popup_user_interface, |scroll_user_interface| {
                            for type_option in filtered_type_options {
                                let row_icon = if Self::module_field_type_option_uses_icon(type_option.kind) {
                                    Some(DataTypeToIconConverter::convert_data_type_to_icon(
                                        type_option.data_type_ref.get_data_type_id(),
                                        &self.app_context.theme.icon_library,
                                    ))
                                } else {
                                    None
                                };
                                let item_response =
                                    scroll_user_interface.add(ComboBoxItemView::new(self.app_context.clone(), &type_option.label, row_icon, width));

                                if item_response.clicked() {
                                    data_type_selection.select_single_data_type(type_option.data_type_ref);
                                    scroll_user_interface
                                        .ctx()
                                        .data_mut(|data| data.insert_temp(search_storage_id, String::new()));
                                    *should_close = true;
                                }
                            }
                        });
                },
            )
            .width(width)
            .height(Self::TOOLBAR_HEIGHT),
        );
    }

    fn render_delete_confirmation_take_over(
        &self,
        user_interface: &mut Ui,
        title: &str,
        display_name: &str,
        description_text: &str,
        is_description_warning: bool,
    ) -> bool {
        let theme = &self.app_context.theme;
        let mut did_confirm_delete = false;
        let description_color = if is_description_warning { theme.warning } else { theme.foreground_preview };

        user_interface.allocate_ui_with_layout(
            user_interface.available_size(),
            Layout::centered_and_justified(Direction::TopDown),
            |user_interface| {
                let panel_width = user_interface.available_width();

                user_interface.add(
                    GroupBox::new_from_theme(theme, title, |user_interface| {
                        user_interface.vertical_centered(|user_interface| {
                            user_interface.label(
                                RichText::new(display_name)
                                    .font(theme.font_library.font_ubuntu_mono_bold.font_header.clone())
                                    .color(theme.foreground),
                            );
                            user_interface.add_space(6.0);
                            user_interface.label(RichText::new(description_text).color(description_color));
                        });

                        user_interface.add_space(12.0);
                        user_interface.allocate_ui(vec2(user_interface.available_width(), 32.0), |user_interface| {
                            let button_size = vec2(120.0, 28.0);
                            let button_spacing = 12.0;
                            let total_button_row_width = button_size.x * 2.0 + button_spacing;
                            let side_spacing = ((user_interface.available_width() - total_button_row_width) * 0.5).max(0.0);

                            user_interface.horizontal(|user_interface| {
                                user_interface.add_space(side_spacing);
                                user_interface.spacing_mut().item_spacing.x = button_spacing;

                                let button_cancel = self.draw_sized_action_button(
                                    user_interface,
                                    "Cancel",
                                    button_size,
                                    theme.background_control_secondary,
                                    theme.background_control_secondary_dark,
                                    true,
                                );

                                if button_cancel.clicked() {
                                    SymbolExplorerViewData::cancel_take_over_state(self.symbol_explorer_view_data.clone());
                                }

                                let button_confirm_delete = user_interface.add_sized(
                                    button_size,
                                    eframe::egui::Button::new(RichText::new("Delete").color(theme.foreground))
                                        .fill(theme.background_control_danger)
                                        .stroke(Stroke::new(1.0, theme.background_control_danger_dark)),
                                );

                                if button_confirm_delete.clicked() {
                                    did_confirm_delete = true;
                                }
                            });
                        });
                    })
                    .desired_width(panel_width),
                );
            },
        );

        did_confirm_delete
    }

    fn render_define_field_take_over(
        &self,
        user_interface: &mut Ui,
        project_symbol_catalog: &ProjectSymbolCatalog,
        module_name: &str,
        segment_offset: u64,
        segment_length: u64,
        define_field_draft: &DefineFieldDraft,
    ) {
        let theme = &self.app_context.theme;
        let mut edited_define_field_draft = define_field_draft.clone();
        let mut define_field_plan_result = Err(String::from("Field is not ready."));
        let mut should_cancel_take_over = false;
        let mut should_create_field = false;

        user_interface.allocate_ui_with_layout(
            user_interface.available_size(),
            Layout::centered_and_justified(Direction::TopDown),
            |user_interface| {
                let panel_width = user_interface.available_width();

                user_interface.add(
                    GroupBox::new_from_theme(theme, "Define Field", |user_interface| {
                        user_interface.label(RichText::new(format!("{} + 0x{:X}", module_name, segment_offset)).color(theme.foreground_preview));
                        user_interface.add_space(8.0);

                        user_interface.label(RichText::new("Name").color(theme.foreground));
                        user_interface.add_space(2.0);
                        self.render_string_data_value_box(
                            user_interface,
                            &mut edited_define_field_draft.display_name,
                            "field_name",
                            "symbol_explorer_define_field_name",
                            user_interface.available_width(),
                        );
                        user_interface.add_space(8.0);

                        let max_relative_offset = segment_length.saturating_sub(1);
                        user_interface.label(RichText::new(format!("Offset in u8[] (0 to {})", max_relative_offset)).color(theme.foreground));
                        user_interface.add_space(2.0);
                        self.render_offset_data_value_box(
                            user_interface,
                            &mut edited_define_field_draft.relative_offset_text,
                            &mut edited_define_field_draft.relative_offset_format,
                            "0",
                            "symbol_explorer_define_field_offset",
                            user_interface.available_width(),
                        );

                        if let Some(offset_warning) = Self::build_define_field_offset_warning(&edited_define_field_draft, segment_length, |struct_layout_id| {
                            self.resolve_define_field_struct_layout_id_size(project_symbol_catalog, struct_layout_id)
                        }) {
                            user_interface.add_space(4.0);
                            user_interface.label(RichText::new(offset_warning).color(theme.warning));
                        }
                        user_interface.add_space(8.0);

                        user_interface.horizontal(|user_interface| {
                            user_interface.spacing_mut().item_spacing.x = 4.0;
                            let pointer_sizes = Self::filter_registered_pointer_sizes(
                                &self
                                    .app_context
                                    .engine_unprivileged_state
                                    .get_registered_data_type_refs(),
                            );
                            let selector_width = Self::DEFINE_FIELD_CONTAINER_SELECTOR_WIDTH.min(user_interface.available_width());
                            self.render_define_field_container_selector(
                                user_interface,
                                &mut edited_define_field_draft.container_type,
                                &pointer_sizes,
                                &format!("symbol_explorer_define_field_container_{}_{}", module_name, segment_offset),
                                selector_width,
                            );

                            let type_selector_width = user_interface.available_width();
                            self.render_module_field_type_combo(
                                user_interface,
                                project_symbol_catalog,
                                &mut edited_define_field_draft.data_type_selection,
                                &format!("symbol_explorer_define_field_type_{}_{}", module_name, segment_offset),
                                type_selector_width,
                            );
                        });

                        define_field_plan_result =
                            Self::build_define_field_plan(&edited_define_field_draft, module_name, segment_offset, segment_length, |struct_layout_id| {
                                self.resolve_define_field_struct_layout_id_size(project_symbol_catalog, struct_layout_id)
                            });

                        if let Err(validation_error) = define_field_plan_result.as_ref() {
                            if validation_error == "Field name is required." {
                                user_interface.add_space(6.0);
                                user_interface.label(RichText::new(validation_error).color(theme.error_red));
                            }
                        }

                        user_interface.add_space(12.0);
                        user_interface.allocate_ui(vec2(user_interface.available_width(), 32.0), |user_interface| {
                            let button_size = vec2(120.0, 28.0);
                            let button_spacing = 12.0;
                            let total_button_row_width = button_size.x * 2.0 + button_spacing;
                            let side_spacing = ((user_interface.available_width() - total_button_row_width) * 0.5).max(0.0);

                            user_interface.horizontal(|user_interface| {
                                user_interface.add_space(side_spacing);
                                user_interface.spacing_mut().item_spacing.x = button_spacing;

                                let button_cancel = user_interface.add_sized(
                                    button_size,
                                    eframe::egui::Button::new(RichText::new("Cancel").color(theme.foreground))
                                        .fill(theme.background_control_secondary)
                                        .stroke(Stroke::new(1.0, theme.background_control_secondary_dark)),
                                );

                                if button_cancel.clicked() {
                                    should_cancel_take_over = true;
                                }

                                let can_create_field = define_field_plan_result.is_ok();
                                let create_fill = if can_create_field {
                                    theme.background_control_primary
                                } else {
                                    theme.background_control_secondary
                                };
                                let create_stroke = if can_create_field {
                                    theme.background_control_primary_dark
                                } else {
                                    theme.background_control_secondary_dark
                                };
                                let button_create =
                                    self.draw_sized_action_button(user_interface, "Create", button_size, create_fill, create_stroke, can_create_field);

                                if button_create.clicked() {
                                    should_create_field = true;
                                }
                            });
                        });
                    })
                    .desired_width(panel_width),
                );
            },
        );

        if should_cancel_take_over {
            SymbolExplorerViewData::cancel_take_over_state(self.symbol_explorer_view_data.clone());
            return;
        }

        if should_create_field {
            if let Ok(define_field_plan) = define_field_plan_result {
                SymbolExplorerViewData::cancel_take_over_state(self.symbol_explorer_view_data.clone());
                self.create_define_field_from_u8_span_edit_target(module_name, define_field_plan);
                return;
            }
        }

        SymbolExplorerViewData::set_define_field_draft(self.symbol_explorer_view_data.clone(), edited_define_field_draft);
    }

    fn calculate_symbol_tree_context_menu_width(
        app_context: &AppContext,
        user_interface: &mut Ui,
        item_labels: &[&str],
    ) -> f32 {
        let mut longest_label_width: f32 = 0.0;

        user_interface.ctx().fonts_mut(|fonts| {
            for item_label in item_labels {
                let galley = fonts.layout_no_wrap(
                    (*item_label).to_string(),
                    app_context
                        .theme
                        .font_library
                        .font_noto_sans
                        .font_normal
                        .clone(),
                    app_context.theme.foreground,
                );
                longest_label_width = longest_label_width.max(galley.size().x);
            }
        });

        ToolbarMenuItemView::row_width_from_text_width(longest_label_width).ceil()
    }

    #[allow(dead_code)]
    fn render_symbol_tree_list_legacy(
        &self,
        user_interface: &mut Ui,
        symbol_tree_entries: &[SymbolTreeEntry],
        selected_entry: Option<&SymbolExplorerSelection>,
    ) {
        user_interface.label(
            RichText::new(format!(
                "Symbol Tree ({})",
                symbol_tree_entries
                    .iter()
                    .filter(|symbol_tree_entry| matches!(symbol_tree_entry.get_kind(), SymbolTreeEntryKind::ModuleSpace { .. }))
                    .count()
            ))
            .font(
                self.app_context
                    .theme
                    .font_library
                    .font_noto_sans
                    .font_header
                    .clone(),
            )
            .color(self.app_context.theme.foreground),
        );
        user_interface.add_space(6.0);

        for symbol_tree_entry in symbol_tree_entries {
            let is_selected = matches!(
                selected_entry,
                Some(SymbolExplorerSelection::SymbolClaim(selected_symbol_locator_key))
                    if matches!(symbol_tree_entry.get_kind(), SymbolTreeEntryKind::SymbolClaim { symbol_locator_key } if selected_symbol_locator_key == symbol_locator_key)
            ) || matches!(
                selected_entry,
                Some(SymbolExplorerSelection::DerivedNode(selected_node_key)) if selected_node_key == symbol_tree_entry.get_node_key()
            );

            user_interface.horizontal(|user_interface| {
                user_interface.add_space(symbol_tree_entry.get_depth() as f32 * 16.0);

                if symbol_tree_entry.can_expand() {
                    let expansion_label = if symbol_tree_entry.is_expanded() { "▾" } else { "▸" };

                    if self
                        .draw_text_button(user_interface, expansion_label, self.app_context.theme.background_control_secondary, true, 24.0)
                        .clicked()
                    {
                        SymbolExplorerViewData::toggle_tree_node_expansion(self.symbol_explorer_view_data.clone(), symbol_tree_entry.get_node_key());
                    }
                } else {
                    user_interface.add_space(24.0);
                }

                let row_label = format!(
                    "{}  [{}{}]",
                    symbol_tree_entry.get_display_name(),
                    symbol_tree_entry.get_symbol_type_id(),
                    symbol_tree_entry.get_container_type()
                );
                let response = user_interface.selectable_label(is_selected, RichText::new(row_label).color(self.app_context.theme.foreground));

                if response.clicked() {
                    let selection = match symbol_tree_entry.get_kind() {
                        SymbolTreeEntryKind::ModuleSpace { module_name, .. } => Some(SymbolExplorerSelection::ModuleRoot(module_name.to_string())),
                        SymbolTreeEntryKind::U8Segment { .. } => Some(SymbolExplorerSelection::DerivedNode(symbol_tree_entry.get_node_key().to_string())),
                        SymbolTreeEntryKind::SymbolClaim { symbol_locator_key } => Some(SymbolExplorerSelection::SymbolClaim(symbol_locator_key.to_string())),
                        SymbolTreeEntryKind::StructField | SymbolTreeEntryKind::PointerTarget => {
                            Some(SymbolExplorerSelection::DerivedNode(symbol_tree_entry.get_node_key().to_string()))
                        }
                    };

                    if let Some(selection) = selection {
                        SymbolExplorerViewData::set_selected_entry(self.symbol_explorer_view_data.clone(), Some(selection));
                    }
                }
            });

            user_interface.label(
                RichText::new(symbol_tree_entry.get_locator().to_string())
                    .font(
                        self.app_context
                            .theme
                            .font_library
                            .font_noto_sans
                            .font_small
                            .clone(),
                    )
                    .color(self.app_context.theme.foreground_preview),
            );
            user_interface.add_space(6.0);
        }
    }

    fn render_symbol_tree_list(
        &self,
        user_interface: &mut Ui,
        project_symbol_catalog: &ProjectSymbolCatalog,
        symbol_tree_entries: &[SymbolTreeEntry],
        preview_values_by_node_key: &HashMap<String, String>,
        selected_entry: Option<&SymbolExplorerSelection>,
        inline_rename_tree_node_key: Option<&str>,
        context_menu_target: Option<&SymbolExplorerContextMenuTarget>,
        shared_struct_viewer_focus_target: Option<&StructViewerFocusTarget>,
        allow_interaction: bool,
    ) {
        user_interface.label(
            RichText::new(format!(
                "Symbol Tree ({})",
                symbol_tree_entries
                    .iter()
                    .filter(|symbol_tree_entry| matches!(symbol_tree_entry.get_kind(), SymbolTreeEntryKind::ModuleSpace { .. }))
                    .count()
            ))
            .font(
                self.app_context
                    .theme
                    .font_library
                    .font_noto_sans
                    .font_header
                    .clone(),
            )
            .color(self.app_context.theme.foreground),
        );
        user_interface.add_space(6.0);

        for symbol_tree_entry in symbol_tree_entries {
            let is_locally_selected = matches!(
                selected_entry,
                Some(SymbolExplorerSelection::ModuleRoot(selected_module_name))
                    if matches!(symbol_tree_entry.get_kind(), SymbolTreeEntryKind::ModuleSpace { module_name, .. } if selected_module_name == module_name)
            ) || matches!(
                selected_entry,
                Some(SymbolExplorerSelection::SymbolClaim(selected_symbol_locator_key))
                    if matches!(symbol_tree_entry.get_kind(), SymbolTreeEntryKind::SymbolClaim { symbol_locator_key } if selected_symbol_locator_key == symbol_locator_key)
            ) || matches!(
                selected_entry,
                Some(SymbolExplorerSelection::DerivedNode(selected_node_key)) if selected_node_key == symbol_tree_entry.get_node_key()
            );
            let is_selected = is_locally_selected
                && (matches!(symbol_tree_entry.get_kind(), SymbolTreeEntryKind::ModuleSpace { .. })
                    || Self::is_symbol_tree_entry_struct_viewer_focused(symbol_tree_entry, shared_struct_viewer_focus_target));

            let is_inline_rename_row = inline_rename_tree_node_key
                .is_some_and(|active_inline_rename_tree_node_key| symbol_tree_entry.get_node_key() == active_inline_rename_tree_node_key);

            if is_inline_rename_row {
                let rename_target_key = symbol_tree_entry.get_node_key();
                let rename_text_storage_id = Self::inline_rename_text_storage_id(rename_target_key);
                let rename_highlight_storage_id = Self::inline_rename_highlight_storage_id(rename_target_key);
                let mut rename_text = user_interface
                    .ctx()
                    .data_mut(|data| data.get_temp::<String>(rename_text_storage_id))
                    .unwrap_or_else(|| symbol_tree_entry.get_display_name().to_string());
                let mut should_highlight_text = user_interface
                    .ctx()
                    .data_mut(|data| data.get_temp::<bool>(rename_highlight_storage_id))
                    .unwrap_or(true);
                let inline_rename_response = SymbolTreeInlineRenameView::new(
                    self.app_context.clone(),
                    rename_target_key,
                    symbol_tree_entry,
                    &mut rename_text,
                    &mut should_highlight_text,
                    is_selected,
                )
                .show(user_interface);

                if inline_rename_response.should_commit {
                    let trimmed_rename_text = rename_text.trim().to_string();

                    if !trimmed_rename_text.is_empty() && trimmed_rename_text != symbol_tree_entry.get_display_name() {
                        match symbol_tree_entry.get_kind() {
                            SymbolTreeEntryKind::ModuleSpace { module_name, .. } => self.rename_module_root(module_name, trimmed_rename_text),
                            SymbolTreeEntryKind::SymbolClaim { symbol_locator_key } => self.rename_symbol_claim(symbol_locator_key, trimmed_rename_text),
                            SymbolTreeEntryKind::U8Segment { module_name, offset, length } => {
                                self.rename_u8_segment(module_name, *offset, *length, trimmed_rename_text)
                            }
                            _ => {}
                        }
                    }

                    self.clear_inline_rename_state(user_interface, rename_target_key);
                }

                if inline_rename_response.should_cancel {
                    self.clear_inline_rename_state(user_interface, rename_target_key);
                }

                user_interface.ctx().data_mut(|data| {
                    data.insert_temp(rename_text_storage_id, rename_text);
                    data.insert_temp(rename_highlight_storage_id, should_highlight_text);
                });

                continue;
            }

            let preview_value = preview_values_by_node_key
                .get(symbol_tree_entry.get_node_key())
                .map(String::as_str)
                .unwrap_or("");
            let size_in_bytes = resolve_symbol_tree_entry_size_in_bytes(project_symbol_catalog, symbol_tree_entry, |data_type_ref| {
                self.app_context
                    .engine_unprivileged_state
                    .get_default_value(data_type_ref)
                    .map(|default_value| default_value.get_size_in_bytes())
            });
            let size_preview_text = Self::format_symbol_tree_size_preview(size_in_bytes);
            let size_tooltip_text = Self::format_symbol_tree_size_tooltip(size_in_bytes);
            let symbol_tree_entry_view_response = SymbolTreeEntryView::new(
                self.app_context.clone(),
                symbol_tree_entry,
                &size_preview_text,
                &size_tooltip_text,
                preview_value,
                is_selected,
            )
            .show(user_interface);

            if allow_interaction && symbol_tree_entry_view_response.did_click_expand_arrow {
                if let Some(selection) = Self::build_selection_for_tree_entry(symbol_tree_entry) {
                    SymbolExplorerViewData::set_selected_entry(self.symbol_explorer_view_data.clone(), Some(selection));
                }

                SymbolExplorerViewData::toggle_tree_node_expansion(self.symbol_explorer_view_data.clone(), symbol_tree_entry.get_node_key());
            }

            if allow_interaction && symbol_tree_entry_view_response.row_response.double_clicked() && !symbol_tree_entry_view_response.did_click_expand_arrow {
                if let Some(selection) = Self::build_selection_for_tree_entry(symbol_tree_entry) {
                    SymbolExplorerViewData::set_selected_entry(self.symbol_explorer_view_data.clone(), Some(selection));
                }

                if let Some(u8_span_edit_target) = Self::build_u8_span_edit_target(symbol_tree_entry) {
                    SymbolExplorerViewData::begin_define_field_from_u8_segment(
                        self.symbol_explorer_view_data.clone(),
                        u8_span_edit_target.module_name,
                        u8_span_edit_target.offset,
                        u8_span_edit_target.length,
                    );
                } else if !matches!(symbol_tree_entry.get_kind(), SymbolTreeEntryKind::ModuleSpace { .. }) {
                    self.focus_symbol_tree_entry_for_edit(project_symbol_catalog, symbol_tree_entry);
                }

                continue;
            }

            if allow_interaction && symbol_tree_entry_view_response.did_click_row {
                let Some(selection) = Self::build_selection_for_tree_entry(symbol_tree_entry) else {
                    continue;
                };

                SymbolExplorerViewData::set_selected_entry(self.symbol_explorer_view_data.clone(), Some(selection));
                if !matches!(symbol_tree_entry.get_kind(), SymbolTreeEntryKind::ModuleSpace { .. }) {
                    self.focus_symbol_tree_entry_in_struct_viewer(project_symbol_catalog, symbol_tree_entry);
                }
            }

            if allow_interaction && symbol_tree_entry_view_response.row_response.secondary_clicked() {
                let Some(selection) = Self::build_selection_for_tree_entry(symbol_tree_entry) else {
                    continue;
                };
                let context_menu_position = symbol_tree_entry_view_response
                    .row_response
                    .interact_pointer_pos()
                    .unwrap_or(symbol_tree_entry_view_response.row_response.rect.left_top());

                SymbolExplorerViewData::set_selected_entry(self.symbol_explorer_view_data.clone(), Some(selection));
                if !matches!(symbol_tree_entry.get_kind(), SymbolTreeEntryKind::ModuleSpace { .. }) {
                    self.focus_symbol_tree_entry_in_struct_viewer(project_symbol_catalog, symbol_tree_entry);
                }
                SymbolExplorerViewData::show_context_menu(
                    self.symbol_explorer_view_data.clone(),
                    SymbolExplorerContextMenuTarget::new(symbol_tree_entry.get_node_key().to_string(), context_menu_position),
                );
            }

            if allow_interaction
                && context_menu_target
                    .as_ref()
                    .is_some_and(|context_menu_target| context_menu_target.get_tree_node_key() == symbol_tree_entry.get_node_key())
            {
                let can_open_symbol_tree_entry = !matches!(symbol_tree_entry.get_kind(), SymbolTreeEntryKind::ModuleSpace { .. });
                let can_rename_symbol_tree_entry = matches!(
                    symbol_tree_entry.get_kind(),
                    SymbolTreeEntryKind::ModuleSpace { .. } | SymbolTreeEntryKind::SymbolClaim { .. } | SymbolTreeEntryKind::U8Segment { .. }
                );
                let context_menu_symbol_claim = match symbol_tree_entry.get_kind() {
                    SymbolTreeEntryKind::SymbolClaim { symbol_locator_key } => project_symbol_catalog
                        .get_symbol_claims()
                        .iter()
                        .find(|symbol_claim| symbol_claim.get_symbol_locator_key() == *symbol_locator_key),
                    _ => None,
                };
                let context_menu_module_name = match symbol_tree_entry.get_kind() {
                    SymbolTreeEntryKind::ModuleSpace { module_name, .. } => Some(module_name.as_str()),
                    _ => None,
                };
                let context_menu_module_child_range_target =
                    Self::build_module_child_range_target(project_symbol_catalog, symbol_tree_entry, |data_type_ref| {
                        self.app_context
                            .engine_unprivileged_state
                            .get_default_value(data_type_ref)
                            .map(|default_value| default_value.get_size_in_bytes())
                    });
                let context_menu_u8_span_edit_target = Self::build_u8_span_edit_target(symbol_tree_entry);
                let can_delete_symbol_tree_entry =
                    context_menu_module_child_range_target.is_some() || context_menu_symbol_claim.is_some() || context_menu_module_name.is_some();
                let mut context_menu_labels = Vec::new();
                if can_open_symbol_tree_entry {
                    context_menu_labels.extend(["Open in Memory Viewer", "Open in Code Viewer"]);
                }
                if context_menu_u8_span_edit_target.is_some() {
                    context_menu_labels.push("Define Field...");
                    context_menu_labels.push("Split in Half");
                }
                if can_rename_symbol_tree_entry {
                    context_menu_labels.push("Rename");
                }
                context_menu_labels.push("New Module");
                if can_delete_symbol_tree_entry {
                    context_menu_labels.push("Delete");
                }
                let context_menu_width = Self::calculate_symbol_tree_context_menu_width(self.app_context.as_ref(), user_interface, &context_menu_labels);
                let mut is_context_menu_open = true;

                ContextMenu::new(
                    self.app_context.clone(),
                    "symbol_tree_context_menu",
                    context_menu_target
                        .as_ref()
                        .map(|context_menu_target| context_menu_target.get_position())
                        .unwrap_or(symbol_tree_entry_view_response.row_response.rect.left_top()),
                    |user_interface, should_close| {
                        if can_open_symbol_tree_entry {
                            if user_interface
                                .add(
                                    ToolbarMenuItemView::new(
                                        self.app_context.clone(),
                                        "Open in Memory Viewer",
                                        "symbol_tree_ctx_open_memory_viewer",
                                        &None,
                                        context_menu_width,
                                    )
                                    .icon(
                                        self.app_context
                                            .theme
                                            .icon_library
                                            .icon_handle_scan_collect_values
                                            .clone(),
                                    ),
                                )
                                .clicked()
                            {
                                self.focus_memory_viewer_for_locator(symbol_tree_entry.get_locator());
                                *should_close = true;
                            }

                            if user_interface
                                .add(
                                    ToolbarMenuItemView::new(
                                        self.app_context.clone(),
                                        "Open in Code Viewer",
                                        "symbol_tree_ctx_open_code_viewer",
                                        &None,
                                        context_menu_width,
                                    )
                                    .icon(
                                        self.app_context
                                            .theme
                                            .icon_library
                                            .icon_handle_project_cpu_instruction
                                            .clone(),
                                    ),
                                )
                                .clicked()
                            {
                                self.focus_code_viewer_for_locator(symbol_tree_entry.get_locator());
                                *should_close = true;
                            }
                        }

                        if can_open_symbol_tree_entry && can_rename_symbol_tree_entry {
                            user_interface.separator();
                        }

                        if let Some(u8_span_edit_target) = context_menu_u8_span_edit_target.as_ref() {
                            if user_interface
                                .add(
                                    ToolbarMenuItemView::new(
                                        self.app_context.clone(),
                                        "Define Field...",
                                        "symbol_tree_ctx_define_field",
                                        &None,
                                        context_menu_width,
                                    )
                                    .icon(
                                        self.app_context
                                            .theme
                                            .icon_library
                                            .icon_handle_data_type_blue_blocks_4
                                            .clone(),
                                    ),
                                )
                                .clicked()
                            {
                                SymbolExplorerViewData::begin_define_field_from_u8_segment(
                                    self.symbol_explorer_view_data.clone(),
                                    u8_span_edit_target.module_name.clone(),
                                    u8_span_edit_target.offset,
                                    u8_span_edit_target.length,
                                );
                                *should_close = true;
                            }

                            if user_interface
                                .add(
                                    ToolbarMenuItemView::new(
                                        self.app_context.clone(),
                                        "Split in Half",
                                        "symbol_tree_ctx_split_in_half",
                                        &None,
                                        context_menu_width,
                                    )
                                    .icon(
                                        self.app_context
                                            .theme
                                            .icon_library
                                            .icon_handle_data_type_purple_blocks_array
                                            .clone(),
                                    ),
                                )
                                .clicked()
                            {
                                self.split_u8_span_edit_target_in_half(u8_span_edit_target);
                                *should_close = true;
                            }
                        }

                        if context_menu_u8_span_edit_target.is_some() && can_rename_symbol_tree_entry {
                            user_interface.separator();
                        }

                        if can_rename_symbol_tree_entry
                            && user_interface
                                .add(
                                    ToolbarMenuItemView::new(self.app_context.clone(), "Rename", "symbol_tree_ctx_rename", &None, context_menu_width).icon(
                                        self.app_context
                                            .theme
                                            .icon_library
                                            .icon_handle_common_edit
                                            .clone(),
                                    ),
                                )
                                .clicked()
                        {
                            SymbolExplorerViewData::begin_inline_rename(self.symbol_explorer_view_data.clone(), symbol_tree_entry.get_node_key().to_string());
                            *should_close = true;
                        }

                        if can_open_symbol_tree_entry || context_menu_u8_span_edit_target.is_some() || can_rename_symbol_tree_entry {
                            user_interface.separator();
                        }

                        if user_interface
                            .add(
                                ToolbarMenuItemView::new(self.app_context.clone(), "New Module", "symbol_tree_ctx_new_module", &None, context_menu_width).icon(
                                    self.app_context
                                        .theme
                                        .icon_library
                                        .icon_handle_common_add
                                        .clone(),
                                ),
                            )
                            .clicked()
                        {
                            SymbolExplorerViewData::begin_create_module_root(self.symbol_explorer_view_data.clone());
                            *should_close = true;
                        }

                        if can_delete_symbol_tree_entry {
                            user_interface.separator();

                            if user_interface
                                .add(
                                    ToolbarMenuItemView::new(self.app_context.clone(), "Delete", "symbol_tree_ctx_delete", &None, context_menu_width).icon(
                                        self.app_context
                                            .theme
                                            .icon_library
                                            .icon_handle_common_delete
                                            .clone(),
                                    ),
                                )
                                .clicked()
                            {
                                self.request_delete_for_selection(
                                    context_menu_symbol_claim,
                                    context_menu_module_name,
                                    context_menu_module_child_range_target.as_ref(),
                                );
                                *should_close = true;
                            }
                        }
                    },
                )
                .width(context_menu_width)
                .corner_radius(8)
                .show(user_interface, &mut is_context_menu_open);

                if !is_context_menu_open {
                    SymbolExplorerViewData::hide_context_menu(self.symbol_explorer_view_data.clone());
                }
            }
        }
    }

    fn build_selection_for_tree_entry(symbol_tree_entry: &SymbolTreeEntry) -> Option<SymbolExplorerSelection> {
        match symbol_tree_entry.get_kind() {
            SymbolTreeEntryKind::ModuleSpace { module_name, .. } => Some(SymbolExplorerSelection::ModuleRoot(module_name.to_string())),
            SymbolTreeEntryKind::SymbolClaim { symbol_locator_key } => Some(SymbolExplorerSelection::SymbolClaim(symbol_locator_key.to_string())),
            SymbolTreeEntryKind::StructField | SymbolTreeEntryKind::U8Segment { .. } | SymbolTreeEntryKind::PointerTarget => {
                Some(SymbolExplorerSelection::DerivedNode(symbol_tree_entry.get_node_key().to_string()))
            }
        }
    }

    fn render_create_module_root_details(
        &self,
        user_interface: &mut Ui,
    ) {
        let original_draft = self
            .symbol_explorer_view_data
            .read("Symbol explorer module root create details")
            .map(|symbol_explorer_view_data| symbol_explorer_view_data.get_module_root_create_draft().clone())
            .unwrap_or_default();
        let mut edited_draft = original_draft.clone();
        let details_width = user_interface.available_width().max(1.0);

        user_interface.add(
            GroupBox::new_from_theme(&self.app_context.theme, "New Module", |user_interface| {
                user_interface.label(RichText::new("Module Name").color(self.app_context.theme.foreground));
                self.render_string_data_value_box(
                    user_interface,
                    &mut edited_draft.module_name,
                    "Module name",
                    Self::CREATE_DISPLAY_NAME_DATA_VALUE_BOX_ID,
                    user_interface.available_width(),
                );
                user_interface.add_space(6.0);

                user_interface.label(RichText::new("Initial u8[] Size").color(self.app_context.theme.foreground));
                user_interface.add(TextEdit::singleline(&mut edited_draft.size_text).hint_text("0x123400 or 1192960"));
            })
            .desired_width(details_width),
        );

        if edited_draft != original_draft {
            SymbolExplorerViewData::set_module_root_create_draft(self.symbol_explorer_view_data.clone(), edited_draft.clone());
        }
    }

    fn parse_u64_draft(numeric_draft: &str) -> Option<u64> {
        let trimmed_numeric_draft = numeric_draft.trim();

        if trimmed_numeric_draft.is_empty() {
            return None;
        }

        if let Some(hex_numeric_draft) = trimmed_numeric_draft
            .strip_prefix("0x")
            .or_else(|| trimmed_numeric_draft.strip_prefix("0X"))
        {
            u64::from_str_radix(hex_numeric_draft, 16).ok()
        } else {
            trimmed_numeric_draft.parse::<u64>().ok()
        }
    }

    fn build_module_root_create_request_from_draft(edited_draft: &ModuleRootCreateDraft) -> Option<ProjectSymbolsCreateModuleRequest> {
        let parsed_size = Self::parse_u64_draft(&edited_draft.size_text);

        if edited_draft.module_name.trim().is_empty() {
            return None;
        }

        Some(ProjectSymbolsCreateModuleRequest {
            module_name: edited_draft.module_name.trim().to_string(),
            size: parsed_size?,
        })
    }

    fn render_create_module_root_take_over(
        &self,
        user_interface: &mut Ui,
        create_module_root_request: Option<ProjectSymbolsCreateModuleRequest>,
    ) {
        let theme = &self.app_context.theme;

        user_interface.allocate_ui_with_layout(
            user_interface.available_size(),
            Layout::centered_and_justified(Direction::TopDown),
            |user_interface| {
                let panel_width = user_interface.available_width().min(520.0).max(320.0);

                ScrollArea::vertical()
                    .id_salt("symbol_explorer_create_module_root_take_over")
                    .auto_shrink([false, false])
                    .show(user_interface, |user_interface| {
                        user_interface.allocate_ui_with_layout(
                            vec2(panel_width, user_interface.available_height()),
                            Layout::top_down(Align::Min),
                            |user_interface| {
                                self.render_create_module_root_details(user_interface);
                                user_interface.add_space(12.0);
                                user_interface.horizontal_centered(|user_interface| {
                                    let button_size = [96.0, 30.0];
                                    let button_cancel = user_interface.add_sized(
                                        button_size,
                                        eframe::egui::Button::new(RichText::new("Cancel").color(theme.foreground))
                                            .fill(theme.background_control_secondary)
                                            .stroke(Stroke::new(1.0, theme.background_control_secondary_dark)),
                                    );

                                    if button_cancel.clicked() {
                                        SymbolExplorerViewData::set_selected_entry(self.symbol_explorer_view_data.clone(), None);
                                    }

                                    let can_create_module = create_module_root_request.is_some();
                                    let button_create = user_interface.add_enabled(
                                        can_create_module,
                                        eframe::egui::Button::new(RichText::new("Create").color(if can_create_module {
                                            theme.foreground
                                        } else {
                                            theme.foreground_preview
                                        }))
                                        .min_size(vec2(button_size[0], button_size[1]))
                                        .fill(theme.background_control_primary)
                                        .stroke(Stroke::new(1.0, theme.background_control_primary_dark)),
                                    );

                                    if button_create.clicked() {
                                        if let Some(project_symbols_create_module_request) = create_module_root_request.clone() {
                                            self.create_module_root(project_symbols_create_module_request);
                                        }
                                    }
                                });
                            },
                        );
                    });
            },
        );
    }
}

impl Widget for SymbolExplorerView {
    fn ui(
        self,
        user_interface: &mut Ui,
    ) -> Response {
        let Some(project_symbol_catalog) = self.get_opened_project_symbol_catalog() else {
            self.app_context
                .engine_unprivileged_state
                .set_virtual_snapshot_queries(Self::POINTER_CHILDREN_VIRTUAL_SNAPSHOT_ID, Self::POINTER_CHILDREN_REFRESH_INTERVAL, Vec::new());
            self.app_context
                .engine_unprivileged_state
                .set_virtual_snapshot_queries(Self::PREVIEW_VALUES_VIRTUAL_SNAPSHOT_ID, Self::PREVIEW_VALUES_REFRESH_INTERVAL, Vec::new());

            return user_interface
                .allocate_ui_with_layout(
                    user_interface.available_size(),
                    Layout::centered_and_justified(Direction::TopDown),
                    |user_interface| {
                        user_interface.label(RichText::new("Open a project to browse the Symbol Tree.").color(self.app_context.theme.foreground_preview));
                    },
                )
                .response;
        };

        let shared_struct_viewer_focus_target = self
            .struct_viewer_view_data
            .read("Symbol explorer shared struct viewer focus target")
            .and_then(|struct_viewer_view_data| struct_viewer_view_data.get_focus_target().cloned());
        let suppress_default_selection = matches!(
            shared_struct_viewer_focus_target,
            Some(StructViewerFocusTarget::ProjectHierarchy { .. }) | Some(StructViewerFocusTarget::SymbolTable { .. })
        );

        SymbolExplorerViewData::synchronize_selection(self.symbol_explorer_view_data.clone(), &project_symbol_catalog, suppress_default_selection);
        SymbolExplorerViewData::synchronize_inline_rename(self.symbol_explorer_view_data.clone(), &project_symbol_catalog);
        SymbolExplorerViewData::synchronize_take_over_state(self.symbol_explorer_view_data.clone(), &project_symbol_catalog);
        let expanded_tree_node_keys = self
            .symbol_explorer_view_data
            .read("Symbol explorer expanded tree nodes")
            .map(|symbol_explorer_view_data| symbol_explorer_view_data.get_expanded_tree_node_keys().clone())
            .unwrap_or_default();
        let structural_symbol_tree_entries = build_symbol_tree_entries(&project_symbol_catalog, &expanded_tree_node_keys, &HashMap::new(), |data_type_ref| {
            self.app_context
                .engine_unprivileged_state
                .get_default_value(data_type_ref)
                .map(|default_value| default_value.get_size_in_bytes())
        });
        self.sync_pointer_child_virtual_snapshot(&project_symbol_catalog, &structural_symbol_tree_entries);
        let resolved_pointer_targets_by_node_key = self.collect_resolved_pointer_targets_by_node_key();
        let symbol_tree_entries = build_symbol_tree_entries(
            &project_symbol_catalog,
            &expanded_tree_node_keys,
            &resolved_pointer_targets_by_node_key,
            |data_type_ref| {
                self.app_context
                    .engine_unprivileged_state
                    .get_default_value(data_type_ref)
                    .map(|default_value| default_value.get_size_in_bytes())
            },
        );
        self.sync_symbol_preview_virtual_snapshot(&project_symbol_catalog, &symbol_tree_entries);
        let preview_values_by_node_key = self.collect_preview_values_by_node_key(&symbol_tree_entries);
        SymbolExplorerViewData::synchronize_selection_to_tree_entries(self.symbol_explorer_view_data.clone(), &symbol_tree_entries);
        let (selected_entry, take_over_state, inline_rename_tree_node_key, context_menu_target, current_module_root_create_draft, current_define_field_draft) =
            self.symbol_explorer_view_data
                .read("Symbol explorer view")
                .map(|symbol_explorer_view_data| {
                    (
                        symbol_explorer_view_data.get_selected_entry().cloned(),
                        symbol_explorer_view_data.get_take_over_state().cloned(),
                        symbol_explorer_view_data
                            .get_inline_rename_tree_node_key()
                            .map(str::to_string),
                        symbol_explorer_view_data.get_context_menu_target().cloned(),
                        symbol_explorer_view_data.get_module_root_create_draft().clone(),
                        symbol_explorer_view_data.get_define_field_draft().clone(),
                    )
                })
                .unwrap_or((None, None, None, None, Default::default(), Default::default()));
        let selected_symbol_claim = match selected_entry.as_ref() {
            Some(SymbolExplorerSelection::SymbolClaim(selected_symbol_locator_key)) => project_symbol_catalog
                .get_symbol_claims()
                .iter()
                .find(|symbol_claim| symbol_claim.get_symbol_locator_key() == *selected_symbol_locator_key),
            _ => None,
        };
        let selected_module_name = match selected_entry.as_ref() {
            Some(SymbolExplorerSelection::ModuleRoot(module_name)) if project_symbol_catalog.find_symbol_module(module_name).is_some() => {
                Some(module_name.to_string())
            }
            _ => None,
        };
        let selected_symbol_tree_entry = Self::build_selected_symbol_tree_entry(&symbol_tree_entries, selected_entry.as_ref());
        let selected_module_child_range_target = selected_symbol_tree_entry.and_then(|symbol_tree_entry| {
            Self::build_module_child_range_target(&project_symbol_catalog, symbol_tree_entry, |data_type_ref| {
                self.app_context
                    .engine_unprivileged_state
                    .get_default_value(data_type_ref)
                    .map(|default_value| default_value.get_size_in_bytes())
            })
        });
        let create_module_root_request = match selected_entry.as_ref() {
            Some(SymbolExplorerSelection::CreateModuleRoot) => Self::build_module_root_create_request_from_draft(&current_module_root_create_draft),
            _ => None,
        };
        self.sync_selected_symbol_into_struct_viewer(&project_symbol_catalog, selected_symbol_tree_entry);
        let theme = self.app_context.theme.clone();
        let is_delete_confirmation_active = take_over_state.is_some();
        let is_inline_rename_active = inline_rename_tree_node_key.is_some();
        let is_create_module_root_active = matches!(selected_entry.as_ref(), Some(SymbolExplorerSelection::CreateModuleRoot));
        let can_use_standard_toolbar_actions = !is_delete_confirmation_active && !is_inline_rename_active && !is_create_module_root_active;
        let is_window_focused = self
            .app_context
            .window_focus_manager
            .is_window_focused(Self::WINDOW_ID);
        let can_handle_window_shortcuts = self
            .app_context
            .window_focus_manager
            .can_window_handle_shortcuts(user_interface.ctx(), Self::WINDOW_ID);

        if is_window_focused && is_delete_confirmation_active && user_interface.input(|input_state| input_state.key_pressed(Key::Enter)) {
            match take_over_state.as_ref() {
                Some(SymbolExplorerTakeOverState::DeleteSymbolClaimConfirmation { symbol_locator_key, .. }) => self.delete_symbol_claim(symbol_locator_key),
                Some(SymbolExplorerTakeOverState::DeleteModuleRootConfirmation { module_name }) => self.delete_module_root(module_name),
                Some(SymbolExplorerTakeOverState::DeleteModuleRangeConfirmation {
                    module_name,
                    offset,
                    length,
                    mode,
                    ..
                }) => self.delete_module_range(module_name, *offset, *length, *mode),
                Some(SymbolExplorerTakeOverState::DefineFieldFromU8Segment { .. }) => {}
                None => {}
            }
        }

        if is_window_focused && is_delete_confirmation_active && user_interface.input(|input_state| input_state.key_pressed(Key::Escape)) {
            SymbolExplorerViewData::cancel_take_over_state(self.symbol_explorer_view_data.clone());
        }

        if is_window_focused && is_create_module_root_active && user_interface.input(|input_state| input_state.key_pressed(Key::Escape)) {
            SymbolExplorerViewData::set_selected_entry(self.symbol_explorer_view_data.clone(), None);
        }

        if is_window_focused && is_create_module_root_active && user_interface.input(|input_state| input_state.key_pressed(Key::Enter)) {
            if let Some(project_symbols_create_module_request) = create_module_root_request.clone() {
                self.create_module_root(project_symbols_create_module_request);
            }
        }

        if !is_delete_confirmation_active
            && !is_inline_rename_active
            && !is_create_module_root_active
            && can_handle_window_shortcuts
            && user_interface.input(|input_state| input_state.key_pressed(Key::Delete))
        {
            self.request_delete_for_selection(
                selected_symbol_claim,
                selected_module_name.as_deref(),
                selected_module_child_range_target.as_ref(),
            );
        }

        let can_rename_selected_entry = selected_symbol_tree_entry.is_some_and(|symbol_tree_entry| {
            matches!(
                symbol_tree_entry.get_kind(),
                SymbolTreeEntryKind::ModuleSpace { .. } | SymbolTreeEntryKind::SymbolClaim { .. } | SymbolTreeEntryKind::U8Segment { .. }
            )
        });
        let can_open_selected_entry =
            selected_symbol_tree_entry.is_some_and(|symbol_tree_entry| !matches!(symbol_tree_entry.get_kind(), SymbolTreeEntryKind::ModuleSpace { .. }));

        if !is_delete_confirmation_active
            && !is_inline_rename_active
            && can_handle_window_shortcuts
            && user_interface.input(|input_state| input_state.key_pressed(Key::F2))
        {
            if can_rename_selected_entry {
                if let Some(symbol_tree_entry) = selected_symbol_tree_entry {
                    SymbolExplorerViewData::begin_inline_rename(self.symbol_explorer_view_data.clone(), symbol_tree_entry.get_node_key().to_string());
                }
            }
        }

        if is_window_focused && is_inline_rename_active && user_interface.input(|input_state| input_state.key_pressed(Key::Escape)) {
            if let Some(active_inline_rename_tree_node_key) = inline_rename_tree_node_key.as_deref() {
                self.clear_inline_rename_state(user_interface, active_inline_rename_tree_node_key);
            }
        }

        user_interface
            .allocate_ui_with_layout(user_interface.available_size(), Layout::top_down(Align::Min), |user_interface| {
                let mut list_user_interface = user_interface.new_child(
                    UiBuilder::new()
                        .max_rect(user_interface.available_rect_before_wrap())
                        .layout(Layout::top_down(Align::Min)),
                );
                let toolbar_action = SymbolExplorerToolbarView::new(self.app_context.clone())
                    .can_create_module_root(can_use_standard_toolbar_actions)
                    .can_rename_selected_entry(can_rename_selected_entry && can_use_standard_toolbar_actions)
                    .can_delete_selected_entry(
                        (selected_module_child_range_target.is_some() || selected_symbol_claim.is_some() || selected_module_name.is_some())
                            && can_use_standard_toolbar_actions,
                    )
                    .can_open_in_code_viewer(can_open_selected_entry && can_use_standard_toolbar_actions)
                    .can_open_in_memory_viewer(can_open_selected_entry && can_use_standard_toolbar_actions)
                    .show(&mut list_user_interface);

                match toolbar_action {
                    Some(SymbolExplorerToolbarAction::CreateModuleRoot) => {
                        SymbolExplorerViewData::begin_create_module_root(self.symbol_explorer_view_data.clone());
                    }
                    Some(SymbolExplorerToolbarAction::RenameSelectedEntry) => {
                        if can_rename_selected_entry {
                            if let Some(symbol_tree_entry) = selected_symbol_tree_entry {
                                SymbolExplorerViewData::begin_inline_rename(
                                    self.symbol_explorer_view_data.clone(),
                                    symbol_tree_entry.get_node_key().to_string(),
                                );
                            }
                        }
                    }
                    Some(SymbolExplorerToolbarAction::DeleteSelectedEntry) => {
                        self.request_delete_for_selection(
                            selected_symbol_claim,
                            selected_module_name.as_deref(),
                            selected_module_child_range_target.as_ref(),
                        );
                    }
                    Some(SymbolExplorerToolbarAction::OpenSelectedInCodeViewer) => {
                        if let Some(symbol_tree_entry) = selected_symbol_tree_entry {
                            self.focus_code_viewer_for_locator(symbol_tree_entry.get_locator());
                        }
                    }
                    Some(SymbolExplorerToolbarAction::OpenSelectedInMemoryViewer) => {
                        if let Some(symbol_tree_entry) = selected_symbol_tree_entry {
                            self.focus_memory_viewer_for_locator(symbol_tree_entry.get_locator());
                        }
                    }
                    None => {}
                }

                match take_over_state.as_ref() {
                    Some(SymbolExplorerTakeOverState::DeleteSymbolClaimConfirmation {
                        symbol_locator_key,
                        display_name,
                    }) => {
                        list_user_interface.add_space(8.0);
                        if self.render_delete_confirmation_take_over(
                            &mut list_user_interface,
                            "Delete this symbol",
                            display_name,
                            "This removes the authored symbol from the project.",
                            false,
                        ) {
                            self.delete_symbol_claim(symbol_locator_key);
                        }

                        return;
                    }
                    Some(SymbolExplorerTakeOverState::DeleteModuleRootConfirmation { module_name }) => {
                        list_user_interface.add_space(8.0);
                        if self.render_delete_confirmation_take_over(
                            &mut list_user_interface,
                            "Delete this module",
                            module_name,
                            "This removes the module root and all symbol claims inside it.",
                            false,
                        ) {
                            self.delete_module_root(module_name);
                        }

                        return;
                    }
                    Some(SymbolExplorerTakeOverState::DeleteModuleRangeConfirmation {
                        module_name,
                        offset,
                        length,
                        display_name,
                        mode,
                    }) => {
                        let delete_confirmation_description = Self::build_delete_module_range_confirmation_description(module_name, *length, *mode);

                        list_user_interface.add_space(8.0);
                        if self.render_delete_confirmation_take_over(
                            &mut list_user_interface,
                            "Delete this field",
                            display_name,
                            &delete_confirmation_description.text,
                            delete_confirmation_description.is_warning,
                        ) {
                            self.delete_module_range(module_name, *offset, *length, *mode);
                        }

                        return;
                    }
                    Some(SymbolExplorerTakeOverState::DefineFieldFromU8Segment {
                        module_name, offset, length, ..
                    }) => {
                        list_user_interface.add_space(8.0);
                        self.render_define_field_take_over(
                            &mut list_user_interface,
                            &project_symbol_catalog,
                            module_name,
                            *offset,
                            *length,
                            &current_define_field_draft,
                        );

                        return;
                    }
                    None => {}
                }

                if matches!(selected_entry.as_ref(), Some(SymbolExplorerSelection::CreateModuleRoot)) {
                    list_user_interface.add_space(8.0);
                    self.render_create_module_root_take_over(&mut list_user_interface, create_module_root_request.clone());

                    return;
                }

                list_user_interface.add_space(8.0);
                ScrollArea::vertical()
                    .id_salt("symbol_explorer_list")
                    .auto_shrink([false, false])
                    .show(&mut list_user_interface, |user_interface| {
                        self.render_symbol_tree_list(
                            user_interface,
                            &project_symbol_catalog,
                            &symbol_tree_entries,
                            &preview_values_by_node_key,
                            selected_entry.as_ref(),
                            inline_rename_tree_node_key.as_deref(),
                            context_menu_target.as_ref(),
                            shared_struct_viewer_focus_target.as_ref(),
                            !is_inline_rename_active,
                        );
                        if project_symbol_catalog.is_empty() {
                            user_interface.add_space(12.0);
                            user_interface.label(
                                RichText::new("This project has no authored symbols yet.")
                                    .font(theme.font_library.font_noto_sans.font_normal.clone())
                                    .color(theme.foreground_preview),
                            );
                        }
                    });
            })
            .response
    }
}

#[cfg(test)]
mod tests {
    use super::{ModuleFieldTypeOption, ModuleFieldTypeOptionKind, SymbolExplorerView};
    use crate::ui::widgets::controls::data_type_selector::data_type_selection::DataTypeSelection;
    use crate::views::struct_viewer::view_data::{struct_viewer_focus_target::StructViewerFocusTarget, struct_viewer_view_data::StructViewerViewData};
    use crate::views::symbol_explorer::view_data::symbol_explorer_view_data::DefineFieldDraft;
    use crate::views::symbol_explorer::view_data::symbol_tree_entry::{SymbolTreeEntry, SymbolTreeEntryKind};
    use squalr_engine_api::commands::project_symbols::delete::project_symbols_delete_request::ProjectSymbolsDeleteModuleRangeMode;
    use squalr_engine_api::registries::symbols::struct_layout_descriptor::StructLayoutDescriptor;
    use squalr_engine_api::structures::{
        data_types::{
            built_in_types::{string::utf8::data_type_string_utf8::DataTypeStringUtf8, u32::data_type_u32::DataTypeU32},
            data_type_ref::DataTypeRef,
        },
        data_values::{anonymous_value_string_format::AnonymousValueStringFormat, container_type::ContainerType},
        pointer_scans::pointer_scan_pointer_size::PointerScanPointerSize,
        projects::{
            project_items::built_in_types::project_item_type_address::ProjectItemTypeAddress, project_symbol_catalog::ProjectSymbolCatalog,
            project_symbol_claim::ProjectSymbolClaim, project_symbol_locator::ProjectSymbolLocator,
        },
        structs::{symbolic_field_definition::SymbolicFieldDefinition, symbolic_struct_definition::SymbolicStructDefinition, valued_struct::ValuedStruct},
    };

    fn create_symbol_claim_tree_entry(
        display_name: &str,
        symbol_type_id: &str,
    ) -> SymbolTreeEntry {
        SymbolTreeEntry::new(
            String::from("claim:absolute:1234"),
            SymbolTreeEntryKind::SymbolClaim {
                symbol_locator_key: String::from("absolute:1234"),
            },
            1,
            display_name.to_string(),
            display_name.to_string(),
            String::from("absolute:1234"),
            ProjectSymbolLocator::new_absolute_address(0x1234),
            symbol_type_id.to_string(),
            ContainerType::None,
            false,
            false,
        )
    }

    fn create_module_tree_entry(module_name: &str) -> SymbolTreeEntry {
        SymbolTreeEntry::new(
            format!("module:{}", module_name),
            SymbolTreeEntryKind::ModuleSpace {
                module_name: module_name.to_string(),
                size: 0x2000,
            },
            0,
            module_name.to_string(),
            module_name.to_string(),
            String::new(),
            ProjectSymbolLocator::new_absolute_address(0),
            String::from("u8"),
            ContainerType::ArrayFixed(0x2000),
            true,
            false,
        )
    }

    fn create_module_symbol_claim_tree_entry() -> SymbolTreeEntry {
        SymbolTreeEntry::new(
            String::from("claim:module:game.exe:4"),
            SymbolTreeEntryKind::SymbolClaim {
                symbol_locator_key: String::from("module:game.exe:4"),
            },
            1,
            String::from("Health"),
            String::from("game.exe.Health"),
            String::from("module:game.exe:4"),
            ProjectSymbolLocator::new_module_offset(String::from("game.exe"), 0x04),
            String::from("u32"),
            ContainerType::None,
            false,
            false,
        )
    }

    fn create_module_u8_array_symbol_claim_tree_entry() -> SymbolTreeEntry {
        SymbolTreeEntry::new(
            String::from("claim:module:game.exe:20"),
            SymbolTreeEntryKind::SymbolClaim {
                symbol_locator_key: String::from("module:game.exe:20"),
            },
            1,
            String::from("u8_00000020"),
            String::from("game.exe.u8_00000020"),
            String::from("module:game.exe:20"),
            ProjectSymbolLocator::new_module_offset(String::from("game.exe"), 0x20),
            String::from("u8"),
            ContainerType::ArrayFixed(0x80),
            false,
            false,
        )
    }

    fn create_u8_segment_tree_entry() -> SymbolTreeEntry {
        SymbolTreeEntry::new(
            String::from("u8:game.exe:0:1234"),
            SymbolTreeEntryKind::U8Segment {
                module_name: String::from("game.exe"),
                offset: 0,
                length: 0x1234,
            },
            1,
            String::from("u8_00000000"),
            String::from("game.exe.u8_00000000"),
            String::new(),
            ProjectSymbolLocator::new_module_offset(String::from("game.exe"), 0),
            String::from("u8"),
            ContainerType::ArrayFixed(0x1234),
            false,
            false,
        )
    }

    fn create_fixed_array_symbol_claim_tree_entry(
        data_type_id: &str,
        array_length: u64,
    ) -> SymbolTreeEntry {
        SymbolTreeEntry::new(
            format!("claim:module:game.exe:40:{}", data_type_id),
            SymbolTreeEntryKind::SymbolClaim {
                symbol_locator_key: String::from("module:game.exe:40"),
            },
            1,
            format!("{}_array", data_type_id),
            format!("game.exe.{}_array", data_type_id),
            String::from("module:game.exe:40"),
            ProjectSymbolLocator::new_module_offset(String::from("game.exe"), 0x40),
            data_type_id.to_string(),
            ContainerType::ArrayFixed(array_length),
            false,
            false,
        )
    }

    #[test]
    fn build_u8_span_edit_target_handles_all_visible_u8_arrays() {
        let u8_segment_entry = create_u8_segment_tree_entry();
        let module_u8_array_symbol_claim_entry = create_module_u8_array_symbol_claim_tree_entry();

        let u8_segment_target = SymbolExplorerView::build_u8_span_edit_target(&u8_segment_entry).expect("Expected visible u8[] target.");
        let u8_field_target = SymbolExplorerView::build_u8_span_edit_target(&module_u8_array_symbol_claim_entry).expect("Expected visible u8[] target.");

        assert_eq!(u8_segment_target.module_name, "game.exe");
        assert_eq!(u8_segment_target.offset, 0);
        assert_eq!(u8_segment_target.length, 0x1234);
        assert_eq!(u8_field_target.module_name, "game.exe");
        assert_eq!(u8_field_target.offset, 0x20);
        assert_eq!(u8_field_target.length, 0x80);
    }

    #[test]
    fn build_u8_span_edit_target_ignores_typed_module_fields() {
        let module_symbol_claim_entry = create_module_symbol_claim_tree_entry();

        assert_eq!(SymbolExplorerView::build_u8_span_edit_target(&module_symbol_claim_entry), None);
    }

    #[test]
    fn struct_viewer_focus_target_key_includes_display_name_and_type() {
        let player_entry = create_symbol_claim_tree_entry("Player", "i32");
        let manager_entry = create_symbol_claim_tree_entry("Player Manager", "u64");

        let player_focus_key = SymbolExplorerView::build_struct_viewer_focus_target_key(Some(&player_entry));
        let manager_focus_key = SymbolExplorerView::build_struct_viewer_focus_target_key(Some(&manager_entry));

        assert_ne!(player_focus_key, manager_focus_key);
    }

    #[test]
    fn symbol_tree_entry_is_struct_viewer_focused_when_focus_target_matches_row_key() {
        let player_entry = create_symbol_claim_tree_entry("Player", "i32");
        let focus_target = SymbolExplorerView::build_struct_viewer_focus_target(Some(&player_entry));

        assert!(SymbolExplorerView::is_symbol_tree_entry_struct_viewer_focused(
            &player_entry,
            focus_target.as_ref(),
        ));
    }

    #[test]
    fn symbol_tree_entry_is_not_struct_viewer_focused_for_other_origin() {
        let player_entry = create_symbol_claim_tree_entry("Player", "i32");
        let focus_target = StructViewerFocusTarget::ProjectHierarchy {
            project_item_paths: Vec::new(),
        };

        assert!(!SymbolExplorerView::is_symbol_tree_entry_struct_viewer_focused(
            &player_entry,
            Some(&focus_target),
        ));
    }

    #[test]
    fn resolve_first_editable_struct_viewer_field_name_skips_read_only_fields() {
        let valued_struct = ValuedStruct::new_anonymous(vec![
            DataTypeStringUtf8::get_value_from_primitive_string("u32").to_named_valued_struct_field(String::from("type"), true),
            DataTypeStringUtf8::get_value_from_primitive_string("123").to_named_valued_struct_field(String::from("value"), false),
        ]);

        assert_eq!(
            SymbolExplorerView::resolve_first_editable_struct_viewer_field_name(&valued_struct),
            Some(String::from("value"))
        );
    }

    #[test]
    fn build_selection_for_tree_entry_selects_module_roots_and_u8_segments() {
        let module_entry = create_module_tree_entry("game.exe");
        let u8_segment_entry = create_u8_segment_tree_entry();

        assert_eq!(
            SymbolExplorerView::build_selection_for_tree_entry(&module_entry),
            Some(crate::views::symbol_explorer::view_data::symbol_explorer_view_data::SymbolExplorerSelection::ModuleRoot(String::from("game.exe")))
        );
        assert_eq!(
            SymbolExplorerView::build_selection_for_tree_entry(&u8_segment_entry),
            Some(crate::views::symbol_explorer::view_data::symbol_explorer_view_data::SymbolExplorerSelection::DerivedNode(String::from("u8:game.exe:0:1234")))
        );
    }

    #[test]
    fn symbol_tree_entry_preview_queries_include_u8_segments_but_not_modules() {
        let module_entry = create_module_tree_entry("game.exe");
        let u8_segment_entry = create_u8_segment_tree_entry();

        assert!(!SymbolExplorerView::symbol_tree_entry_should_query_preview(&module_entry));
        assert!(SymbolExplorerView::symbol_tree_entry_should_query_preview(&u8_segment_entry));
    }

    #[test]
    fn format_symbol_tree_size_preview_uses_scaled_byte_units() {
        assert_eq!(SymbolExplorerView::format_symbol_tree_size_preview(4), "4 B");
        assert_eq!(SymbolExplorerView::format_symbol_tree_size_preview(1024), "1 KB");
        assert_eq!(SymbolExplorerView::format_symbol_tree_size_preview(1536), "1.5 KB");
        assert_eq!(SymbolExplorerView::format_symbol_tree_size_preview(1024 * 1024), "1 MB");
    }

    #[test]
    fn format_symbol_tree_size_tooltip_keeps_raw_bytes_for_kb_and_larger() {
        assert_eq!(SymbolExplorerView::format_symbol_tree_size_tooltip(512), "");
        assert_eq!(SymbolExplorerView::format_symbol_tree_size_tooltip(1536), "1536 bytes");
    }

    #[test]
    fn build_module_child_range_target_handles_u8_segments_and_direct_module_claims() {
        let project_symbol_catalog = ProjectSymbolCatalog::new_with_symbol_claims(
            Vec::new(),
            vec![ProjectSymbolClaim::new_module_offset(
                String::from("Health"),
                String::from("game.exe"),
                0x04,
                String::from("u32"),
            )],
        );
        let u8_segment_entry = create_u8_segment_tree_entry();
        let module_symbol_claim_entry = create_module_symbol_claim_tree_entry();
        let u8_segment_target = SymbolExplorerView::build_module_child_range_target(&project_symbol_catalog, &u8_segment_entry, |data_type_ref| {
            (data_type_ref.get_data_type_id() == "u8").then_some(1)
        })
        .expect("Expected u8 segment to resolve as a module child range.");
        let symbol_claim_target = SymbolExplorerView::build_module_child_range_target(&project_symbol_catalog, &module_symbol_claim_entry, |data_type_ref| {
            match data_type_ref.get_data_type_id() {
                "u8" => Some(1),
                "u32" => Some(4),
                _ => None,
            }
        })
        .expect("Expected direct module symbol claim to resolve as a module child range.");

        assert_eq!(u8_segment_target.module_name, "game.exe");
        assert_eq!(u8_segment_target.offset, 0);
        assert_eq!(u8_segment_target.length, 0x1234);
        assert_eq!(u8_segment_target.delete_mode, ProjectSymbolsDeleteModuleRangeMode::ShiftLeft);
        assert_eq!(symbol_claim_target.module_name, "game.exe");
        assert_eq!(symbol_claim_target.offset, 0x04);
        assert_eq!(symbol_claim_target.length, 4);
        assert_eq!(symbol_claim_target.delete_mode, ProjectSymbolsDeleteModuleRangeMode::ReplaceWithU8);
    }

    #[test]
    fn build_delete_module_range_confirmation_description_marks_shift_left_as_warning() {
        let delete_confirmation_description =
            SymbolExplorerView::build_delete_module_range_confirmation_description("winmine.exe", 389, ProjectSymbolsDeleteModuleRangeMode::ShiftLeft);

        assert_eq!(
            delete_confirmation_description.text,
            "WARNING: winmine.exe will be 389 byte(s) smaller. Proceeding fields will be shifted left."
        );
        assert!(delete_confirmation_description.is_warning);
    }

    #[test]
    fn build_delete_module_range_confirmation_description_keeps_replace_with_u8_non_warning() {
        let delete_confirmation_description =
            SymbolExplorerView::build_delete_module_range_confirmation_description("winmine.exe", 389, ProjectSymbolsDeleteModuleRangeMode::ReplaceWithU8);

        assert_eq!(
            delete_confirmation_description.text,
            "This removes the field definition and preserves the module bytes as u8[]."
        );
        assert!(!delete_confirmation_description.is_warning);
    }

    #[test]
    fn build_module_field_type_options_includes_builtins_and_struct_layouts_without_pointer_variants() {
        let project_symbol_catalog = ProjectSymbolCatalog::new(vec![StructLayoutDescriptor::new(
            String::from("player.stats"),
            SymbolicStructDefinition::new(
                String::from("player.stats"),
                vec![SymbolicFieldDefinition::new(
                    DataTypeRef::new("u32"),
                    ContainerType::None,
                )],
            ),
        )]);
        let type_options = SymbolExplorerView::build_module_field_type_options(&project_symbol_catalog);

        assert!(
            type_options
                .iter()
                .any(|type_option| { type_option.data_type_ref == DataTypeRef::new("i32") && type_option.kind == ModuleFieldTypeOptionKind::BuiltIn })
        );
        assert!(type_options.iter().any(|type_option| {
            type_option.data_type_ref == DataTypeRef::new("player.stats") && type_option.kind == ModuleFieldTypeOptionKind::StructLayout
        }));
        assert!(
            !type_options
                .iter()
                .any(|type_option| type_option.data_type_ref == DataTypeRef::new("player.stats*(u64)"))
        );
    }

    #[test]
    fn filter_module_field_type_options_matches_struct_layouts() {
        let type_options = vec![
            ModuleFieldTypeOption {
                data_type_ref: DataTypeRef::new("i32"),
                label: String::from("i32"),
                kind: ModuleFieldTypeOptionKind::BuiltIn,
            },
            ModuleFieldTypeOption {
                data_type_ref: DataTypeRef::new("player.stats"),
                label: String::from("player.stats"),
                kind: ModuleFieldTypeOptionKind::StructLayout,
            },
        ];
        let filtered_type_options = SymbolExplorerView::filter_module_field_type_options(&type_options, "stats");

        assert_eq!(filtered_type_options.len(), 1);
        assert!(
            filtered_type_options
                .iter()
                .all(|type_option| { !SymbolExplorerView::module_field_type_option_uses_icon(type_option.kind) })
        );
    }

    #[test]
    fn filter_registered_pointer_sizes_omits_plugin_backed_sizes_when_unregistered() {
        let pointer_sizes = SymbolExplorerView::filter_registered_pointer_sizes(&[
            DataTypeRef::new("u32"),
            DataTypeRef::new("u32be"),
            DataTypeRef::new("u64"),
            DataTypeRef::new("u64be"),
        ]);

        assert_eq!(
            pointer_sizes,
            vec![
                PointerScanPointerSize::Pointer32,
                PointerScanPointerSize::Pointer32be,
                PointerScanPointerSize::Pointer64,
                PointerScanPointerSize::Pointer64be,
            ]
        );
    }

    #[test]
    fn filter_registered_pointer_sizes_includes_plugin_backed_sizes_when_registered() {
        let pointer_sizes = SymbolExplorerView::filter_registered_pointer_sizes(&[
            DataTypeRef::new("u24"),
            DataTypeRef::new("u24be"),
            DataTypeRef::new("u32"),
            DataTypeRef::new("u64"),
        ]);

        assert_eq!(
            pointer_sizes,
            vec![
                PointerScanPointerSize::Pointer24,
                PointerScanPointerSize::Pointer24be,
                PointerScanPointerSize::Pointer32,
                PointerScanPointerSize::Pointer64,
            ]
        );
    }

    #[test]
    fn build_define_field_plan_offsets_into_u8_segment() {
        let define_field_draft = DefineFieldDraft {
            display_name: String::from("health"),
            relative_offset_text: String::from("0x10"),
            relative_offset_format: AnonymousValueStringFormat::Hexadecimal,
            container_type: ContainerType::None,
            data_type_selection: DataTypeSelection::new(DataTypeRef::new("i32")),
        };
        let define_field_plan = SymbolExplorerView::build_define_field_plan(&define_field_draft, "game.exe", 0x100, 0x40, |struct_layout_id| {
            (struct_layout_id == "i32").then_some(4)
        })
        .expect("Expected valid define-field request.");

        assert_eq!(define_field_plan.project_symbols_create_request.display_name, "health");
        assert_eq!(
            define_field_plan
                .project_symbols_create_request
                .struct_layout_id,
            "i32"
        );
        assert_eq!(define_field_plan.project_symbols_create_request.module_name, Some(String::from("game.exe")));
        assert_eq!(define_field_plan.project_symbols_create_request.offset, Some(0x110));
    }

    #[test]
    fn build_define_field_plan_accepts_pointer_container_for_struct_selection() {
        let define_field_draft = DefineFieldDraft {
            display_name: String::from("player_stats_pointer"),
            relative_offset_text: String::from("0"),
            relative_offset_format: AnonymousValueStringFormat::Decimal,
            container_type: ContainerType::Pointer(PointerScanPointerSize::Pointer64),
            data_type_selection: DataTypeSelection::new(DataTypeRef::new("player.stats")),
        };
        let define_field_plan = SymbolExplorerView::build_define_field_plan(&define_field_draft, "game.exe", 0x100, 0x40, |struct_layout_id| {
            (struct_layout_id == "player.stats*(u64)").then_some(8)
        })
        .expect("Expected pointer-to-struct define-field request.");

        assert_eq!(
            define_field_plan
                .project_symbols_create_request
                .struct_layout_id,
            "player.stats*(u64)"
        );
        assert_eq!(define_field_plan.project_symbols_create_request.offset, Some(0x100));
    }

    #[test]
    fn build_define_field_plan_accepts_pointer_container_for_builtin_selection() {
        let define_field_draft = DefineFieldDraft {
            display_name: String::from("health_pointer"),
            relative_offset_text: String::from("0"),
            relative_offset_format: AnonymousValueStringFormat::Decimal,
            container_type: ContainerType::Pointer(PointerScanPointerSize::Pointer32),
            data_type_selection: DataTypeSelection::new(DataTypeRef::new("i32")),
        };
        let define_field_plan = SymbolExplorerView::build_define_field_plan(&define_field_draft, "game.exe", 0x100, 0x40, |struct_layout_id| {
            (struct_layout_id == "i32*(u32)").then_some(4)
        })
        .expect("Expected pointer-to-builtin define-field request.");

        assert_eq!(
            define_field_plan
                .project_symbols_create_request
                .struct_layout_id,
            "i32*(u32)"
        );
        assert_eq!(define_field_plan.project_symbols_create_request.offset, Some(0x100));
    }

    #[test]
    fn build_define_field_plan_rejects_out_of_bounds_type() {
        let define_field_draft = DefineFieldDraft {
            display_name: String::from("health"),
            relative_offset_text: String::from("0x3E"),
            relative_offset_format: AnonymousValueStringFormat::Hexadecimal,
            container_type: ContainerType::None,
            data_type_selection: DataTypeSelection::new(DataTypeRef::new("i32")),
        };
        let define_field_plan = SymbolExplorerView::build_define_field_plan(&define_field_draft, "game.exe", 0x100, 0x40, |struct_layout_id| {
            (struct_layout_id == "i32").then_some(4)
        });

        assert!(define_field_plan.is_err());
    }

    #[test]
    fn parse_define_field_relative_offset_accepts_hex_and_decimal() {
        assert_eq!(
            SymbolExplorerView::parse_define_field_relative_offset("0x10", AnonymousValueStringFormat::Decimal),
            Ok(16)
        );
        assert_eq!(
            SymbolExplorerView::parse_define_field_relative_offset("10", AnonymousValueStringFormat::Hexadecimal),
            Ok(16)
        );
        assert_eq!(
            SymbolExplorerView::parse_define_field_relative_offset("16", AnonymousValueStringFormat::Decimal),
            Ok(16)
        );
        assert_eq!(
            SymbolExplorerView::parse_define_field_relative_offset("10000", AnonymousValueStringFormat::Binary),
            Ok(16)
        );
    }

    #[test]
    fn normalize_symbol_memory_struct_prepends_claim_metadata_and_keeps_value_rows_editable() {
        let symbol_claim_tree_entry = create_symbol_claim_tree_entry("Player", "i32");
        let valued_struct = ValuedStruct::new_anonymous(vec![
            DataTypeU32::get_value_from_primitive(100).to_named_valued_struct_field(String::from("health"), false),
        ]);

        let normalized_struct = SymbolExplorerView::normalize_symbol_memory_struct(valued_struct, &symbol_claim_tree_entry, true, Some(4));
        let normalized_fields = normalized_struct.get_fields();

        assert_eq!(normalized_fields[0].get_name(), SymbolExplorerView::STRUCT_VIEWER_SYMBOL_NAME_FIELD);
        assert_eq!(
            normalized_fields[1].get_name(),
            ProjectItemTypeAddress::PROPERTY_SYMBOLIC_STRUCT_DEFINITION_REFERENCE
        );
        assert!(normalized_fields[1].get_is_read_only());
        assert_eq!(normalized_fields[2].get_name(), ProjectItemTypeAddress::PROPERTY_ADDRESS);
        assert_eq!(normalized_fields[3].get_name(), ProjectItemTypeAddress::PROPERTY_MODULE);
        assert_eq!(normalized_fields[4].get_name(), SymbolExplorerView::STRUCT_VIEWER_SYMBOL_SIZE_FIELD);
        assert_eq!(normalized_fields[5].get_name(), SymbolExplorerView::STRUCT_VIEWER_SYMBOL_PATH_FIELD);
        assert_eq!(normalized_fields[6].get_name(), "health");
        assert!(!normalized_fields[6].get_is_read_only());
    }

    #[test]
    fn build_external_value_symbol_struct_routes_arrays_through_memory_viewer_value_field() {
        let symbol_tree_entry = create_u8_segment_tree_entry();
        let symbol_struct = SymbolExplorerView::build_external_value_symbol_struct(&symbol_tree_entry, false, Some(0x1234));
        let fields = symbol_struct.get_fields();

        assert!(
            symbol_struct
                .get_field(ProjectItemTypeAddress::PROPERTY_SYMBOLIC_STRUCT_DEFINITION_REFERENCE)
                .is_some()
        );
        assert!(
            symbol_struct
                .get_field(ProjectItemTypeAddress::PROPERTY_FREEZE_DISPLAY_VALUE)
                .is_some()
        );
        assert_eq!(
            fields
                .iter()
                .find(|field| field.get_name() == ProjectItemTypeAddress::PROPERTY_SYMBOLIC_STRUCT_DEFINITION_REFERENCE)
                .map(|field| field.get_is_read_only()),
            Some(true)
        );
        assert_eq!(
            fields
                .iter()
                .find(|field| field.get_name() == ProjectItemTypeAddress::PROPERTY_FREEZE_DISPLAY_VALUE)
                .map(|field| field.get_is_read_only()),
            Some(true)
        );
    }

    #[test]
    fn build_external_value_symbol_struct_is_not_limited_to_u8_arrays() {
        let symbol_tree_entry = create_fixed_array_symbol_claim_tree_entry("u16", 4);

        assert!(SymbolExplorerView::symbol_tree_entry_should_use_external_value_viewer(&symbol_tree_entry));

        let symbol_struct = SymbolExplorerView::build_external_value_symbol_struct(&symbol_tree_entry, true, Some(8));

        assert_eq!(
            symbol_struct
                .get_field(ProjectItemTypeAddress::PROPERTY_SYMBOLIC_STRUCT_DEFINITION_REFERENCE)
                .map(StructViewerViewData::read_utf8_field_text),
            Some(String::from("u16[4]"))
        );
        assert!(
            symbol_struct
                .get_field(ProjectItemTypeAddress::PROPERTY_FREEZE_DISPLAY_VALUE)
                .is_some()
        );
    }
}
