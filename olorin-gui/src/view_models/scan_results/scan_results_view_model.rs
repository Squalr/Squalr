use crate::MainWindowView;
use crate::ScanResultViewData;
use crate::ScanResultsViewModelBindings;
use crate::comparers::scan_result_comparer::ScanResultComparer;
use crate::converters::scan_result_converter::ScanResultConverter;
use crate::models::audio::audio_player::AudioPlayer;
use crate::models::audio::audio_player::SoundType;
use crate::view_models::struct_viewer::struct_viewer_domain::StructViewerDomain;
use crate::view_models::struct_viewer::struct_viewer_view_model::StructViewerViewModel;
use olorin_engine::command_executors::engine_request_executor::EngineCommandRequestExecutor;
use olorin_engine::engine_execution_context::EngineExecutionContext;
use olorin_engine_api::commands::scan_results::add_to_project::scan_results_add_to_project_request::ScanResultsAddToProjectRequest;
use olorin_engine_api::commands::scan_results::delete::scan_results_delete_request::ScanResultsDeleteRequest;
use olorin_engine_api::commands::scan_results::freeze::scan_results_freeze_request::ScanResultsFreezeRequest;
use olorin_engine_api::commands::scan_results::query::scan_results_query_request::ScanResultsQueryRequest;
use olorin_engine_api::commands::scan_results::refresh::scan_results_refresh_request::ScanResultsRefreshRequest;
use olorin_engine_api::commands::scan_results::set_property::scan_results_set_property_request::ScanResultsSetPropertyRequest;
use olorin_engine_api::conversions::conversions::Conversions;
use olorin_engine_api::dependency_injection::dependency_container::DependencyContainer;
use olorin_engine_api::events::scan_results::updated::scan_results_updated_event::ScanResultsUpdatedEvent;
use olorin_engine_api::structures::data_values::data_value::DataValue;
use olorin_engine_api::structures::scan_results::scan_result::ScanResult;
use olorin_engine_api::structures::scan_results::scan_result_base::ScanResultBase;
use slint::ComponentHandle;
use slint::Model;
use slint::SharedString;
use slint_mvvm::view_binding::ViewBinding;
use slint_mvvm::view_collection_binding::ViewCollectionBinding;
use slint_mvvm_macros::create_view_bindings;
use slint_mvvm_macros::create_view_model_collection;
use std::cmp;
use std::sync::Arc;
use std::sync::RwLock;
use std::sync::atomic::AtomicI32;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;
use std::thread;
use std::time::Duration;

pub struct ScanResultsViewModel {
    view_binding: Arc<ViewBinding<MainWindowView>>,
    audio_player: Arc<AudioPlayer>,
    base_scan_results_collection: Arc<RwLock<Vec<ScanResult>>>,
    scan_results_collection: ViewCollectionBinding<ScanResultViewData, ScanResult, MainWindowView>,
    engine_execution_context: Arc<EngineExecutionContext>,
    current_page_index: Arc<AtomicU64>,
    cached_last_page_index: Arc<AtomicU64>,
    struct_viewer_view_model: Arc<StructViewerViewModel>,
    selection_index_start: Arc<AtomicI32>,
    selection_index_end: Arc<AtomicI32>,
}

impl ScanResultsViewModel {
    pub fn register(dependency_container: &DependencyContainer) {
        dependency_container.resolve_all(Self::on_dependencies_resolved);
    }

