use crate::MainWindowView;
use crate::ScanResultViewData;
use crate::ScanResultsViewModelBindings;
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
use squalr_engine_api::commands::scan_results::query::scan_results_query_request::ScanResultsQueryRequest;
use squalr_engine_api::commands::scan_results::refresh::scan_results_refresh_request::ScanResultsRefreshRequest;
use squalr_engine_api::structures::scan_results::scan_result::ScanResult;
use squalr_engine_api::structures::scan_results::scan_result_base::ScanResultBase;
use squalr_engine_common::conversions::Conversions;
use std::sync::Arc;
use std::sync::RwLock;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;
use std::thread;
use std::time::Duration;

pub struct ScanResultsViewModel {
    view_binding: ViewBinding<MainWindowView>,
    base_scan_results_collection: Arc<RwLock<Vec<ScanResultBase>>>,
    scan_results_collection: ViewCollectionBinding<ScanResultViewData, ScanResult, MainWindowView>,
    engine_execution_context: Arc<EngineExecutionContext>,
    current_page_index: Arc<AtomicU64>,
    cached_last_page_index: Arc<AtomicU64>,
}

impl ScanResultsViewModel {
    pub fn new(
        view_binding: ViewBinding<MainWindowView>,
        engine_execution_context: Arc<EngineExecutionContext>,
    ) -> Self {
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

        let view: ScanResultsViewModel = ScanResultsViewModel {
            view_binding: view_binding.clone(),
            base_scan_results_collection: base_scan_results_collection.clone(),
            scan_results_collection: scan_results_collection.clone(),
            engine_execution_context: engine_execution_context.clone(),
            current_page_index: current_page_index.clone(),
            cached_last_page_index: cached_last_page_index.clone(),
        };

        create_view_bindings!(view_binding, {
            ScanResultsViewModelBindings => {
                on_navigate_first_page() -> [view_binding, engine_execution_context, base_scan_results_collection, scan_results_collection, current_page_index, cached_last_page_index] -> Self::on_navigate_first_page,
                on_navigate_last_page() -> [view_binding, engine_execution_context, base_scan_results_collection, scan_results_collection, current_page_index, cached_last_page_index] -> Self::on_navigate_last_page,
                on_navigate_previous_page() -> [view_binding, engine_execution_context, base_scan_results_collection, scan_results_collection, current_page_index, cached_last_page_index] -> Self::on_navigate_previous_page,
                on_navigate_next_page() -> [view_binding, engine_execution_context, base_scan_results_collection, scan_results_collection, current_page_index, cached_last_page_index] -> Self::on_navigate_next_page,
                on_add_result_range(start_index: i32, end_index: i32) -> [] -> Self::on_add_result_range,
                on_page_index_text_changed(new_page_index_text: SharedString) -> [view_binding, engine_execution_context, base_scan_results_collection, scan_results_collection, current_page_index, cached_last_page_index] -> Self::on_page_index_text_changed,
            },
        });

        view.poll_scan_results();

        view
    }

    fn poll_scan_results(&self) {
        let view_binding = self.view_binding.clone();
        let base_scan_results_collection = self.base_scan_results_collection.clone();
        let scan_results_collection = self.scan_results_collection.clone();
        let engine_execution_context = self.engine_execution_context.clone();
        let current_page_index = self.current_page_index.clone();
        let cached_last_page_index = self.cached_last_page_index.clone();

        thread::spawn(move || {
            loop {
                let has_scan_results = match base_scan_results_collection.read() {
                    Ok(base_scan_results_collection) => base_scan_results_collection.len() > 0,
                    Err(_) => false,
                };

                if has_scan_results {
                    Self::refresh_scan_results(
                        engine_execution_context.clone(),
                        base_scan_results_collection.clone(),
                        scan_results_collection.clone(),
                    );
                } else {
                    Self::query_scan_results(
                        view_binding.clone(),
                        engine_execution_context.clone(),
                        base_scan_results_collection.clone(),
                        scan_results_collection.clone(),
                        current_page_index.clone(),
                        cached_last_page_index.clone(),
                    );
                }
                thread::sleep(Duration::from_millis(100));
            }
        });
    }

    fn load_current_page_index(
        current_page_index: &Arc<AtomicU64>,
        cached_last_page_index: &Arc<AtomicU64>,
    ) -> u64 {
        current_page_index
            .load(Ordering::Relaxed)
            .clamp(0, cached_last_page_index.load(Ordering::Acquire))
    }

