use squalr_engine_api::commands::scan_results::add_to_project::scan_results_add_to_project_request::ScanResultsAddToProjectRequest;
use squalr_engine_api::commands::scan_results::delete::scan_results_delete_request::ScanResultsDeleteRequest;
use squalr_engine_api::commands::scan_results::freeze::scan_results_freeze_request::ScanResultsFreezeRequest;
use squalr_engine_api::dependency_injection::dependency::Dependency;
use squalr_engine_api::engine::engine_execution_context::EngineExecutionContext;
use squalr_engine_api::structures::scan_results::scan_result_base::ScanResultBase;
use squalr_engine_api::structures::scan_results::scan_result_ref::ScanResultRef;
use squalr_engine_api::{
    commands::{
        engine_command_request::EngineCommandRequest,
        scan_results::{
            query::scan_results_query_request::ScanResultsQueryRequest, refresh::scan_results_refresh_request::ScanResultsRefreshRequest,
            set_property::scan_results_set_property_request::ScanResultsSetPropertyRequest,
        },
    },
    conversions::conversions::Conversions,
    events::scan_results::updated::scan_results_updated_event::ScanResultsUpdatedEvent,
    structures::{data_values::anonymous_value::AnonymousValue, scan_results::scan_result::ScanResult},
};
use std::cmp::{self};
use std::sync::{Arc, RwLockReadGuard};
use std::{thread, time::Duration};

pub struct ElementScannerResultsViewData {
    // audio_player: AudioPlayer,
    pub current_scan_results: Vec<ScanResult>,
    pub current_page_index: u64,
    pub cached_last_page_index: u64,
    pub selection_index_start: i32,
    pub selection_index_end: i32,
    pub result_count: u64,
    pub byte_size_in_metric: String,
}

impl ElementScannerResultsViewData {
    pub fn new() -> Self {
        Self {
            current_scan_results: Vec::new(),
            current_page_index: 0,
            cached_last_page_index: 0,
            selection_index_start: 0,
            selection_index_end: 0,
            result_count: 0,
            byte_size_in_metric: String::new(),
        }
    }

    pub fn poll_scan_results(
        element_scanner_results_view_data: Dependency<Self>,
        engine_execution_context: Arc<EngineExecutionContext>,
    ) {
        let engine_execution_context_clone = engine_execution_context.clone();
        let element_scanner_results_view_data_clone = element_scanner_results_view_data.clone();

        // Requery all scan results if they update.
        {
            engine_execution_context.listen_for_engine_event::<ScanResultsUpdatedEvent>(move |scan_results_updated_event| {
                let element_scanner_results_view_data = element_scanner_results_view_data_clone.clone();
                let engine_execution_context = engine_execution_context_clone.clone();
                let play_sound = !scan_results_updated_event.is_new_scan;

                Self::query_scan_results(element_scanner_results_view_data, engine_execution_context, play_sound);
            });
        }

        let engine_execution_context_clone = engine_execution_context.clone();
        let element_scanner_results_view_data_clone = element_scanner_results_view_data.clone();

        // Refresh scan values on a loop. JIRA: This should be coming from settings. We can probably cache, and have some mechanism for getting latest val.
        thread::spawn(move || {
            loop {
                let element_scanner_results_view_data = element_scanner_results_view_data_clone.clone();
                let engine_execution_context = engine_execution_context_clone.clone();

                Self::refresh_scan_results(element_scanner_results_view_data, engine_execution_context);

                thread::sleep(Duration::from_millis(100));
            }
        });
    }

    pub fn set_selected_scan_results_value(
        &self,
        engine_execution_context: Arc<EngineExecutionContext>,
        field_namespace: String,
        anonymous_value: AnonymousValue,
    ) {
        let scan_result_refs = self
            .current_scan_results
            .iter()
            .map(|scan_result| scan_result.get_base_result().get_scan_result_ref().clone())
            .collect();

        let scan_results_set_property_request = ScanResultsSetPropertyRequest {
            scan_result_refs,
            field_namespace,
            anonymous_value,
        };

        scan_results_set_property_request.send(&engine_execution_context, move |scan_results_set_property_response| {});
    }

