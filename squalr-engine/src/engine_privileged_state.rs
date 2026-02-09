use crate::engine_bindings::interprocess::interprocess_engine_api_privileged_bindings::InterprocessEngineApiPrivilegedBindings;
use crate::engine_bindings::standalone::standalone_engine_api_privileged_bindings::StandalonePrivilegedEngine;
use crate::engine_initialization_error::EngineInitializationError;
use crate::engine_mode::EngineMode;
use squalr_engine_api::engine::engine_api_priviliged_bindings::EngineApiPrivilegedBindings;
use squalr_engine_session::os::engine_os_provider::EngineOsProviders;
use std::sync::{Arc, RwLock};

pub use squalr_engine_session::engine_privileged_state::EnginePrivilegedState;

pub fn create_engine_privileged_state(engine_mode: EngineMode) -> Result<Arc<EnginePrivilegedState>, EngineInitializationError> {
    create_engine_privileged_state_with_os_providers(engine_mode, EngineOsProviders::default())
}

pub fn create_engine_privileged_state_with_os_providers(
    engine_mode: EngineMode,
    os_providers: EngineOsProviders,
) -> Result<Arc<EnginePrivilegedState>, EngineInitializationError> {
    let standalone_bindings = match engine_mode {
        EngineMode::Standalone => Some(Arc::new(RwLock::new(StandalonePrivilegedEngine::new()))),
        _ => None,
    };
    let interprocess_bindings = match engine_mode {
        EngineMode::PrivilegedShell => Some(Arc::new(RwLock::new(InterprocessEngineApiPrivilegedBindings::new()))),
        _ => None,
    };

    let engine_bindings: Arc<RwLock<dyn EngineApiPrivilegedBindings>> = match engine_mode {
        EngineMode::Standalone => standalone_bindings
            .clone()
            .expect("Standalone engine mode must always provide standalone privileged bindings."),
        EngineMode::PrivilegedShell => interprocess_bindings
            .clone()
            .expect("Privileged shell mode must always provide interprocess privileged bindings."),
        EngineMode::UnprivilegedHost => unreachable!("Privileged state should never be created on an unprivileged host."),
    };

    let engine_privileged_state =
        EnginePrivilegedState::new(engine_bindings, os_providers).map_err(EngineInitializationError::process_monitoring_start_failed)?;

    if let Some(standalone_bindings) = standalone_bindings.as_ref() {
        match standalone_bindings.write() {
            Ok(mut standalone_bindings_guard) => {
                if let Err(error) = standalone_bindings_guard.initialize(&engine_privileged_state) {
                    return Err(EngineInitializationError::privileged_bindings_initialize_failed(
                        "initializing standalone privileged bindings",
                        error,
                    ));
                }
            }
            Err(error) => {
                return Err(EngineInitializationError::privileged_bindings_lock_failed(
                    "initializing standalone privileged bindings",
                    error.to_string(),
                ));
            }
        }
    }

    if let Some(interprocess_bindings) = interprocess_bindings.as_ref() {
        match interprocess_bindings.write() {
            Ok(mut interprocess_bindings_guard) => {
                if let Err(error) = interprocess_bindings_guard.initialize(&engine_privileged_state) {
                    return Err(EngineInitializationError::privileged_bindings_initialize_failed(
                        "initializing interprocess privileged bindings",
                        error,
                    ));
                }
            }
            Err(error) => {
                return Err(EngineInitializationError::privileged_bindings_lock_failed(
                    "initializing interprocess privileged bindings",
                    error.to_string(),
                ));
            }
        }
    }

    Ok(engine_privileged_state)
}

#[cfg(test)]
mod tests {
    use crate::engine_mode::EngineMode;
    use squalr_engine_api::structures::processes::opened_process_info::OpenedProcessInfo;
    use squalr_engine_api::structures::processes::process_info::ProcessInfo;
    use squalr_engine_session::os::ProcessQueryError;
    use squalr_engine_session::os::ProcessQueryOptions;
    use squalr_engine_session::os::engine_os_provider::{EngineOsProviders, ProcessQueryProvider};
    use std::sync::Arc;

    struct FailingProcessQueryProvider;

    impl ProcessQueryProvider for FailingProcessQueryProvider {
        fn start_monitoring(&self) -> Result<(), ProcessQueryError> {
            Err(ProcessQueryError::internal("start_monitoring", "simulated startup failure"))
        }

        fn get_processes(
            &self,
            _process_query_options: ProcessQueryOptions,
        ) -> Vec<ProcessInfo> {
            vec![]
        }

        fn open_process(
            &self,
            _process_info: &ProcessInfo,
        ) -> Result<OpenedProcessInfo, ProcessQueryError> {
            Err(ProcessQueryError::internal("open_process", "not implemented in test provider"))
        }

        fn close_process(
            &self,
            _handle: u64,
        ) -> Result<(), ProcessQueryError> {
            Err(ProcessQueryError::internal("close_process", "not implemented in test provider"))
        }
    }

    #[test]
    fn create_engine_privileged_state_with_os_providers_fails_fast_when_process_monitoring_fails() {
        let mut engine_os_providers = EngineOsProviders::default();
        engine_os_providers.process_query = Arc::new(FailingProcessQueryProvider);

        let initialization_result = super::create_engine_privileged_state_with_os_providers(EngineMode::Standalone, engine_os_providers);

        assert!(initialization_result.is_err());

        if let Err(error) = initialization_result {
            assert!(
                error
                    .to_string()
                    .contains("Failed to start process monitoring during privileged engine bootstrap")
            );
        }
    }
}
