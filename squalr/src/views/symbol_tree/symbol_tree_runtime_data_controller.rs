use crate::app_context::AppContext;
use crate::views::symbol_tree::view_data::symbol_tree_scalar_value::SymbolTreeScalarValue;
use squalr_engine_api::commands::{
    memory::read::{memory_read_request::MemoryReadRequest, memory_read_response::MemoryReadResponse},
    privileged_command_request::PrivilegedCommandRequest,
    privileged_command_response::TypedPrivilegedCommandResponse,
};
use squalr_engine_api::engine::engine_execution_context::EngineExecutionContext;
use squalr_engine_api::structures::data_types::data_type_ref::DataTypeRef;
use squalr_engine_api::structures::data_values::{
    container_type::ContainerType,
    data_value_preview_formatter::{DataValuePreviewFormatOptions, DataValuePreviewFormatter},
};
use squalr_engine_api::structures::details::DetailsProjection;
use squalr_engine_api::structures::memory::{
    pointer::Pointer,
    symbolic_pointer_chain::{SymbolicPointerChain, SymbolicPointerChainLink},
};
use squalr_engine_api::structures::projects::{
    project_symbol_catalog::ProjectSymbolCatalog,
    project_symbol_locator::ProjectSymbolLocator,
    symbol_layouts::symbol_layout_size_resolver::SymbolLayoutSizeResolver,
    symbol_tree::details::SymbolTreeDetailsProjection,
    symbol_tree::symbol_tree::SymbolTree,
    symbol_tree::symbol_tree_node::{ResolvedPointerTarget, SymbolTreeNode, SymbolTreeNodeKind},
};
use squalr_engine_api::structures::structs::{
    symbolic_field_definition::SymbolicFieldDefinition, symbolic_resolver_definition::SymbolicResolverEvaluationError,
    symbolic_struct_definition::SymbolicStructDefinition,
};
use squalr_engine_session::virtual_snapshots::virtual_snapshot_query::VirtualSnapshotQuery;
use squalr_engine_session::virtual_snapshots::virtual_snapshot_query_result::VirtualSnapshotQueryResult;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::str::FromStr;
use std::sync::Arc;
use std::sync::mpsc;
use std::time::Duration;

pub struct SymbolTreeRuntimeDataController {
    app_context: Arc<AppContext>,
}

pub struct SymbolTreeRuntimeData {
    pub symbol_tree_entries: Vec<SymbolTreeNode>,
    pub preview_values_by_node_key: HashMap<String, String>,
}

impl SymbolTreeRuntimeDataController {
    const POINTER_CHILDREN_VIRTUAL_SNAPSHOT_ID: &'static str = "symbol_tree_pointer_children";
    const SCALAR_VALUES_VIRTUAL_SNAPSHOT_ID: &'static str = "symbol_tree_scalar_values";
    const PREVIEW_VALUES_VIRTUAL_SNAPSHOT_ID: &'static str = "symbol_tree_preview_values";
    const PREVIEW_FORMAT_OPTIONS: DataValuePreviewFormatOptions = DataValuePreviewFormatOptions::new(3, 24);
    const POINTER_CHILDREN_REFRESH_INTERVAL: Duration = Duration::from_millis(250);
    const SCALAR_VALUES_REFRESH_INTERVAL: Duration = Duration::from_millis(250);
    const PREVIEW_VALUES_REFRESH_INTERVAL: Duration = Duration::from_millis(250);

    pub fn new(app_context: Arc<AppContext>) -> Self {
        Self { app_context }
    }

    pub fn clear_virtual_snapshots(&self) {
        self.app_context
            .engine_unprivileged_state
            .set_virtual_snapshot_queries(Self::POINTER_CHILDREN_VIRTUAL_SNAPSHOT_ID, Self::POINTER_CHILDREN_REFRESH_INTERVAL, Vec::new());
        self.app_context
            .engine_unprivileged_state
            .set_virtual_snapshot_queries(Self::SCALAR_VALUES_VIRTUAL_SNAPSHOT_ID, Self::SCALAR_VALUES_REFRESH_INTERVAL, Vec::new());
        self.app_context
            .engine_unprivileged_state
            .set_virtual_snapshot_queries(Self::PREVIEW_VALUES_VIRTUAL_SNAPSHOT_ID, Self::PREVIEW_VALUES_REFRESH_INTERVAL, Vec::new());
    }

