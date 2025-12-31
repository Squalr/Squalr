use arc_swap::Guard;
use squalr_engine_api::commands::scan_results::add_to_project::scan_results_add_to_project_request::ScanResultsAddToProjectRequest;
use squalr_engine_api::commands::scan_results::delete::scan_results_delete_request::ScanResultsDeleteRequest;
use squalr_engine_api::commands::scan_results::freeze::scan_results_freeze_request::ScanResultsFreezeRequest;
use squalr_engine_api::dependency_injection::dependency::Dependency;
use squalr_engine_api::dependency_injection::write_guard::WriteGuard;
use squalr_engine_api::engine::engine_execution_context::EngineExecutionContext;
use squalr_engine_api::structures::data_values::display_value::DisplayValue;
use squalr_engine_api::structures::data_values::display_value_type::DisplayValueType;
use squalr_engine_api::structures::scan_results::scan_result_base::ScanResultBase;
use squalr_engine_api::structures::scan_results::scan_result_ref::ScanResultRef;
use squalr_engine_api::structures::structs::container_type::ContainerType;
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
use std::sync::Arc;
use std::{thread, time::Duration};

#[derive(Clone)]
pub struct ElementScannerResultsViewData {
    // audio_player: AudioPlayer,
    pub value_splitter_ratio: f32,
    pub previous_value_splitter_ratio: f32,
    pub current_scan_results: Vec<ScanResult>,
    pub current_page_index: u64,
    pub cached_last_page_index: u64,
    pub selection_index_start: Option<i32>,
    pub selection_index_end: Option<i32>,
    pub result_count: u64,
    pub stats_string: String,
    pub edit_value: DisplayValue,
    pub is_querying_scan_results: bool,
    pub is_refreshing_scan_results: bool,
    pub is_setting_property: bool,
}

impl ElementScannerResultsViewData {
    pub const DEFAULT_VALUE_SPLITTER_RATIO: f32 = 0.35;
    pub const DEFAULT_PREVIOUS_VALUE_SPLITTER_RATIO: f32 = 0.70;

