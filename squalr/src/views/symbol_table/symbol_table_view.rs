use crate::app_context::AppContext;
use crate::ui::widgets::controls::{button::Button as ThemeButton, groupbox::GroupBox, state_layer::StateLayer};
use crate::views::{
    code_viewer::{code_viewer_view::CodeViewerView, view_data::code_viewer_view_data::CodeViewerViewData},
    memory_viewer::{memory_viewer_view::MemoryViewerView, view_data::memory_viewer_view_data::MemoryViewerViewData},
    struct_viewer::view_data::struct_viewer_focus_target::StructViewerFocusTarget,
    struct_viewer::view_data::struct_viewer_view_data::StructViewerViewData,
    symbol_explorer::view_data::symbol_tree_entry::{SymbolTreeEntry, SymbolTreeEntryKind, build_symbol_tree_entries},
    symbol_table::{
        symbol_table_toolbar_view::{SymbolTableToolbarAction, SymbolTableToolbarView},
        view_data::symbol_table_view_data::{RootedSymbolCreateDraft, RootedSymbolDraftLocatorMode, SymbolTableTakeOverState, SymbolTableViewData},
    },
};
use eframe::egui::{
    Align, Color32, ComboBox, Direction, Key, Layout, Rect, Response, RichText, ScrollArea, Sense, TextEdit, Ui, UiBuilder, Widget, pos2, vec2,
};
use epaint::{CornerRadius, Stroke, StrokeKind};
use squalr_engine_api::commands::{
    memory::{
        read::{memory_read_request::MemoryReadRequest, memory_read_response::MemoryReadResponse},
        write::memory_write_request::MemoryWriteRequest,
    },
    privileged_command_request::PrivilegedCommandRequest,
    privileged_command_response::TypedPrivilegedCommandResponse,
    project_symbols::{
        create::project_symbols_create_request::ProjectSymbolsCreateRequest, delete::project_symbols_delete_request::ProjectSymbolsDeleteRequest,
        rename::project_symbols_rename_request::ProjectSymbolsRenameRequest, update::project_symbols_update_request::ProjectSymbolsUpdateRequest,
    },
    unprivileged_command_request::UnprivilegedCommandRequest,
};
use squalr_engine_api::dependency_injection::dependency::Dependency;
use squalr_engine_api::engine::engine_execution_context::EngineExecutionContext;
use squalr_engine_api::structures::{
    data_types::{built_in_types::string::utf8::data_type_string_utf8::DataTypeStringUtf8, data_type_ref::DataTypeRef},
    data_values::container_type::ContainerType,
    projects::{project_root_symbol_locator::ProjectRootSymbolLocator, project_symbol_catalog::ProjectSymbolCatalog},
    structs::{
        symbolic_field_definition::SymbolicFieldDefinition, symbolic_struct_definition::SymbolicStructDefinition, valued_struct::ValuedStruct,
        valued_struct_field::ValuedStructField,
    },
};
use std::collections::{HashMap, HashSet};
use std::str::FromStr;
use std::sync::{Arc, mpsc};
use std::time::Duration;

#[derive(Clone)]
pub struct SymbolTableView {
    app_context: Arc<AppContext>,
    symbol_table_view_data: Dependency<SymbolTableViewData>,
    struct_viewer_view_data: Dependency<StructViewerViewData>,
    memory_viewer_view_data: Dependency<MemoryViewerViewData>,
    code_viewer_view_data: Dependency<CodeViewerViewData>,
}

impl SymbolTableView {
    pub const WINDOW_ID: &'static str = "window_symbol_table";
    const SEARCH_ROW_HEIGHT: f32 = 32.0;
    const TABLE_ROW_HEIGHT: f32 = 28.0;
    const STRUCT_VIEWER_SYMBOL_NAME_FIELD: &'static str = "display_name";
    const STRUCT_VIEWER_SYMBOL_KEY_FIELD: &'static str = "symbol_key";
    const STRUCT_VIEWER_SYMBOL_TYPE_FIELD: &'static str = "type";

