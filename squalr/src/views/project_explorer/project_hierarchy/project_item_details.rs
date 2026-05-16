use crate::views::struct_viewer::view_data::struct_viewer_view_data::StructViewerViewData;
use squalr_engine_api::commands::memory::query::memory_query_request::MemoryQueryRequest;
use squalr_engine_api::commands::memory::query::memory_query_response::MemoryQueryResponse;
use squalr_engine_api::commands::memory::read::memory_read_request::MemoryReadRequest;
use squalr_engine_api::commands::memory::read::memory_read_response::MemoryReadResponse;
use squalr_engine_api::commands::privileged_command_request::PrivilegedCommandRequest;
use squalr_engine_api::commands::privileged_command_response::TypedPrivilegedCommandResponse;
use squalr_engine_api::engine::engine_execution_context::EngineExecutionContext;
use squalr_engine_api::plugins::instruction_set::normalize_instruction_data_type_id;
use squalr_engine_api::structures::data_types::built_in_types::string::utf8::data_type_string_utf8::DataTypeStringUtf8;
use squalr_engine_api::structures::data_types::data_type_ref::DataTypeRef;
use squalr_engine_api::structures::data_values::{
    anonymous_value_string::AnonymousValueString,
    container_type::ContainerType,
    data_value_preview_formatter::{DataValuePreviewFormatOptions, DataValuePreviewFormatter},
};
use squalr_engine_api::structures::memory::normalized_module::NormalizedModule;
use squalr_engine_api::structures::memory::{pointer::Pointer, pointer_chain_segment::PointerChainSegment};
use squalr_engine_api::structures::pointer_scans::pointer_scan_pointer_size::PointerScanPointerSize;
use squalr_engine_api::structures::projects::project_info::ProjectInfo;
use squalr_engine_api::structures::projects::project_items::built_in_types::{
    project_item_type_address::ProjectItemTypeAddress, project_item_type_address_target::ProjectItemAddressTarget,
    project_item_type_directory::ProjectItemTypeDirectory, project_item_type_pointer::ProjectItemTypePointer,
};
use squalr_engine_api::structures::projects::project_items::project_item::ProjectItem;
use squalr_engine_api::structures::structs::valued_struct::ValuedStruct;
use squalr_engine_api::structures::structs::valued_struct_field::{ValuedStructField, ValuedStructFieldData};
use squalr_engine_api::structures::structs::{symbolic_field_definition::SymbolicFieldDefinition, symbolic_struct_definition::SymbolicStructDefinition};
use squalr_engine_session::{
    engine_unprivileged_state::EngineUnprivilegedState,
    virtual_snapshots::{virtual_snapshot_query::VirtualSnapshotQuery, virtual_snapshot_query_result::VirtualSnapshotQueryResult},
};
use std::path::Path;
use std::str::FromStr;
use std::sync::Arc;
use std::sync::mpsc;
use std::time::Duration;

#[derive(Clone)]
pub struct ProjectItemValueEditContext {
    pub project_item_name: String,
    pub value_field_name: String,
    pub validation_data_type_ref: DataTypeRef,
    pub initial_value_edit: AnonymousValueString,
}

pub struct ProjectItemDetails;

impl ProjectItemDetails {
    pub const TARGET_FIELD_POINTER_OFFSETS: &'static str = StructViewerViewData::VIRTUAL_FIELD_PROJECT_ITEM_POINTER_OFFSETS;
    pub const TARGET_FIELD_POINTER_SIZE: &'static str = StructViewerViewData::VIRTUAL_FIELD_PROJECT_ITEM_POINTER_SIZE;

    const PROJECT_ITEM_PREVIEW_FORMAT_OPTIONS: DataValuePreviewFormatOptions = DataValuePreviewFormatOptions::new(4, 96);

