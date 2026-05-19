use crate::app_context::AppContext;
use squalr_engine_api::commands::{
    project_symbols::{
        delete_layout::project_symbols_delete_layout_request::ProjectSymbolsDeleteLayoutRequest,
        upsert_layout::project_symbols_upsert_layout_request::ProjectSymbolsUpsertLayoutRequest,
    },
    unprivileged_command_request::UnprivilegedCommandRequest,
};
use squalr_engine_api::registries::symbols::struct_layout_descriptor::StructLayoutDescriptor;
use std::sync::Arc;

#[derive(Clone)]
pub struct SymbolLayoutCommandDispatcher {
    app_context: Arc<AppContext>,
}

impl SymbolLayoutCommandDispatcher {
    pub fn new(app_context: Arc<AppContext>) -> Self {
        Self { app_context }
    }

    pub fn persist_symbol_layout_descriptor(
        &self,
        original_struct_layout_id: Option<String>,
        struct_layout_descriptor: &StructLayoutDescriptor,
    ) {
        ProjectSymbolsUpsertLayoutRequest::from_struct_layout_descriptor(original_struct_layout_id, struct_layout_descriptor).send(
            &self.app_context.engine_unprivileged_state,
            |response| {
                if !response.success {
                    log::error!(
                        "Failed to persist symbol layout `{}` through project-symbols upsert-layout command: {}.",
                        response.struct_layout_id,
                        response.error.as_deref().unwrap_or("unknown error")
                    );
                }
            },
        );
    }

    pub fn delete_symbol_layout(
        &self,
        layout_id: &str,
    ) {
        ProjectSymbolsDeleteLayoutRequest::new(layout_id).send(&self.app_context.engine_unprivileged_state, |response| {
            if !response.success {
                log::error!(
                    "Failed to delete symbol layout `{}` through project-symbols delete-layout command: {}.",
                    response.struct_layout_id,
                    response.error.as_deref().unwrap_or("unknown error")
                );
            }
        });
    }
}