    fn load_current_page_index(element_scanner_results_view_data: &RwLockReadGuard<'_, ElementScannerResultsViewData>) -> u64 {
        element_scanner_results_view_data
            .current_page_index
            .clamp(0, element_scanner_results_view_data.cached_last_page_index)
    }

    fn query_scan_results(
        element_scanner_results_view_data: Dependency<Self>,
        engine_execution_context: Arc<EngineExecutionContext>,
        play_sound: bool,
    ) {
        let element_scanner_results_view_data_clone = element_scanner_results_view_data.clone();
        let element_scanner_results_view_data = match element_scanner_results_view_data.read() {
            Ok(element_scanner_results_view_data) => element_scanner_results_view_data,
            Err(error) => {
                log::error!("Failed to acquire read lock on element scanner results view data: {}", error);

                return;
            }
        };
        let engine_execution_context_clone = engine_execution_context.clone();
        let page_index = Self::load_current_page_index(&element_scanner_results_view_data);
        let scan_results_query_request = ScanResultsQueryRequest { page_index };

        drop(element_scanner_results_view_data);

        scan_results_query_request.send(&engine_execution_context, move |scan_results_query_response| {
            let element_scanner_results_view_data_clone_clone = element_scanner_results_view_data_clone.clone();
            let mut element_scanner_results_view_data = match element_scanner_results_view_data_clone.write() {
                Ok(element_scanner_results_view_data) => element_scanner_results_view_data,
                Err(error) => {
                    log::error!("Failed to acquire read lock on element scanner results view data: {}", error);

                    return;
                }
            };

            // let audio_player = &self.audio_player;
            let byte_size_in_metric = Conversions::value_to_metric_size(scan_results_query_response.total_size_in_bytes);
            let result_count = scan_results_query_response.result_count;

            element_scanner_results_view_data.cached_last_page_index = scan_results_query_response.last_page_index;
            element_scanner_results_view_data.result_count = result_count;
            element_scanner_results_view_data.byte_size_in_metric = format!("{} (Count: {})", byte_size_in_metric, result_count);
            element_scanner_results_view_data.current_scan_results = scan_results_query_response.scan_results;

            if play_sound {
                if result_count > 0 {
                    // audio_player.play_sound(SoundType::Success);
                } else {
                    // audio_player.play_sound(SoundType::Warn);
                }
            }

            drop(element_scanner_results_view_data);

            Self::refresh_scan_results(element_scanner_results_view_data_clone_clone, engine_execution_context_clone);
        });
    }

    /// Fetches up-to-date values and module information for the current scan results, then updates the UI.
    fn refresh_scan_results(
        element_scanner_results_view_data: Dependency<Self>,
        engine_execution_context: Arc<EngineExecutionContext>,
    ) {
        let element_scanner_results_view_data_clone = element_scanner_results_view_data.clone();
        let element_scanner_results_view_data = match element_scanner_results_view_data.read() {
            Ok(element_scanner_results_view_data) => element_scanner_results_view_data,
            Err(error) => {
                log::error!("Failed to acquire read lock on element scanner results view data: {}", error);

                return;
            }
        };
        let engine_execution_context = &engine_execution_context;

        // Fire a request to get all scan result data needed for display.
        let scan_results_refresh_request = ScanResultsRefreshRequest {
            scan_result_refs: element_scanner_results_view_data
                .current_scan_results
                .iter()
                .map(|scan_result| scan_result.get_base_result().get_scan_result_ref().clone())
                .collect(),
        };

        drop(element_scanner_results_view_data);

        scan_results_refresh_request.send(engine_execution_context, move |scan_results_refresh_response| {
            let mut element_scanner_results_view_data = match element_scanner_results_view_data_clone.write() {
                Ok(element_scanner_results_view_data) => element_scanner_results_view_data,
                Err(error) => {
                    log::error!("Failed to acquire write lock on element scanner results view data: {}", error);

                    return;
                }
            };

            // Update UI with refreshed, full scan result values.
            element_scanner_results_view_data.current_scan_results = scan_results_refresh_response.scan_results;
        });
    }