    pub fn new() -> Self {
        Self {
            value_splitter_ratio: Self::DEFAULT_VALUE_SPLITTER_RATIO,
            previous_value_splitter_ratio: Self::DEFAULT_PREVIOUS_VALUE_SPLITTER_RATIO,
            current_scan_results: Vec::new(),
            current_page_index: 0,
            cached_last_page_index: 0,
            selection_index_start: None,
            selection_index_end: None,
            result_count: 0,
            stats_string: String::new(),
            edit_value: DisplayValue::new(String::new(), DisplayValueType::Decimal, ContainerType::None),
            is_querying_scan_results: false,
            is_refreshing_scan_results: false,
            is_setting_property: false,
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

    pub fn navigate_first_page(
        element_scanner_results_view_data: Dependency<Self>,
        engine_execution_context: Arc<EngineExecutionContext>,
    ) {
        let new_page_index = 0;

        Self::set_page_index(element_scanner_results_view_data, engine_execution_context, new_page_index);
    }

    pub fn navigate_last_page(
        element_scanner_results_view_data: Dependency<Self>,
        engine_execution_context: Arc<EngineExecutionContext>,
    ) {
        let cached_last_page_index = match element_scanner_results_view_data.read("Element scanner results navigation last") {
            Some(element_scanner_results_view_data) => element_scanner_results_view_data.cached_last_page_index,
            None => return,
        };
        let cached_last_page_index = cached_last_page_index;
        let new_page_index = cached_last_page_index;

        Self::set_page_index(element_scanner_results_view_data, engine_execution_context, new_page_index);
    }

    pub fn navigate_previous_page(
        element_scanner_results_view_data: Dependency<Self>,
        engine_execution_context: Arc<EngineExecutionContext>,
    ) {
        let element_scanner_results_view_data_clone = element_scanner_results_view_data.clone();
        let element_scanner_results_view_data = match element_scanner_results_view_data.read("Element scanner results navigation previous") {
            Some(element_scanner_results_view_data) => element_scanner_results_view_data,
            None => return,
        };
        let new_page_index = Self::load_current_page_index(&element_scanner_results_view_data).saturating_sub(1);

        drop(element_scanner_results_view_data);

        Self::set_page_index(element_scanner_results_view_data_clone, engine_execution_context, new_page_index);
    }

    pub fn navigate_next_page(
        element_scanner_results_view_data: Dependency<Self>,
        engine_execution_context: Arc<EngineExecutionContext>,
    ) {
        let element_scanner_results_view_data_clone = element_scanner_results_view_data.clone();
        let element_scanner_results_view_data = match element_scanner_results_view_data.read("Element scanner results navigation next") {
            Some(element_scanner_results_view_data) => element_scanner_results_view_data,
            None => return,
        };
        let new_page_index = Self::load_current_page_index(&element_scanner_results_view_data).saturating_add(1);

        drop(element_scanner_results_view_data);

        Self::set_page_index(element_scanner_results_view_data_clone, engine_execution_context, new_page_index);
    }

    pub fn set_selected_scan_results_value(
        element_scanner_results_view_data: Dependency<Self>,
        engine_execution_context: Arc<EngineExecutionContext>,
        field_namespace: String,
        anonymous_value: AnonymousValue,
    ) {
        let element_scanner_results_view_data_clone = element_scanner_results_view_data.clone();
        let mut element_scanner_results_view_data = match element_scanner_results_view_data.write("Set selected scan results") {
            Some(element_scanner_results_view_data) => element_scanner_results_view_data,
            None => return,
        };
        let scan_result_refs = element_scanner_results_view_data
            .current_scan_results
            .iter()
            .map(|scan_result| scan_result.get_base_result().get_scan_result_ref().clone())
            .collect();

        let scan_results_set_property_request = ScanResultsSetPropertyRequest {
            scan_result_refs,
            field_namespace,
            anonymous_value,
        };

        element_scanner_results_view_data.is_setting_property = true;

        scan_results_set_property_request.send(&engine_execution_context, move |scan_results_set_property_response| {
            let mut element_scanner_results_view_data = match element_scanner_results_view_data_clone.write("Set selected scan results response") {
                Some(element_scanner_results_view_data) => element_scanner_results_view_data,
                None => return,
            };

            element_scanner_results_view_data.is_setting_property = false;
        });
    }

    fn load_current_page_index(element_scanner_results_view_data: &Guard<Arc<ElementScannerResultsViewData>>) -> u64 {
        element_scanner_results_view_data
            .current_page_index
            .clamp(0, element_scanner_results_view_data.cached_last_page_index)
    }

    fn load_current_page_index_write(element_scanner_results_view_data: &WriteGuard<'_, ElementScannerResultsViewData>) -> u64 {
        element_scanner_results_view_data
            .current_page_index
            .clamp(0, element_scanner_results_view_data.cached_last_page_index)
    }

    fn query_scan_results(
        element_scanner_results_view_data: Dependency<Self>,
        engine_execution_context: Arc<EngineExecutionContext>,
        play_sound: bool,
    ) {
        if element_scanner_results_view_data
            .read("Query scan results")
            .map(|element_scanner_results_view_data| element_scanner_results_view_data.is_querying_scan_results)
            .unwrap_or(false)
        {
            return;
        }

        let element_scanner_results_view_data_clone = element_scanner_results_view_data.clone();
        let mut element_scanner_results_view_data = match element_scanner_results_view_data.write("Query scan results") {
            Some(element_scanner_results_view_data) => element_scanner_results_view_data,
            None => return,
        };
        let page_index = Self::load_current_page_index_write(&element_scanner_results_view_data);
        let scan_results_query_request = ScanResultsQueryRequest { page_index };

        element_scanner_results_view_data.is_querying_scan_results = true;

        scan_results_query_request.send(&engine_execution_context, move |scan_results_query_response| {
            // let audio_player = &self.audio_player;
            let byte_size_in_metric = Conversions::value_to_metric_size(scan_results_query_response.total_size_in_bytes);
            let result_count = scan_results_query_response.result_count;

            if let Some(mut element_scanner_results_view_data) = element_scanner_results_view_data_clone.write("Query scan results response") {
                element_scanner_results_view_data.is_querying_scan_results = false;
                element_scanner_results_view_data.cached_last_page_index = scan_results_query_response.last_page_index;
                element_scanner_results_view_data.result_count = result_count;
                element_scanner_results_view_data.stats_string = format!("{} (Count: {})", byte_size_in_metric, result_count);
                element_scanner_results_view_data.current_scan_results = scan_results_query_response.scan_results;
            }

            if play_sound {
                if result_count > 0 {
                    // audio_player.play_sound(SoundType::Success);
                } else {
                    // audio_player.play_sound(SoundType::Warn);
                }
            }
        });
    }

    /// Fetches up-to-date values and module information for the current scan results, then updates the UI.
    fn refresh_scan_results(
        element_scanner_results_view_data: Dependency<Self>,
        engine_execution_context: Arc<EngineExecutionContext>,
    ) {
        if element_scanner_results_view_data
            .read("Refresh scan results")
            .map(|element_scanner_results_view_data| {
                element_scanner_results_view_data.is_querying_scan_results || element_scanner_results_view_data.is_refreshing_scan_results
            })
            .unwrap_or(false)
        {
            return;
        }

        let element_scanner_results_view_data_clone = element_scanner_results_view_data.clone();
        let mut element_scanner_results_view_data = match element_scanner_results_view_data.write("Refresh scan results") {
            Some(element_scanner_results_view_data) => element_scanner_results_view_data,
            None => return,
        };
        let engine_execution_context = &engine_execution_context;

        element_scanner_results_view_data.is_refreshing_scan_results = true;

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
            let mut element_scanner_results_view_data = match element_scanner_results_view_data_clone.write("Refresh scan results response") {
                Some(element_scanner_results_view_data) => element_scanner_results_view_data,
                None => return,
            };

            // Update UI with refreshed, full scan result values.
            element_scanner_results_view_data.is_refreshing_scan_results = false;
            element_scanner_results_view_data.current_scan_results = scan_results_refresh_response.scan_results;
        });
    }