    pub fn new(app_context: Arc<AppContext>) -> Self {
        let symbol_table_view_data = app_context
            .dependency_container
            .register(SymbolTableViewData::new());
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
            symbol_table_view_data,
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

    fn build_rooted_symbol_entries(
        &self,
        project_symbol_catalog: &ProjectSymbolCatalog,
    ) -> Vec<SymbolTreeEntry> {
        let mut rooted_symbol_entries = build_symbol_tree_entries(project_symbol_catalog, &HashSet::new(), &HashMap::new(), |data_type_ref| {
            self.app_context
                .engine_unprivileged_state
                .get_default_value(data_type_ref)
                .map(|default_value| default_value.get_size_in_bytes())
        });

        rooted_symbol_entries.retain(|symbol_tree_entry| matches!(symbol_tree_entry.get_kind(), SymbolTreeEntryKind::RootedSymbol { .. }));
        rooted_symbol_entries.sort_by(|left_symbol, right_symbol| {
            left_symbol
                .get_display_name()
                .to_ascii_lowercase()
                .cmp(&right_symbol.get_display_name().to_ascii_lowercase())
                .then_with(|| {
                    left_symbol
                        .get_root_symbol_key()
                        .cmp(right_symbol.get_root_symbol_key())
                })
        });

        rooted_symbol_entries
    }

    fn focus_memory_viewer_for_locator(
        &self,
        root_locator: &ProjectRootSymbolLocator,
    ) {
        MemoryViewerViewData::request_focus_address(
            self.memory_viewer_view_data.clone(),
            self.app_context.engine_unprivileged_state.clone(),
            root_locator.get_focus_address(),
            root_locator.get_focus_module_name().to_string(),
        );

        match self.app_context.docking_manager.write() {
            Ok(mut docking_manager) => {
                docking_manager.set_window_visibility(MemoryViewerView::WINDOW_ID, true);
                docking_manager.select_tab_by_window_id(MemoryViewerView::WINDOW_ID);
            }
            Err(error) => {
                log::error!("Failed to acquire docking manager while opening memory viewer from Symbol Table: {}", error);
            }
        }
    }

    fn focus_code_viewer_for_locator(
        &self,
        root_locator: &ProjectRootSymbolLocator,
    ) {
        CodeViewerViewData::request_focus_address(
            self.code_viewer_view_data.clone(),
            self.app_context.engine_unprivileged_state.clone(),
            root_locator.get_focus_address(),
            root_locator.get_focus_module_name().to_string(),
        );

        match self.app_context.docking_manager.write() {
            Ok(mut docking_manager) => {
                docking_manager.set_window_visibility(CodeViewerView::WINDOW_ID, true);
                docking_manager.select_tab_by_window_id(CodeViewerView::WINDOW_ID);
            }
            Err(error) => {
                log::error!("Failed to acquire docking manager while opening code viewer from Symbol Table: {}", error);
            }
        }
    }

    fn focus_rooted_symbol_in_struct_viewer(
        &self,
        project_symbol_catalog: &ProjectSymbolCatalog,
        rooted_symbol_entry: &SymbolTreeEntry,
    ) {
        let symbol_struct = self.build_symbol_struct_for_rooted_symbol(project_symbol_catalog, rooted_symbol_entry);
        let struct_viewer_edit_callback = self.build_struct_viewer_edit_callback(project_symbol_catalog, rooted_symbol_entry);

        StructViewerViewData::focus_valued_struct_with_focus_target(
            self.struct_viewer_view_data.clone(),
            self.app_context.engine_unprivileged_state.clone(),
            symbol_struct,
            struct_viewer_edit_callback,
            Some(StructViewerFocusTarget::SymbolTable {
                symbol_key: rooted_symbol_entry.get_root_symbol_key().to_string(),
            }),
        );
    }

    fn build_symbol_struct_for_rooted_symbol(
        &self,
        project_symbol_catalog: &ProjectSymbolCatalog,
        rooted_symbol_entry: &SymbolTreeEntry,
    ) -> ValuedStruct {
        let Some(symbolic_struct_definition) = self.build_named_symbolic_struct_definition_for_rooted_symbol(project_symbol_catalog, rooted_symbol_entry)
        else {
            return Self::build_symbol_struct_fallback(rooted_symbol_entry, "Unable to resolve a struct definition for the selected symbol.");
        };

        let engine_execution_context: Arc<dyn EngineExecutionContext> = self.app_context.engine_unprivileged_state.clone();
        let memory_read_response = Self::dispatch_memory_read_request(
            &engine_execution_context,
            rooted_symbol_entry.get_locator().get_focus_address(),
            rooted_symbol_entry.get_locator().get_focus_module_name(),
            &symbolic_struct_definition,
        );
        let Some(memory_read_response) = memory_read_response else {
            return Self::build_symbol_struct_fallback(rooted_symbol_entry, "Timed out while reading the selected symbol from memory.");
        };

        if !memory_read_response.success {
            return Self::build_symbol_struct_fallback(rooted_symbol_entry, "The selected symbol could not be read from memory.");
        }

        Self::normalize_symbol_memory_struct(memory_read_response.valued_struct, rooted_symbol_entry)
    }

    fn build_named_symbolic_struct_definition_for_rooted_symbol(
        &self,
        project_symbol_catalog: &ProjectSymbolCatalog,
        rooted_symbol_entry: &SymbolTreeEntry,
    ) -> Option<SymbolicStructDefinition> {
        self.build_symbolic_struct_definition_for_symbol_type(project_symbol_catalog, rooted_symbol_entry.get_symbol_type_id())
            .map(|symbolic_struct_definition| {
                if !symbolic_struct_definition.get_fields().is_empty() {
                    return symbolic_struct_definition;
                }

                SymbolicStructDefinition::new_anonymous(vec![SymbolicFieldDefinition::new(
                    DataTypeRef::new(rooted_symbol_entry.get_symbol_type_id()),
                    rooted_symbol_entry.get_container_type(),
                )])
            })
    }

    fn build_symbolic_struct_definition_for_symbol_type(
        &self,
        project_symbol_catalog: &ProjectSymbolCatalog,
        symbol_type_id: &str,
    ) -> Option<SymbolicStructDefinition> {
        Self::build_symbolic_struct_definition_for_symbol_type_static(project_symbol_catalog, symbol_type_id)
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
            ContainerType::None,
        )]))
    }

    fn normalize_symbol_memory_struct(
        valued_struct: ValuedStruct,
        rooted_symbol_entry: &SymbolTreeEntry,
    ) -> ValuedStruct {
        let mut normalized_fields = vec![
            DataTypeStringUtf8::get_value_from_primitive_string(rooted_symbol_entry.get_display_name())
                .to_named_valued_struct_field(Self::STRUCT_VIEWER_SYMBOL_NAME_FIELD.to_string(), false),
            DataTypeStringUtf8::get_value_from_primitive_string(rooted_symbol_entry.get_root_symbol_key())
                .to_named_valued_struct_field(Self::STRUCT_VIEWER_SYMBOL_KEY_FIELD.to_string(), true),
            DataTypeStringUtf8::get_value_from_primitive_string(&rooted_symbol_entry.get_promoted_symbol_type_id())
                .to_named_valued_struct_field(Self::STRUCT_VIEWER_SYMBOL_TYPE_FIELD.to_string(), false),
        ];

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
        rooted_symbol_entry: &SymbolTreeEntry,
        status_text: &str,
    ) -> ValuedStruct {
        ValuedStruct::new_anonymous(vec![
            DataTypeStringUtf8::get_value_from_primitive_string(rooted_symbol_entry.get_display_name())
                .to_named_valued_struct_field(Self::STRUCT_VIEWER_SYMBOL_NAME_FIELD.to_string(), false),
            DataTypeStringUtf8::get_value_from_primitive_string(rooted_symbol_entry.get_root_symbol_key())
                .to_named_valued_struct_field(Self::STRUCT_VIEWER_SYMBOL_KEY_FIELD.to_string(), true),
            DataTypeStringUtf8::get_value_from_primitive_string(&rooted_symbol_entry.get_promoted_symbol_type_id())
                .to_named_valued_struct_field(Self::STRUCT_VIEWER_SYMBOL_TYPE_FIELD.to_string(), false),
            DataTypeStringUtf8::get_value_from_primitive_string(&rooted_symbol_entry.get_locator().to_string())
                .to_named_valued_struct_field(String::from("locator"), true),
            DataTypeStringUtf8::get_value_from_primitive_string(status_text).to_named_valued_struct_field(String::from("status"), true),
        ])
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

    fn build_struct_viewer_edit_callback(
        &self,
        project_symbol_catalog: &ProjectSymbolCatalog,
        rooted_symbol_entry: &SymbolTreeEntry,
    ) -> Arc<dyn Fn(ValuedStructField) + Send + Sync> {
        let rooted_symbol_entry = rooted_symbol_entry.clone();
        let project_symbol_catalog = project_symbol_catalog.clone();
        let engine_unprivileged_state = self.app_context.engine_unprivileged_state.clone();
        let engine_execution_context: Arc<dyn EngineExecutionContext> = engine_unprivileged_state.clone();

        Arc::new(move |edited_field: ValuedStructField| {
            if edited_field.get_name() == Self::STRUCT_VIEWER_SYMBOL_NAME_FIELD {
                let next_display_name = StructViewerViewData::read_utf8_field_text(&edited_field)
                    .trim()
                    .to_string();
                if next_display_name.is_empty() || next_display_name == rooted_symbol_entry.get_display_name() {
                    return;
                }

                ProjectSymbolsRenameRequest {
                    symbol_key: rooted_symbol_entry.get_root_symbol_key().to_string(),
                    display_name: next_display_name,
                }
                .send(&engine_unprivileged_state, |_project_symbols_rename_response| {});
                return;
            }

            if edited_field.get_name() == Self::STRUCT_VIEWER_SYMBOL_TYPE_FIELD {
                let next_struct_layout_id = StructViewerViewData::read_utf8_field_text(&edited_field)
                    .trim()
                    .to_string();
                if next_struct_layout_id.is_empty() || next_struct_layout_id == rooted_symbol_entry.get_promoted_symbol_type_id() {
                    return;
                }

                ProjectSymbolsUpdateRequest {
                    symbol_key: rooted_symbol_entry.get_root_symbol_key().to_string(),
                    display_name: None,
                    struct_layout_id: Some(next_struct_layout_id),
                }
                .send(&engine_unprivileged_state, |_project_symbols_update_response| {});
                return;
            }

            let Some(memory_write_request) =
                Self::build_memory_write_request_for_symbol_value_edit(&engine_execution_context, &project_symbol_catalog, &rooted_symbol_entry, &edited_field)
            else {
                return;
            };

            memory_write_request.send(&engine_unprivileged_state, |memory_write_response| {
                if !memory_write_response.success {
                    log::warn!("Symbol Table struct-viewer memory write command failed.");
                }
            });
        })
    }

    fn build_memory_write_request_for_symbol_value_edit(
        engine_execution_context: &Arc<dyn EngineExecutionContext>,
        project_symbol_catalog: &ProjectSymbolCatalog,
        rooted_symbol_entry: &SymbolTreeEntry,
        edited_field: &ValuedStructField,
    ) -> Option<MemoryWriteRequest> {
        let edited_data_value = edited_field.get_data_value()?;
        let symbolic_struct_definition =
            Self::build_named_symbolic_struct_definition_for_value_edit(engine_execution_context, project_symbol_catalog, rooted_symbol_entry)?;
        let field_offset = Self::resolve_symbol_struct_field_offset(engine_execution_context, &symbolic_struct_definition, edited_field.get_name())?;
        let address = rooted_symbol_entry
            .get_locator()
            .get_focus_address()
            .checked_add(field_offset)?;

        Some(MemoryWriteRequest {
            address,
            module_name: rooted_symbol_entry
                .get_locator()
                .get_focus_module_name()
                .to_string(),
            value: edited_data_value.get_value_bytes().clone(),
        })
    }

    fn build_named_symbolic_struct_definition_for_value_edit(
        engine_execution_context: &Arc<dyn EngineExecutionContext>,
        project_symbol_catalog: &ProjectSymbolCatalog,
        rooted_symbol_entry: &SymbolTreeEntry,
    ) -> Option<SymbolicStructDefinition> {
        let symbolic_struct_definition =
            Self::build_symbolic_struct_definition_for_symbol_type_static(project_symbol_catalog, rooted_symbol_entry.get_symbol_type_id())?;

        if !symbolic_struct_definition.get_fields().is_empty() {
            return Some(symbolic_struct_definition);
        }

        if engine_execution_context
            .get_default_value(&DataTypeRef::new(rooted_symbol_entry.get_symbol_type_id()))
            .is_none()
        {
            return None;
        }

        Some(SymbolicStructDefinition::new_anonymous(vec![SymbolicFieldDefinition::new(
            DataTypeRef::new(rooted_symbol_entry.get_symbol_type_id()),
            rooted_symbol_entry.get_container_type(),
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
        let element_size_in_bytes = Self::resolve_element_size_in_bytes(
            engine_execution_context,
            symbolic_field_definition.get_data_type_ref().get_data_type_id(),
            visited_type_ids,
        )?;

        Some(
            symbolic_field_definition
                .get_container_type()
                .get_total_size_in_bytes(element_size_in_bytes),
        )
    }

    fn resolve_element_size_in_bytes(
        engine_execution_context: &Arc<dyn EngineExecutionContext>,
        symbol_type_id: &str,
        visited_type_ids: &mut HashSet<String>,
    ) -> Option<u64> {
        if !visited_type_ids.insert(symbol_type_id.to_string()) {
            return None;
        }

        let result = if let Some(default_value) = engine_execution_context.get_default_value(&DataTypeRef::new(symbol_type_id)) {
            Some(default_value.get_size_in_bytes())
        } else if let Ok(symbolic_field_definition) = SymbolicFieldDefinition::from_str(symbol_type_id) {
            Self::resolve_symbolic_field_size_in_bytes(engine_execution_context, &symbolic_field_definition, visited_type_ids)
        } else if let Ok(symbolic_struct_definition) = SymbolicStructDefinition::from_str(symbol_type_id) {
            Self::resolve_symbolic_struct_size_in_bytes(engine_execution_context, &symbolic_struct_definition, visited_type_ids)
        } else {
            None
        };

        visited_type_ids.remove(symbol_type_id);

        result
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
                            "Unexpected response variant for Symbol Table memory read request: {:?}",
                            unexpected_response
                        )),
                    };
                    let _ = memory_read_response_sender.send(conversion_result);
                }),
            ),
            Err(error) => {
                log::error!("Failed to acquire engine bindings lock for Symbol Table memory read request: {}", error);
                return None;
            }
        };

        if let Err(error) = dispatch_result {
            log::error!("Failed to dispatch Symbol Table memory read request: {}", error);
            return None;
        }

        match memory_read_response_receiver.recv_timeout(Duration::from_secs(2)) {
            Ok(Ok(memory_read_response)) => Some(memory_read_response),
            Ok(Err(error)) => {
                log::error!("Failed to convert Symbol Table memory read response: {}", error);
                None
            }
            Err(error) => {
                log::error!("Timed out waiting for Symbol Table memory read response: {}", error);
                None
            }
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

    fn build_rooted_symbol_create_request_from_draft(
        edited_draft: &RootedSymbolCreateDraft,
        project_symbol_catalog: &ProjectSymbolCatalog,
    ) -> Option<ProjectSymbolsCreateRequest> {
        let parsed_address = Self::parse_u64_draft(&edited_draft.address_text);
        let parsed_offset = Self::parse_u64_draft(&edited_draft.offset_text);
        let has_valid_type_id = !edited_draft.struct_layout_id.trim().is_empty()
            && project_symbol_catalog
                .get_struct_layout_descriptors()
                .iter()
                .any(|struct_layout_descriptor| struct_layout_descriptor.get_struct_layout_id() == edited_draft.struct_layout_id.trim());
        let has_valid_locator = match edited_draft.locator_mode {
            RootedSymbolDraftLocatorMode::AbsoluteAddress => parsed_address.is_some(),
            RootedSymbolDraftLocatorMode::ModuleOffset => !edited_draft.module_name.trim().is_empty() && parsed_offset.is_some(),
        };

        if edited_draft.display_name.trim().is_empty() || !has_valid_type_id || !has_valid_locator {
            return None;
        }

        Some(ProjectSymbolsCreateRequest {
            display_name: edited_draft.display_name.trim().to_string(),
            struct_layout_id: edited_draft.struct_layout_id.trim().to_string(),
            address: match edited_draft.locator_mode {
                RootedSymbolDraftLocatorMode::AbsoluteAddress => parsed_address,
                RootedSymbolDraftLocatorMode::ModuleOffset => None,
            },
            module_name: match edited_draft.locator_mode {
                RootedSymbolDraftLocatorMode::AbsoluteAddress => None,
                RootedSymbolDraftLocatorMode::ModuleOffset => Some(edited_draft.module_name.trim().to_string()),
            },
            offset: match edited_draft.locator_mode {
                RootedSymbolDraftLocatorMode::AbsoluteAddress => None,
                RootedSymbolDraftLocatorMode::ModuleOffset => parsed_offset,
            },
            metadata: Default::default(),
        })
    }

    fn create_rooted_symbol(
        &self,
        project_symbols_create_request: ProjectSymbolsCreateRequest,
    ) {
        let symbol_table_view_data = self.symbol_table_view_data.clone();

        project_symbols_create_request.send(&self.app_context.engine_unprivileged_state, move |project_symbols_create_response| {
            if project_symbols_create_response.success && !project_symbols_create_response.created_symbol_key.is_empty() {
                SymbolTableViewData::set_selected_symbol_key(symbol_table_view_data.clone(), Some(project_symbols_create_response.created_symbol_key));
                SymbolTableViewData::cancel_take_over_state(symbol_table_view_data);
            }
        });
    }

    fn delete_rooted_symbol(
        &self,
        symbol_key: &str,
    ) {
        let deleted_symbol_key = symbol_key.to_string();
        let symbol_table_view_data = self.symbol_table_view_data.clone();
        let struct_viewer_view_data = self.struct_viewer_view_data.clone();

        ProjectSymbolsDeleteRequest {
            symbol_keys: vec![deleted_symbol_key.clone()],
        }
        .send(&self.app_context.engine_unprivileged_state, move |_project_symbols_delete_response| {
            SymbolTableViewData::cancel_take_over_state(symbol_table_view_data.clone());

            let current_focus_target = struct_viewer_view_data
                .read("Symbol table current struct viewer focus target after delete")
                .and_then(|struct_viewer_view_data| struct_viewer_view_data.get_focus_target().cloned());

            if matches!(
                current_focus_target,
                Some(StructViewerFocusTarget::SymbolTable {
                    symbol_key: focused_symbol_key,
                }) if focused_symbol_key == deleted_symbol_key
            ) {
                StructViewerViewData::clear_focus(struct_viewer_view_data.clone());
            }
        });
    }

    fn render_search_row(
        &self,
        user_interface: &mut Ui,
        current_filter_text: &str,
    ) {
        let theme = &self.app_context.theme;
        let (search_rect, _search_response) =
            user_interface.allocate_exact_size(vec2(user_interface.available_width(), Self::SEARCH_ROW_HEIGHT), Sense::hover());
        let mut search_user_interface = user_interface.new_child(
            UiBuilder::new()
                .max_rect(search_rect)
                .layout(Layout::left_to_right(Align::Center)),
        );
        let mut next_filter_text = current_filter_text.to_string();
        let search_edit_response = search_user_interface.add_sized(
            vec2(search_rect.width(), 26.0),
            TextEdit::singleline(&mut next_filter_text)
                .hint_text("Filter rooted symbols by name, locator, type, or key.")
                .background_color(theme.background_control)
                .text_color(theme.foreground),
        );

        if search_edit_response.changed() {
            SymbolTableViewData::set_filter_text(self.symbol_table_view_data.clone(), next_filter_text);
        }
    }

    fn render_delete_confirmation_take_over(
        &self,
        user_interface: &mut Ui,
        display_name: &str,
        symbol_key: &str,
    ) {
        let theme = &self.app_context.theme;

        user_interface.allocate_ui_with_layout(
            user_interface.available_size(),
            Layout::centered_and_justified(Direction::TopDown),
            |user_interface| {
                user_interface.add(
                    GroupBox::new_from_theme(theme, "Delete Rooted Symbol", |user_interface| {
                        user_interface.label(RichText::new(format!("Delete `{}`?", display_name)).color(theme.foreground));
                        user_interface.add_space(4.0);
                        user_interface.label(RichText::new(symbol_key).color(theme.foreground_preview));
                        user_interface.add_space(12.0);
                        user_interface.horizontal(|user_interface| {
                            let cancel_response = user_interface.add_sized(
                                vec2(96.0, 28.0),
                                ThemeButton::new_from_theme(theme).with_tooltip_text("Cancel symbol deletion."),
                            );
                            if cancel_response.clicked() {
                                SymbolTableViewData::cancel_take_over_state(self.symbol_table_view_data.clone());
                            }
                            let delete_response =
                                user_interface.add_sized(vec2(96.0, 28.0), ThemeButton::new_from_theme(theme).with_tooltip_text("Delete rooted symbol."));
                            if delete_response.clicked() {
                                self.delete_rooted_symbol(symbol_key);
                            }
                        });
                    })
                    .desired_width(360.0),
                );
            },
        );
    }

    fn render_create_rooted_symbol_take_over(
        &self,
        user_interface: &mut Ui,
        project_symbol_catalog: &ProjectSymbolCatalog,
        rooted_symbol_create_draft: &RootedSymbolCreateDraft,
    ) {
        let theme = &self.app_context.theme;
        let mut edited_draft = rooted_symbol_create_draft.clone();
        let can_create_rooted_symbol = Self::build_rooted_symbol_create_request_from_draft(&edited_draft, project_symbol_catalog).is_some();

        user_interface.allocate_ui_with_layout(
            user_interface.available_size(),
            Layout::centered_and_justified(Direction::TopDown),
            |user_interface| {
                user_interface.add(
                    GroupBox::new_from_theme(theme, "New Rooted Symbol", |user_interface| {
                        user_interface.label(RichText::new("Display Name").color(theme.foreground_preview));
                        user_interface.add(
                            TextEdit::singleline(&mut edited_draft.display_name)
                                .background_color(theme.background_control)
                                .text_color(theme.foreground),
                        );
                        user_interface.add_space(8.0);
                        user_interface.label(RichText::new("Type").color(theme.foreground_preview));
                        ComboBox::from_id_salt("symbol_table_create_rooted_symbol_type")
                            .selected_text(if edited_draft.struct_layout_id.is_empty() {
                                "Select a type"
                            } else {
                                &edited_draft.struct_layout_id
                            })
                            .show_ui(user_interface, |user_interface| {
                                for struct_layout_descriptor in project_symbol_catalog.get_struct_layout_descriptors() {
                                    user_interface.selectable_value(
                                        &mut edited_draft.struct_layout_id,
                                        struct_layout_descriptor.get_struct_layout_id().to_string(),
                                        struct_layout_descriptor.get_struct_layout_id(),
                                    );
                                }
                            });
                        user_interface.add_space(8.0);
                        user_interface.label(RichText::new("Locator").color(theme.foreground_preview));
                        user_interface.horizontal(|user_interface| {
                            user_interface.selectable_value(
                                &mut edited_draft.locator_mode,
                                RootedSymbolDraftLocatorMode::AbsoluteAddress,
                                "Absolute Address",
                            );
                            user_interface.selectable_value(&mut edited_draft.locator_mode, RootedSymbolDraftLocatorMode::ModuleOffset, "Module + Offset");
                        });
                        user_interface.add_space(4.0);

                        match edited_draft.locator_mode {
                            RootedSymbolDraftLocatorMode::AbsoluteAddress => {
                                user_interface.label(RichText::new("Address").color(theme.foreground_preview));
                                user_interface.add(
                                    TextEdit::singleline(&mut edited_draft.address_text)
                                        .hint_text("0x12345678")
                                        .background_color(theme.background_control)
                                        .text_color(theme.foreground),
                                );
                            }
                            RootedSymbolDraftLocatorMode::ModuleOffset => {
                                user_interface.label(RichText::new("Module Name").color(theme.foreground_preview));
                                user_interface.add(
                                    TextEdit::singleline(&mut edited_draft.module_name)
                                        .background_color(theme.background_control)
                                        .text_color(theme.foreground),
                                );
                                user_interface.add_space(4.0);
                                user_interface.label(RichText::new("Offset").color(theme.foreground_preview));
                                user_interface.add(
                                    TextEdit::singleline(&mut edited_draft.offset_text)
                                        .hint_text("0x1234")
                                        .background_color(theme.background_control)
                                        .text_color(theme.foreground),
                                );
                            }
                        }

                        user_interface.add_space(12.0);
                        user_interface.horizontal(|user_interface| {
                            let cancel_response = user_interface.add_sized(
                                vec2(96.0, 28.0),
                                ThemeButton::new_from_theme(theme).with_tooltip_text("Cancel rooted-symbol creation."),
                            );
                            if cancel_response.clicked() {
                                SymbolTableViewData::cancel_take_over_state(self.symbol_table_view_data.clone());
                            }
                            let create_response = user_interface.add_sized(
                                vec2(96.0, 28.0),
                                ThemeButton::new_from_theme(theme)
                                    .with_tooltip_text("Create rooted symbol.")
                                    .disabled(!can_create_rooted_symbol),
                            );
                            if create_response.clicked() {
                                if let Some(project_symbols_create_request) =
                                    Self::build_rooted_symbol_create_request_from_draft(&edited_draft, project_symbol_catalog)
                                {
                                    self.create_rooted_symbol(project_symbols_create_request);
                                }
                            }
                        });
                    })
                    .desired_width(420.0),
                );
            },
        );

        SymbolTableViewData::set_rooted_symbol_create_draft(self.symbol_table_view_data.clone(), edited_draft);
    }

    fn render_table_header(
        &self,
        user_interface: &mut Ui,
    ) {
        let theme = &self.app_context.theme;
        let (header_rect, _) = user_interface.allocate_exact_size(vec2(user_interface.available_width(), Self::TABLE_ROW_HEIGHT), Sense::hover());

        user_interface
            .painter()
            .rect_filled(header_rect, CornerRadius::ZERO, theme.background_primary);

        let total_width = header_rect.width();
        let name_width = (total_width * 0.22).max(140.0);
        let locator_width = (total_width * 0.34).max(200.0);
        let type_width = (total_width * 0.20).max(140.0);
        let key_width = (total_width - name_width - locator_width - type_width - 16.0).max(80.0);

        self.draw_column_text(header_rect, 8.0, name_width, "Name", theme.foreground_preview, true, user_interface);
        self.draw_column_text(
            header_rect,
            8.0 + name_width,
            locator_width,
            "Locator",
            theme.foreground_preview,
            true,
            user_interface,
        );
        self.draw_column_text(
            header_rect,
            8.0 + name_width + locator_width,
            type_width,
            "Type",
            theme.foreground_preview,
            true,
            user_interface,
        );
        self.draw_column_text(
            header_rect,
            8.0 + name_width + locator_width + type_width,
            key_width,
            "Key",
            theme.foreground_preview,
            true,
            user_interface,
        );
    }

    fn render_rooted_symbol_row(
        &self,
        user_interface: &mut Ui,
        rooted_symbol_entry: &SymbolTreeEntry,
        is_selected: bool,
    ) -> Response {
        let theme = &self.app_context.theme;
        let (row_rect, row_response) = user_interface.allocate_exact_size(vec2(user_interface.available_width(), Self::TABLE_ROW_HEIGHT), Sense::click());

        if is_selected {
            user_interface
                .painter()
                .rect_filled(row_rect, CornerRadius::ZERO, theme.selected_background);
            user_interface
                .painter()
                .rect_stroke(row_rect, CornerRadius::ZERO, Stroke::new(1.0, theme.selected_border), StrokeKind::Inside);
        }

        StateLayer {
            bounds_min: row_rect.min,
            bounds_max: row_rect.max,
            enabled: true,
            pressed: row_response.is_pointer_button_down_on(),
            has_hover: row_response.hovered(),
            has_focus: false,
            corner_radius: CornerRadius::ZERO,
            border_width: 0.0,
            hover_color: theme.hover_tint,
            pressed_color: theme.pressed_tint,
            border_color: theme.background_control_secondary_dark,
            border_color_focused: theme.background_control_secondary_dark,
        }
        .ui(user_interface);

        let total_width = row_rect.width();
        let name_width = (total_width * 0.22).max(140.0);
        let locator_width = (total_width * 0.34).max(200.0);
        let type_width = (total_width * 0.20).max(140.0);
        let key_width = (total_width - name_width - locator_width - type_width - 16.0).max(80.0);
        let primary_text_color = theme.foreground;
        let secondary_text_color = theme.foreground_preview;

        self.draw_column_text(
            row_rect,
            8.0,
            name_width,
            rooted_symbol_entry.get_display_name(),
            primary_text_color,
            false,
            user_interface,
        );
        self.draw_column_text(
            row_rect,
            8.0 + name_width,
            locator_width,
            &rooted_symbol_entry.get_locator().to_string(),
            secondary_text_color,
            false,
            user_interface,
        );
        self.draw_column_text(
            row_rect,
            8.0 + name_width + locator_width,
            type_width,
            &rooted_symbol_entry.get_promoted_symbol_type_id(),
            secondary_text_color,
            false,
            user_interface,
        );
        self.draw_column_text(
            row_rect,
            8.0 + name_width + locator_width + type_width,
            key_width,
            rooted_symbol_entry.get_root_symbol_key(),
            secondary_text_color,
            false,
            user_interface,
        );

        row_response
    }

    fn draw_column_text(
        &self,
        row_rect: Rect,
        left_offset: f32,
        column_width: f32,
        text: &str,
        color: Color32,
        is_header: bool,
        user_interface: &mut Ui,
    ) {
        let text_rect = Rect::from_min_max(
            pos2(row_rect.min.x + left_offset, row_rect.min.y),
            pos2(row_rect.min.x + left_offset + column_width.max(1.0), row_rect.max.y),
        );
        let mut text_user_interface = user_interface.new_child(
            UiBuilder::new()
                .max_rect(text_rect)
                .layout(Layout::left_to_right(Align::Center)),
        );
        text_user_interface.set_clip_rect(text_rect);

        let rich_text = if is_header {
            RichText::new(text).strong().color(color)
        } else {
            RichText::new(text).color(color)
        };
        text_user_interface.label(rich_text);
    }

    fn rooted_symbol_matches_filter(
        rooted_symbol_entry: &SymbolTreeEntry,
        filter_text: &str,
    ) -> bool {
        let trimmed_filter_text = filter_text.trim();
        if trimmed_filter_text.is_empty() {
            return true;
        }

        let normalized_filter_text = trimmed_filter_text.to_ascii_lowercase();

        [
            rooted_symbol_entry.get_display_name().to_string(),
            rooted_symbol_entry.get_root_symbol_key().to_string(),
            rooted_symbol_entry.get_promoted_symbol_type_id(),
            rooted_symbol_entry.get_locator().to_string(),
        ]
        .iter()
        .any(|candidate_text| {
            candidate_text
                .to_ascii_lowercase()
                .contains(&normalized_filter_text)
        })
    }
}

