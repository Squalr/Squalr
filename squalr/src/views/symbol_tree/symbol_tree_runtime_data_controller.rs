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
    symbol_tree::operations::build_symbol_tree::ResolvedPointerTarget,
    symbol_tree::symbol_tree::SymbolTree,
    symbol_tree::symbol_tree_node::{SymbolTreeNode, SymbolTreeNodeKind},
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
    const PREVIEW_FORMAT_OPTIONS: DataValuePreviewFormatOptions = DataValuePreviewFormatOptions::new(3, 24, 48);
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

    pub fn has_in_flight_virtual_snapshot_refresh(&self) -> bool {
        [
            Self::POINTER_CHILDREN_VIRTUAL_SNAPSHOT_ID,
            Self::SCALAR_VALUES_VIRTUAL_SNAPSHOT_ID,
            Self::PREVIEW_VALUES_VIRTUAL_SNAPSHOT_ID,
        ]
        .into_iter()
        .filter_map(|virtual_snapshot_id| {
            self.app_context
                .engine_unprivileged_state
                .get_virtual_snapshot(virtual_snapshot_id)
        })
        .any(|virtual_snapshot| virtual_snapshot.get_is_refresh_in_progress())
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
            expanded_tree_node_keys,
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
        self.sync_pointer_child_virtual_snapshot(
            project_symbol_catalog,
            &symbol_tree_entries,
            expanded_tree_node_keys,
            resolver_pointer_snapshot_queries.borrow().clone(),
        );
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

        if !SymbolTreeDetailsProjection::should_include_runtime_value_fields(symbol_tree_entry) {
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
        expanded_tree_node_keys: &HashSet<String>,
        additional_pointer_snapshot_queries: Vec<VirtualSnapshotQuery>,
    ) {
        let mut pointer_snapshot_queries = self.build_pointer_snapshot_queries(project_symbol_catalog, symbol_tree_entries, expanded_tree_node_keys);

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
        expanded_tree_node_keys: &HashSet<String>,
    ) -> Vec<VirtualSnapshotQuery> {
        symbol_tree_entries
            .iter()
            .filter(|symbol_tree_entry| {
                symbol_tree_entry.can_expand()
                    && expanded_tree_node_keys.contains(symbol_tree_entry.get_node_key())
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

#[cfg(test)]
mod tests {
    use super::SymbolTreeRuntimeDataController;
    use crate::{
        app_context::AppContext,
        models::docking::{docking_manager::DockingManager, hierarchy::dock_node::DockNode},
        ui::theme::Theme,
    };
    use eframe::egui::Context;
    use squalr_engine::engine_bindings::standalone::standalone_engine_api_unprivileged_bindings::StandaloneEngineApiUnprivilegedBindings;
    use squalr_engine::engine_mode::EngineMode;
    use squalr_engine::engine_privileged_state::create_engine_privileged_state;
    use squalr_engine_api::{
        engine::engine_api_unprivileged_bindings::EngineApiUnprivilegedBindings,
        engine::engine_execution_context::EngineExecutionContext,
        plugins::symbol_tree::{
            symbol_tree_action::{
                DataTypeRegistryStore, ProcessMemoryStore, ProjectSymbolStore, SymbolTreeActionContext, SymbolTreeActionSelection, SymbolTreeActionServices,
                SymbolTreeWindowStore,
            },
            symbol_tree_plugin::SymbolTreePlugin,
        },
        registries::symbols::struct_layout_descriptor::StructLayoutDescriptor,
        structures::data_types::data_type_ref::DataTypeRef,
        structures::data_values::container_type::ContainerType,
        structures::projects::{
            project_symbol_catalog::ProjectSymbolCatalog, project_symbol_module::ProjectSymbolModule, project_symbol_module_field::ProjectSymbolModuleField,
        },
        structures::structs::symbolic_field_definition::SymbolicFieldDefinition,
        structures::structs::symbolic_struct_definition::SymbolicStructDefinition,
    };
    use squalr_engine_session::{
        engine_unprivileged_state::{EngineUnprivilegedState, EngineUnprivilegedStateOptions},
        os::ProcessQueryError,
        virtual_snapshots::virtual_snapshot_query::VirtualSnapshotQuery,
    };
    use squalr_plugin_binary_symbols::BinarySymbolsPlugin;
    use std::{
        collections::HashSet,
        path::Path,
        str::FromStr,
        sync::{Arc, Mutex, RwLock},
        thread,
        time::{Duration, Instant},
    };

    struct TestProjectSymbolStore {
        project_symbol_catalog: Arc<Mutex<ProjectSymbolCatalog>>,
    }

    impl ProjectSymbolStore for TestProjectSymbolStore {
        fn read_catalog(&self) -> Result<ProjectSymbolCatalog, String> {
            self.project_symbol_catalog
                .lock()
                .map(|project_symbol_catalog| project_symbol_catalog.clone())
                .map_err(|error| format!("Failed to acquire test project symbol catalog lock for read: {error}"))
        }

        fn write_catalog(
            &self,
            _reason: &str,
            update_catalog: Box<dyn FnOnce(&mut ProjectSymbolCatalog) -> Result<(), String> + Send>,
        ) -> Result<(), String> {
            let mut project_symbol_catalog = self
                .project_symbol_catalog
                .lock()
                .map_err(|error| format!("Failed to acquire test project symbol catalog lock for write: {error}"))?;

            update_catalog(&mut project_symbol_catalog)
        }
    }

    struct TestProcessMemoryStore {
        engine_unprivileged_state: Arc<EngineUnprivilegedState>,
    }

    impl ProcessMemoryStore for TestProcessMemoryStore {
        fn read_module_bytes(
            &self,
            module_name: &str,
            offset: u64,
            length: u64,
        ) -> Result<Vec<u8>, String> {
            let Some(memory_read_response) = SymbolTreeRuntimeDataController::dispatch_memory_read_request(
                &(self.engine_unprivileged_state.clone() as Arc<dyn EngineExecutionContext>),
                offset,
                module_name,
                &SymbolicStructDefinition::new_anonymous(vec![SymbolicFieldDefinition::new(
                    DataTypeRef::new("u8"),
                    ContainerType::ArrayFixed(length),
                )]),
            ) else {
                return Err(format!("Timed out reading {module_name}+0x{offset:X} for test process memory store."));
            };

            if !memory_read_response.success {
                return Err(format!("Failed reading {module_name}+0x{offset:X} for test process memory store."));
            }

            Ok(memory_read_response.valued_struct.get_bytes())
        }
    }

    struct TestDataTypeRegistryStore {
        engine_unprivileged_state: Arc<EngineUnprivilegedState>,
    }

    impl DataTypeRegistryStore for TestDataTypeRegistryStore {
        fn get_registered_data_type_refs(&self) -> Vec<DataTypeRef> {
            self.engine_unprivileged_state.get_registered_data_type_refs()
        }

        fn get_unit_size_in_bytes(
            &self,
            data_type_ref: &DataTypeRef,
        ) -> u64 {
            self.engine_unprivileged_state
                .get_unit_size_in_bytes(data_type_ref)
        }
    }

    struct TestSymbolTreeWindowStore;

    impl SymbolTreeWindowStore for TestSymbolTreeWindowStore {
        fn request_refresh(&self) {}

        fn focus_tree_node(
            &self,
            _tree_node_key: &str,
        ) {
        }
    }

    struct TestSymbolTreeActionServices {
        project_symbol_store: TestProjectSymbolStore,
        process_memory_store: TestProcessMemoryStore,
        data_type_registry_store: TestDataTypeRegistryStore,
        symbol_tree_window_store: TestSymbolTreeWindowStore,
    }

    impl SymbolTreeActionServices for TestSymbolTreeActionServices {
        fn symbol_store(&self) -> &dyn ProjectSymbolStore {
            &self.project_symbol_store
        }

        fn process_memory(&self) -> &dyn ProcessMemoryStore {
            &self.process_memory_store
        }

        fn data_type_registry(&self) -> &dyn DataTypeRegistryStore {
            &self.data_type_registry_store
        }

        fn symbol_tree_window(&self) -> &dyn SymbolTreeWindowStore {
            &self.symbol_tree_window_store
        }
    }

    #[cfg(target_os = "macos")]
    fn create_self_attached_app_context() -> Result<(Arc<AppContext>, String), ProcessQueryError> {
        let engine_privileged_state = create_engine_privileged_state(EngineMode::Standalone)
            .map_err(|error| ProcessQueryError::internal("create_engine_privileged_state", error.to_string()))?;
        let current_process_name = std::env::current_exe()
            .ok()
            .and_then(|current_executable_path| {
                current_executable_path
                    .file_name()
                    .map(|file_name| file_name.to_string_lossy().to_string())
            })
            .unwrap_or_else(|| String::from("cargo"));
        let current_process_info =
            squalr_engine_api::structures::processes::process_info::ProcessInfo::new(std::process::id(), current_process_name.clone(), true, None);
        let opened_process_info = engine_privileged_state
            .get_os_providers()
            .process_query
            .open_process(&current_process_info)?;
        engine_privileged_state
            .get_process_manager()
            .set_opened_process(opened_process_info.clone());

        let module_name = Path::new(&current_process_name)
            .file_name()
            .map(|file_name| file_name.to_string_lossy().to_string())
            .unwrap_or(current_process_name);
        let engine_bindings: Arc<RwLock<dyn EngineApiUnprivilegedBindings>> =
            Arc::new(RwLock::new(StandaloneEngineApiUnprivilegedBindings::new(&engine_privileged_state)));
        let engine_unprivileged_state =
            EngineUnprivilegedState::new_with_options(engine_bindings, EngineUnprivilegedStateOptions { enable_console_logging: false });
        engine_unprivileged_state.initialize();

        let deadline = Instant::now() + Duration::from_secs(2);
        while engine_unprivileged_state
            .get_registered_data_type_refs()
            .is_empty()
            && Instant::now() < deadline
        {
            thread::sleep(Duration::from_millis(10));
        }

        let egui_context = Context::default();
        let app_context = Arc::new(AppContext::new(
            egui_context.clone(),
            Arc::new(Theme::new(&egui_context)),
            Arc::new(RwLock::new(DockingManager::new(DockNode::default()))),
            engine_unprivileged_state,
        ));

        Ok((app_context, module_name))
    }

    #[cfg(target_os = "macos")]
    fn populate_self_attached_macho_symbol_catalog(
        app_context: &Arc<AppContext>,
        module_name: &str,
    ) -> ProjectSymbolCatalog {
        let project_symbol_catalog = Arc::new(Mutex::new(ProjectSymbolCatalog::new_with_modules_and_symbol_claims(
            vec![ProjectSymbolModule::new(module_name.to_string(), 0x1000)],
            Vec::new(),
            Vec::new(),
        )));
        let services = TestSymbolTreeActionServices {
            project_symbol_store: TestProjectSymbolStore {
                project_symbol_catalog: project_symbol_catalog.clone(),
            },
            process_memory_store: TestProcessMemoryStore {
                engine_unprivileged_state: app_context.engine_unprivileged_state.clone(),
            },
            data_type_registry_store: TestDataTypeRegistryStore {
                engine_unprivileged_state: app_context.engine_unprivileged_state.clone(),
            },
            symbol_tree_window_store: TestSymbolTreeWindowStore,
        };
        let plugin = BinarySymbolsPlugin::new();
        let action = plugin
            .symbol_tree_actions()
            .into_iter()
            .find(|symbol_tree_action| symbol_tree_action.action_id() == "builtin.symbols.binary.populate-binary-symbols")
            .expect("Expected binary symbols plugin to expose the populate action.");

        action
            .execute(
                &SymbolTreeActionContext::new(SymbolTreeActionSelection::ModuleRoot {
                    module_name: module_name.to_string(),
                }),
                &services,
            )
            .expect("Expected Mach-O population for self-attached module to succeed.");

        project_symbol_catalog
            .lock()
            .expect("Expected test project symbol catalog lock after Mach-O population.")
            .clone()
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn build_runtime_data_populates_self_attach_preview_values_for_expanded_module_fields() {
        let (app_context, module_name) = create_self_attached_app_context().expect("Expected self-attach app context setup to succeed.");
        let mut project_symbol_catalog =
            ProjectSymbolCatalog::new_with_modules_and_symbol_claims(vec![ProjectSymbolModule::new(module_name.clone(), 0x1000)], Vec::new(), Vec::new());
        project_symbol_catalog
            .find_symbol_module_mut(&module_name)
            .expect("Expected test module to exist in project symbol catalog.")
            .get_fields_mut()
            .push(ProjectSymbolModuleField::new(String::from("magic"), 0, String::from("u32")));

        let expanded_tree_node_keys = HashSet::from([format!("module:{module_name}")]);
        let symbol_tree_runtime_data_controller = SymbolTreeRuntimeDataController::new(app_context);

        let first_runtime_data = symbol_tree_runtime_data_controller.build_runtime_data(&project_symbol_catalog, &expanded_tree_node_keys);
        let magic_node_key = first_runtime_data
            .symbol_tree_entries
            .iter()
            .find(|symbol_tree_entry| symbol_tree_entry.get_display_name() == "magic")
            .map(|symbol_tree_entry| symbol_tree_entry.get_node_key().to_string())
            .expect("Expected expanded symbol tree to include the test module field.");

        thread::sleep(Duration::from_millis(300));

        let second_runtime_data = symbol_tree_runtime_data_controller.build_runtime_data(&project_symbol_catalog, &expanded_tree_node_keys);
        let preview_value = second_runtime_data
            .preview_values_by_node_key
            .get(&magic_node_key)
            .cloned()
            .unwrap_or_default();

        assert!(
            !preview_value.is_empty(),
            "Expected symbol tree preview value for self-attached expanded module field to be populated on the second refresh."
        );
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn build_runtime_data_populates_self_attach_preview_values_for_nested_struct_fields() {
        let (app_context, module_name) = create_self_attached_app_context().expect("Expected self-attach app context setup to succeed.");
        let mut project_symbol_catalog = ProjectSymbolCatalog::new_with_modules_and_symbol_claims(
            vec![ProjectSymbolModule::new(module_name.clone(), 0x1000)],
            vec![StructLayoutDescriptor::new(
                String::from("test.header"),
                SymbolicStructDefinition::from_str("magic:u32 @ +0;cputype:u32 @ +4").expect("Expected nested struct layout definition to parse."),
            )],
            Vec::new(),
        );
        project_symbol_catalog
            .find_symbol_module_mut(&module_name)
            .expect("Expected test module to exist in project symbol catalog.")
            .get_fields_mut()
            .push(ProjectSymbolModuleField::new(String::from("Header"), 0, String::from("test.header")));

        let mut expanded_tree_node_keys = HashSet::from([format!("module:{module_name}")]);
        let symbol_tree_runtime_data_controller = SymbolTreeRuntimeDataController::new(app_context);
        let first_runtime_data = symbol_tree_runtime_data_controller.build_runtime_data(&project_symbol_catalog, &expanded_tree_node_keys);
        let header_node_key = first_runtime_data
            .symbol_tree_entries
            .iter()
            .find(|symbol_tree_entry| symbol_tree_entry.get_display_name() == "Header")
            .map(|symbol_tree_entry| symbol_tree_entry.get_node_key().to_string())
            .expect("Expected expanded symbol tree to include the nested struct module field.");
        expanded_tree_node_keys.insert(header_node_key);

        thread::sleep(Duration::from_millis(300));

        let second_runtime_data = symbol_tree_runtime_data_controller.build_runtime_data(&project_symbol_catalog, &expanded_tree_node_keys);
        let magic_node_key = second_runtime_data
            .symbol_tree_entries
            .iter()
            .find(|symbol_tree_entry| symbol_tree_entry.get_display_name() == "magic")
            .map(|symbol_tree_entry| symbol_tree_entry.get_node_key().to_string())
            .expect("Expected expanded nested struct field node to exist.");

        thread::sleep(Duration::from_millis(300));

        let third_runtime_data = symbol_tree_runtime_data_controller.build_runtime_data(&project_symbol_catalog, &expanded_tree_node_keys);
        let preview_value = third_runtime_data
            .preview_values_by_node_key
            .get(&magic_node_key)
            .cloned()
            .unwrap_or_default();

        assert!(
            !preview_value.is_empty(),
            "Expected nested self-attach struct field preview value to be populated after refresh."
        );
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn build_runtime_data_populates_self_attach_preview_values_for_populated_macho_leaf_fields() {
        let (app_context, module_name) = create_self_attached_app_context().expect("Expected self-attach app context setup to succeed.");
        let project_symbol_catalog = populate_self_attached_macho_symbol_catalog(&app_context, &module_name);
        let symbol_tree_runtime_data_controller = SymbolTreeRuntimeDataController::new(app_context);
        let mut expanded_tree_node_keys = HashSet::from([format!("module:{module_name}")]);

        let first_runtime_data = symbol_tree_runtime_data_controller.build_runtime_data(&project_symbol_catalog, &expanded_tree_node_keys);
        let mach_headers_node_key = first_runtime_data
            .symbol_tree_entries
            .iter()
            .find(|symbol_tree_entry| symbol_tree_entry.get_display_name() == "Mach-O Headers")
            .map(|symbol_tree_entry| symbol_tree_entry.get_node_key().to_string())
            .expect("Expected Mach-O population to add the Mach-O Headers root field.");
        expanded_tree_node_keys.insert(mach_headers_node_key);

        thread::sleep(Duration::from_millis(300));

        let second_runtime_data = symbol_tree_runtime_data_controller.build_runtime_data(&project_symbol_catalog, &expanded_tree_node_keys);
        let header_node_key = second_runtime_data
            .symbol_tree_entries
            .iter()
            .find(|symbol_tree_entry| symbol_tree_entry.get_display_name() == "Header")
            .map(|symbol_tree_entry| symbol_tree_entry.get_node_key().to_string())
            .expect("Expected expanded Mach-O headers tree to include the Header node.");
        expanded_tree_node_keys.insert(header_node_key);

        thread::sleep(Duration::from_millis(300));

        let third_runtime_data = symbol_tree_runtime_data_controller.build_runtime_data(&project_symbol_catalog, &expanded_tree_node_keys);
        let magic_symbol_tree_entry = third_runtime_data
            .symbol_tree_entries
            .iter()
            .find(|symbol_tree_entry| symbol_tree_entry.get_display_name() == "magic")
            .expect("Expected expanded Mach-O header tree to include the magic field.");
        let magic_node_key = magic_symbol_tree_entry.get_node_key().to_string();

        let preview_query = symbol_tree_runtime_data_controller
            .build_symbol_preview_virtual_snapshot_query(&project_symbol_catalog, magic_symbol_tree_entry)
            .expect("Expected Mach-O magic leaf to produce a preview query.");
        let VirtualSnapshotQuery::Address {
            address,
            module_name: preview_module_name,
            symbolic_struct_definition,
            ..
        } = preview_query
        else {
            panic!("Expected Mach-O leaf preview query to use address mode.");
        };
        let direct_memory_read_response = SymbolTreeRuntimeDataController::dispatch_memory_read_request(
            &(symbol_tree_runtime_data_controller
                .app_context
                .engine_unprivileged_state
                .clone() as Arc<dyn EngineExecutionContext>),
            address,
            &preview_module_name,
            &symbolic_struct_definition,
        )
        .expect("Expected direct preview memory read to return a response.");

        let preview_deadline = Instant::now() + Duration::from_secs(5);
        let mut preview_value = String::new();
        let mut preview_snapshot_generation = 0_u64;
        let mut preview_snapshot_query_count = 0_usize;
        let mut preview_snapshot_in_progress = false;

        while Instant::now() < preview_deadline {
            thread::sleep(Duration::from_millis(250));

            let runtime_data = symbol_tree_runtime_data_controller.build_runtime_data(&project_symbol_catalog, &expanded_tree_node_keys);
            preview_value = runtime_data
                .preview_values_by_node_key
                .get(&magic_node_key)
                .cloned()
                .unwrap_or_default();

            if let Some(preview_virtual_snapshot) = symbol_tree_runtime_data_controller
                .app_context
                .engine_unprivileged_state
                .get_virtual_snapshot(SymbolTreeRuntimeDataController::PREVIEW_VALUES_VIRTUAL_SNAPSHOT_ID)
            {
                preview_snapshot_generation = preview_virtual_snapshot.get_generation();
                preview_snapshot_query_count = preview_virtual_snapshot.get_queries().len();
                preview_snapshot_in_progress = preview_virtual_snapshot.get_is_refresh_in_progress();
            }

            if !preview_value.is_empty() {
                break;
            }
        }

        assert!(
            direct_memory_read_response.success,
            "Expected direct Mach-O preview read for self-attach to succeed."
        );
        assert!(
            !preview_value.is_empty(),
            "Expected populated Mach-O leaf fields to show self-attach preview values after refresh. node_key={magic_node_key}, display_type_id={}, locator={}, direct_read_success={}, preview_generation={}, preview_query_count={}, preview_refresh_in_progress={}",
            magic_symbol_tree_entry.get_display_type_id(),
            magic_symbol_tree_entry.get_locator(),
            direct_memory_read_response.success,
            preview_snapshot_generation,
            preview_snapshot_query_count,
            preview_snapshot_in_progress,
        );
    }
}