    fn set_page_index(
        element_scanner_results_view_data: Dependency<Self>,
        engine_execution_context: Arc<EngineExecutionContext>,
        new_page_index: u64,
    ) {
        if element_scanner_results_view_data
            .read("Set page index")
            .map(|element_scanner_results_view_data| element_scanner_results_view_data.is_querying_scan_results)
            .unwrap_or(false)
        {
            return;
        }

        let element_scanner_results_view_data_clone = element_scanner_results_view_data.clone();
        let mut element_scanner_results_view_data = match element_scanner_results_view_data.write("Set page index") {
            Some(element_scanner_results_view_data) => element_scanner_results_view_data,
            None => return,
        };
        let new_page_index = new_page_index.clamp(0, element_scanner_results_view_data.cached_last_page_index);

        // If the new index is the same as the current one, do nothing.
        if new_page_index == element_scanner_results_view_data.current_page_index {
            return;
        }

        element_scanner_results_view_data.current_page_index = new_page_index;

        // Clear out our selected items.
        element_scanner_results_view_data.selection_index_start = None;
        element_scanner_results_view_data.selection_index_end = None;

        drop(element_scanner_results_view_data);

        // Refresh scan results with the new page index. // JIRA: Should happen in the loop technically, but we need to make the MVVM bindings deadlock resistant.
        Self::query_scan_results(element_scanner_results_view_data_clone, engine_execution_context, false);
    }