    fn on_dependencies_resolved(
        dependency_container: DependencyContainer,
        (view_binding, engine_execution_context, audio_player, struct_viewer_view_model): (
            Arc<ViewBinding<MainWindowView>>,
            Arc<EngineExecutionContext>,
            Arc<AudioPlayer>,
            Arc<StructViewerViewModel>,
        ),
    ) {
        // Create a binding that allows us to easily update the view's scan results.
        let scan_results_collection = create_view_model_collection!(
            view_binding -> MainWindowView,
            ScanResultsViewModelBindings -> { set_scan_results, get_scan_results },
            ScanResultConverter -> [],
            ScanResultComparer -> [],
        );

        let current_page_index = Arc::new(AtomicU64::new(0));
        let cached_last_page_index = Arc::new(AtomicU64::new(0));
        let base_scan_results_collection = Arc::new(RwLock::new(vec![]));

        let view_model = Arc::new(ScanResultsViewModel {
            view_binding: view_binding.clone(),
            audio_player,
            base_scan_results_collection: base_scan_results_collection.clone(),
            scan_results_collection: scan_results_collection.clone(),
            engine_execution_context: engine_execution_context.clone(),
            current_page_index: current_page_index.clone(),
            cached_last_page_index: cached_last_page_index.clone(),
            struct_viewer_view_model,
            selection_index_start: Arc::new(AtomicI32::new(-1)),
            selection_index_end: Arc::new(AtomicI32::new(-1)),
        });

        {
            let view_model = view_model.clone();

            create_view_bindings!(view_binding, {
                ScanResultsViewModelBindings => {
                    on_navigate_first_page() -> [view_model] -> Self::on_navigate_first_page,
                    on_navigate_last_page() -> [view_model] -> Self::on_navigate_last_page,
                    on_navigate_previous_page() -> [view_model] -> Self::on_navigate_previous_page,
                    on_navigate_next_page() -> [view_model] -> Self::on_navigate_next_page,
                    on_page_index_text_changed(new_page_index_text: SharedString) -> [view_model] -> Self::on_page_index_text_changed,
                    on_set_scan_result_selection_start(local_scan_result_indices: i32) -> [view_model] -> Self::on_set_scan_result_selection_start,
                    on_set_scan_result_selection_end(local_scan_result_indices: i32) -> [view_model] -> Self::on_set_scan_result_selection_end,
                    on_add_scan_results_to_project() -> [view_model] -> Self::on_add_scan_results_to_project,
                    on_delete_selected_scan_results() -> [view_model] -> Self::on_delete_selected_scan_results,
                    on_set_scan_result_frozen(local_scan_result_index: i32, is_frozen: bool) -> [view_model] -> Self::on_set_scan_result_frozen,
                    on_toggle_selected_scan_results_frozen() -> [view_model] -> Self::on_toggle_selected_scan_results_frozen,
                },
            });
        }

        Self::poll_scan_results(view_model.clone());

        dependency_container.register::<ScanResultsViewModel>(view_model);
    }

    pub fn set_selected_scan_results_value(
        &self,
        field_namespace: String,
        data_value: DataValue,
    ) {
        let scan_results = self
            .base_scan_results_collection
            .read()
            .map(|collection| {
                collection
                    .iter()
                    .map(|scan_result| scan_result.get_base_result().clone())
                    .collect()
            })
            .unwrap_or_default();

        let scan_results_set_property_request = ScanResultsSetPropertyRequest {
            scan_results,
            field_namespace,
            data_value,
        };

        scan_results_set_property_request.send(&self.engine_execution_context, move |scan_results_set_property_response| {});
    }

    fn poll_scan_results(view_model: Arc<ScanResultsViewModel>) {
        let engine_execution_context = &view_model.engine_execution_context;

        // Requery all scan results if they update.
        {
            let view_model = view_model.clone();

            engine_execution_context.listen_for_engine_event::<ScanResultsUpdatedEvent>(move |scan_results_updated_event| {
                let play_sound = !scan_results_updated_event.is_new_scan;
                Self::query_scan_results(view_model.clone(), play_sound);
            });
        }

        // Refresh scan values on a loop. JIRA: This should be coming from settings. We can probably cache, and have some mechanism for getting latest val.
        thread::spawn(move || {
            loop {
                Self::refresh_scan_results(view_model.clone());

                thread::sleep(Duration::from_millis(100));
            }
        });
    }

    fn load_current_page_index(view_model: &Arc<ScanResultsViewModel>) -> u64 {
        let current_page_index = &view_model.current_page_index;
        let cached_last_page_index = &view_model.cached_last_page_index;

        current_page_index
            .load(Ordering::Relaxed)
            .clamp(0, cached_last_page_index.load(Ordering::Acquire))
    }

    fn query_scan_results(
        view_model: Arc<ScanResultsViewModel>,
        play_sound: bool,
    ) {
        let engine_execution_context = &view_model.engine_execution_context;
        let page_index = Self::load_current_page_index(&view_model);
        let scan_results_query_request = ScanResultsQueryRequest { page_index };
        let view_model = view_model.clone();

        scan_results_query_request.send(engine_execution_context, move |scan_results_query_response| {
            let view_binding = &view_model.view_binding;
            let cached_last_page_index = &view_model.cached_last_page_index;
            let base_scan_results_collection = &view_model.base_scan_results_collection;
            let view_model = view_model.clone();
            let result_count = scan_results_query_response.result_count;
            let byte_count = scan_results_query_response.total_size_in_bytes;

            cached_last_page_index.store(scan_results_query_response.last_page_index, Ordering::Release);

            if let Ok(mut base_scan_results_collection) = base_scan_results_collection.write() {
                *base_scan_results_collection = scan_results_query_response.scan_results;
            }

            view_binding.execute_on_ui_thread(move |main_window_view, _| {
                let audio_player = &view_model.audio_player;
                let scan_results_bindings = main_window_view.global::<ScanResultsViewModelBindings>();
                let byte_size_in_metric = Conversions::value_to_metric_size(byte_count);

                scan_results_bindings.set_result_statistics(format!("{} (Count: {})", byte_size_in_metric, result_count).into());

                if play_sound {
                    if result_count > 0 {
                        audio_player.play_sound(SoundType::Success);
                    } else {
                        audio_player.play_sound(SoundType::Warn);
                    }
                }

                Self::refresh_scan_results(view_model);
            });
        });
    }