    pub fn can_open_project_item_in_memory_viewer(project_item: &ProjectItem) -> bool {
        let project_item_type_id = project_item.get_item_type().get_project_item_type_id();

        project_item_type_id == ProjectItemTypeAddress::PROJECT_ITEM_TYPE_ID || project_item_type_id == ProjectItemTypePointer::PROJECT_ITEM_TYPE_ID
    }

    pub fn build_struct_view_properties(project_item: &ProjectItem) -> ValuedStruct {
        let mut fields = project_item
            .get_properties()
            .get_fields()
            .iter()
            .filter(|valued_struct_field| Self::should_show_project_item_detail_field(project_item, valued_struct_field.get_name()))
            .map(|valued_struct_field| {
                let is_runtime_value_field = Self::is_runtime_value_field(valued_struct_field.get_name());
                let projected_field_data = Self::project_address_item_target_detail_field_data(project_item, valued_struct_field)
                    .unwrap_or_else(|| valued_struct_field.get_field_data().clone());

                ValuedStructField::new(
                    valued_struct_field.get_name().to_string(),
                    projected_field_data,
                    if is_runtime_value_field {
                        false
                    } else {
                        valued_struct_field.get_is_read_only()
                    },
                )
            })
            .collect::<Vec<_>>();

        if project_item.get_item_type().get_project_item_type_id() == ProjectItemTypeAddress::PROJECT_ITEM_TYPE_ID {
            let mut project_item = project_item.clone();
            let address_target = ProjectItemTypeAddress::get_address_target(&mut project_item);

            Self::append_project_item_address_target_fields(&mut fields, &address_target);
        }

        ValuedStruct::new_anonymous(fields)
    }

    pub fn copy_project_item_preview_fields(
        source_project_item: &ProjectItem,
        target_project_item: &mut ProjectItem,
    ) {
        let preview_value = Self::read_project_item_preview_value(source_project_item);
        let project_item_type_id = target_project_item.get_item_type().get_project_item_type_id();

        if project_item_type_id == ProjectItemTypeAddress::PROJECT_ITEM_TYPE_ID {
            ProjectItemTypeAddress::set_field_freeze_data_value_interpreter(target_project_item, &preview_value);
        } else if project_item_type_id == ProjectItemTypePointer::PROJECT_ITEM_TYPE_ID {
            ProjectItemTypePointer::set_field_freeze_data_value_interpreter(target_project_item, &preview_value);
        }
    }

    pub fn apply_project_item_address_target_edit(
        project_item: &mut ProjectItem,
        edited_field: &ValuedStructField,
    ) -> bool {
        if project_item.get_item_type().get_project_item_type_id() != ProjectItemTypeAddress::PROJECT_ITEM_TYPE_ID {
            return false;
        }

        let edited_field_name = edited_field.get_name();

        let mut updated_address_target = ProjectItemTypeAddress::get_address_target(project_item);
        let did_update_address_target = match edited_field_name {
            ProjectItemTypeAddress::PROPERTY_MODULE => {
                let Some(edited_module_name) = Self::extract_string_value_from_edited_field_allow_empty(edited_field) else {
                    return false;
                };

                updated_address_target.set_module_name(edited_module_name);
                true
            }
            Self::TARGET_FIELD_POINTER_SIZE => {
                let Some(pointer_size_text) = Self::extract_string_value_from_edited_field(edited_field) else {
                    return false;
                };

                if let Ok(pointer_size) = PointerScanPointerSize::from_str(&pointer_size_text) {
                    updated_address_target.set_pointer_size(pointer_size);
                    true
                } else {
                    log::warn!("Ignoring unknown project address pointer size: {}", pointer_size_text);
                    false
                }
            }
            Self::TARGET_FIELD_POINTER_OFFSETS => {
                let Some(pointer_offsets) = Self::extract_pointer_offsets_from_edited_field(edited_field) else {
                    return false;
                };

                updated_address_target.set_pointer_offsets(Self::ensure_minimum_pointer_offsets(pointer_offsets));
                true
            }
            _ => false,
        };

        if did_update_address_target {
            ProjectItemTypeAddress::set_address_target(project_item, updated_address_target);
            true
        } else {
            false
        }
    }