    fn query_scan_results(
        view_binding: ViewBinding<MainWindowView>,
        engine_execution_context: Arc<EngineExecutionContext>,
        base_scan_results_collection: Arc<RwLock<Vec<ScanResultBase>>>,
        scan_results_collection: ViewCollectionBinding<ScanResultViewData, ScanResult, MainWindowView>,
        current_page_index: Arc<AtomicU64>,
        cached_last_page_index: Arc<AtomicU64>,
    ) {
        let page_index = Self::load_current_page_index(&current_page_index, &cached_last_page_index);
        let scan_results_query_request = ScanResultsQueryRequest { page_index };
        let scan_results_collection = scan_results_collection.clone();
        let engine_execution_context_clone = engine_execution_context.clone();

        scan_results_query_request.send(&engine_execution_context, move |scan_results_query_response| {
            cached_last_page_index.store(scan_results_query_response.last_page_index, Ordering::Release);

            let result_count = scan_results_query_response.result_count;
            let byte_count = scan_results_query_response.total_size_in_bytes;

            if let Ok(mut base_scan_results_collection) = base_scan_results_collection.write() {
                *base_scan_results_collection = scan_results_query_response.scan_results;
            }

            view_binding.execute_on_ui_thread(move |main_window_view, _| {
                let scan_results_bindings = main_window_view.global::<ScanResultsViewModelBindings>();
                let byte_size_in_metric = Conversions::value_to_metric_size(byte_count);

                scan_results_bindings.set_result_statistics(format!("{} (Count: {})", byte_size_in_metric, result_count).into());

                Self::refresh_scan_results(
                    engine_execution_context_clone,
                    base_scan_results_collection.clone(),
                    scan_results_collection.clone(),
                );
            });
        });
    }

    /// Fetches up-to-date values and module information for the current scan results, then updates the UI.
    fn refresh_scan_results(
        engine_execution_context: Arc<EngineExecutionContext>,
        base_scan_results_collection: Arc<RwLock<Vec<ScanResultBase>>>,
        scan_results_collection: ViewCollectionBinding<ScanResultViewData, ScanResult, MainWindowView>,
    ) {
        // Gather the current/incomplete scan results.
        let scan_results_to_refresh = match base_scan_results_collection.read() {
            Ok(base_scan_results_collection) => base_scan_results_collection.clone(),
            Err(_) => vec![],
        };
        let scan_results_collection = scan_results_collection.clone();

        // Fire a request to get all scan result data needed for display.
        let scan_results_refresh_request = ScanResultsRefreshRequest {
            scan_results: scan_results_to_refresh,
        };

        scan_results_refresh_request.send(&engine_execution_context, move |scan_results_refresh_response| {
            // Update UI with refreshed, full scan result values.
            scan_results_collection.update_from_source(scan_results_refresh_response.scan_results);
        });
    }

    fn set_page_index(
        view_binding: ViewBinding<MainWindowView>,
        engine_execution_context: Arc<EngineExecutionContext>,
        base_scan_results_collection: Arc<RwLock<Vec<ScanResultBase>>>,
        scan_results_collection: ViewCollectionBinding<ScanResultViewData, ScanResult, MainWindowView>,
        current_page_index: Arc<AtomicU64>,
        cached_last_page_index: Arc<AtomicU64>,
        new_page_index: u64,
    ) {
        let new_page_index = new_page_index.clamp(0, cached_last_page_index.load(Ordering::Acquire));
        let view_binding_clone = view_binding.clone();
        let current_page_index_clone = current_page_index.clone();

        view_binding.execute_on_ui_thread(move |main_window_view, _| {
            let scan_results_bindings = main_window_view.global::<ScanResultsViewModelBindings>();
            // If the new index is the same as the current one, do nothing
            if new_page_index == current_page_index.load(Ordering::Acquire) {
                return;
            }

            current_page_index.store(new_page_index, Ordering::Release);

            // Update the view binding with the cleaned numeric string
            scan_results_bindings.set_current_page_index_string(SharedString::from(new_page_index.to_string()));
        });

        // Refresh scan results with the new page index
        Self::query_scan_results(
            view_binding_clone,
            engine_execution_context,
            base_scan_results_collection,
            scan_results_collection,
            current_page_index_clone,
            cached_last_page_index,
        );
    }

