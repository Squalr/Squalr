use crate::command_executors::privileged_request_executor::PrivilegedCommandRequestExecutor;
use crate::engine_privileged_state::EnginePrivilegedState;
use squalr_engine_api::commands::memory::query::memory_query_request::MemoryQueryRequest;
use squalr_engine_api::commands::memory::query::memory_query_response::MemoryQueryResponse;
use squalr_engine_api::plugins::memory_view::PageRetrievalMode as ApiPageRetrievalMode;
use squalr_engine_session::os::PageRetrievalMode as SessionPageRetrievalMode;
use std::sync::Arc;

impl PrivilegedCommandRequestExecutor for MemoryQueryRequest {
    type ResponseType = MemoryQueryResponse;

    fn execute(
        &self,
        engine_privileged_state: &Arc<EnginePrivilegedState>,
    ) -> <Self as PrivilegedCommandRequestExecutor>::ResponseType {
        let Some(opened_process_info) = engine_privileged_state
            .get_process_manager()
            .get_opened_process()
        else {
            return MemoryQueryResponse::default();
        };
        let os_providers = engine_privileged_state.get_os_providers();
        let page_retrieval_mode = match self.page_retrieval_mode {
            ApiPageRetrievalMode::FromSettings => SessionPageRetrievalMode::FromSettings,
            ApiPageRetrievalMode::FromUserMode => SessionPageRetrievalMode::FromUserMode,
            ApiPageRetrievalMode::FromNonModules => SessionPageRetrievalMode::FromNonModules,
            ApiPageRetrievalMode::FromModules => SessionPageRetrievalMode::FromModules,
            ApiPageRetrievalMode::FromVirtualModules => SessionPageRetrievalMode::FromVirtualModules,
        };
        let mut virtual_pages = os_providers
            .memory_query
            .get_memory_page_bounds(&opened_process_info, page_retrieval_mode);
        let mut modules = os_providers.memory_query.get_modules(&opened_process_info);

        virtual_pages.sort();
        modules.sort_by_key(|normalized_module| normalized_module.get_base_address());

        MemoryQueryResponse {
            virtual_pages,
            modules,
            success: true,
        }
    }
}