    pub fn resolve_pointer_write_target(
        engine_execution_context: &Arc<dyn EngineExecutionContext>,
        pointer: &Pointer,
    ) -> Option<(u64, String)> {
        let mut current_address = pointer.get_address();
        let mut current_module_name = pointer.get_module_name().to_string();

        for pointer_chain_segment in pointer.get_offset_segments() {
            let pointer_offset = pointer_chain_segment.as_offset()?;
            let pointer_value = Self::read_pointer_value(engine_execution_context, current_address, &current_module_name, pointer.get_pointer_size())?;
            current_address = Pointer::apply_pointer_offset(pointer_value, pointer_offset)?;
            current_module_name.clear();
        }

        Some((current_address, current_module_name))
    }

    pub fn dispatch_memory_query_request(engine_unprivileged_state: &Arc<EngineUnprivilegedState>) -> Option<MemoryQueryResponse> {
        let memory_query_request = MemoryQueryRequest::default();
        let memory_query_command = memory_query_request.to_engine_command();
        let (memory_query_response_sender, memory_query_response_receiver) = mpsc::channel();

        let dispatch_result = match engine_unprivileged_state.get_bindings().read() {
            Ok(engine_bindings) => engine_bindings.dispatch_privileged_command(
                memory_query_command,
                Box::new(move |engine_response| {
                    let conversion_result = match MemoryQueryResponse::from_engine_response(engine_response) {
                        Ok(memory_query_response) => Ok(memory_query_response),
                        Err(unexpected_response) => Err(format!(
                            "Unexpected response variant for project hierarchy memory query request: {:?}",
                            unexpected_response
                        )),
                    };
                    let _ = memory_query_response_sender.send(conversion_result);
                }),
            ),
            Err(error) => {
                log::error!("Failed to acquire engine bindings lock for project hierarchy memory query request: {}", error);
                return None;
            }
        };

        if let Err(error) = dispatch_result {
            log::error!("Failed to dispatch project hierarchy memory query request: {}", error);
            return None;
        }

        match memory_query_response_receiver.recv_timeout(Duration::from_secs(2)) {
            Ok(Ok(memory_query_response)) => Some(memory_query_response),
            Ok(Err(error)) => {
                log::error!("Failed to convert project hierarchy memory query response: {}", error);
                None
            }
            Err(error) => {
                log::error!("Timed out waiting for project hierarchy memory query response: {}", error);
                None
            }
        }
    }

    pub fn resolve_module_relative_address(
        modules: &[NormalizedModule],
        address: u64,
        module_name: &str,
    ) -> Option<u64> {
        modules
            .iter()
            .find(|normalized_module| {
                normalized_module
                    .get_module_name()
                    .eq_ignore_ascii_case(module_name)
            })
            .and_then(|normalized_module| normalized_module.get_base_address().checked_add(address))
    }

    pub fn should_open_project_item_in_code_viewer(project_item: &ProjectItem) -> bool {
        Self::resolve_project_item_symbolic_struct_namespace(project_item)
            .and_then(|symbolic_struct_namespace| normalize_instruction_data_type_id(&symbolic_struct_namespace))
            .map(|data_type_id| matches!(data_type_id.as_str(), "i_x86" | "i_x64"))
            .unwrap_or(false)
    }