    fn set_page_index(
        element_scanner_results_view_data: Dependency<Self>,
        engine_execution_context: Arc<EngineExecutionContext>,
        new_page_index: u64,
    ) {
        let element_scanner_results_view_data_clone = element_scanner_results_view_data.clone();
        let mut element_scanner_results_view_data = match element_scanner_results_view_data.write() {
            Ok(element_scanner_results_view_data) => element_scanner_results_view_data,
            Err(error) => {
                log::error!("Failed to acquire write lock on element scanner results view data: {}", error);

                return;
            }
        };
        let new_page_index = new_page_index.clamp(0, element_scanner_results_view_data.cached_last_page_index);

        // If the new index is the same as the current one, do nothing.
        if new_page_index == element_scanner_results_view_data.current_page_index {
            return;
        }

        element_scanner_results_view_data.current_page_index = new_page_index;

        // Refresh scan results with the new page index. // JIRA: Should happen in the loop technically, but we need to make the MVVM bindings deadlock resistant.
        Self::query_scan_results(element_scanner_results_view_data_clone, engine_execution_context, false);
    }

    fn on_page_index_text_changed(
        element_scanner_results_view_data: Dependency<Self>,
        engine_execution_context: Arc<EngineExecutionContext>,
        new_page_index_text: &str,
    ) {
        // Extract numeric part from new_page_index_text and parse it to u64, defaulting to 0.
        let new_page_index = new_page_index_text
            .chars()
            .take_while(|char| char.is_digit(10))
            .collect::<String>()
            .parse::<u64>()
            .unwrap_or(0);

        Self::set_page_index(element_scanner_results_view_data, engine_execution_context, new_page_index);
    }

    fn on_navigate_first_page(
        element_scanner_results_view_data: Dependency<Self>,
        engine_execution_context: Arc<EngineExecutionContext>,
    ) {
        let new_page_index = 0;

        Self::set_page_index(element_scanner_results_view_data, engine_execution_context, new_page_index);
    }

    fn on_navigate_last_page(
        element_scanner_results_view_data: Dependency<Self>,
        engine_execution_context: Arc<EngineExecutionContext>,
    ) {
        let cached_last_page_index = match element_scanner_results_view_data.read() {
            Ok(element_scanner_results_view_data) => element_scanner_results_view_data.cached_last_page_index,
            Err(error) => {
                log::error!("Failed to acquire read lock on element scanner results view data: {}", error);

                return;
            }
        };
        let cached_last_page_index = cached_last_page_index;
        let new_page_index = cached_last_page_index;

        Self::set_page_index(element_scanner_results_view_data, engine_execution_context, new_page_index);
    }

    fn on_navigate_previous_page(
        element_scanner_results_view_data: Dependency<Self>,
        engine_execution_context: Arc<EngineExecutionContext>,
    ) {
        let element_scanner_results_view_data_clone = element_scanner_results_view_data.clone();
        let element_scanner_results_view_data = match element_scanner_results_view_data.read() {
            Ok(element_scanner_results_view_data) => element_scanner_results_view_data,
            Err(error) => {
                log::error!("Failed to acquire read lock on element scanner results view data: {}", error);

                return;
            }
        };
        let new_page_index = Self::load_current_page_index(&element_scanner_results_view_data).saturating_sub(1);

        Self::set_page_index(element_scanner_results_view_data_clone, engine_execution_context, new_page_index);
    }

    fn on_navigate_next_page(
        element_scanner_results_view_data: Dependency<Self>,
        engine_execution_context: Arc<EngineExecutionContext>,
    ) {
        let element_scanner_results_view_data_clone = element_scanner_results_view_data.clone();
        let element_scanner_results_view_data = match element_scanner_results_view_data.read() {
            Ok(element_scanner_results_view_data) => element_scanner_results_view_data,
            Err(error) => {
                log::error!("Failed to acquire read lock on element scanner results view data: {}", error);

                return;
            }
        };
        let new_page_index = Self::load_current_page_index(&element_scanner_results_view_data).saturating_add(1);

        Self::set_page_index(element_scanner_results_view_data_clone, engine_execution_context, new_page_index);
    }

