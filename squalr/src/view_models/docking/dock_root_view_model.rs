use crate::DockRootViewModelBindings;
use crate::DockedWindowViewData;
use crate::MainWindowView;
use crate::WindowViewModelBindings;
use crate::models::docking::docked_window_node::DockedWindowNode;
use crate::models::docking::docking_layout::DockingLayout;
use crate::view_models::docking::dock_panel_comparer::DockPanelComparer;
use crate::view_models::docking::dock_panel_converter::DockPanelConverter;
use crate::view_models::docking::docked_window_view_model::DockedWindowViewModel;
use crate::view_models::output::output_view_model::OutputViewModel;
use crate::view_models::process_selector::process_selector_view_model::ProcessSelectorViewModel;
use crate::view_models::scanners::manual_scan_view_model::ManualScanViewModel;
use crate::view_models::settings::memory_settings_view_model::MemorySettingsViewModel;
use crate::view_models::settings::scan_settings_view_model::ScanSettingsViewModel;
use slint::ComponentHandle;
use slint::Model;
use slint::SharedString;
use slint_mvvm::view_binding::ViewBinding;
use slint_mvvm::view_collection_binding::ViewCollectionBinding;
use slint_mvvm::view_data_converter::ViewDataConverter;
use slint_mvvm_macros::create_view_bindings;
use slint_mvvm_macros::create_view_model_collection;
use std::borrow::BorrowMut;
use std::sync::Arc;
use std::sync::Mutex;

pub struct DockRootViewModel {
    view_binding: ViewBinding<MainWindowView>,
    _docking_layout: Arc<Mutex<DockingLayout>>,
    _dock_panel_collection: ViewCollectionBinding<DockedWindowViewData, DockedWindowNode, MainWindowView>,
    docked_window_view_model: Arc<DockedWindowViewModel>,
    manual_scan_view_model: Arc<ManualScanViewModel>,
    memory_settings_view_model: Arc<MemorySettingsViewModel>,
    output_view_model: Arc<OutputViewModel>,
    process_selector_view_model: Arc<ProcessSelectorViewModel>,
    scan_settings_view_model: Arc<ScanSettingsViewModel>,
}

impl DockRootViewModel {
    pub fn new(view_binding: ViewBinding<MainWindowView>) -> Self {
        let docking_layout = Arc::new(Mutex::new(DockingLayout::default()));

        // Create a binding that allows us to easily update the view's process list.
        let dock_panel_collection = create_view_model_collection!(
            view_binding -> MainWindowView,
            DockRootViewModelBindings -> { set_dock_panels, get_dock_panels },
            DockPanelConverter -> [docking_layout],
            DockPanelComparer -> [],
        );

        dock_panel_collection.update_from_source(docking_layout.lock().unwrap().get_all_nodes());

        let view: DockRootViewModel = DockRootViewModel {
            view_binding: view_binding.clone(),
            _docking_layout: docking_layout.clone(),
            _dock_panel_collection: dock_panel_collection.clone(),
            docked_window_view_model: Arc::new(DockedWindowViewModel::new(view_binding.clone(), docking_layout.clone())),
            manual_scan_view_model: Arc::new(ManualScanViewModel::new(view_binding.clone())),
            memory_settings_view_model: Arc::new(MemorySettingsViewModel::new(view_binding.clone())),
            output_view_model: Arc::new(OutputViewModel::new(view_binding.clone())),
            process_selector_view_model: Arc::new(ProcessSelectorViewModel::new(view_binding.clone())),
            scan_settings_view_model: Arc::new(ScanSettingsViewModel::new(view_binding.clone())),
        };

        create_view_bindings!(view_binding, {
            WindowViewModelBindings => {
                on_minimize() -> [view_binding] -> Self::on_minimize,
                on_maximize() -> [view_binding] -> Self::on_maximize,
                on_close() -> [] -> Self::on_close,
                on_double_clicked() -> [view_binding] -> Self::on_double_clicked,
                on_drag(delta_x: i32, delta_y: i32) -> [view_binding] -> Self::on_drag
            },
            DockRootViewModelBindings => {
                on_update_dock_root_size(width: f32, height: f32) -> [docking_layout, dock_panel_collection] -> Self::on_update_dock_root_size,
                on_update_dock_root_width(width: f32) -> [docking_layout, dock_panel_collection] -> Self::on_update_dock_root_width,
                on_update_dock_root_height(height: f32) -> [docking_layout, dock_panel_collection] -> Self::on_update_dock_root_height,
                on_get_docked_window_data(identifier: SharedString) -> [docking_layout, dock_panel_collection] -> Self::on_get_docked_window_data
            }
        });

        view
    }