    /// Fetches up-to-date values and module information for the current scan results, then updates the UI.
    fn refresh_scan_results(view_model: Arc<ScanResultsViewModel>) {
        let scan_results_collection = view_model.scan_results_collection.clone();
        let base_scan_results_collection = &view_model.base_scan_results_collection;
        let engine_execution_context = &view_model.engine_execution_context;

        // Gather the current/incomplete scan results.
        let scan_results_to_refresh = match base_scan_results_collection.read() {
            Ok(base_scan_results_collection) => base_scan_results_collection.clone(),
            Err(_) => vec![],
        };

        // Fire a request to get all scan result data needed for display.
        let scan_results_refresh_request = ScanResultsRefreshRequest {
            scan_results: scan_results_to_refresh
                .iter()
                .map(|scan_result| scan_result.get_valued_result().clone())
                .collect(),
        };

        scan_results_refresh_request.send(engine_execution_context, move |scan_results_refresh_response| {
            // Update UI with refreshed, full scan result values.
            scan_results_collection.update_from_source(scan_results_refresh_response.scan_results);
        });
    }

    fn set_page_index(
        view_model: Arc<ScanResultsViewModel>,
        new_page_index: u64,
    ) {
        let view_binding = view_model.view_binding.clone();
        let view_model_clone = view_model.clone();

        view_binding.execute_on_ui_thread(move |main_window_view, _| {
            let cached_last_page_index = &view_model.cached_last_page_index;
            let current_page_index = &view_model.current_page_index;
            let new_page_index = new_page_index.clamp(0, cached_last_page_index.load(Ordering::Acquire));

            let scan_results_bindings = main_window_view.global::<ScanResultsViewModelBindings>();
            // If the new index is the same as the current one, do nothing.
            if new_page_index == current_page_index.load(Ordering::Acquire) {
                return;
            }

            current_page_index.store(new_page_index, Ordering::Release);

            // Update the view binding with the cleaned numeric string.
            scan_results_bindings.set_current_page_index_string(SharedString::from(new_page_index.to_string()));
        });

        // Refresh scan results with the new page index. // JIRA: Should happen in the loop technically, but we need to make the MVVM bindings deadlock resistant.
        Self::query_scan_results(view_model_clone, false);
    }

    fn on_page_index_text_changed(
        view_model: Arc<ScanResultsViewModel>,
        new_page_index_text: SharedString,
    ) {
        // Extract numeric part from new_page_index_text and parse it to u64, defaulting to 0.
        let new_page_index = new_page_index_text
            .chars()
            .take_while(|char| char.is_digit(10))
            .collect::<String>()
            .parse::<u64>()
            .unwrap_or(0);

        Self::set_page_index(view_model, new_page_index);
    }

    fn on_navigate_first_page(view_model: Arc<ScanResultsViewModel>) {
        let new_page_index = 0;

        Self::set_page_index(view_model, new_page_index);
    }

    fn on_navigate_last_page(view_model: Arc<ScanResultsViewModel>) {
        let cached_last_page_index = &view_model.cached_last_page_index;
        let new_page_index = cached_last_page_index.load(Ordering::Acquire);

        Self::set_page_index(view_model, new_page_index);
    }

    fn on_navigate_previous_page(view_model: Arc<ScanResultsViewModel>) {
        let new_page_index = Self::load_current_page_index(&view_model).saturating_sub(1);

        Self::set_page_index(view_model, new_page_index);
    }

    fn on_navigate_next_page(view_model: Arc<ScanResultsViewModel>) {
        let new_page_index = Self::load_current_page_index(&view_model).saturating_add(1);

        Self::set_page_index(view_model, new_page_index);
    }

    fn on_set_scan_result_selection_start(
        view_model: Arc<ScanResultsViewModel>,
        scan_result_collection_start_index: i32,
    ) {
        view_model
            .selection_index_start
            .store(scan_result_collection_start_index, Ordering::Release);

        let struct_viewer_view_model = &view_model.struct_viewer_view_model;
        let scan_results = Self::collect_selected_scan_results(&view_model);

        if !scan_results.is_empty() {
            struct_viewer_view_model.set_selected_structs(
                StructViewerDomain::ScanResult,
                scan_results
                    .iter()
                    .map(|scan_result| scan_result.as_property_struct())
                    .collect(),
            );
        }
    }

