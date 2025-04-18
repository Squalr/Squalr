use crate::MainWindowView;
use crate::ScanResultViewData;
use crate::ScanResultsViewModelBindings;
use crate::models::audio::audio_player::AudioPlayer;
use crate::models::audio::audio_player::SoundType;
use crate::view_models::scan_results::scan_result_comparer::ScanResultComparer;
use crate::view_models::scan_results::scan_result_converter::ScanResultConverter;
use slint::ComponentHandle;
use slint::SharedString;
use slint_mvvm::view_binding::ViewBinding;
use slint_mvvm::view_collection_binding::ViewCollectionBinding;
use slint_mvvm_macros::create_view_bindings;
use slint_mvvm_macros::create_view_model_collection;
use squalr_engine::command_executors::engine_request_executor::EngineCommandRequestExecutor;
use squalr_engine::engine_execution_context::EngineExecutionContext;
use squalr_engine_api::commands::scan_results::freeze::scan_results_freeze_request::ScanResultsFreezeRequest;
use squalr_engine_api::commands::scan_results::query::scan_results_query_request::ScanResultsQueryRequest;
use squalr_engine_api::commands::scan_results::refresh::scan_results_refresh_request::ScanResultsRefreshRequest;
use squalr_engine_api::events::scan_results::updated::scan_results_updated_event::ScanResultsUpdatedEvent;
use squalr_engine_api::structures::scan_results::scan_result::ScanResult;
use squalr_engine_api::structures::scan_results::scan_result_valued::ScanResultValued;
use squalr_engine_common::conversions::Conversions;
use std::sync::Arc;
use std::sync::RwLock;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;
use std::thread;
use std::time::Duration;

pub struct ScanResultsViewModel {
    view_binding: ViewBinding<MainWindowView>,
    audio_player: Arc<AudioPlayer>,
    base_scan_results_collection: Arc<RwLock<Vec<ScanResultValued>>>,
    scan_results_collection: ViewCollectionBinding<ScanResultViewData, ScanResult, MainWindowView>,
    engine_execution_context: Arc<EngineExecutionContext>,
    current_page_index: Arc<AtomicU64>,
    cached_last_page_index: Arc<AtomicU64>,
}

impl ScanResultsViewModel {
    pub fn new(
        view_binding: ViewBinding<MainWindowView>,
        audio_player: Arc<AudioPlayer>,
        engine_execution_context: Arc<EngineExecutionContext>,
    ) -> Arc<Self> {
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

        let view = Arc::new(ScanResultsViewModel {
            view_binding: view_binding.clone(),
            audio_player: audio_player.clone(),
            base_scan_results_collection: base_scan_results_collection.clone(),
            scan_results_collection: scan_results_collection.clone(),
            engine_execution_context: engine_execution_context.clone(),
            current_page_index: current_page_index.clone(),
            cached_last_page_index: cached_last_page_index.clone(),
        });

        {
            let view = view.clone();

            create_view_bindings!(view_binding, {
                ScanResultsViewModelBindings => {
                    on_navigate_first_page() -> [view] -> Self::on_navigate_first_page,
                    on_navigate_last_page() -> [view] -> Self::on_navigate_last_page,
                    on_navigate_previous_page() -> [view] -> Self::on_navigate_previous_page,
                    on_navigate_next_page() -> [view] -> Self::on_navigate_next_page,
                    on_add_result_range(local_start_index: i32, local_end_index: i32) -> [] -> Self::on_add_result_range,
                    on_page_index_text_changed(new_page_index_text: SharedString) -> [view] -> Self::on_page_index_text_changed,
                    on_delete_scan_result(local_scan_result_index: i32) -> [engine_execution_context] -> Self::on_delete_scan_result,
                    on_set_scan_result_frozen(local_scan_result_index: i32, is_frozen: bool) -> [engine_execution_context, base_scan_results_collection] -> Self::on_set_scan_result_frozen,
                },
            });
        }

        Self::poll_scan_results(view.clone());

        view
    }

    fn poll_scan_results(scan_results_view_model: Arc<ScanResultsViewModel>) {
        let engine_execution_context = &scan_results_view_model.engine_execution_context;

        // Requery all scan results if they update.
        {
            let scan_results_view_model = scan_results_view_model.clone();

            engine_execution_context.listen_for_engine_event::<ScanResultsUpdatedEvent>(move |scan_results_updated_event| {
                let play_sound = !scan_results_updated_event.is_new_scan;
                Self::query_scan_results(scan_results_view_model.clone(), play_sound);
            });
        }

        // Refresh scan values on a loop. JIRA: This should be coming from settings. We can probably cache, and have some mechanism for getting latest val.
        thread::spawn(move || {
            loop {
                Self::refresh_scan_results(scan_results_view_model.clone());

                thread::sleep(Duration::from_millis(100));
            }
        });
    }

    fn load_current_page_index(scan_results_view_model: &Arc<ScanResultsViewModel>) -> u64 {
        let current_page_index = &scan_results_view_model.current_page_index;
        let cached_last_page_index = &scan_results_view_model.cached_last_page_index;

        current_page_index
            .load(Ordering::Relaxed)
            .clamp(0, cached_last_page_index.load(Ordering::Acquire))
    }