    fn on_get_docked_window_data(
        docking_layout: Arc<Mutex<DockingLayout>>,
        dock_panel_collection: ViewCollectionBinding<DockedWindowViewData, DockedWindowNode, MainWindowView>,
        identifier: SharedString,
    ) -> DockedWindowViewData {
        let converter = DockPanelConverter::new(docking_layout.clone());
        let nodes = docking_layout.lock().unwrap().get_all_nodes();

        for node in nodes {
            if node.window_identifier == *identifier {
                return converter.convert_to_view_data(&node);
            }
        }

        DockedWindowViewData::default()
    }

    pub fn initialize(&self) {
        self.show();
    }

    pub fn show(&self) {
        if let Ok(handle) = self.view_binding.get_view_handle().lock() {
            if let Some(view) = handle.upgrade() {
                if let Err(err) = view.show() {
                    log::error!("Error showing the main window: {err}");
                }
            }
        }
    }

    pub fn hide(&self) {
        if let Ok(handle) = self.view_binding.get_view_handle().lock() {
            if let Some(view) = handle.upgrade() {
                if let Err(err) = view.hide() {
                    log::error!("Error hiding the main window: {err}");
                }
            }
        }
    }

    pub fn get_docked_window_view_model(&self) -> &Arc<DockedWindowViewModel> {
        &self.docked_window_view_model
    }

    pub fn get_manual_scan_view_model(&self) -> &Arc<ManualScanViewModel> {
        &self.manual_scan_view_model
    }

    pub fn get_memory_settings_view_model(&self) -> &Arc<MemorySettingsViewModel> {
        &self.memory_settings_view_model
    }

    pub fn get_output_view_model(&self) -> &Arc<OutputViewModel> {
        &self.output_view_model
    }

    pub fn get_process_selector_view_model(&self) -> &Arc<ProcessSelectorViewModel> {
        &self.process_selector_view_model
    }

    pub fn get_scan_settings_view_model(&self) -> &Arc<ScanSettingsViewModel> {
        &self.scan_settings_view_model
    }

    fn on_minimize(view_binding: ViewBinding<MainWindowView>) {
        view_binding.execute_on_ui_thread(move |main_window_view, _view_binding| {
            let window = main_window_view.window();
            window.set_minimized(true);
        });
    }

    fn on_maximize(view_binding: ViewBinding<MainWindowView>) {
        view_binding.execute_on_ui_thread(move |main_window_view, _view_binding| {
            let window = main_window_view.window();
            window.set_maximized(!window.is_maximized());
        });
    }

    fn on_close() {
        if let Err(e) = slint::quit_event_loop() {
            log::error!("Failed to quit event loop: {}", e);
        }
    }

    fn on_double_clicked(view_binding: ViewBinding<MainWindowView>) {
        view_binding.execute_on_ui_thread(move |main_window_view, _view_binding| {
            let window = main_window_view.window();
            window.set_maximized(!window.is_maximized());
        });
    }

    fn on_drag(
        view_binding: ViewBinding<MainWindowView>,
        delta_x: i32,
        delta_y: i32,
    ) {
        view_binding.execute_on_ui_thread(move |main_window_view, _view_binding| {
            let window = main_window_view.window();
            let mut position = window.position();
            position.x += delta_x;
            position.y += delta_y;
            window.set_position(position);
        });
    }

    fn on_update_dock_root_size(
        docking_layout: Arc<Mutex<DockingLayout>>,
        dock_panel_collection: ViewCollectionBinding<DockedWindowViewData, DockedWindowNode, MainWindowView>,
        width: f32,
        height: f32,
    ) -> f32 {
        docking_layout
            .lock()
            .unwrap()
            .borrow_mut()
            .set_available_size(width, height);
        dock_panel_collection.update_from_source(docking_layout.lock().unwrap().get_all_nodes());
        0.0
    }

    fn on_update_dock_root_width(
        docking_layout: Arc<Mutex<DockingLayout>>,
        dock_panel_collection: ViewCollectionBinding<DockedWindowViewData, DockedWindowNode, MainWindowView>,
        width: f32,
    ) {
        docking_layout
            .lock()
            .unwrap()
            .borrow_mut()
            .set_available_width(width);
        dock_panel_collection.update_from_source(docking_layout.lock().unwrap().get_all_nodes());
    }

    fn on_update_dock_root_height(
        docking_layout: Arc<Mutex<DockingLayout>>,
        dock_panel_collection: ViewCollectionBinding<DockedWindowViewData, DockedWindowNode, MainWindowView>,
        height: f32,
    ) {
        docking_layout
            .lock()
            .unwrap()
            .borrow_mut()
            .set_available_height(height);
        dock_panel_collection.update_from_source(docking_layout.lock().unwrap().get_all_nodes());
    }
}