    pub fn build_project_item_value_edit_context(
        engine_unprivileged_state: &Arc<EngineUnprivilegedState>,
        opened_project_info: Option<&ProjectInfo>,
        project_item: &ProjectItem,
    ) -> Option<ProjectItemValueEditContext> {
        let project_item_type_id = project_item.get_item_type().get_project_item_type_id();
        let value_field_name = if project_item_type_id == ProjectItemTypeAddress::PROJECT_ITEM_TYPE_ID {
            ProjectItemTypeAddress::PROPERTY_FREEZE_DISPLAY_VALUE
        } else if project_item_type_id == ProjectItemTypePointer::PROJECT_ITEM_TYPE_ID {
            ProjectItemTypePointer::PROPERTY_FREEZE_DISPLAY_VALUE
        } else {
            return None;
        };
        let value_field = project_item.get_properties().get_field(value_field_name)?;
        let value_data_value = value_field.get_data_value()?;
        let symbolic_struct_namespace = Self::resolve_project_item_symbolic_struct_namespace(project_item);
        let symbolic_field_definition = symbolic_struct_namespace
            .as_deref()
            .and_then(|symbolic_struct_namespace| SymbolicFieldDefinition::from_str(symbolic_struct_namespace).ok());
        let validation_data_type_ref = symbolic_field_definition
            .as_ref()
            .map(|symbolic_field_definition| symbolic_field_definition.get_data_type_ref().clone())
            .unwrap_or_else(|| value_data_value.get_data_type_ref().clone());
        let container_type = symbolic_field_definition
            .map(|symbolic_field_definition| symbolic_field_definition.get_container_type())
            .unwrap_or(ContainerType::None);
        let default_format = engine_unprivileged_state.get_default_anonymous_value_string_format(&validation_data_type_ref);
        let engine_execution_context: Arc<dyn EngineExecutionContext> = engine_unprivileged_state.clone();
        let initial_value_edit = symbolic_struct_namespace
            .as_deref()
            .and_then(|symbolic_struct_namespace| {
                Self::read_project_item_runtime_value_from_memory(&engine_execution_context, opened_project_info, project_item, symbolic_struct_namespace)
            })
            .unwrap_or_else(|| {
                let raw_display_value = String::from_utf8(value_data_value.get_value_bytes().clone()).unwrap_or_default();

                AnonymousValueString::new(raw_display_value, default_format, container_type)
            });

        Some(ProjectItemValueEditContext {
            project_item_name: project_item.get_field_name(),
            value_field_name: value_field_name.to_string(),
            validation_data_type_ref,
            initial_value_edit,
        })
    }

    pub fn build_project_item_value_edit_display_values(
        engine_unprivileged_state: &Arc<EngineUnprivilegedState>,
        validation_data_type_ref: &DataTypeRef,
        value_edit: &AnonymousValueString,
    ) -> Vec<AnonymousValueString> {
        let Ok(data_value) = engine_unprivileged_state.deanonymize_value_string(validation_data_type_ref, value_edit) else {
            return Vec::new();
        };

        engine_unprivileged_state
            .anonymize_value_to_supported_formats(&data_value)
            .unwrap_or_else(|_| vec![value_edit.clone()])
    }

    pub fn build_project_item_virtual_snapshot_query(
        opened_project_info: Option<&ProjectInfo>,
        project_item_path: &Path,
        project_item: &ProjectItem,
        engine_unprivileged_state: &Arc<EngineUnprivilegedState>,
    ) -> Option<VirtualSnapshotQuery> {
        let query_id = project_item_path.to_string_lossy().to_string();
        let symbolic_struct_namespace = Self::resolve_project_item_symbolic_struct_namespace(project_item)?;
        let symbolic_struct_definition = Self::build_project_item_preview_symbolic_struct_definition(engine_unprivileged_state, &symbolic_struct_namespace)?;
        let project_item_type_id = project_item.get_item_type().get_project_item_type_id();

        if project_item_type_id == ProjectItemTypeAddress::PROJECT_ITEM_TYPE_ID {
            let mut project_item = project_item.clone();
            let address_target = ProjectItemTypeAddress::get_address_target(&mut project_item);

            let runtime_pointer = Self::resolve_address_target_runtime_pointer(opened_project_info, &address_target)?;

            return if runtime_pointer.get_offset_segments().is_empty() {
                Some(VirtualSnapshotQuery::Address {
                    query_id,
                    address: runtime_pointer.get_address(),
                    module_name: runtime_pointer.get_module_name().to_string(),
                    symbolic_struct_definition,
                })
            } else {
                Some(VirtualSnapshotQuery::Pointer {
                    query_id,
                    pointer: runtime_pointer,
                    symbolic_struct_definition,
                })
            };
        }

        if project_item_type_id == ProjectItemTypePointer::PROJECT_ITEM_TYPE_ID {
            return Some(VirtualSnapshotQuery::Pointer {
                query_id,
                pointer: ProjectItemTypePointer::get_field_pointer(project_item),
                symbolic_struct_definition,
            });
        }

        None
    }