    pub fn build_runtime_data(
        &self,
        project_symbol_catalog: &ProjectSymbolCatalog,
        expanded_tree_node_keys: &HashSet<String>,
    ) -> SymbolTreeRuntimeData {
        let scalar_values_by_query_id = self.collect_scalar_values_by_query_id();
        let scalar_snapshot_queries = RefCell::new(Vec::new());
        let resolve_primitive_size_in_bytes = |data_type_ref: &DataTypeRef| {
            self.app_context
                .engine_unprivileged_state
                .get_default_value(data_type_ref)
                .map(|default_value| default_value.get_size_in_bytes())
        };
        let read_scalar_field = |project_symbol_locator: &ProjectSymbolLocator, field_definition: &SymbolicFieldDefinition, field_size_in_bytes: u64| {
            let scalar_query_id = SymbolTreeScalarValue::query_id(project_symbol_locator, field_definition);

            if let Some(scalar_snapshot_query) =
                SymbolTreeScalarValue::build_query(project_symbol_locator, field_definition, field_size_in_bytes, |data_type_ref| {
                    self.app_context
                        .engine_unprivileged_state
                        .supports_scalar_integer_values(data_type_ref)
                })
            {
                scalar_snapshot_queries.borrow_mut().push(scalar_snapshot_query);
            }

            if let Some(scalar_value) = scalar_values_by_query_id.get(&scalar_query_id) {
                return Ok(Some(*scalar_value));
            }

            Ok(None)
        };
        let previous_resolved_pointer_targets_by_node_key = self.collect_resolved_pointer_targets_by_node_key();
        let resolver_pointer_snapshot_queries = RefCell::new(Vec::new());
        let resolve_relative_pointer_chain = |root_locator: &ProjectSymbolLocator, pointer_chain: &SymbolicPointerChain| {
            Self::resolve_relative_pointer_chain_from_pointer_snapshot(
                &previous_resolved_pointer_targets_by_node_key,
                &resolver_pointer_snapshot_queries,
                root_locator,
                pointer_chain,
            )
        };
        let resolve_global_pointer_chain = |pointer_chain: &SymbolicPointerChain| {
            Self::resolve_global_pointer_chain_from_pointer_snapshot(
                project_symbol_catalog,
                &previous_resolved_pointer_targets_by_node_key,
                &resolver_pointer_snapshot_queries,
                pointer_chain,
            )
        };
        let structural_symbol_tree_entries = SymbolTree::build_with_scalar_reader_and_pointer_chains(
            project_symbol_catalog,
            expanded_tree_node_keys,
            &HashMap::new(),
            resolve_primitive_size_in_bytes,
            read_scalar_field,
            resolve_relative_pointer_chain,
            resolve_global_pointer_chain,
        )
        .into_nodes();

        self.sync_symbol_scalar_virtual_snapshot(scalar_snapshot_queries.borrow().clone());
        self.sync_pointer_child_virtual_snapshot(
            project_symbol_catalog,
            &structural_symbol_tree_entries,
            resolver_pointer_snapshot_queries.borrow().clone(),
        );

        let resolved_pointer_targets_by_node_key = self.collect_resolved_pointer_targets_by_node_key();
        let resolve_relative_pointer_chain = |root_locator: &ProjectSymbolLocator, pointer_chain: &SymbolicPointerChain| {
            Self::resolve_relative_pointer_chain_from_pointer_snapshot(
                &resolved_pointer_targets_by_node_key,
                &resolver_pointer_snapshot_queries,
                root_locator,
                pointer_chain,
            )
        };
        let resolve_global_pointer_chain = |pointer_chain: &SymbolicPointerChain| {
            Self::resolve_global_pointer_chain_from_pointer_snapshot(
                project_symbol_catalog,
                &resolved_pointer_targets_by_node_key,
                &resolver_pointer_snapshot_queries,
                pointer_chain,
            )
        };
        let symbol_tree_entries = SymbolTree::build_with_scalar_reader_and_pointer_chains(
            project_symbol_catalog,
            expanded_tree_node_keys,
            &resolved_pointer_targets_by_node_key,
            resolve_primitive_size_in_bytes,
            read_scalar_field,
            resolve_relative_pointer_chain,
            resolve_global_pointer_chain,
        )
        .into_nodes();

        self.sync_symbol_scalar_virtual_snapshot(scalar_snapshot_queries.borrow().clone());
        self.sync_pointer_child_virtual_snapshot(project_symbol_catalog, &symbol_tree_entries, resolver_pointer_snapshot_queries.borrow().clone());
        self.sync_symbol_preview_virtual_snapshot(project_symbol_catalog, &symbol_tree_entries);

        let preview_values_by_node_key = self.collect_preview_values_by_node_key(&symbol_tree_entries);

        SymbolTreeRuntimeData {
            symbol_tree_entries,
            preview_values_by_node_key,
        }
    }

