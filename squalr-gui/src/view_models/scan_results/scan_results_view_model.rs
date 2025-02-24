use crate::MainWindowView;
use crate::ScanResultDataView;
use crate::ScanResultsViewModelBindings;
use crate::view_models::scan_results::scan_result_comparer::ScanResultComparer;
use crate::view_models::scan_results::scan_result_converter::ScanResultConverter;
use slint::ComponentHandle;
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
use std::thread;
use std::time::Duration;

pub struct ScanResultsViewModel {
    view_binding: ViewBinding<MainWindowView>,
    scan_results_collection: ViewCollectionBinding<ScanResultDataView, ScanResult, MainWindowView>,
    engine_execution_context: Arc<EngineExecutionContext>,
    current_page_index: Arc<u64>,
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

        let view: ScanResultsViewModel = ScanResultsViewModel {
            view_binding: view_binding.clone(),
            scan_results_collection: scan_results_collection,
            engine_execution_context: engine_execution_context.clone(),
            current_page_index: Arc::new(0),
        };

        create_view_bindings!(view_binding, {
            ScanResultsViewModelBindings => {
                on_navigate_first_page() -> [] -> Self::on_navigate_first_page,
                on_navigate_last_page() -> [] -> Self::on_navigate_last_page,
                on_navigate_previous_page() -> [] -> Self::on_navigate_previous_page,
                on_navigate_next_page() -> [] -> Self::on_navigate_next_page,
                on_add_result_range(start_index: i32, end_index: i32) -> [] -> Self::on_add_result_range,
            },
        });

        view.poll_scan_results();

        view
    }

    fn poll_scan_results(&self) {
        let view_binding = self.view_binding.clone();
        let scan_results_collection = self.scan_results_collection.clone();
        let engine_execution_context = self.engine_execution_context.clone();
        let current_page_index = self.current_page_index.clone();

        thread::spawn(move || {
            loop {
                let scan_results_list_request = ScanResultsListRequest {
                    page_index: *current_page_index,
                    // TODO
                    data_type: squalr_engine_common::values::data_type::DataType::I32(Endian::Big),
                };
                let scan_results_collection = scan_results_collection.clone();

                scan_results_list_request.send(&engine_execution_context, move |scan_results_list_response| {
                    scan_results_collection.update_from_source(scan_results_list_response.scan_results);
                });

                thread::sleep(Duration::from_millis(100));
            }
        });
    }

    fn on_add_result_range(
        start_index: i32,
        end_index: i32,
    ) {
    }

    fn on_navigate_first_page() {
        //
    }

    fn on_navigate_last_page() {
        //
    }

    fn on_navigate_previous_page() {
        //
    }

    fn on_navigate_next_page() {
        //
    }
}
