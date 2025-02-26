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
use squalr_engine::commands::engine_request::EngineRequest;
use squalr_engine::commands::scan_results::list::scan_results_list_request::ScanResultsListRequest;
use squalr_engine::engine_execution_context::EngineExecutionContext;
use squalr_engine_common::values::endian::Endian;
use squalr_engine_scanning::results::scan_result::ScanResult;
use std::sync::Arc;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;
use std::thread;
use std::time::Duration;

pub struct ScanResultsViewModel {
    _view_binding: ViewBinding<MainWindowView>,
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

        let view: ScanResultsViewModel = ScanResultsViewModel {
            _view_binding: view_binding.clone(),
            scan_results_collection: scan_results_collection.clone(),
            engine_execution_context: engine_execution_context.clone(),
            current_page_index: current_page_index.clone(),
            cached_last_page_index: cached_last_page_index.clone(),
        };

        create_view_bindings!(view_binding, {
            ScanResultsViewModelBindings => {
                on_navigate_first_page() -> [view_binding, engine_execution_context, scan_results_collection, current_page_index, cached_last_page_index] -> Self::on_navigate_first_page,
                on_navigate_last_page() -> [view_binding, engine_execution_context, scan_results_collection, current_page_index, cached_last_page_index] -> Self::on_navigate_last_page,
                on_navigate_previous_page() -> [view_binding, engine_execution_context, scan_results_collection, current_page_index, cached_last_page_index] -> Self::on_navigate_previous_page,
                on_navigate_next_page() -> [view_binding, engine_execution_context, scan_results_collection, current_page_index, cached_last_page_index] -> Self::on_navigate_next_page,
                on_add_result_range(start_index: i32, end_index: i32) -> [] -> Self::on_add_result_range,
                on_page_index_text_changed(new_page_index_text: SharedString) -> [view_binding, engine_execution_context, scan_results_collection, current_page_index, cached_last_page_index] -> Self::on_page_index_text_changed,
            },
        });

        view.poll_scan_results();

        view
    }

    fn poll_scan_results(&self) {
        let scan_results_collection = self.scan_results_collection.clone();
        let engine_execution_context = self.engine_execution_context.clone();
        let current_page_index = self.current_page_index.clone();
        let cached_last_page_index = self.cached_last_page_index.clone();

        thread::spawn(move || {
            loop {
                Self::refresh_scan_results(
                    engine_execution_context.clone(),
                    scan_results_collection.clone(),
                    current_page_index.clone(),
                    cached_last_page_index.clone(),
                );
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

    fn refresh_scan_results(
        engine_execution_context: Arc<EngineExecutionContext>,
        scan_results_collection: ViewCollectionBinding<ScanResultViewData, ScanResult, MainWindowView>,
        current_page_index: Arc<AtomicU64>,
        cached_last_page_index: Arc<AtomicU64>,
    ) {
        let page_index = Self::load_current_page_index(&current_page_index, &cached_last_page_index);
        let scan_results_list_request = ScanResultsListRequest {
            page_index,
            // TODO
            data_type: squalr_engine_common::values::data_type::DataType::I32(Endian::Little),
        };
        let scan_results_collection = scan_results_collection.clone();

        scan_results_list_request.send(&engine_execution_context, move |scan_results_list_response| {
            cached_last_page_index.store(scan_results_list_response.last_page_index, Ordering::Release);

            scan_results_collection.update_from_source(scan_results_list_response.scan_results);
        });
    }

    fn set_page_index(
        view_binding: ViewBinding<MainWindowView>,
        engine_execution_context: Arc<EngineExecutionContext>,
        scan_results_collection: ViewCollectionBinding<ScanResultViewData, ScanResult, MainWindowView>,
        current_page_index: Arc<AtomicU64>,
        cached_last_page_index: Arc<AtomicU64>,
        new_page_index: u64,
    ) {
        let new_page_index = new_page_index.clamp(0, cached_last_page_index.load(Ordering::Acquire));

        view_binding.execute_on_ui_thread(move |main_window_view, _| {
            let scan_results_bindings = main_window_view.global::<ScanResultsViewModelBindings>();
            // If the new index is the same as the current one, do nothing
            if new_page_index == current_page_index.load(Ordering::Acquire) {
                return;
            }

            current_page_index.store(new_page_index, Ordering::Release);

            // Update the view binding with the cleaned numeric string
            scan_results_bindings.set_current_page_index_string(SharedString::from(new_page_index.to_string()));

            // Refresh scan results with the new page index
            Self::refresh_scan_results(engine_execution_context, scan_results_collection, current_page_index, cached_last_page_index);
        });
    }

    fn on_add_result_range(
        start_index: i32,
        end_index: i32,
    ) {
    }

    fn on_page_index_text_changed(
        view_binding: ViewBinding<MainWindowView>,
        engine_execution_context: Arc<EngineExecutionContext>,
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
            scan_results_collection,
            current_page_index,
            cached_last_page_index,
            new_page_index,
        );
    }

    fn on_navigate_first_page(
        view_binding: ViewBinding<MainWindowView>,
        engine_execution_context: Arc<EngineExecutionContext>,
        scan_results_collection: ViewCollectionBinding<ScanResultViewData, ScanResult, MainWindowView>,
        current_page_index: Arc<AtomicU64>,
        cached_last_page_index: Arc<AtomicU64>,
    ) {
        let new_page_index = 0;

        Self::set_page_index(
            view_binding,
            engine_execution_context,
            scan_results_collection,
            current_page_index,
            cached_last_page_index,
            new_page_index,
        );
    }

    fn on_navigate_last_page(
        view_binding: ViewBinding<MainWindowView>,
        engine_execution_context: Arc<EngineExecutionContext>,
        scan_results_collection: ViewCollectionBinding<ScanResultViewData, ScanResult, MainWindowView>,
        current_page_index: Arc<AtomicU64>,
        cached_last_page_index: Arc<AtomicU64>,
    ) {
        let new_page_index = cached_last_page_index.load(Ordering::Acquire);

        Self::set_page_index(
            view_binding,
            engine_execution_context,
            scan_results_collection,
            current_page_index,
            cached_last_page_index,
            new_page_index,
        );
    }

    fn on_navigate_previous_page(
        view_binding: ViewBinding<MainWindowView>,
        engine_execution_context: Arc<EngineExecutionContext>,
        scan_results_collection: ViewCollectionBinding<ScanResultViewData, ScanResult, MainWindowView>,
        current_page_index: Arc<AtomicU64>,
        cached_last_page_index: Arc<AtomicU64>,
    ) {
        let new_page_index = Self::load_current_page_index(&current_page_index, &cached_last_page_index).saturating_sub(1);

        Self::set_page_index(
            view_binding,
            engine_execution_context,
            scan_results_collection,
            current_page_index,
            cached_last_page_index,
            new_page_index,
        );
    }

    fn on_navigate_next_page(
        view_binding: ViewBinding<MainWindowView>,
        engine_execution_context: Arc<EngineExecutionContext>,
        scan_results_collection: ViewCollectionBinding<ScanResultViewData, ScanResult, MainWindowView>,
        current_page_index: Arc<AtomicU64>,
        cached_last_page_index: Arc<AtomicU64>,
    ) {
        let new_page_index = Self::load_current_page_index(&current_page_index, &cached_last_page_index).saturating_add(1);

        Self::set_page_index(
            view_binding,
            engine_execution_context,
            scan_results_collection,
            current_page_index,
            cached_last_page_index,
            new_page_index,
        );
    }
}