    fn query_scan_results(
        scan_results_view_model: Arc<ScanResultsViewModel>,
        play_sound: bool,
    ) {
        let engine_execution_context = &scan_results_view_model.engine_execution_context;
        let page_index = Self::load_current_page_index(&scan_results_view_model);
        let scan_results_query_request = ScanResultsQueryRequest { page_index };
        let scan_results_view_model = scan_results_view_model.clone();

        scan_results_query_request.send(engine_execution_context, move |scan_results_query_response| {
            let view_binding = &scan_results_view_model.view_binding;
            let cached_last_page_index = &scan_results_view_model.cached_last_page_index;
            let base_scan_results_collection = &scan_results_view_model.base_scan_results_collection;
            let scan_results_view_model = scan_results_view_model.clone();
            let result_count = scan_results_query_response.result_count;
            let byte_count = scan_results_query_response.total_size_in_bytes;

            cached_last_page_index.store(scan_results_query_response.last_page_index, Ordering::Release);

            if let Ok(mut base_scan_results_collection) = base_scan_results_collection.write() {
                *base_scan_results_collection = scan_results_query_response.scan_results;
            }

            view_binding.execute_on_ui_thread(move |main_window_view, _| {
                let audio_player = &scan_results_view_model.audio_player;
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

                Self::refresh_scan_results(scan_results_view_model);
            });
        });
    }

    /// Fetches up-to-date values and module information for the current scan results, then updates the UI.
    fn refresh_scan_results(scan_results_view_model: Arc<ScanResultsViewModel>) {
        let scan_results_collection = scan_results_view_model.scan_results_collection.clone();
        let base_scan_results_collection = &scan_results_view_model.base_scan_results_collection;
        let engine_execution_context = &scan_results_view_model.engine_execution_context;

        // Gather the current/incomplete scan results.
        let scan_results_to_refresh = match base_scan_results_collection.read() {
            Ok(base_scan_results_collection) => base_scan_results_collection.clone(),
            Err(_) => vec![],
        };

        // Fire a request to get all scan result data needed for display.
        let scan_results_refresh_request = ScanResultsRefreshRequest {
            scan_results: scan_results_to_refresh,
        };

        scan_results_refresh_request.send(engine_execution_context, move |scan_results_refresh_response| {
            // Update UI with refreshed, full scan result values.
            scan_results_collection.update_from_source(scan_results_refresh_response.scan_results);
        });
    }

    fn set_page_index(
        scan_results_view_model: Arc<ScanResultsViewModel>,
        new_page_index: u64,
    ) {
        let view_binding = scan_results_view_model.view_binding.clone();
        let scan_results_view_model_clone = scan_results_view_model.clone();

        view_binding.execute_on_ui_thread(move |main_window_view, _| {
            let cached_last_page_index = &scan_results_view_model.cached_last_page_index;
            let current_page_index = &scan_results_view_model.current_page_index;
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
        Self::query_scan_results(scan_results_view_model_clone, false);
    }

    fn on_add_result_range(
        start_index: i32,
        end_index: i32,
    ) {
    }

    fn on_page_index_text_changed(
        scan_results_view_model: Arc<ScanResultsViewModel>,
        new_page_index_text: SharedString,
    ) {
        // Extract numeric part from new_page_index_text and parse it to u64, defaulting to 0.
        let new_page_index = new_page_index_text
            .chars()
            .take_while(|c| c.is_digit(10))
            .collect::<String>()
            .parse::<u64>()
            .unwrap_or(0);

        Self::set_page_index(scan_results_view_model, new_page_index);
    }

    fn on_navigate_first_page(scan_results_view_model: Arc<ScanResultsViewModel>) {
        let new_page_index = 0;

        Self::set_page_index(scan_results_view_model, new_page_index);
    }

    fn on_navigate_last_page(scan_results_view_model: Arc<ScanResultsViewModel>) {
        let cached_last_page_index = &scan_results_view_model.cached_last_page_index;
        let new_page_index = cached_last_page_index.load(Ordering::Acquire);

        Self::set_page_index(scan_results_view_model, new_page_index);
    }

    fn on_navigate_previous_page(scan_results_view_model: Arc<ScanResultsViewModel>) {
        let new_page_index = Self::load_current_page_index(&scan_results_view_model).saturating_sub(1);

        Self::set_page_index(scan_results_view_model, new_page_index);
    }

    fn on_navigate_next_page(scan_results_view_model: Arc<ScanResultsViewModel>) {
        let new_page_index = Self::load_current_page_index(&scan_results_view_model).saturating_add(1);

        Self::set_page_index(scan_results_view_model, new_page_index);
    }

    fn on_delete_scan_result(
        engine_execution_context: Arc<EngineExecutionContext>,
        scan_result_index: i32,
    ) {
    }

    fn on_set_scan_result_frozen(
        engine_execution_context: Arc<EngineExecutionContext>,
        base_scan_results_collection: Arc<RwLock<Vec<ScanResultValued>>>,
        local_scan_result_index: i32,
        is_frozen: bool,
    ) {
        // Gather the current/incomplete scan results.
        let scan_results_to_refresh = match base_scan_results_collection.read() {
            Ok(base_scan_results_collection) => base_scan_results_collection.clone(),
            Err(_) => vec![],
        };
        if let Some(scan_result) = scan_results_to_refresh
            .get(local_scan_result_index as usize)
            .map(|scan_result| scan_result.get_scan_result_base().clone())
        {
            let scan_results_freeze_request = ScanResultsFreezeRequest { scan_result, is_frozen };

            scan_results_freeze_request.send(&engine_execution_context, |_scan_results_freeze_response| {});
        }
    }
}