    fn on_set_scan_result_selection_start(
        element_scanner_results_view_data: Dependency<Self>,
        engine_execution_context: Arc<EngineExecutionContext>,
        scan_result_collection_start_index: i32,
    ) {
        let mut element_scanner_results_view_data = match element_scanner_results_view_data.write() {
            Ok(element_scanner_results_view_data) => element_scanner_results_view_data,
            Err(error) => {
                log::error!("Failed to acquire write lock on element scanner results view data: {}", error);

                return;
            }
        };
        element_scanner_results_view_data.selection_index_start = scan_result_collection_start_index;

        /*
        let scan_results = Self::collect_selected_scan_results();

        if !scan_results.is_empty() {
            let struct_viewer_view_model = &self.struct_viewer_view_model;
            struct_viewer_self.set_selected_structs(
                StructViewerDomain::ScanResult,
                scan_results
                    .iter()
                    .map(|scan_result| scan_result.as_property_struct())
                    .collect(),
            );

        }*/
    }

    fn on_set_scan_result_selection_end(
        element_scanner_results_view_data: Dependency<Self>,
        engine_execution_context: Arc<EngineExecutionContext>,
        scan_result_collection_end_index: i32,
    ) {
        /*
        self.selection_index_end = scan_result_collection_end_index;
        let scan_results = self.collect_selected_scan_results();

        if !scan_results.is_empty() {
            let struct_viewer_view_model = &self.struct_viewer_view_model;
            struct_viewer_self.set_selected_structs(
                StructViewerDomain::ScanResult,
                scan_results
                    .iter()
                    .map(|scan_result| scan_result.as_property_struct())
                    .collect(),
            );

        }*/
    }

    fn on_add_scan_results_to_project(
        element_scanner_results_view_data: Dependency<Self>,
        engine_execution_context: Arc<EngineExecutionContext>,
    ) {
        let scan_result_refs = Self::collect_selected_scan_result_refs(element_scanner_results_view_data, engine_execution_context.clone());

        if !scan_result_refs.is_empty() {
            let engine_execution_context = &engine_execution_context;
            let scan_results_add_to_project_request = ScanResultsAddToProjectRequest { scan_result_refs };

            scan_results_add_to_project_request.send(engine_execution_context, |_response| {});
        }
    }

    fn on_delete_selected_scan_results(
        element_scanner_results_view_data: Dependency<Self>,
        engine_execution_context: Arc<EngineExecutionContext>,
    ) {
        let scan_result_refs = Self::collect_selected_scan_result_refs(element_scanner_results_view_data, engine_execution_context.clone());

        if !scan_result_refs.is_empty() {
            let engine_execution_context = &engine_execution_context;
            let scan_results_delete_request = ScanResultsDeleteRequest { scan_result_refs };

            scan_results_delete_request.send(engine_execution_context, |_response| {});
        }
    }

    fn on_set_scan_result_frozen(
        element_scanner_results_view_data: Dependency<Self>,
        engine_execution_context: Arc<EngineExecutionContext>,
        local_scan_result_index: i32,
        is_frozen: bool,
    ) {
        let local_scan_result_indices_vec = (local_scan_result_index..=local_scan_result_index).collect::<Vec<_>>();
        let scan_result_refs = Self::collect_scan_result_refs_by_indicies(
            element_scanner_results_view_data,
            engine_execution_context.clone(),
            &&local_scan_result_indices_vec,
        );

        if !scan_result_refs.is_empty() {
            let engine_execution_context = &engine_execution_context;
            let scan_results_freeze_request = ScanResultsFreezeRequest { scan_result_refs, is_frozen };

            scan_results_freeze_request.send(engine_execution_context, |_response| {});
        }
    }