    pub fn set_page_index_string(
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

    pub fn set_scan_result_selection_start(
        element_scanner_results_view_data: Dependency<Self>,
        engine_execution_context: Arc<EngineExecutionContext>,
        scan_result_collection_start_index: Option<i32>,
    ) {
        let mut element_scanner_results_view_data = match element_scanner_results_view_data.write("Set scan result selection start") {
            Some(element_scanner_results_view_data) => element_scanner_results_view_data,
            None => return,
        };
        element_scanner_results_view_data.selection_index_start = scan_result_collection_start_index;
        element_scanner_results_view_data.selection_index_end = None;

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

    pub fn set_scan_result_selection_end(
        element_scanner_results_view_data: Dependency<Self>,
        engine_execution_context: Arc<EngineExecutionContext>,
        scan_result_collection_end_index: Option<i32>,
    ) {
        let mut element_scanner_results_view_data = match element_scanner_results_view_data.write("Set scan result selection end") {
            Some(element_scanner_results_view_data) => element_scanner_results_view_data,
            None => return,
        };
        element_scanner_results_view_data.selection_index_end = scan_result_collection_end_index;

        /*
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

    pub fn add_scan_results_to_project(
        element_scanner_results_view_data: Dependency<Self>,
        engine_execution_context: Arc<EngineExecutionContext>,
    ) {
        let scan_result_refs = Self::collect_selected_scan_result_refs(element_scanner_results_view_data);

        if !scan_result_refs.is_empty() {
            let engine_execution_context = &engine_execution_context;
            let scan_results_add_to_project_request = ScanResultsAddToProjectRequest { scan_result_refs };

            scan_results_add_to_project_request.send(engine_execution_context, |_response| {});
        }
    }

    pub fn delete_selected_scan_results(
        element_scanner_results_view_data: Dependency<Self>,
        engine_execution_context: Arc<EngineExecutionContext>,
    ) {
        let scan_result_refs = Self::collect_selected_scan_result_refs(element_scanner_results_view_data);

        if !scan_result_refs.is_empty() {
            let engine_execution_context = &engine_execution_context;
            let scan_results_delete_request = ScanResultsDeleteRequest { scan_result_refs };

            scan_results_delete_request.send(engine_execution_context, |_response| {});
        }
    }

    pub fn set_scan_result_frozen(
        element_scanner_results_view_data: Dependency<Self>,
        engine_execution_context: Arc<EngineExecutionContext>,
        local_scan_result_index: i32,
        is_frozen: bool,
    ) {
        let local_scan_result_indices_vec = (local_scan_result_index..=local_scan_result_index).collect::<Vec<_>>();
        let scan_result_refs = Self::collect_scan_result_refs_by_indicies(element_scanner_results_view_data, &&local_scan_result_indices_vec);

        if !scan_result_refs.is_empty() {
            let engine_execution_context = &engine_execution_context;
            let scan_results_freeze_request = ScanResultsFreezeRequest { scan_result_refs, is_frozen };

            scan_results_freeze_request.send(engine_execution_context, |_response| {});
        }
    }

    pub fn toggle_selected_scan_results_frozen(
        element_scanner_results_view_data: Dependency<Self>,
        engine_execution_context: Arc<EngineExecutionContext>,
        is_frozen: bool,
    ) {
        let scan_result_refs = Self::collect_selected_scan_result_refs(element_scanner_results_view_data.clone());

        if !scan_result_refs.is_empty() {
            let engine_execution_context = &engine_execution_context;
            let scan_results_freeze_request = ScanResultsFreezeRequest { scan_result_refs, is_frozen };

            scan_results_freeze_request.send(engine_execution_context, |_response| {});
        }
    }

    fn collect_selected_scan_result_refs(element_scanner_results_view_data: Dependency<Self>) -> Vec<ScanResultRef> {
        Self::collect_selected_scan_results(element_scanner_results_view_data)
            .into_iter()
            .map(|scan_result| scan_result.get_base_result().get_scan_result_ref().clone())
            .collect()
    }

    fn collect_scan_result_refs_by_indicies(
        element_scanner_results_view_data: Dependency<Self>,
        local_scan_result_indices: &[i32],
    ) -> Vec<ScanResultRef> {
        Self::collect_scan_result_bases_by_indicies(element_scanner_results_view_data, local_scan_result_indices)
            .into_iter()
            .map(|scan_result| scan_result.get_scan_result_ref().clone())
            .collect()
    }

    fn collect_selected_scan_results(element_scanner_results_view_data: Dependency<Self>) -> Vec<ScanResult> {
        let element_scanner_results_view_data = match element_scanner_results_view_data.read("Collect selected scan results") {
            Some(element_scanner_results_view_data) => element_scanner_results_view_data,
            None => return Vec::new(),
        };

        // Pull out the optional bounds.
        let mut initial_selection_index_start = element_scanner_results_view_data.selection_index_start;
        let mut initial_selection_index_end = element_scanner_results_view_data.selection_index_end;

        // If either start or end is invalid, set the start and end to the same value (single selection).
        match (initial_selection_index_start, initial_selection_index_end) {
            (Some(start), None) => {
                initial_selection_index_end = Some(start);
            }
            (None, Some(end)) => {
                initial_selection_index_start = Some(end);
            }
            _ => {}
        }

        // If both are invalid, return empty.
        let (Some(start), Some(end)) = (initial_selection_index_start, initial_selection_index_end) else {
            return vec![];
        };

        let selection_index_start = cmp::min(start, end);
        let selection_index_end = cmp::max(start, end);
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
        local_scan_result_indices: &[i32],
    ) -> Vec<ScanResultBase> {
        let element_scanner_results_view_data = match element_scanner_results_view_data.read("Collect scan result bases") {
            Some(element_scanner_results_view_data) => element_scanner_results_view_data,
            None => return Vec::new(),
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