    fn on_add_result_range(
        start_index: i32,
        end_index: i32,
    ) {
    }

    fn on_page_index_text_changed(
        view_binding: ViewBinding<MainWindowView>,
        engine_execution_context: Arc<EngineExecutionContext>,
        base_scan_results_collection: Arc<RwLock<Vec<ScanResultBase>>>,
        scan_results_collection: ViewCollectionBinding<ScanResultViewData, ScanResult, MainWindowView>,
        current_page_index: Arc<AtomicU64>,
        cached_last_page_index: Arc<AtomicU64>,
        new_page_index_text: SharedString,
    ) {
        // Extract numeric part from new_page_index_text and parse it to u64, defaulting to 0.
        let new_page_index = new_page_index_text
            .chars()
            .take_while(|c| c.is_digit(10))
            .collect::<String>()
            .parse::<u64>()
            .unwrap_or(0);

        Self::set_page_index(
            view_binding,
            engine_execution_context,
            base_scan_results_collection,
            scan_results_collection,
            current_page_index,
            cached_last_page_index,
            new_page_index,
        );
    }

    fn on_navigate_first_page(
        view_binding: ViewBinding<MainWindowView>,
        engine_execution_context: Arc<EngineExecutionContext>,
        base_scan_results_collection: Arc<RwLock<Vec<ScanResultBase>>>,
        scan_results_collection: ViewCollectionBinding<ScanResultViewData, ScanResult, MainWindowView>,
        current_page_index: Arc<AtomicU64>,
        cached_last_page_index: Arc<AtomicU64>,
    ) {
        let new_page_index = 0;

        Self::set_page_index(
            view_binding,
            engine_execution_context,
            base_scan_results_collection,
            scan_results_collection,
            current_page_index,
            cached_last_page_index,
            new_page_index,
        );
    }

    fn on_navigate_last_page(
        view_binding: ViewBinding<MainWindowView>,
        engine_execution_context: Arc<EngineExecutionContext>,
        base_scan_results_collection: Arc<RwLock<Vec<ScanResultBase>>>,
        scan_results_collection: ViewCollectionBinding<ScanResultViewData, ScanResult, MainWindowView>,
        current_page_index: Arc<AtomicU64>,
        cached_last_page_index: Arc<AtomicU64>,
    ) {
        let new_page_index = cached_last_page_index.load(Ordering::Acquire);

        Self::set_page_index(
            view_binding,
            engine_execution_context,
            base_scan_results_collection,
            scan_results_collection,
            current_page_index,
            cached_last_page_index,
            new_page_index,
        );
    }

    fn on_navigate_previous_page(
        view_binding: ViewBinding<MainWindowView>,
        engine_execution_context: Arc<EngineExecutionContext>,
        base_scan_results_collection: Arc<RwLock<Vec<ScanResultBase>>>,
        scan_results_collection: ViewCollectionBinding<ScanResultViewData, ScanResult, MainWindowView>,
        current_page_index: Arc<AtomicU64>,
        cached_last_page_index: Arc<AtomicU64>,
    ) {
        let new_page_index = Self::load_current_page_index(&current_page_index, &cached_last_page_index).saturating_sub(1);

        Self::set_page_index(
            view_binding,
            engine_execution_context,
            base_scan_results_collection,
            scan_results_collection,
            current_page_index,
            cached_last_page_index,
            new_page_index,
        );
    }

    fn on_navigate_next_page(
        view_binding: ViewBinding<MainWindowView>,
        engine_execution_context: Arc<EngineExecutionContext>,
        base_scan_results_collection: Arc<RwLock<Vec<ScanResultBase>>>,
        scan_results_collection: ViewCollectionBinding<ScanResultViewData, ScanResult, MainWindowView>,
        current_page_index: Arc<AtomicU64>,
        cached_last_page_index: Arc<AtomicU64>,
    ) {
        let new_page_index = Self::load_current_page_index(&current_page_index, &cached_last_page_index).saturating_add(1);

        Self::set_page_index(
            view_binding,
            engine_execution_context,
            base_scan_results_collection,
            scan_results_collection,
            current_page_index,
            cached_last_page_index,
            new_page_index,
        );
    }
}