impl Widget for SymbolTableView {
    fn ui(
        self,
        user_interface: &mut Ui,
    ) -> Response {
        let Some(project_symbol_catalog) = self.get_opened_project_symbol_catalog() else {
            return user_interface
                .allocate_ui_with_layout(
                    user_interface.available_size(),
                    Layout::centered_and_justified(Direction::TopDown),
                    |user_interface| {
                        user_interface.label(
                            RichText::new("Open a project to browse rooted symbols in the symbol table.").color(self.app_context.theme.foreground_preview),
                        );
                    },
                )
                .response;
        };

        SymbolTableViewData::synchronize_selection(self.symbol_table_view_data.clone(), &project_symbol_catalog);
        SymbolTableViewData::synchronize_rooted_symbol_create_draft(self.symbol_table_view_data.clone(), &project_symbol_catalog);
        SymbolTableViewData::synchronize_take_over_state(self.symbol_table_view_data.clone(), &project_symbol_catalog);
        let rooted_symbol_entries = self.build_rooted_symbol_entries(&project_symbol_catalog);
        let (selected_symbol_key, take_over_state, filter_text, rooted_symbol_create_draft) = self
            .symbol_table_view_data
            .read("Symbol table view")
            .map(|symbol_table_view_data| {
                (
                    symbol_table_view_data
                        .get_selected_symbol_key()
                        .map(str::to_string),
                    symbol_table_view_data.get_take_over_state().cloned(),
                    symbol_table_view_data.get_filter_text().to_string(),
                    symbol_table_view_data.get_rooted_symbol_create_draft().clone(),
                )
            })
            .unwrap_or((None, None, String::new(), RootedSymbolCreateDraft::default()));
        let selected_rooted_symbol_entry = selected_symbol_key.as_ref().and_then(|selected_symbol_key| {
            rooted_symbol_entries
                .iter()
                .find(|rooted_symbol_entry| rooted_symbol_entry.get_root_symbol_key() == selected_symbol_key)
        });
        let can_create_rooted_symbol = !project_symbol_catalog
            .get_struct_layout_descriptors()
            .is_empty();

        if matches!(take_over_state, Some(SymbolTableTakeOverState::DeleteConfirmation { .. }))
            && user_interface.input(|input_state| input_state.key_pressed(Key::Escape))
        {
            SymbolTableViewData::cancel_take_over_state(self.symbol_table_view_data.clone());
        }

        if matches!(take_over_state, Some(SymbolTableTakeOverState::DeleteConfirmation { .. }))
            && user_interface.input(|input_state| input_state.key_pressed(Key::Enter))
        {
            if let Some(SymbolTableTakeOverState::DeleteConfirmation { symbol_key, .. }) = take_over_state.as_ref() {
                self.delete_rooted_symbol(symbol_key);
            }
        }

        if take_over_state.is_none() && user_interface.input(|input_state| input_state.key_pressed(Key::Delete)) {
            if let Some(rooted_symbol_entry) = selected_rooted_symbol_entry {
                SymbolTableViewData::request_delete_confirmation(
                    self.symbol_table_view_data.clone(),
                    rooted_symbol_entry.get_root_symbol_key().to_string(),
                    rooted_symbol_entry.get_display_name().to_string(),
                );
            }
        }

        if take_over_state.is_none() && user_interface.input(|input_state| input_state.key_pressed(Key::Enter)) {
            if let Some(rooted_symbol_entry) = selected_rooted_symbol_entry {
                self.focus_memory_viewer_for_locator(rooted_symbol_entry.get_locator());
            }
        }

        user_interface
            .allocate_ui_with_layout(user_interface.available_size(), Layout::top_down(Align::Min), |user_interface| {
                let mut content_user_interface = user_interface.new_child(
                    UiBuilder::new()
                        .max_rect(
                            user_interface
                                .available_rect_before_wrap()
                                .shrink2(vec2(10.0, 8.0)),
                        )
                        .layout(Layout::top_down(Align::Min)),
                );

                let toolbar_action = SymbolTableToolbarView::new(self.app_context.clone())
                    .can_create_rooted_symbol(can_create_rooted_symbol && take_over_state.is_none())
                    .can_delete_rooted_symbol(selected_rooted_symbol_entry.is_some() && take_over_state.is_none())
                    .can_open_in_code_viewer(selected_rooted_symbol_entry.is_some() && take_over_state.is_none())
                    .can_open_in_memory_viewer(selected_rooted_symbol_entry.is_some() && take_over_state.is_none())
                    .show(&mut content_user_interface);

                match toolbar_action {
                    Some(SymbolTableToolbarAction::CreateRootedSymbol) => {
                        SymbolTableViewData::begin_create_rooted_symbol(self.symbol_table_view_data.clone(), &project_symbol_catalog);
                    }
                    Some(SymbolTableToolbarAction::OpenSelectedInCodeViewer) => {
                        if let Some(rooted_symbol_entry) = selected_rooted_symbol_entry {
                            self.focus_code_viewer_for_locator(rooted_symbol_entry.get_locator());
                        }
                    }
                    Some(SymbolTableToolbarAction::OpenSelectedInMemoryViewer) => {
                        if let Some(rooted_symbol_entry) = selected_rooted_symbol_entry {
                            self.focus_memory_viewer_for_locator(rooted_symbol_entry.get_locator());
                        }
                    }
                    Some(SymbolTableToolbarAction::DeleteSelectedRootedSymbol) => {
                        if let Some(rooted_symbol_entry) = selected_rooted_symbol_entry {
                            SymbolTableViewData::request_delete_confirmation(
                                self.symbol_table_view_data.clone(),
                                rooted_symbol_entry.get_root_symbol_key().to_string(),
                                rooted_symbol_entry.get_display_name().to_string(),
                            );
                        }
                    }
                    None => {}
                }

                content_user_interface.add_space(4.0);
                self.render_search_row(&mut content_user_interface, &filter_text);
                content_user_interface.add_space(8.0);

                match take_over_state.as_ref() {
                    Some(SymbolTableTakeOverState::DeleteConfirmation { symbol_key, display_name }) => {
                        self.render_delete_confirmation_take_over(&mut content_user_interface, display_name, symbol_key);
                    }
                    Some(SymbolTableTakeOverState::CreateRootedSymbol) => {
                        self.render_create_rooted_symbol_take_over(&mut content_user_interface, &project_symbol_catalog, &rooted_symbol_create_draft);
                    }
                    None => {
                        self.render_table_header(&mut content_user_interface);
                        ScrollArea::vertical()
                            .id_salt("symbol_table_rows")
                            .auto_shrink([false, false])
                            .show(&mut content_user_interface, |user_interface| {
                                for rooted_symbol_entry in rooted_symbol_entries
                                    .iter()
                                    .filter(|rooted_symbol_entry| Self::rooted_symbol_matches_filter(rooted_symbol_entry, &filter_text))
                                {
                                    let is_selected = selected_symbol_key
                                        .as_deref()
                                        .is_some_and(|selected_symbol_key| rooted_symbol_entry.get_root_symbol_key() == selected_symbol_key);
                                    let row_response = self.render_rooted_symbol_row(user_interface, rooted_symbol_entry, is_selected);

                                    if row_response.clicked() {
                                        SymbolTableViewData::set_selected_symbol_key(
                                            self.symbol_table_view_data.clone(),
                                            Some(rooted_symbol_entry.get_root_symbol_key().to_string()),
                                        );
                                        self.focus_rooted_symbol_in_struct_viewer(&project_symbol_catalog, rooted_symbol_entry);
                                    }

                                    if row_response.double_clicked() {
                                        self.focus_memory_viewer_for_locator(rooted_symbol_entry.get_locator());
                                    }
                                }
                            });
                    }
                }
            })
            .response
    }
}