    pub fn build_project_item_preview_value_from_virtual_snapshot_result(
        engine_unprivileged_state: &Arc<EngineUnprivilegedState>,
        _opened_project_info: Option<&ProjectInfo>,
        project_item: &ProjectItem,
        virtual_snapshot_query_result: &VirtualSnapshotQueryResult,
    ) -> String {
        let Some(memory_read_response) = virtual_snapshot_query_result.memory_read_response.as_ref() else {
            return String::new();
        };

        if !memory_read_response.success {
            return String::new();
        }

        let first_read_field_data_value = memory_read_response
            .valued_struct
            .get_fields()
            .first()
            .and_then(|valued_struct_field| valued_struct_field.get_data_value());
        let Some(first_read_field_data_value) = first_read_field_data_value else {
            return String::new();
        };

        let default_anonymous_value_string_format =
            engine_unprivileged_state.get_default_anonymous_value_string_format(first_read_field_data_value.get_data_type_ref());
        let symbolic_field_container_type = Self::resolve_project_item_symbolic_container_type(project_item);
        let preview_was_truncated = Self::project_item_preview_was_truncated(project_item);

        engine_unprivileged_state
            .anonymize_value(first_read_field_data_value, default_anonymous_value_string_format)
            .map(|anonymous_value_string| {
                DataValuePreviewFormatter::format_anonymous_value_preview(
                    &anonymous_value_string,
                    symbolic_field_container_type,
                    preview_was_truncated,
                    Self::PROJECT_ITEM_PREVIEW_FORMAT_OPTIONS,
                )
            })
            .unwrap_or_default()
    }

    pub fn resolve_project_item_runtime_value_target(
        engine_execution_context: &Arc<dyn EngineExecutionContext>,
        opened_project_info: Option<&ProjectInfo>,
        project_item: &ProjectItem,
    ) -> Option<(u64, String)> {
        let project_item_type_id = project_item.get_item_type().get_project_item_type_id();

        if project_item_type_id == ProjectItemTypeAddress::PROJECT_ITEM_TYPE_ID {
            let mut project_item = project_item.clone();
            let address_target = ProjectItemTypeAddress::get_address_target(&mut project_item);

            return Self::resolve_project_item_address_target(engine_execution_context, opened_project_info, &address_target);
        }

        if project_item_type_id == ProjectItemTypePointer::PROJECT_ITEM_TYPE_ID {
            let pointer = ProjectItemTypePointer::get_field_pointer(project_item);

            return Self::resolve_pointer_write_target(engine_execution_context, &pointer);
        }

        None
    }

    pub fn resolve_address_target_runtime_pointer(
        opened_project_info: Option<&ProjectInfo>,
        address_target: &ProjectItemAddressTarget,
    ) -> Option<Pointer> {
        if let Some(opened_project_info) = opened_project_info {
            address_target.to_runtime_pointer_resolving_symbols(opened_project_info.get_project_symbol_catalog())
        } else {
            address_target.to_runtime_pointer()
        }
    }

    pub fn resolve_project_item_runtime_value_byte_count(
        engine_unprivileged_state: &Arc<EngineUnprivilegedState>,
        project_item: &ProjectItem,
    ) -> Option<u64> {
        let symbolic_struct_namespace = Self::resolve_project_item_symbolic_struct_namespace(project_item)?;
        let symbolic_field_definition = SymbolicFieldDefinition::from_str(&symbolic_struct_namespace).ok()?;
        let unit_size_in_bytes = engine_unprivileged_state
            .get_default_value(symbolic_field_definition.get_data_type_ref())
            .map(|default_value| default_value.get_size_in_bytes())
            .unwrap_or(1);

        Some(
            symbolic_field_definition
                .get_container_type()
                .get_total_size_in_bytes(unit_size_in_bytes),
        )
    }

