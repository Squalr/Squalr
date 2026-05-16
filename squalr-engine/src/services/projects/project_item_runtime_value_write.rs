use crate::services::projects::project_item_symbol_resolution::{resolve_project_item_locator, resolve_project_item_struct_layout_id};
use crate::services::projects::project_symbol_runtime_value_write::{
    ProjectSymbolRuntimeValueWritePlanRequest, build_project_symbol_runtime_value_write_request,
};
use squalr_engine_api::commands::memory::write::memory_write_request::MemoryWriteRequest;
use squalr_engine_api::engine::engine_execution_context::EngineExecutionContext;
use squalr_engine_api::structures::data_values::{anonymous_value_string::AnonymousValueString, container_type::ContainerType};
use squalr_engine_api::structures::projects::project_items::project_item::ProjectItem;
use squalr_engine_api::structures::projects::project_symbol_catalog::ProjectSymbolCatalog;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct ProjectItemRuntimeValueWritePlanRequest {
    pub field_name: String,
    pub anonymous_value_string: AnonymousValueString,
}

pub fn build_project_item_runtime_value_write_request(
    engine_execution_context: &Arc<dyn EngineExecutionContext>,
    project_symbol_catalog: &ProjectSymbolCatalog,
    project_item: &ProjectItem,
    write_plan_request: &ProjectItemRuntimeValueWritePlanRequest,
) -> Result<MemoryWriteRequest, String> {
    let project_symbol_locator = resolve_project_item_locator(engine_execution_context, project_symbol_catalog, project_item)
        .ok_or_else(|| String::from("Unable to resolve project item runtime target."))?;
    let symbol_type_id = resolve_project_item_struct_layout_id(project_symbol_catalog, project_item)
        .ok_or_else(|| String::from("Unable to resolve project item value type."))?;
    let project_symbol_write_plan_request = ProjectSymbolRuntimeValueWritePlanRequest {
        address: project_symbol_locator.get_focus_address(),
        module_name: project_symbol_locator.get_focus_module_name().to_string(),
        symbol_type_id,
        container_type: ContainerType::None,
        field_name: write_plan_request.field_name.clone(),
        anonymous_value_string: write_plan_request.anonymous_value_string.clone(),
    };

    build_project_symbol_runtime_value_write_request(engine_execution_context, project_symbol_catalog, &project_symbol_write_plan_request)
}

#[cfg(test)]
mod tests {
    use super::{ProjectItemRuntimeValueWritePlanRequest, build_project_item_runtime_value_write_request};
    use crate::command_executors::project_symbols::test_support::{MockProjectSymbolsBindings, create_engine_unprivileged_state};
    use squalr_engine_api::engine::engine_execution_context::EngineExecutionContext;
    use squalr_engine_api::structures::data_types::built_in_types::u32::data_type_u32::DataTypeU32;
    use squalr_engine_api::structures::data_values::{anonymous_value_string::AnonymousValueString, anonymous_value_string_format::AnonymousValueStringFormat};
    use squalr_engine_api::structures::projects::project_items::built_in_types::project_item_type_address::ProjectItemTypeAddress;
    use squalr_engine_api::structures::projects::project_symbol_catalog::ProjectSymbolCatalog;
    use std::sync::Arc;

    #[test]
    fn build_project_item_runtime_value_write_request_resolves_address_item() {
        let project_item = ProjectItemTypeAddress::new_project_item("Health", 0x1234, "", "", DataTypeU32::get_value_from_primitive(0));
        let project_symbol_catalog = ProjectSymbolCatalog::default();
        let engine_unprivileged_state = create_engine_unprivileged_state(MockProjectSymbolsBindings::new());
        let engine_execution_context: Arc<dyn EngineExecutionContext> = engine_unprivileged_state.clone();

        let memory_write_request = build_project_item_runtime_value_write_request(
            &engine_execution_context,
            &project_symbol_catalog,
            &project_item,
            &ProjectItemRuntimeValueWritePlanRequest {
                field_name: String::from("value"),
                anonymous_value_string: AnonymousValueString::new(String::from("255"), AnonymousValueStringFormat::Decimal, Default::default()),
            },
        )
        .expect("Expected project item runtime write to resolve.");

        assert_eq!(memory_write_request.address, 0x1234);
        assert_eq!(memory_write_request.value, 255_u32.to_ne_bytes());
    }
}