    fn on_toggle_selected_scan_results_frozen(
        element_scanner_results_view_data: Dependency<Self>,
        engine_execution_context: Arc<EngineExecutionContext>,
    ) {
        /*
        let scan_results = self.collect_selected_scan_result_bases();

        if !scan_results.is_empty() {
            let engine_execution_context = &engine_execution_context;
            let scan_results_freeze_request = ScanResultsFreezeRequest { scan_results, is_frozen };

            scan_results_freeze_request.send(engine_execution_context, |_response| {});
        }*/
    }

    fn collect_selected_scan_result_refs(
        element_scanner_results_view_data: Dependency<Self>,
        engine_execution_context: Arc<EngineExecutionContext>,
    ) -> Vec<ScanResultRef> {
        Self::collect_selected_scan_results(element_scanner_results_view_data)
            .into_iter()
            .map(|scan_result| scan_result.get_base_result().get_scan_result_ref().clone())
            .collect()
    }

    fn collect_scan_result_refs_by_indicies(
        element_scanner_results_view_data: Dependency<Self>,
        engine_execution_context: Arc<EngineExecutionContext>,
        local_scan_result_indices: &[i32],
    ) -> Vec<ScanResultRef> {
        Self::collect_scan_result_bases_by_indicies(element_scanner_results_view_data, engine_execution_context, local_scan_result_indices)
            .into_iter()
            .map(|scan_result| scan_result.get_scan_result_ref().clone())
            .collect()
    }

    fn collect_selected_scan_result_bases(
        element_scanner_results_view_data: Dependency<Self>,
        engine_execution_context: Arc<EngineExecutionContext>,
    ) -> Vec<ScanResultBase> {
        Self::collect_selected_scan_results(element_scanner_results_view_data)
            .into_iter()
            .map(|scan_result| scan_result.get_base_result().clone())
            .collect()
    }

    fn collect_selected_scan_results(element_scanner_results_view_data: Dependency<Self>) -> Vec<ScanResult> {
        let element_scanner_results_view_data = match element_scanner_results_view_data.read() {
            Ok(element_scanner_results_view_data) => element_scanner_results_view_data,
            Err(error) => {
                log::error!("Failed to acquire read lock on element scanner results view data: {}", error);

                return Vec::new();
            }
        };
        let mut initial_selection_index_start = element_scanner_results_view_data.selection_index_start;
        let mut initial_selection_index_end = element_scanner_results_view_data.selection_index_end;

        // If either start or end is invalid, set the start and end to the same value (single selection).
        if initial_selection_index_start < 0 && initial_selection_index_end >= 0 {
            initial_selection_index_start = initial_selection_index_end;
        } else if initial_selection_index_end < 0 && initial_selection_index_start >= 0 {
            initial_selection_index_end = initial_selection_index_start;
        }

        // If both are invalid, return empty
        if initial_selection_index_start < 0 || initial_selection_index_end < 0 {
            return vec![];
        }

        let selection_index_start = cmp::min(initial_selection_index_start, initial_selection_index_end);
        let selection_index_end = cmp::max(initial_selection_index_start, initial_selection_index_end);

        let local_scan_result_indices = selection_index_start..=selection_index_end;

        local_scan_result_indices
            .filter_map(|index| {
                element_scanner_results_view_data
                    .current_scan_results
                    .get(index as usize)
                    .cloned()
            })
            .collect()
    }

    fn collect_scan_result_bases_by_indicies(
        element_scanner_results_view_data: Dependency<Self>,
        engine_execution_context: Arc<EngineExecutionContext>,
        local_scan_result_indices: &[i32],
    ) -> Vec<ScanResultBase> {
        let element_scanner_results_view_data = match element_scanner_results_view_data.read() {
            Ok(element_scanner_results_view_data) => element_scanner_results_view_data,
            Err(error) => {
                log::error!("Failed to acquire read lock on element scanner results view data: {}", error);

                return Vec::new();
            }
        };
        let scan_results = local_scan_result_indices
            .iter()
            .filter_map(|index| {
                element_scanner_results_view_data
                    .current_scan_results
                    .get(*index as usize)
                    .map(|scan_result| scan_result.get_base_result().clone())
            })
            .collect();

        scan_results
    }
}