    pub fn is_runtime_value_field(field_name: &str) -> bool {
        field_name == ProjectItemTypeAddress::PROPERTY_FREEZE_DISPLAY_VALUE || field_name == ProjectItemTypePointer::PROPERTY_FREEZE_DISPLAY_VALUE
    }

    pub fn should_apply_struct_field_edit_to_project_item(
        project_item_type_id: &str,
        edited_field_name: &str,
    ) -> bool {
        if Self::is_runtime_value_field(edited_field_name) {
            return false;
        }

        !(edited_field_name == ProjectItem::PROPERTY_NAME && project_item_type_id == ProjectItemTypeDirectory::PROJECT_ITEM_TYPE_ID)
    }

    fn should_show_project_item_detail_field(
        project_item: &ProjectItem,
        field_name: &str,
    ) -> bool {
        if field_name == ProjectItemTypeAddress::PROPERTY_TARGET {
            return false;
        }

        if project_item.get_item_type().get_project_item_type_id() == ProjectItemTypeAddress::PROJECT_ITEM_TYPE_ID
            && field_name == ProjectItemTypeAddress::PROPERTY_ADDRESS
        {
            return false;
        }

        if project_item.get_item_type().get_project_item_type_id() == ProjectItemTypePointer::PROJECT_ITEM_TYPE_ID {
            return field_name != ProjectItemTypePointer::PROPERTY_EVALUATED_POINTER_PATH;
        }

        true
    }

    fn project_address_item_target_detail_field_data(
        project_item: &ProjectItem,
        valued_struct_field: &ValuedStructField,
    ) -> Option<ValuedStructFieldData> {
        if project_item.get_item_type().get_project_item_type_id() != ProjectItemTypeAddress::PROJECT_ITEM_TYPE_ID {
            return None;
        }

        let mut project_item = project_item.clone();
        let address_target = ProjectItemTypeAddress::get_address_target(&mut project_item);

        match valued_struct_field.get_name() {
            ProjectItemTypeAddress::PROPERTY_MODULE => Some(ValuedStructFieldData::Value(DataTypeStringUtf8::get_value_from_primitive_string(
                address_target.get_module_name(),
            ))),
            _ => None,
        }
    }

    fn append_project_item_address_target_fields(
        fields: &mut Vec<ValuedStructField>,
        address_target: &ProjectItemAddressTarget,
    ) {
        fields.push(
            DataTypeStringUtf8::get_value_from_primitive_string(
                address_target
                    .get_pointer_size()
                    .to_data_type_ref()
                    .get_data_type_id(),
            )
            .to_named_valued_struct_field(Self::TARGET_FIELD_POINTER_SIZE.to_string(), false),
        );
        fields.push(
            DataTypeStringUtf8::get_value_from_primitive_string(&Self::format_pointer_offsets(&Self::ensure_minimum_pointer_offsets(
                address_target.get_pointer_offsets().to_vec(),
            )))
            .to_named_valued_struct_field(Self::TARGET_FIELD_POINTER_OFFSETS.to_string(), true),
        );
    }

    fn read_project_item_preview_value(project_item: &ProjectItem) -> String {
        let project_item_type_id = project_item.get_item_type().get_project_item_type_id();

        if project_item_type_id == ProjectItemTypeAddress::PROJECT_ITEM_TYPE_ID {
            let mut project_item = project_item.clone();

            ProjectItemTypeAddress::get_field_freeze_data_value_interpreter(&mut project_item)
        } else if project_item_type_id == ProjectItemTypePointer::PROJECT_ITEM_TYPE_ID {
            ProjectItemTypePointer::get_field_freeze_data_value_interpreter(project_item)
        } else {
            String::new()
        }
    }

