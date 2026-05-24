use crate::app_context::AppContext;
use squalr_engine_api::commands::{
    project_symbols::{
        delete_resolver::project_symbols_delete_resolver_request::ProjectSymbolsDeleteResolverRequest,
        upsert_resolver::project_symbols_upsert_resolver_request::ProjectSymbolsUpsertResolverRequest,
    },
    unprivileged_command_request::UnprivilegedCommandRequest,
};
use squalr_engine_api::registries::symbols::symbolic_resolver_descriptor::SymbolicResolverDescriptor;
use std::sync::Arc;

#[derive(Clone)]
pub struct SymbolResolverCommandDispatcher {
    app_context: Arc<AppContext>,
}

impl SymbolResolverCommandDispatcher {
    pub fn new(app_context: Arc<AppContext>) -> Self {
        Self { app_context }
    }

    pub fn persist_resolver_descriptor(
        &self,
        original_resolver_id: Option<String>,
        resolver_descriptor: &SymbolicResolverDescriptor,
    ) {
        let Ok(request) = ProjectSymbolsUpsertResolverRequest::from_resolver_descriptor(original_resolver_id, resolver_descriptor) else {
            log::error!(
                "Failed to serialize symbol resolver `{}` for persistence.",
                resolver_descriptor.get_resolver_id()
            );
            return;
        };

        request.send(&self.app_context.engine_unprivileged_state, |response| {
            if !response.success {
                log::error!(
                    "Failed to persist symbol resolver `{}` through project-symbols upsert-resolver command: {}.",
                    response.resolver_id,
                    response.error.as_deref().unwrap_or("unknown error")
                );
            }
        });
    }

    pub fn delete_resolver(
        &self,
        resolver_id: &str,
    ) {
        ProjectSymbolsDeleteResolverRequest::new(resolver_id).send(&self.app_context.engine_unprivileged_state, |response| {
            if !response.success {
                log::error!(
                    "Failed to delete symbol resolver `{}` through project-symbols delete-resolver command: {}.",
                    response.resolver_id,
                    response.error.as_deref().unwrap_or("unknown error")
                );
            }
        });
    }
}
