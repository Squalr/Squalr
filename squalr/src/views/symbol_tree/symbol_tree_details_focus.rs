use crate::app_context::AppContext;
use crate::views::{
    struct_viewer::view_data::struct_viewer_focus_target::StructViewerFocusTarget, struct_viewer::view_data::struct_viewer_view_data::StructViewerViewData,
    symbol_tree::symbol_tree_runtime_data_controller::SymbolTreeRuntimeDataController,
};
use squalr_engine_api::commands::{
    project::save::project_save_request::ProjectSaveRequest,
    project_symbols::write_value::project_symbols_write_value_request::ProjectSymbolsWriteValueRequest,
    unprivileged_command_request::UnprivilegedCommandRequest,
};
use squalr_engine_api::dependency_injection::dependency::Dependency;
use squalr_engine_api::engine::engine_execution_context::EngineExecutionContext;
use squalr_engine_api::structures::data_values::{
    anonymous_value_string::AnonymousValueString, anonymous_value_string_format::AnonymousValueStringFormat, container_type::ContainerType,
    data_value::DataValue,
};
use squalr_engine_api::structures::details::{DetailsEdit, DetailsFieldSource, DetailsValue};
use squalr_engine_api::structures::projects::{project_symbol_catalog::ProjectSymbolCatalog, symbol_tree::symbol_tree_node::SymbolTreeNode};
use std::sync::Arc;

pub struct SymbolTreeDetailsFocus {
    app_context: Arc<AppContext>,
    struct_viewer_view_data: Dependency<StructViewerViewData>,
}

impl SymbolTreeDetailsFocus {
    pub fn new(
        app_context: Arc<AppContext>,
        struct_viewer_view_data: Dependency<StructViewerViewData>,
    ) -> Self {
        Self {
            app_context,
            struct_viewer_view_data,
        }
    }

    pub fn build_struct_viewer_focus_target_key(selected_symbol_tree_entry: Option<&SymbolTreeNode>) -> Option<String> {
        selected_symbol_tree_entry.map(|symbol_tree_entry| {
            format!(
                "{}|{}|{}",
                symbol_tree_entry.get_node_key(),
                symbol_tree_entry.get_display_name(),
                symbol_tree_entry.get_display_type_id()
            )
        })
    }

    pub fn build_struct_viewer_focus_target(selected_symbol_tree_entry: Option<&SymbolTreeNode>) -> Option<StructViewerFocusTarget> {
        Self::build_struct_viewer_focus_target_key(selected_symbol_tree_entry).map(|selection_key| StructViewerFocusTarget::SymbolTree { selection_key })
    }

    pub fn focus_symbol_tree_entry_in_struct_viewer(
        &self,
        project_symbol_catalog: &ProjectSymbolCatalog,
        selected_symbol_tree_entry: &SymbolTreeNode,
    ) {
        let focus_target = Self::build_struct_viewer_focus_target(Some(selected_symbol_tree_entry));
        let details_projection = SymbolTreeRuntimeDataController::new(self.app_context.clone())
            .build_symbol_details_projection_for_tree_entry(project_symbol_catalog, selected_symbol_tree_entry);
        let details_edit_callback = self.build_symbol_details_edit_callback(selected_symbol_tree_entry);

        StructViewerViewData::focus_details_projection_with_focus_target(
            self.struct_viewer_view_data.clone(),
            self.app_context.engine_unprivileged_state.clone(),
            details_projection,
            details_edit_callback,
            focus_target,
        );
    }

    pub fn sync_selected_symbol_into_struct_viewer(
        &self,
        project_symbol_catalog: &ProjectSymbolCatalog,
        selected_symbol_tree_entry: Option<&SymbolTreeNode>,
    ) {
        let current_focus_target = self
            .struct_viewer_view_data
            .read("Symbol tree current struct viewer focus target")
            .and_then(|struct_viewer_view_data| struct_viewer_view_data.get_focus_target().cloned());
        let desired_focus_target = Self::build_struct_viewer_focus_target(selected_symbol_tree_entry);

        if current_focus_target == desired_focus_target {
            return;
        }

        let Some(selected_symbol_tree_entry) = selected_symbol_tree_entry else {
            if matches!(current_focus_target, Some(StructViewerFocusTarget::SymbolTree { .. })) {
                StructViewerViewData::clear_focus(self.struct_viewer_view_data.clone());
            }
            return;
        };

        if matches!(current_focus_target, Some(StructViewerFocusTarget::ProjectHierarchy { .. })) {
            return;
        }

        self.focus_symbol_tree_entry_in_struct_viewer(project_symbol_catalog, selected_symbol_tree_entry);
    }