    pub fn build_symbol_details_projection_for_tree_entry(
        &self,
        project_symbol_catalog: &ProjectSymbolCatalog,
        symbol_tree_entry: &SymbolTreeNode,
    ) -> DetailsProjection {
        let include_symbol_claim_metadata = SymbolTreeDetailsProjection::include_symbol_claim_metadata(symbol_tree_entry);
        let engine_execution_context: Arc<dyn EngineExecutionContext> = self.app_context.engine_unprivileged_state.clone();
        let symbol_size_in_bytes = Self::resolve_symbol_tree_entry_size_for_struct_viewer(&engine_execution_context, symbol_tree_entry);

        if Self::symbol_tree_entry_should_use_external_value_viewer(symbol_tree_entry) {
            return SymbolTreeDetailsProjection::build_external_value(symbol_tree_entry, include_symbol_claim_metadata, symbol_size_in_bytes);
        }

        if let SymbolTreeNodeKind::ModuleSpace { module_name, .. } = symbol_tree_entry.get_kind() {
            let metadata_type_id = project_symbol_catalog
                .get_struct_layout_descriptors()
                .iter()
                .any(|struct_layout_descriptor| struct_layout_descriptor.get_struct_layout_id() == module_name)
                .then_some(module_name.as_str());

            return SymbolTreeDetailsProjection::build_with_metadata_type_id(
                symbol_tree_entry,
                include_symbol_claim_metadata,
                symbol_size_in_bytes,
                None,
                None,
                metadata_type_id,
            );
        }

        if matches!(symbol_tree_entry.get_kind(), SymbolTreeNodeKind::UnassignedSegment { .. }) {
            return SymbolTreeDetailsProjection::build(symbol_tree_entry, include_symbol_claim_metadata, symbol_size_in_bytes, None, None);
        }

        let Some(symbolic_struct_definition) = self.build_named_symbolic_struct_definition_for_symbol_tree_entry(project_symbol_catalog, symbol_tree_entry)
        else {
            return SymbolTreeDetailsProjection::build(
                symbol_tree_entry,
                include_symbol_claim_metadata,
                symbol_size_in_bytes,
                None,
                Some("Unable to resolve a struct definition for the selected symbol."),
            );
        };

        let memory_read_response = Self::dispatch_memory_read_request(
            &engine_execution_context,
            symbol_tree_entry.get_locator().get_focus_address(),
            symbol_tree_entry.get_locator().get_focus_module_name(),
            &symbolic_struct_definition,
        );
        let Some(memory_read_response) = memory_read_response else {
            return SymbolTreeDetailsProjection::build(
                symbol_tree_entry,
                include_symbol_claim_metadata,
                symbol_size_in_bytes,
                None,
                Some("Timed out while reading the selected symbol from memory."),
            );
        };

        if !memory_read_response.success {
            return SymbolTreeDetailsProjection::build(
                symbol_tree_entry,
                include_symbol_claim_metadata,
                symbol_size_in_bytes,
                None,
                Some("The selected symbol could not be read from memory."),
            );
        }

        SymbolTreeDetailsProjection::build(
            symbol_tree_entry,
            include_symbol_claim_metadata,
            symbol_size_in_bytes,
            Some(&memory_read_response.valued_struct),
            None,
        )
    }

    fn sync_pointer_child_virtual_snapshot(
        &self,
        project_symbol_catalog: &ProjectSymbolCatalog,
        symbol_tree_entries: &[SymbolTreeNode],
        additional_pointer_snapshot_queries: Vec<VirtualSnapshotQuery>,
    ) {
        let mut pointer_snapshot_queries = self.build_pointer_snapshot_queries(project_symbol_catalog, symbol_tree_entries);

        pointer_snapshot_queries.extend(additional_pointer_snapshot_queries);

        self.app_context
            .engine_unprivileged_state
            .set_virtual_snapshot_queries(
                Self::POINTER_CHILDREN_VIRTUAL_SNAPSHOT_ID,
                Self::POINTER_CHILDREN_REFRESH_INTERVAL,
                SymbolTreeScalarValue::deduplicate_queries_by_id(pointer_snapshot_queries),
            );
        self.app_context
            .engine_unprivileged_state
            .request_virtual_snapshot_refresh(Self::POINTER_CHILDREN_VIRTUAL_SNAPSHOT_ID);
    }

