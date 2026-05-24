use squalr_engine::services::projects::project_item_symbol_resolution::{
    resolve_address_target_runtime_pointer_with_optional_catalog, resolve_project_item_struct_layout_id,
};
use squalr_engine_api::engine::engine_execution_context::EngineExecutionContext;
use squalr_engine_api::structures::data_values::{
    anonymous_value_string_format::AnonymousValueStringFormat,
    container_type::ContainerType,
    data_value_preview_formatter::{DataValuePreviewFormatOptions, DataValuePreviewFormatter},
};
use squalr_engine_api::structures::projects::project_info::ProjectInfo;
use squalr_engine_api::structures::projects::project_items::built_in_types::{
    project_item_type_address::ProjectItemTypeAddress, project_item_type_pointer::ProjectItemTypePointer,
};
use squalr_engine_api::structures::projects::project_items::project_item::ProjectItem;
use squalr_engine_api::structures::projects::project_symbol_catalog::ProjectSymbolCatalog;
use squalr_engine_api::structures::structs::{symbolic_field_definition::SymbolicFieldDefinition, symbolic_struct_definition::SymbolicStructDefinition};
use squalr_engine_session::{
    engine_unprivileged_state::EngineUnprivilegedState,
    virtual_snapshots::{virtual_snapshot_query::VirtualSnapshotQuery, virtual_snapshot_query_result::VirtualSnapshotQueryResult},
};
use std::path::Path;
use std::str::FromStr;
use std::sync::Arc;

pub struct ProjectItemPreviewDetails;

impl ProjectItemPreviewDetails {
    const PROJECT_ITEM_PREVIEW_FORMAT_OPTIONS: DataValuePreviewFormatOptions = DataValuePreviewFormatOptions::new(4, 96, 96);

    pub fn copy_project_item_preview_fields(
        source_project_item: &ProjectItem,
        target_project_item: &mut ProjectItem,
    ) {
        let preview_value = Self::read_project_item_preview_value(source_project_item);
        let preview_display_format = Self::read_project_item_preview_display_format(source_project_item);
        let project_item_type_id = target_project_item.get_item_type().get_project_item_type_id();

        if project_item_type_id == ProjectItemTypeAddress::PROJECT_ITEM_TYPE_ID {
            ProjectItemTypeAddress::set_field_freeze_data_value_interpreter(target_project_item, &preview_value);
            if let Some(preview_display_format) = preview_display_format {
                ProjectItemTypeAddress::set_field_freeze_display_format(target_project_item, preview_display_format);
            }
        } else if project_item_type_id == ProjectItemTypePointer::PROJECT_ITEM_TYPE_ID {
            ProjectItemTypePointer::set_field_freeze_data_value_interpreter(target_project_item, &preview_value);
            if let Some(preview_display_format) = preview_display_format {
                ProjectItemTypePointer::set_field_freeze_display_format(target_project_item, preview_display_format);
            }
        }
    }

    pub fn build_project_item_virtual_snapshot_query(
        opened_project_info: Option<&ProjectInfo>,
        project_item_path: &Path,
        project_item: &ProjectItem,
        engine_unprivileged_state: &Arc<EngineUnprivilegedState>,
    ) -> Option<VirtualSnapshotQuery> {
        let query_id = project_item_path.to_string_lossy().to_string();
        let symbolic_struct_namespace = resolve_project_item_struct_layout_id(&ProjectSymbolCatalog::default(), project_item)?;
        let symbolic_struct_definition = Self::build_project_item_preview_symbolic_struct_definition(engine_unprivileged_state, &symbolic_struct_namespace)?;
        let project_item_type_id = project_item.get_item_type().get_project_item_type_id();

        if project_item_type_id == ProjectItemTypeAddress::PROJECT_ITEM_TYPE_ID {
            let mut project_item = project_item.clone();
            let address_target = ProjectItemTypeAddress::get_address_target(&mut project_item);

            let project_symbol_catalog = opened_project_info.map(|opened_project_info| opened_project_info.get_project_symbol_catalog());
            let runtime_pointer = resolve_address_target_runtime_pointer_with_optional_catalog(project_symbol_catalog, &address_target)?;

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

        let default_anonymous_value_string_format = Self::read_project_item_preview_display_format(project_item)
            .unwrap_or_else(|| engine_unprivileged_state.get_default_anonymous_value_string_format(first_read_field_data_value.get_data_type_ref()));
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

    fn read_project_item_preview_display_format(project_item: &ProjectItem) -> Option<AnonymousValueStringFormat> {
        let project_item_type_id = project_item.get_item_type().get_project_item_type_id();

        if project_item_type_id == ProjectItemTypeAddress::PROJECT_ITEM_TYPE_ID {
            ProjectItemTypeAddress::get_field_freeze_display_format(project_item)
        } else if project_item_type_id == ProjectItemTypePointer::PROJECT_ITEM_TYPE_ID {
            ProjectItemTypePointer::get_field_freeze_display_format(project_item)
        } else {
            None
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

    fn resolve_project_item_symbolic_container_type(project_item: &ProjectItem) -> ContainerType {
        resolve_project_item_struct_layout_id(&ProjectSymbolCatalog::default(), project_item)
            .and_then(|symbolic_struct_namespace| SymbolicFieldDefinition::from_str(&symbolic_struct_namespace).ok())
            .map(|symbolic_field_definition| symbolic_field_definition.get_container_type())
            .unwrap_or(ContainerType::None)
    }

    fn project_item_preview_was_truncated(project_item: &ProjectItem) -> bool {
        let Some(symbolic_struct_namespace) = resolve_project_item_struct_layout_id(&ProjectSymbolCatalog::default(), project_item) else {
            return false;
        };
        let Some(symbolic_field_definition) = SymbolicFieldDefinition::from_str(&symbolic_struct_namespace).ok() else {
            return false;
        };

        DataValuePreviewFormatter::array_preview_was_truncated(symbolic_field_definition.get_container_type())
    }
}