    fn build_symbol_details_edit_callback(
        &self,
        selected_symbol_tree_entry: &SymbolTreeNode,
    ) -> Arc<dyn Fn(DetailsEdit) + Send + Sync> {
        let selected_symbol_tree_entry = selected_symbol_tree_entry.clone();
        let engine_unprivileged_state = self.app_context.engine_unprivileged_state.clone();

        Arc::new(move |details_edit: DetailsEdit| match details_edit.get_value() {
            DetailsValue::Empty => {}
            DetailsValue::DisplayFormat(display_format) => {
                let project_manager = engine_unprivileged_state.get_project_manager();
                let opened_project_lock = project_manager.get_opened_project();
                let mut opened_project_guard = match opened_project_lock.write() {
                    Ok(opened_project_guard) => opened_project_guard,
                    Err(error) => {
                        log::error!("Failed to acquire opened project lock while saving symbol display format: {}", error);
                        return;
                    }
                };
                let Some(opened_project) = opened_project_guard.as_mut() else {
                    return;
                };

                opened_project
                    .get_project_manifest_mut()
                    .set_symbol_display_format(selected_symbol_tree_entry.get_node_key().to_string(), *display_format);
                opened_project
                    .get_project_info_mut()
                    .set_has_unsaved_changes(true);
                drop(opened_project_guard);

                ProjectSaveRequest {}.send(&engine_unprivileged_state, |project_save_response| {
                    if !project_save_response.success {
                        log::warn!("Failed to save project after Symbol Tree display format update.");
                    }
                });
            }
            details_value => {
                let DetailsFieldSource::ProjectSymbolRuntimeValue { field_path } = details_edit.get_source() else {
                    return;
                };
                let field_name = field_path
                    .last()
                    .cloned()
                    .unwrap_or_else(|| String::from("value"));
                let Some(anonymous_value_string) = Self::details_value_to_anonymous_value_string(engine_unprivileged_state.as_ref(), details_value) else {
                    return;
                };

                ProjectSymbolsWriteValueRequest {
                    address: selected_symbol_tree_entry.get_locator().get_focus_address(),
                    module_name: selected_symbol_tree_entry
                        .get_locator()
                        .get_focus_module_name()
                        .to_string(),
                    symbol_type_id: selected_symbol_tree_entry.get_symbol_type_id().to_string(),
                    container_type: selected_symbol_tree_entry.get_container_type(),
                    field_name,
                    anonymous_value_string,
                }
                .send(&engine_unprivileged_state, |project_symbols_write_value_response| {
                    if !project_symbols_write_value_response.success {
                        log::warn!(
                            "Symbol Tree details value write command failed: {}",
                            project_symbols_write_value_response
                                .error
                                .as_deref()
                                .unwrap_or("unknown error")
                        );
                    }
                });
            }
        })
    }

    fn details_value_to_anonymous_value_string(
        engine_execution_context: &dyn EngineExecutionContext,
        details_value: &DetailsValue,
    ) -> Option<AnonymousValueString> {
        match details_value {
            DetailsValue::AnonymousValue(anonymous_value_string) => Some(anonymous_value_string.clone()),
            DetailsValue::DataValue(data_value) => Self::data_value_to_anonymous_value_string(engine_execution_context, data_value),
            DetailsValue::Text(text) => Some(AnonymousValueString::new(text.clone(), AnonymousValueStringFormat::String, ContainerType::None)),
            DetailsValue::Boolean(value) => Some(AnonymousValueString::new(
                value.to_string(),
                AnonymousValueStringFormat::Bool,
                ContainerType::None,
            )),
            DetailsValue::UnsignedInteger(value) => Some(AnonymousValueString::new(
                value.to_string(),
                AnonymousValueStringFormat::Decimal,
                ContainerType::None,
            )),
            DetailsValue::SignedInteger(value) => Some(AnonymousValueString::new(
                value.to_string(),
                AnonymousValueStringFormat::Decimal,
                ContainerType::None,
            )),
            DetailsValue::DisplayFormat(display_format) => Some(AnonymousValueString::new(
                display_format.to_string(),
                AnonymousValueStringFormat::String,
                ContainerType::None,
            )),
            DetailsValue::Empty => Some(AnonymousValueString::new(
                String::new(),
                AnonymousValueStringFormat::String,
                ContainerType::None,
            )),
        }
    }

    fn data_value_to_anonymous_value_string(
        engine_execution_context: &dyn EngineExecutionContext,
        data_value: &DataValue,
    ) -> Option<AnonymousValueString> {
        let anonymous_value_string_format = engine_execution_context.get_default_anonymous_value_string_format(data_value.get_data_type_ref());

        engine_execution_context
            .anonymize_value(data_value, anonymous_value_string_format)
            .map_err(|error| {
                log::warn!("Failed to format Symbol Tree details edit: {}", error);
                error
            })
            .ok()
    }
}