    fn build_pointer_snapshot_queries(
        &self,
        project_symbol_catalog: &ProjectSymbolCatalog,
        symbol_tree_entries: &[SymbolTreeNode],
    ) -> Vec<VirtualSnapshotQuery> {
        symbol_tree_entries
            .iter()
            .filter(|symbol_tree_entry| {
                symbol_tree_entry.is_expanded()
                    && !matches!(symbol_tree_entry.get_kind(), SymbolTreeNodeKind::PointerTarget)
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
        symbol_tree_entry: &SymbolTreeNode,
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

    fn resolve_global_pointer_chain_from_pointer_snapshot(
        project_symbol_catalog: &ProjectSymbolCatalog,
        resolved_pointer_targets_by_query_id: &HashMap<String, ResolvedPointerTarget>,
        resolver_pointer_snapshot_queries: &RefCell<Vec<VirtualSnapshotQuery>>,
        pointer_chain: &SymbolicPointerChain,
    ) -> Result<i128, SymbolicResolverEvaluationError> {
        let query_id = Self::global_pointer_chain_query_id(pointer_chain);

        if let Some(resolved_pointer_target) = resolved_pointer_targets_by_query_id.get(&query_id) {
            return Ok(i128::from(resolved_pointer_target.get_target_locator().get_focus_address()));
        }

        let Some(pointer_snapshot_query) = Self::build_global_pointer_chain_virtual_snapshot_query(project_symbol_catalog, pointer_chain, query_id) else {
            return Err(SymbolicResolverEvaluationError::UnknownGlobalPointerChain(pointer_chain.to_string()));
        };

        resolver_pointer_snapshot_queries
            .borrow_mut()
            .push(pointer_snapshot_query);

        Err(SymbolicResolverEvaluationError::UnknownGlobalPointerChain(pointer_chain.to_string()))
    }

    fn resolve_relative_pointer_chain_from_pointer_snapshot(
        resolved_pointer_targets_by_query_id: &HashMap<String, ResolvedPointerTarget>,
        resolver_pointer_snapshot_queries: &RefCell<Vec<VirtualSnapshotQuery>>,
        root_locator: &ProjectSymbolLocator,
        pointer_chain: &SymbolicPointerChain,
    ) -> Result<i128, SymbolicResolverEvaluationError> {
        let query_id = Self::relative_pointer_chain_query_id(root_locator, pointer_chain);

        if let Some(resolved_pointer_target) = resolved_pointer_targets_by_query_id.get(&query_id) {
            return Ok(i128::from(resolved_pointer_target.get_target_locator().get_focus_address()));
        }

        let Some(pointer_snapshot_query) = Self::build_relative_pointer_chain_virtual_snapshot_query(root_locator, pointer_chain, query_id) else {
            return Err(SymbolicResolverEvaluationError::UnknownRelativePointerChain(pointer_chain.to_string()));
        };

        resolver_pointer_snapshot_queries
            .borrow_mut()
            .push(pointer_snapshot_query);

        Err(SymbolicResolverEvaluationError::UnknownRelativePointerChain(pointer_chain.to_string()))
    }

    fn build_global_pointer_chain_virtual_snapshot_query(
        project_symbol_catalog: &ProjectSymbolCatalog,
        pointer_chain: &SymbolicPointerChain,
        query_id: String,
    ) -> Option<VirtualSnapshotQuery> {
        let resolved_pointer_chain = pointer_chain.with_resolved_symbols(|module_name, symbol_name| {
            project_symbol_catalog
                .find_module_symbol_offset_by_display_name(module_name, symbol_name)
                .and_then(|symbol_offset| i64::try_from(symbol_offset).ok())
        })?;
        let root_offset = resolved_pointer_chain.get_numeric_root_offset()?;
        let root_offset = u64::try_from(root_offset).ok()?;

        Some(VirtualSnapshotQuery::Pointer {
            query_id,
            pointer: Pointer::new_with_size_and_segments(
                root_offset,
                resolved_pointer_chain.get_tail_links().to_vec(),
                resolved_pointer_chain.get_module_name().to_string(),
                resolved_pointer_chain.get_pointer_size(),
            ),
            symbolic_struct_definition: SymbolicStructDefinition::new_anonymous(Vec::new()),
        })
    }

    fn build_relative_pointer_chain_virtual_snapshot_query(
        root_locator: &ProjectSymbolLocator,
        pointer_chain: &SymbolicPointerChain,
        query_id: String,
    ) -> Option<VirtualSnapshotQuery> {
        let root_offset = pointer_chain.get_numeric_root_offset()?;
        let root_address = Pointer::apply_pointer_offset(root_locator.get_focus_address(), root_offset)?;

        Some(VirtualSnapshotQuery::Pointer {
            query_id,
            pointer: Pointer::new_with_size_and_segments(
                root_address,
                pointer_chain.get_tail_links().to_vec(),
                root_locator.get_focus_module_name().to_string(),
                pointer_chain.get_pointer_size(),
            ),
            symbolic_struct_definition: SymbolicStructDefinition::new_anonymous(Vec::new()),
        })
    }

    fn global_pointer_chain_query_id(pointer_chain: &SymbolicPointerChain) -> String {
        format!(
            "resolver_pointer:{}:{}:{}",
            pointer_chain.get_module_name(),
            pointer_chain.get_pointer_size(),
            SymbolicPointerChainLink::display_text_list(pointer_chain.get_links())
        )
    }

    fn relative_pointer_chain_query_id(
        root_locator: &ProjectSymbolLocator,
        pointer_chain: &SymbolicPointerChain,
    ) -> String {
        format!(
            "resolver_relative_pointer:{}:{}:{}",
            root_locator.to_locator_key(),
            pointer_chain.get_pointer_size(),
            SymbolicPointerChainLink::display_text_list(pointer_chain.get_links())
        )
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

    fn sync_symbol_scalar_virtual_snapshot(
        &self,
        scalar_snapshot_queries: Vec<VirtualSnapshotQuery>,
    ) {
        self.app_context
            .engine_unprivileged_state
            .set_virtual_snapshot_queries(
                Self::SCALAR_VALUES_VIRTUAL_SNAPSHOT_ID,
                Self::SCALAR_VALUES_REFRESH_INTERVAL,
                SymbolTreeScalarValue::deduplicate_queries_by_id(scalar_snapshot_queries),
            );
        self.app_context
            .engine_unprivileged_state
            .request_virtual_snapshot_refresh(Self::SCALAR_VALUES_VIRTUAL_SNAPSHOT_ID);
    }

    fn collect_scalar_values_by_query_id(&self) -> HashMap<String, i128> {
        let Some(virtual_snapshot) = self
            .app_context
            .engine_unprivileged_state
            .get_virtual_snapshot(Self::SCALAR_VALUES_VIRTUAL_SNAPSHOT_ID)
        else {
            return HashMap::new();
        };

        virtual_snapshot
            .get_query_results()
            .iter()
            .filter_map(|(query_id, virtual_snapshot_query_result)| {
                let memory_read_response = virtual_snapshot_query_result.memory_read_response.as_ref()?;

                if !memory_read_response.success {
                    return None;
                }

                let first_read_field_data_value = memory_read_response
                    .valued_struct
                    .get_fields()
                    .first()
                    .and_then(|valued_struct_field| valued_struct_field.get_data_value())?;
                let scalar_value = self
                    .app_context
                    .engine_unprivileged_state
                    .read_scalar_integer_value(first_read_field_data_value)
                    .ok()
                    .flatten()?;

                Some((query_id.clone(), scalar_value))
            })
            .collect()
    }

    fn sync_symbol_preview_virtual_snapshot(
        &self,
        project_symbol_catalog: &ProjectSymbolCatalog,
        symbol_tree_entries: &[SymbolTreeNode],
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
        symbol_tree_entries: &[SymbolTreeNode],
    ) -> Vec<VirtualSnapshotQuery> {
        symbol_tree_entries
            .iter()
            .filter(|symbol_tree_entry| Self::symbol_tree_entry_should_query_preview(symbol_tree_entry))
            .filter_map(|symbol_tree_entry| self.build_symbol_preview_virtual_snapshot_query(project_symbol_catalog, symbol_tree_entry))
            .collect()
    }

    fn symbol_tree_entry_should_query_preview(symbol_tree_entry: &SymbolTreeNode) -> bool {
        !matches!(
            symbol_tree_entry.get_kind(),
            SymbolTreeNodeKind::ModuleSpace { .. } | SymbolTreeNodeKind::UnassignedSegment { .. }
        )
    }

    fn build_symbol_preview_virtual_snapshot_query(
        &self,
        project_symbol_catalog: &ProjectSymbolCatalog,
        symbol_tree_entry: &SymbolTreeNode,
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
        symbol_tree_entries: &[SymbolTreeNode],
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
        symbol_tree_entry: &SymbolTreeNode,
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
                DataValuePreviewFormatter::format_anonymous_value_preview(
                    &anonymous_value_string,
                    symbol_tree_entry.get_container_type(),
                    DataValuePreviewFormatter::array_preview_was_truncated(symbol_tree_entry.get_container_type()),
                    Self::PREVIEW_FORMAT_OPTIONS,
                )
            })
            .unwrap_or_default()
    }

    fn build_named_symbolic_struct_definition_for_preview(
        &self,
        project_symbol_catalog: &ProjectSymbolCatalog,
        symbol_tree_entry: &SymbolTreeNode,
        truncate_preview_arrays: bool,
    ) -> Option<SymbolicStructDefinition> {
        let entry_field_definition = SymbolicFieldDefinition::from_str(&symbol_tree_entry.get_display_type_id()).ok()?;
        let preview_container_type = if truncate_preview_arrays {
            DataValuePreviewFormatter::limit_array_container_type(entry_field_definition.get_container_type())
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

    fn build_named_symbolic_struct_definition_for_symbol_tree_entry(
        &self,
        project_symbol_catalog: &ProjectSymbolCatalog,
        symbol_tree_entry: &SymbolTreeNode,
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

    fn resolve_symbol_tree_entry_size_for_struct_viewer(
        engine_execution_context: &Arc<dyn EngineExecutionContext>,
        symbol_tree_entry: &SymbolTreeNode,
    ) -> Option<u64> {
        if let SymbolTreeNodeKind::UnassignedSegment { length, .. } = symbol_tree_entry.get_kind() {
            return Some(*length);
        }

        let symbolic_field_definition = SymbolicFieldDefinition::from_str(&symbol_tree_entry.get_display_type_id()).ok()?;

        Self::resolve_symbolic_field_size_in_bytes(engine_execution_context, &symbolic_field_definition, &mut HashSet::new())
    }

    fn symbol_tree_entry_should_use_external_value_viewer(symbol_tree_entry: &SymbolTreeNode) -> bool {
        if matches!(
            symbol_tree_entry.get_kind(),
            SymbolTreeNodeKind::ModuleSpace { .. } | SymbolTreeNodeKind::UnassignedSegment { .. }
        ) {
            return false;
        }

        matches!(symbol_tree_entry.get_container_type(), ContainerType::Array | ContainerType::ArrayFixed(_))
    }

    fn resolve_symbolic_field_size_in_bytes(
        engine_execution_context: &Arc<dyn EngineExecutionContext>,
        symbolic_field_definition: &SymbolicFieldDefinition,
        visited_type_ids: &mut HashSet<String>,
    ) -> Option<u64> {
        SymbolLayoutSizeResolver::resolve_symbolic_field_size_in_bytes(
            symbolic_field_definition,
            |data_type_ref| {
                engine_execution_context
                    .get_default_value(data_type_ref)
                    .map(|default_value| default_value.get_size_in_bytes())
            },
            |struct_layout_id| engine_execution_context.resolve_struct_layout_definition(struct_layout_id),
            visited_type_ids,
        )
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
                            "Unexpected response variant for Symbol Tree memory read request: {:?}",
                            unexpected_response
                        )),
                    };
                    let _ = memory_read_response_sender.send(conversion_result);
                }),
            ),
            Err(error) => {
                log::error!("Failed to acquire engine bindings lock for Symbol Tree memory read request: {}", error);
                return None;
            }
        };

        if let Err(error) = dispatch_result {
            log::error!("Failed to dispatch Symbol Tree memory read request: {}", error);
            return None;
        }

        match memory_read_response_receiver.recv_timeout(Duration::from_secs(2)) {
            Ok(Ok(memory_read_response)) => Some(memory_read_response),
            Ok(Err(error)) => {
                log::error!("Failed to convert Symbol Tree memory read response: {}", error);
                None
            }
            Err(error) => {
                log::error!("Timed out waiting for Symbol Tree memory read response: {}", error);
                None
            }
        }
    }
}