    fn ensure_minimum_pointer_offsets(mut pointer_offsets: Vec<PointerChainSegment>) -> Vec<PointerChainSegment> {
        if pointer_offsets.is_empty() {
            pointer_offsets.push(PointerChainSegment::new_offset(0));
        }

        pointer_offsets
    }

    fn extract_pointer_offsets_from_edited_field(edited_field: &ValuedStructField) -> Option<Vec<PointerChainSegment>> {
        let offsets_text = Self::extract_string_value_from_edited_field(edited_field)?;
        let pointer_offsets = PointerChainSegment::parse_text_list(&offsets_text);

        if pointer_offsets.is_empty() { None } else { Some(pointer_offsets) }
    }

    fn format_pointer_offsets(pointer_offsets: &[PointerChainSegment]) -> String {
        PointerChainSegment::display_text_list(pointer_offsets)
    }

    fn read_pointer_value(
        engine_execution_context: &Arc<dyn EngineExecutionContext>,
        address: u64,
        module_name: &str,
        pointer_size: PointerScanPointerSize,
    ) -> Option<u64> {
        let symbolic_struct_definition = SymbolicStructDefinition::new_anonymous(vec![SymbolicFieldDefinition::new(
            pointer_size.to_data_type_ref(),
            ContainerType::None,
        )]);
        let memory_read_response = Self::dispatch_memory_read_request(engine_execution_context, address, module_name, &symbolic_struct_definition)?;

        if !memory_read_response.success {
            return None;
        }

        let data_value = memory_read_response
            .valued_struct
            .get_fields()
            .first()
            .and_then(|valued_struct_field| valued_struct_field.get_data_value())?;

        pointer_size.read_address_value(data_value)
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
                            "Unexpected response variant for project hierarchy memory read request: {:?}",
                            unexpected_response
                        )),
                    };
                    let _ = memory_read_response_sender.send(conversion_result);
                }),
            ),
            Err(error) => {
                log::error!("Failed to acquire engine bindings lock for project hierarchy memory read request: {}", error);
                return None;
            }
        };

        if let Err(error) = dispatch_result {
            log::error!("Failed to dispatch project hierarchy memory read request: {}", error);
            return None;
        }

        match memory_read_response_receiver.recv_timeout(Duration::from_secs(2)) {
            Ok(Ok(memory_read_response)) => Some(memory_read_response),
            Ok(Err(error)) => {
                log::error!("Failed to convert project hierarchy memory read response: {}", error);
                None
            }
            Err(error) => {
                log::error!("Timed out waiting for project hierarchy memory read response: {}", error);
                None
            }
        }
    }

    fn read_project_item_runtime_value_from_memory(
        engine_execution_context: &Arc<dyn EngineExecutionContext>,
        opened_project_info: Option<&ProjectInfo>,
        project_item: &ProjectItem,
        symbolic_struct_namespace: &str,
    ) -> Option<AnonymousValueString> {
        let (address, module_name) = Self::resolve_project_item_runtime_value_target(engine_execution_context, opened_project_info, project_item)?;
        let symbolic_struct_definition = engine_execution_context.resolve_struct_layout_definition(symbolic_struct_namespace)?;
        let memory_read_response = Self::dispatch_memory_read_request(engine_execution_context, address, &module_name, &symbolic_struct_definition)?;

        if !memory_read_response.success {
            return None;
        }

        let read_data_value = memory_read_response
            .valued_struct
            .get_fields()
            .first()
            .and_then(|valued_struct_field| valued_struct_field.get_data_value())?;
        let default_format = engine_execution_context.get_default_anonymous_value_string_format(read_data_value.get_data_type_ref());

        engine_execution_context
            .anonymize_value(read_data_value, default_format)
            .ok()
    }

    fn resolve_project_item_address_target(
        engine_execution_context: &Arc<dyn EngineExecutionContext>,
        opened_project_info: Option<&ProjectInfo>,
        address_target: &ProjectItemAddressTarget,
    ) -> Option<(u64, String)> {
        let runtime_pointer = Self::resolve_address_target_runtime_pointer(opened_project_info, address_target)?;

        if runtime_pointer.get_offset_segments().is_empty() {
            Some((runtime_pointer.get_address(), runtime_pointer.get_module_name().to_string()))
        } else {
            Self::resolve_pointer_write_target(engine_execution_context, &runtime_pointer)
        }
    }

    fn build_project_item_preview_symbolic_struct_definition(
        engine_unprivileged_state: &Arc<EngineUnprivilegedState>,
        symbolic_struct_namespace: &str,
    ) -> Option<SymbolicStructDefinition> {
        let symbolic_struct_definition = engine_unprivileged_state.resolve_struct_layout_definition(symbolic_struct_namespace)?;
        let preview_field_definition = SymbolicFieldDefinition::from_str(symbolic_struct_namespace).ok();

        let Some(preview_field_definition) = preview_field_definition else {
            return Some(symbolic_struct_definition);
        };

        let preview_container_type = DataValuePreviewFormatter::limit_array_container_type(preview_field_definition.get_container_type());

        if preview_container_type == preview_field_definition.get_container_type() {
            Some(symbolic_struct_definition)
        } else {
            Some(SymbolicStructDefinition::new_anonymous(vec![SymbolicFieldDefinition::new(
                preview_field_definition.get_data_type_ref().clone(),
                preview_container_type,
            )]))
        }
    }

    fn resolve_project_item_symbolic_struct_namespace(project_item: &ProjectItem) -> Option<String> {
        let project_item_type_id = project_item.get_item_type().get_project_item_type_id();

        if project_item_type_id == ProjectItemTypeAddress::PROJECT_ITEM_TYPE_ID {
            let mut project_item = project_item.clone();

            return ProjectItemTypeAddress::get_field_symbolic_struct_definition_reference(&mut project_item).map(|symbolic_struct_reference| {
                symbolic_struct_reference
                    .get_symbolic_struct_namespace()
                    .to_string()
            });
        }

        if project_item_type_id == ProjectItemTypePointer::PROJECT_ITEM_TYPE_ID {
            return ProjectItemTypePointer::get_field_symbolic_struct_definition_reference(project_item).map(|symbolic_struct_reference| {
                symbolic_struct_reference
                    .get_symbolic_struct_namespace()
                    .to_string()
            });
        }

        None
    }

    fn resolve_project_item_symbolic_container_type(project_item: &ProjectItem) -> ContainerType {
        Self::resolve_project_item_symbolic_struct_namespace(project_item)
            .and_then(|symbolic_struct_namespace| SymbolicFieldDefinition::from_str(&symbolic_struct_namespace).ok())
            .map(|symbolic_field_definition| symbolic_field_definition.get_container_type())
            .unwrap_or(ContainerType::None)
    }

    fn project_item_preview_was_truncated(project_item: &ProjectItem) -> bool {
        let Some(symbolic_struct_namespace) = Self::resolve_project_item_symbolic_struct_namespace(project_item) else {
            return false;
        };
        let Some(symbolic_field_definition) = SymbolicFieldDefinition::from_str(&symbolic_struct_namespace).ok() else {
            return false;
        };

        DataValuePreviewFormatter::array_preview_was_truncated(symbolic_field_definition.get_container_type())
    }

    fn extract_string_value_from_edited_field(edited_field: &ValuedStructField) -> Option<String> {
        let edited_text = Self::extract_string_value_from_edited_field_allow_empty(edited_field)?;
        let edited_text = edited_text.trim();

        if edited_text.is_empty() { None } else { Some(edited_text.to_string()) }
    }

    fn extract_string_value_from_edited_field_allow_empty(edited_field: &ValuedStructField) -> Option<String> {
        let data_value = edited_field.get_data_value()?;

        String::from_utf8(data_value.get_value_bytes().clone()).ok()
    }
}
