use crate::command_executors::privileged_request_executor::PrivilegedCommandRequestExecutor;
use crate::engine_privileged_state::EnginePrivilegedState;
use squalr_engine_api::commands::settings::memory::set::memory_settings_set_request::MemorySettingsSetRequest;
use squalr_engine_api::commands::settings::memory::set::memory_settings_set_response::MemorySettingsSetResponse;
use squalr_engine_session::os::MemorySettingsConfig;
use std::sync::Arc;

impl PrivilegedCommandRequestExecutor for MemorySettingsSetRequest {
    type ResponseType = MemorySettingsSetResponse;

    fn execute(
        &self,
        _engine_privileged_state: &Arc<EnginePrivilegedState>,
    ) -> <Self as PrivilegedCommandRequestExecutor>::ResponseType {
        if let Some(memory_type_none) = self.memory_type_none {
            MemorySettingsConfig::set_memory_type_none(memory_type_none);
        }

        if let Some(memory_type_private) = self.memory_type_private {
            MemorySettingsConfig::set_memory_type_private(memory_type_private);
        }

        if let Some(memory_type_image) = self.memory_type_image {
            MemorySettingsConfig::set_memory_type_image(memory_type_image);
        }

        if let Some(memory_type_mapped) = self.memory_type_mapped {
            MemorySettingsConfig::set_memory_type_mapped(memory_type_mapped);
        }

        if let Some(required_write) = self.required_write {
            MemorySettingsConfig::set_required_write(required_write);
        }

        if let Some(required_execute) = self.required_execute {
            MemorySettingsConfig::set_required_execute(required_execute);
        }

        if let Some(required_copy_on_write) = self.required_copy_on_write {
            MemorySettingsConfig::set_required_copy_on_write(required_copy_on_write);
        }

        if let Some(excluded_write) = self.excluded_write {
            MemorySettingsConfig::set_excluded_write(excluded_write);
        }

        if let Some(excluded_execute) = self.excluded_execute {
            MemorySettingsConfig::set_excluded_execute(excluded_execute);
        }

        if let Some(excluded_copy_on_write) = self.excluded_copy_on_write {
            MemorySettingsConfig::set_excluded_copy_on_write(excluded_copy_on_write);
        }

        if let Some(start_address) = self.start_address {
            MemorySettingsConfig::set_start_address(start_address);
        }

        if let Some(end_address) = self.end_address {
            MemorySettingsConfig::set_end_address(end_address);
        }

        if let Some(only_query_usermode) = self.only_query_usermode {
            MemorySettingsConfig::set_only_query_usermode(only_query_usermode);
        }

        MemorySettingsSetResponse {}
    }
}