    fn on_set_scan_result_selection_end(
        view_model: Arc<ScanResultsViewModel>,
        scan_result_collection_end_index: i32,
    ) {
        view_model
            .selection_index_end
            .store(scan_result_collection_end_index, Ordering::Release);

        let struct_viewer_view_model = &view_model.struct_viewer_view_model;
        let scan_results = Self::collect_selected_scan_results(&view_model);

        if !scan_results.is_empty() {
            struct_viewer_view_model.set_selected_structs(
                StructViewerDomain::ScanResult,
                scan_results
                    .iter()
                    .map(|scan_result| scan_result.as_property_struct())
                    .collect(),
            );
        }
    }

    fn on_add_scan_results_to_project(view_model: Arc<ScanResultsViewModel>) {
        let scan_results = Self::collect_selected_scan_result_bases(&view_model);

        if !scan_results.is_empty() {
            let engine_execution_context = &view_model.engine_execution_context;
            let scan_results_add_to_project_request = ScanResultsAddToProjectRequest { scan_results };

            scan_results_add_to_project_request.send(engine_execution_context, |_response| {});
        }
    }

    fn on_delete_selected_scan_results(view_model: Arc<ScanResultsViewModel>) {
        let scan_results = Self::collect_selected_scan_result_bases(&view_model);

        if !scan_results.is_empty() {
            let engine_execution_context = &view_model.engine_execution_context;
            let scan_results_delete_request = ScanResultsDeleteRequest { scan_results };

            scan_results_delete_request.send(engine_execution_context, |_response| {});
        }
    }

    fn on_set_scan_result_frozen(
        view_model: Arc<ScanResultsViewModel>,
        local_scan_result_index: i32,
        is_frozen: bool,
    ) {
        let local_scan_result_indices_vec = (local_scan_result_index..=local_scan_result_index).collect::<Vec<_>>();
        let scan_results = Self::collect_scan_result_bases_by_indicies(&view_model, &local_scan_result_indices_vec);

        if !scan_results.is_empty() {
            let engine_execution_context = &view_model.engine_execution_context;
            let scan_results_freeze_request = ScanResultsFreezeRequest { scan_results, is_frozen };

            scan_results_freeze_request.send(engine_execution_context, |_response| {});
        }
    }

    fn on_toggle_selected_scan_results_frozen(view_model: Arc<ScanResultsViewModel>) {
        /*
        let scan_results = Self::collect_selected_scan_result_bases(&view_model);

        if !scan_results.is_empty() {
            let engine_execution_context = &view_model.engine_execution_context;
            let scan_results_freeze_request = ScanResultsFreezeRequest { scan_results, is_frozen };

            scan_results_freeze_request.send(engine_execution_context, |_response| {});
        }*/
    }

    fn collect_selected_scan_result_bases(view_model: &Arc<ScanResultsViewModel>) -> Vec<ScanResultBase> {
        Self::collect_selected_scan_results(view_model)
            .into_iter()
            .map(|scan_result| scan_result.get_base_result().clone())
            .collect()
    }

    fn collect_selected_scan_results(view_model: &Arc<ScanResultsViewModel>) -> Vec<ScanResult> {
        let base_scan_results_collection = &view_model.base_scan_results_collection;
        let current_scan_results = match base_scan_results_collection.read() {
            Ok(base_scan_results_collection) => base_scan_results_collection.clone(),
            Err(_) => vec![],
        };
        let mut selection_index_start = view_model.selection_index_start.load(Ordering::Acquire);
        let mut selection_index_end = view_model.selection_index_end.load(Ordering::Acquire);

        // If either start or end is invalid, set the start and end to the same value (single selection).
        if selection_index_start < 0 && selection_index_end >= 0 {
            selection_index_start = selection_index_end;
        } else if selection_index_end < 0 && selection_index_start >= 0 {
            selection_index_end = selection_index_start;
        }

        // If both are invalid, return empty
        if selection_index_start < 0 || selection_index_end < 0 {
            return vec![];
        }

        let selection_index_start = cmp::min(selection_index_start, selection_index_end);
        let selection_index_end = cmp::max(selection_index_start, selection_index_end);

        let local_scan_result_indices = selection_index_start..=selection_index_end;
        local_scan_result_indices
            .filter_map(|index| current_scan_results.get(index as usize).cloned())
            .collect()
    }

    fn collect_scan_result_bases_by_indicies(
        view_model: &Arc<ScanResultsViewModel>,
        local_scan_result_indices: &[i32],
    ) -> Vec<ScanResultBase> {
        let base_scan_results_collection = &view_model.base_scan_results_collection;
        let current_scan_results = match base_scan_results_collection.read() {
            Ok(base_scan_results_collection) => base_scan_results_collection.clone(),
            Err(_) => vec![],
        };
        let scan_results = local_scan_result_indices
            .iter()
            .filter_map(|index| {
                current_scan_results
                    .get(*index as usize)
                    .map(|scan_result| scan_result.get_base_result().clone())
            })
            .collect();

        scan_results
    }
}