#[cfg(test)]
mod tests {
    use super::{RootedSymbolCreateDraft, RootedSymbolDraftLocatorMode, SymbolTableView};
    use squalr_engine_api::registries::symbols::struct_layout_descriptor::StructLayoutDescriptor;
    use squalr_engine_api::structures::{
        data_types::data_type_ref::DataTypeRef,
        data_values::container_type::ContainerType,
        projects::project_symbol_catalog::ProjectSymbolCatalog,
        structs::{symbolic_field_definition::SymbolicFieldDefinition, symbolic_struct_definition::SymbolicStructDefinition},
    };

    fn create_project_symbol_catalog() -> ProjectSymbolCatalog {
        ProjectSymbolCatalog::new(vec![StructLayoutDescriptor::new(
            String::from("player.stats"),
            SymbolicStructDefinition::new(
                String::from("player.stats"),
                vec![SymbolicFieldDefinition::new(
                    DataTypeRef::new("u32"),
                    ContainerType::None,
                )],
            ),
        )])
    }

    #[test]
    fn build_rooted_symbol_create_request_accepts_hex_absolute_address() {
        let project_symbol_catalog = create_project_symbol_catalog();
        let rooted_symbol_create_draft = RootedSymbolCreateDraft {
            display_name: String::from("Player"),
            struct_layout_id: String::from("player.stats"),
            locator_mode: RootedSymbolDraftLocatorMode::AbsoluteAddress,
            address_text: String::from("0x1234"),
            module_name: String::new(),
            offset_text: String::new(),
        };

        let project_symbols_create_request =
            SymbolTableView::build_rooted_symbol_create_request_from_draft(&rooted_symbol_create_draft, &project_symbol_catalog)
                .expect("Expected rooted symbol create request for valid absolute-address draft.");

        assert_eq!(project_symbols_create_request.address, Some(0x1234));
        assert_eq!(project_symbols_create_request.module_name, None);
        assert_eq!(project_symbols_create_request.offset, None);
    }

    #[test]
    fn build_rooted_symbol_create_request_rejects_unknown_type_id() {
        let project_symbol_catalog = create_project_symbol_catalog();
        let rooted_symbol_create_draft = RootedSymbolCreateDraft {
            display_name: String::from("Player"),
            struct_layout_id: String::from("missing.type"),
            locator_mode: RootedSymbolDraftLocatorMode::ModuleOffset,
            address_text: String::new(),
            module_name: String::from("game.exe"),
            offset_text: String::from("0x1234"),
        };

        assert!(SymbolTableView::build_rooted_symbol_create_request_from_draft(&rooted_symbol_create_draft, &project_symbol_catalog).is_none());
    }
}
