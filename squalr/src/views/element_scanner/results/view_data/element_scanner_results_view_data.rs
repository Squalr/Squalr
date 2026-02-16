use crate::views::struct_viewer::view_data::struct_viewer_view_data::StructViewerViewData;
use arc_swap::Guard;
use squalr_engine_api::commands::project_items::add::project_items_add_request::ProjectItemsAddRequest;
use squalr_engine_api::commands::scan_results::delete::scan_results_delete_request::ScanResultsDeleteRequest;
use squalr_engine_api::commands::scan_results::freeze::scan_results_freeze_request::ScanResultsFreezeRequest;
use squalr_engine_api::conversions::storage_size_conversions::StorageSizeConversions;
use squalr_engine_api::dependency_injection::dependency::Dependency;
use squalr_engine_api::dependency_injection::write_guard::WriteGuard;
use squalr_engine_api::registries::symbols::symbol_registry::SymbolRegistry;
use squalr_engine_api::structures::data_values::anonymous_value_string_format::AnonymousValueStringFormat;
use squalr_engine_api::structures::data_values::container_type::ContainerType;
use squalr_engine_api::structures::scan_results::scan_result_base::ScanResultBase;
use squalr_engine_api::structures::scan_results::scan_result_ref::ScanResultRef;
use squalr_engine_api::{
    commands::{
        privileged_command_request::PrivilegedCommandRequest, scan_results::query::scan_results_query_request::ScanResultsQueryRequest,
        scan_results::refresh::scan_results_refresh_request::ScanResultsRefreshRequest,
        scan_results::set_property::scan_results_set_property_request::ScanResultsSetPropertyRequest, unprivileged_command_request::UnprivilegedCommandRequest,
    },
    events::scan_results::updated::scan_results_updated_event::ScanResultsUpdatedEvent,
    structures::{data_values::anonymous_value_string::AnonymousValueString, scan_results::scan_result::ScanResult},
};
use squalr_engine_session::engine_unprivileged_state::EngineUnprivilegedState;
use std::ops::RangeInclusive;
use std::path::PathBuf;
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
    pub current_display_string: AnonymousValueString,
    pub is_querying_scan_results: bool,
    pub is_refreshing_scan_results: bool,
    pub is_setting_properties: bool,
    pub is_freezing_entries: bool,
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
            current_display_string: AnonymousValueString::new(String::new(), AnonymousValueStringFormat::Decimal, ContainerType::None),
            is_querying_scan_results: false,
            is_refreshing_scan_results: false,
            is_setting_properties: false,
            is_freezing_entries: false,
        }
    }

    pub fn poll_scan_results(
        element_scanner_results_view_data: Dependency<Self>,
        engine_unprivileged_state: Arc<EngineUnprivilegedState>,
    ) {
        Self::query_scan_results(element_scanner_results_view_data.clone(), engine_unprivileged_state.clone(), false);

        let engine_unprivileged_state_clone = engine_unprivileged_state.clone();
        let element_scanner_results_view_data_clone = element_scanner_results_view_data.clone();

        // Requery all scan results if they update.
        {
            engine_unprivileged_state.listen_for_engine_event::<ScanResultsUpdatedEvent>(move |scan_results_updated_event| {
                let element_scanner_results_view_data = element_scanner_results_view_data_clone.clone();
                let engine_unprivileged_state = engine_unprivileged_state_clone.clone();
                let play_sound = !scan_results_updated_event.is_new_scan;

                Self::query_scan_results(element_scanner_results_view_data, engine_unprivileged_state, play_sound);
            });
        }

        let engine_unprivileged_state_clone = engine_unprivileged_state.clone();
        let element_scanner_results_view_data_clone = element_scanner_results_view_data.clone();

        // Refresh scan values on a loop. JIRA: This should be coming from settings. We can probably cache, and have some mechanism for getting latest val.
        thread::spawn(move || {
            loop {
                let element_scanner_results_view_data = element_scanner_results_view_data_clone.clone();
                let engine_unprivileged_state = engine_unprivileged_state_clone.clone();

                Self::refresh_scan_results(element_scanner_results_view_data, engine_unprivileged_state);

                thread::sleep(Duration::from_millis(100));
            }
        });
    }

    pub fn navigate_first_page(
        element_scanner_results_view_data: Dependency<Self>,
        engine_unprivileged_state: Arc<EngineUnprivilegedState>,
    ) {
        let new_page_index = 0;

        Self::set_page_index(element_scanner_results_view_data, engine_unprivileged_state, new_page_index);
    }

    pub fn navigate_last_page(
        element_scanner_results_view_data: Dependency<Self>,
        engine_unprivileged_state: Arc<EngineUnprivilegedState>,
    ) {
        let cached_last_page_index = match element_scanner_results_view_data.read("Element scanner results navigation last") {
            Some(element_scanner_results_view_data) => element_scanner_results_view_data.cached_last_page_index,
            None => return,
        };
        let cached_last_page_index = cached_last_page_index;
        let new_page_index = cached_last_page_index;

        Self::set_page_index(element_scanner_results_view_data, engine_unprivileged_state, new_page_index);
    }

    pub fn navigate_previous_page(
        element_scanner_results_view_data: Dependency<Self>,
        engine_unprivileged_state: Arc<EngineUnprivilegedState>,
    ) {
        let element_scanner_results_view_data_clone = element_scanner_results_view_data.clone();
        let element_scanner_results_view_data = match element_scanner_results_view_data.read("Element scanner results navigation previous") {
            Some(element_scanner_results_view_data) => element_scanner_results_view_data,
            None => return,
        };
        let new_page_index = Self::load_current_page_index(&element_scanner_results_view_data).saturating_sub(1);

        drop(element_scanner_results_view_data);

        Self::set_page_index(element_scanner_results_view_data_clone, engine_unprivileged_state, new_page_index);
    }

    pub fn navigate_next_page(
        element_scanner_results_view_data: Dependency<Self>,
        engine_unprivileged_state: Arc<EngineUnprivilegedState>,
    ) {
        let element_scanner_results_view_data_clone = element_scanner_results_view_data.clone();
        let element_scanner_results_view_data = match element_scanner_results_view_data.read("Element scanner results navigation next") {
            Some(element_scanner_results_view_data) => element_scanner_results_view_data,
            None => return,
        };
        let new_page_index = Self::load_current_page_index(&element_scanner_results_view_data).saturating_add(1);

        drop(element_scanner_results_view_data);

        Self::set_page_index(element_scanner_results_view_data_clone, engine_unprivileged_state, new_page_index);
    }

    pub fn set_selected_scan_results_value(
        element_scanner_results_view_data: Dependency<Self>,
        engine_unprivileged_state: Arc<EngineUnprivilegedState>,
        field_namespace: &str,
        anonymous_value_string: AnonymousValueString,
    ) {
        let element_scanner_results_view_data_clone = element_scanner_results_view_data.clone();
        let mut element_scanner_results_view_data = match element_scanner_results_view_data.write("Set selected scan results") {
            Some(element_scanner_results_view_data) => element_scanner_results_view_data,
            None => return,
        };
        let scan_result_refs = Self::collect_scan_result_refs_for_selected_range(&element_scanner_results_view_data);

        if scan_result_refs.is_empty() {
            return;
        }

        let scan_results_set_property_request = ScanResultsSetPropertyRequest {
            scan_result_refs,
            field_namespace: field_namespace.to_string(),
            anonymous_value_string,
        };

        element_scanner_results_view_data.is_setting_properties = true;

        // Drop to commit the write before send(), which may execute the callback synchronously.
        drop(element_scanner_results_view_data);

        scan_results_set_property_request.send(&engine_unprivileged_state, move |_scan_results_set_property_response| {
            let mut element_scanner_results_view_data = match element_scanner_results_view_data_clone.write("Set selected scan results response") {
                Some(element_scanner_results_view_data) => element_scanner_results_view_data,
                None => return,
            };

            element_scanner_results_view_data.is_setting_properties = false;
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
        engine_unprivileged_state: Arc<EngineUnprivilegedState>,
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

        // Drop to commit the write before send(), which may execute the callback synchronously.
        drop(element_scanner_results_view_data);

        let element_scanner_results_view_data_for_response = element_scanner_results_view_data_clone.clone();
        let did_dispatch = scan_results_query_request.send(&engine_unprivileged_state, move |scan_results_query_response| {
            // let audio_player = &self.audio_player;
            let byte_size_in_metric = StorageSizeConversions::value_to_metric_size(scan_results_query_response.total_size_in_bytes as u128);
            let result_count = scan_results_query_response.result_count;

            if let Some(mut element_scanner_results_view_data) = element_scanner_results_view_data_for_response.write("Query scan results response") {
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

        if !did_dispatch {
            if let Some(mut element_scanner_results_view_data) = element_scanner_results_view_data_clone.write("Query scan results dispatch failure") {
                element_scanner_results_view_data.is_querying_scan_results = false;
            }
        }
    }

    /// Fetches up-to-date values and module information for the current scan results, then updates the UI.
    fn refresh_scan_results(
        element_scanner_results_view_data: Dependency<Self>,
        engine_unprivileged_state: Arc<EngineUnprivilegedState>,
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
        let engine_unprivileged_state = &engine_unprivileged_state;

        element_scanner_results_view_data.is_refreshing_scan_results = true;

        // Fire a request to get all scan result data needed for display.
        let scan_results_refresh_request = ScanResultsRefreshRequest {
            scan_result_refs: element_scanner_results_view_data
                .current_scan_results
                .iter()
                .map(|scan_result| scan_result.get_base_result().get_scan_result_ref().clone())
                .collect(),
        };

        // Drop to commit the write.
        drop(element_scanner_results_view_data);

        let element_scanner_results_view_data_for_response = element_scanner_results_view_data_clone.clone();
        let did_dispatch = scan_results_refresh_request.send(engine_unprivileged_state, move |scan_results_refresh_response| {
            let mut element_scanner_results_view_data = match element_scanner_results_view_data_for_response.write("Refresh scan results response") {
                Some(element_scanner_results_view_data) => element_scanner_results_view_data,
                None => return,
            };

            // Update UI with refreshed, full scan result values.
            element_scanner_results_view_data.is_refreshing_scan_results = false;
            element_scanner_results_view_data.current_scan_results = scan_results_refresh_response.scan_results;
        });

        if !did_dispatch {
            if let Some(mut element_scanner_results_view_data) = element_scanner_results_view_data_clone.write("Refresh scan results dispatch failure") {
                element_scanner_results_view_data.is_refreshing_scan_results = false;
            }
        }
    }

    fn set_page_index(
        element_scanner_results_view_data: Dependency<Self>,
        engine_unprivileged_state: Arc<EngineUnprivilegedState>,
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

        // Drop to commit the write.
        drop(element_scanner_results_view_data);

        // Refresh scan results with the new page index.
        Self::query_scan_results(element_scanner_results_view_data_clone, engine_unprivileged_state, false);
    }

    pub fn set_page_index_string(
        element_scanner_results_view_data: Dependency<Self>,
        engine_unprivileged_state: Arc<EngineUnprivilegedState>,
        new_page_index_text: &str,
    ) {
        // Extract numeric part from new_page_index_text and parse it to u64, defaulting to 0.
        let new_page_index = new_page_index_text
            .chars()
            .take_while(|char| char.is_digit(10))
            .collect::<String>()
            .parse::<u64>()
            .unwrap_or(0);

        Self::set_page_index(element_scanner_results_view_data, engine_unprivileged_state, new_page_index);
    }

    pub fn set_scan_result_selection_start(
        element_scanner_results_view_data: Dependency<Self>,
        struct_viewer_view_data: Dependency<StructViewerViewData>,
        engine_unprivileged_state: Arc<EngineUnprivilegedState>,
        scan_result_collection_start_index: Option<i32>,
    ) {
        let element_scanner_results_view_data_dependency = element_scanner_results_view_data.clone();
        let mut element_scanner_results_view_data = match element_scanner_results_view_data.write("Set scan result selection start") {
            Some(element_scanner_results_view_data) => element_scanner_results_view_data,
            None => return,
        };
        let mut valued_structs = Vec::new();

        element_scanner_results_view_data.selection_index_start = scan_result_collection_start_index;
        element_scanner_results_view_data.selection_index_end = None;

        Self::for_each_selected_scan_result(&mut element_scanner_results_view_data, |scan_result| {
            valued_structs.push(scan_result.as_valued_struct())
        });

        let element_scanner_results_view_data_clone = element_scanner_results_view_data_dependency.clone();
        let engine_unprivileged_state_clone = engine_unprivileged_state.clone();
        StructViewerViewData::focus_valued_structs(
            struct_viewer_view_data,
            valued_structs,
            Self::create_struct_field_modified_callback(element_scanner_results_view_data_clone, engine_unprivileged_state_clone),
        );
    }

    pub fn set_scan_result_selection_end(
        element_scanner_results_view_data: Dependency<Self>,
        struct_viewer_view_data: Dependency<StructViewerViewData>,
        engine_unprivileged_state: Arc<EngineUnprivilegedState>,
        scan_result_collection_end_index: Option<i32>,
    ) {
        let element_scanner_results_view_data_dependency = element_scanner_results_view_data.clone();
        let mut element_scanner_results_view_data = match element_scanner_results_view_data.write("Set scan result selection end") {
            Some(element_scanner_results_view_data) => element_scanner_results_view_data,
            None => return,
        };
        let mut valued_structs = Vec::new();

        element_scanner_results_view_data.selection_index_end = scan_result_collection_end_index;

        Self::for_each_selected_scan_result(&mut element_scanner_results_view_data, |scan_result| {
            valued_structs.push(scan_result.as_valued_struct())
        });

        let element_scanner_results_view_data_clone = element_scanner_results_view_data_dependency.clone();
        let engine_unprivileged_state_clone = engine_unprivileged_state.clone();
        StructViewerViewData::focus_valued_structs(
            struct_viewer_view_data,
            valued_structs,
            Self::create_struct_field_modified_callback(element_scanner_results_view_data_clone, engine_unprivileged_state_clone),
        );
    }

    fn create_struct_field_modified_callback(
        element_scanner_results_view_data: Dependency<Self>,
        engine_unprivileged_state: Arc<EngineUnprivilegedState>,
    ) -> Arc<dyn Fn(squalr_engine_api::structures::structs::valued_struct_field::ValuedStructField) + Send + Sync> {
        Arc::new(move |modified_field| {
            let Some(modified_data_value) = modified_field.get_data_value() else {
                return;
            };

            if modified_field.get_name() == ScanResult::PROPERTY_NAME_IS_FROZEN {
                let is_frozen = modified_data_value
                    .get_value_bytes()
                    .iter()
                    .any(|frozen_value_byte| *frozen_value_byte != 0);

                Self::toggle_selected_scan_results_frozen(element_scanner_results_view_data.clone(), engine_unprivileged_state.clone(), is_frozen);

                return;
            }

            let symbol_registry = SymbolRegistry::get_instance();
            let data_type_ref = modified_data_value.get_data_type_ref();
            let default_anonymous_value_string_format = symbol_registry.get_default_anonymous_value_string_format(data_type_ref);
            let anonymous_value_string = symbol_registry
                .anonymize_value(modified_data_value, default_anonymous_value_string_format)
                .unwrap_or_else(|error| {
                    log::warn!("Failed to anonymize struct edit value: {}", error);
                    AnonymousValueString::new(String::new(), default_anonymous_value_string_format, ContainerType::None)
                });

            Self::set_selected_scan_results_value(
                element_scanner_results_view_data.clone(),
                engine_unprivileged_state.clone(),
                modified_field.get_name(),
                anonymous_value_string,
            );
        })
    }

    pub fn add_scan_results_to_project(
        element_scanner_results_view_data: Dependency<Self>,
        engine_unprivileged_state: Arc<EngineUnprivilegedState>,
        target_directory_path: Option<PathBuf>,
    ) {
        let scan_result_refs = Self::collect_selected_scan_result_refs(element_scanner_results_view_data);

        if !scan_result_refs.is_empty() {
            let project_items_add_request = ProjectItemsAddRequest {
                scan_result_refs,
                target_directory_path,
            };

            project_items_add_request.send(&engine_unprivileged_state, |_response| {});
        }
    }

    pub fn add_scan_result_to_project_by_index(
        element_scanner_results_view_data: Dependency<Self>,
        engine_unprivileged_state: Arc<EngineUnprivilegedState>,
        local_scan_result_index: i32,
        target_directory_path: Option<PathBuf>,
    ) {
        let local_scan_result_indices = [local_scan_result_index];
        let scan_result_refs = Self::collect_scan_result_refs_by_indicies(element_scanner_results_view_data, &local_scan_result_indices);

        if !scan_result_refs.is_empty() {
            let project_items_add_request = ProjectItemsAddRequest {
                scan_result_refs,
                target_directory_path,
            };

            project_items_add_request.send(&engine_unprivileged_state, |_response| {});
        }
    }

    pub fn delete_selected_scan_results(
        element_scanner_results_view_data: Dependency<Self>,
        engine_unprivileged_state: Arc<EngineUnprivilegedState>,
    ) {
        let scan_result_refs = Self::collect_selected_scan_result_refs(element_scanner_results_view_data);

        if !scan_result_refs.is_empty() {
            let engine_unprivileged_state = &engine_unprivileged_state;
            let scan_results_delete_request = ScanResultsDeleteRequest { scan_result_refs };

            scan_results_delete_request.send(engine_unprivileged_state, |_response| {});
        }
    }

    pub fn set_scan_result_frozen(
        element_scanner_results_view_data: Dependency<Self>,
        engine_unprivileged_state: Arc<EngineUnprivilegedState>,
        local_scan_result_index: i32,
        is_frozen: bool,
    ) {
        let element_scanner_results_view_data_clone = element_scanner_results_view_data.clone();
        let local_scan_result_indices_vec = (local_scan_result_index..=local_scan_result_index).collect::<Vec<_>>();
        let scan_result_refs = Self::collect_scan_result_refs_by_indicies(element_scanner_results_view_data.clone(), &&local_scan_result_indices_vec);
        let mut element_scanner_results_view_data = match element_scanner_results_view_data.write("Element scanner results view data: set scan result frozen") {
            Some(element_scanner_results_view_data) => element_scanner_results_view_data,
            None => return,
        };

        if element_scanner_results_view_data.is_freezing_entries {
            return;
        }

        if let Some(scan_result) = element_scanner_results_view_data
            .current_scan_results
            .get_mut(local_scan_result_index as usize)
        {
            scan_result.set_is_frozen_client_only(is_frozen);
        } else {
            log::warn!("Failed to find scan result to apply client side freeze at index: {}", local_scan_result_index)
        }

        element_scanner_results_view_data.is_freezing_entries = true;

        // Drop to commit the write before send(), which may execute the callback synchronously.
        drop(element_scanner_results_view_data);

        if !scan_result_refs.is_empty() {
            let engine_unprivileged_state = &engine_unprivileged_state;
            let scan_results_freeze_request = ScanResultsFreezeRequest { scan_result_refs, is_frozen };

            scan_results_freeze_request.send(engine_unprivileged_state, move |scan_results_freeze_response| {
                let mut element_scanner_results_view_data =
                    match element_scanner_results_view_data_clone.write("Element scanner results view data: set scan result frozen response") {
                        Some(element_scanner_results_view_data) => element_scanner_results_view_data,
                        None => return,
                    };

                // Revert failures by mapping global -> local, and revert to previous state.
                for failed_scan_result_ref in scan_results_freeze_response.failed_freeze_toggle_scan_result_refs {
                    let global_index = failed_scan_result_ref.get_scan_result_global_index();

                    if let Some(local_index) = Self::find_local_index_by_global_index(&element_scanner_results_view_data, global_index) {
                        if let Some(scan_result) = element_scanner_results_view_data
                            .current_scan_results
                            .get_mut(local_index)
                        {
                            scan_result.set_is_frozen_client_only(!is_frozen);
                        }
                    } else {
                        log::warn!("Failed to find scan result to revert client side freeze (global index: {})", global_index);
                    }
                }

                element_scanner_results_view_data.is_freezing_entries = false;
            });
        }
    }

    pub fn toggle_selected_scan_results_frozen(
        element_scanner_results_view_data: Dependency<Self>,
        engine_unprivileged_state: Arc<EngineUnprivilegedState>,
        is_frozen: bool,
    ) {
        let element_scanner_results_view_data_clone = element_scanner_results_view_data.clone();
        let scan_result_refs = Self::collect_selected_scan_result_refs(element_scanner_results_view_data.clone());
        let mut element_scanner_results_view_data =
            match element_scanner_results_view_data.write("Element scanner results view data: set selected scan results frozen") {
                Some(element_scanner_results_view_data) => element_scanner_results_view_data,
                None => return,
            };

        if element_scanner_results_view_data.is_freezing_entries {
            return;
        }

        Self::for_each_selected_scan_result(&mut element_scanner_results_view_data, |scan_result| {
            scan_result.set_is_frozen_client_only(is_frozen);
        });

        element_scanner_results_view_data.is_freezing_entries = true;

        // Drop to commit the write before send(), which may execute the callback synchronously.
        drop(element_scanner_results_view_data);

        if !scan_result_refs.is_empty() {
            let engine_unprivileged_state = &engine_unprivileged_state;
            let scan_results_freeze_request = ScanResultsFreezeRequest { scan_result_refs, is_frozen };

            scan_results_freeze_request.send(engine_unprivileged_state, move |scan_results_freeze_response| {
                let mut element_scanner_results_view_data =
                    match element_scanner_results_view_data_clone.write("Element scanner results view data: set selected scan results frozen response") {
                        Some(element_scanner_results_view_data) => element_scanner_results_view_data,
                        None => return,
                    };

                // Revert failures by mapping global -> local, and revert to previous state.
                for failed_scan_result_ref in scan_results_freeze_response.failed_freeze_toggle_scan_result_refs {
                    let global_index = failed_scan_result_ref.get_scan_result_global_index();

                    if let Some(local_index) = Self::find_local_index_by_global_index(&element_scanner_results_view_data, global_index) {
                        if let Some(scan_result) = element_scanner_results_view_data
                            .current_scan_results
                            .get_mut(local_index)
                        {
                            scan_result.set_is_frozen_client_only(!is_frozen);
                        }
                    } else {
                        log::warn!("Failed to find scan result to revert client side freeze (global index: {})", global_index);
                    }
                }

                element_scanner_results_view_data.is_freezing_entries = false;
            });
        }
    }

    fn get_selected_results_range(element_scanner_results_view_data: &ElementScannerResultsViewData) -> Option<RangeInclusive<usize>> {
        let start = element_scanner_results_view_data
            .selection_index_start
            .or(element_scanner_results_view_data.selection_index_end)?;
        let end = element_scanner_results_view_data
            .selection_index_end
            .or(element_scanner_results_view_data.selection_index_start)?;
        let (range_low, range_high) = (start.min(end), start.max(end));

        Some(range_low.max(0) as usize..=range_high.max(0) as usize)
    }

    fn for_each_selected_scan_result(
        element_scanner_results_view_data: &mut ElementScannerResultsViewData,
        mut callback: impl FnMut(&mut ScanResult),
    ) {
        let Some(range) = Self::get_selected_results_range(element_scanner_results_view_data) else {
            return;
        };

        for index in range {
            if let Some(scan_result) = element_scanner_results_view_data
                .current_scan_results
                .get_mut(index)
            {
                callback(scan_result);
            }
        }
    }

    fn collect_selected_scan_result_refs(element_scanner_results_view_data: Dependency<Self>) -> Vec<ScanResultRef> {
        let element_scanner_results_view_data = match element_scanner_results_view_data.read("Collect selected scan result refs") {
            Some(element_scanner_results_view_data) => element_scanner_results_view_data,
            None => return Vec::new(),
        };

        Self::collect_scan_result_refs_for_selected_range(&element_scanner_results_view_data)
    }

    fn collect_scan_result_refs_for_selected_range(element_scanner_results_view_data: &ElementScannerResultsViewData) -> Vec<ScanResultRef> {
        let Some(selected_result_range) = Self::get_selected_results_range(element_scanner_results_view_data) else {
            return Vec::new();
        };

        selected_result_range
            .filter_map(|selected_result_index| {
                element_scanner_results_view_data
                    .current_scan_results
                    .get(selected_result_index)
            })
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

    fn find_local_index_by_global_index(
        element_scanner_results_view_data: &ElementScannerResultsViewData,
        global_index: u64,
    ) -> Option<usize> {
        element_scanner_results_view_data
            .current_scan_results
            .iter()
            .position(|scan_result| {
                scan_result
                    .get_base_result()
                    .get_scan_result_ref()
                    .get_scan_result_global_index()
                    == global_index
            })
    }
}

#[cfg(test)]
mod tests {
    use super::ElementScannerResultsViewData;
    use squalr_engine_api::dependency_injection::dependency_container::DependencyContainer;
    use squalr_engine_api::structures::data_types::data_type_ref::DataTypeRef;
    use squalr_engine_api::structures::scan_results::scan_result::ScanResult;
    use squalr_engine_api::structures::scan_results::scan_result_ref::ScanResultRef;
    use squalr_engine_api::structures::scan_results::scan_result_valued::ScanResultValued;

    fn create_scan_result(scan_result_global_index: u64) -> ScanResult {
        let scan_result_valued = ScanResultValued::new(
            0x1000 + scan_result_global_index,
            DataTypeRef::new("u8"),
            String::new(),
            None,
            Vec::new(),
            None,
            Vec::new(),
            ScanResultRef::new(scan_result_global_index),
        );

        ScanResult::new(scan_result_valued, String::new(), 0, None, Vec::new(), false)
    }

    fn create_view_data_with_scan_results(scan_result_global_indices: &[u64]) -> ElementScannerResultsViewData {
        let mut element_scanner_results_view_data = ElementScannerResultsViewData::new();
        element_scanner_results_view_data.current_scan_results = scan_result_global_indices
            .iter()
            .map(|scan_result_global_index| create_scan_result(*scan_result_global_index))
            .collect();

        element_scanner_results_view_data
    }

    #[test]
    fn collect_scan_result_refs_for_selected_range_uses_multi_select_bounds() {
        let mut element_scanner_results_view_data = create_view_data_with_scan_results(&[10, 11, 12, 13]);
        element_scanner_results_view_data.selection_index_start = Some(1);
        element_scanner_results_view_data.selection_index_end = Some(2);

        let selected_scan_result_refs = ElementScannerResultsViewData::collect_scan_result_refs_for_selected_range(&element_scanner_results_view_data);
        let selected_scan_result_global_indices = selected_scan_result_refs
            .iter()
            .map(|scan_result_ref| scan_result_ref.get_scan_result_global_index())
            .collect::<Vec<_>>();

        assert_eq!(selected_scan_result_global_indices, vec![11, 12]);
    }

    #[test]
    fn collect_scan_result_refs_for_selected_range_uses_single_select_when_end_missing() {
        let mut element_scanner_results_view_data = create_view_data_with_scan_results(&[10, 11, 12, 13]);
        element_scanner_results_view_data.selection_index_start = Some(2);
        element_scanner_results_view_data.selection_index_end = None;

        let selected_scan_result_refs = ElementScannerResultsViewData::collect_scan_result_refs_for_selected_range(&element_scanner_results_view_data);
        let selected_scan_result_global_indices = selected_scan_result_refs
            .iter()
            .map(|scan_result_ref| scan_result_ref.get_scan_result_global_index())
            .collect::<Vec<_>>();

        assert_eq!(selected_scan_result_global_indices, vec![12]);
    }

    #[test]
    fn collect_scan_result_refs_for_selected_range_returns_empty_without_selection() {
        let element_scanner_results_view_data = create_view_data_with_scan_results(&[10, 11, 12, 13]);

        let selected_scan_result_refs = ElementScannerResultsViewData::collect_scan_result_refs_for_selected_range(&element_scanner_results_view_data);

        assert!(selected_scan_result_refs.is_empty());
    }

    #[test]
    fn collect_scan_result_refs_by_indicies_returns_requested_index_only() {
        let dependency_container = DependencyContainer::new();
        let element_scanner_results_view_data = dependency_container.register(create_view_data_with_scan_results(&[10, 11, 12, 13]));

        let selected_scan_result_refs = ElementScannerResultsViewData::collect_scan_result_refs_by_indicies(element_scanner_results_view_data, &[2]);
        let selected_scan_result_global_indices = selected_scan_result_refs
            .iter()
            .map(|scan_result_ref| scan_result_ref.get_scan_result_global_index())
            .collect::<Vec<_>>();

        assert_eq!(selected_scan_result_global_indices, vec![12]);
    }
}
