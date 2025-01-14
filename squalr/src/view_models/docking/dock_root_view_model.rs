use crate::DockRootViewModelBindings;
use crate::MainWindowView;
use crate::WindowViewModelBindings;
use crate::models::docking::layout::dock_node::DockNode;
use crate::models::docking::layout::docking_layout::DockingLayout;
use crate::view_models::docking::dock_panel_converter::DockPanelConverter;
use crate::view_models::output::output_view_model::OutputViewModel;
use crate::view_models::process_selector::process_selector_view_model::ProcessSelectorViewModel;
use crate::view_models::scanners::manual_scan_view_model::ManualScanViewModel;
use crate::view_models::settings::memory_settings_view_model::MemorySettingsViewModel;
use crate::view_models::settings::scan_settings_view_model::ScanSettingsViewModel;
use slint::ComponentHandle;
use slint::SharedString;
use slint_mvvm::view_binding::ViewBinding;
use slint_mvvm::view_data_converter::ViewDataConverter;
use slint_mvvm_macros::create_view_bindings;
use std::sync::Arc;
use std::sync::RwLock;

pub struct DockRootViewModel {
    view_binding: ViewBinding<MainWindowView>,
    _docking_layout: Arc<RwLock<DockingLayout>>,
    manual_scan_view_model: Arc<ManualScanViewModel>,
    memory_settings_view_model: Arc<MemorySettingsViewModel>,
    output_view_model: Arc<OutputViewModel>,
    process_selector_view_model: Arc<ProcessSelectorViewModel>,
    scan_settings_view_model: Arc<ScanSettingsViewModel>,
}

impl DockRootViewModel {
    pub fn new(view_binding: ViewBinding<MainWindowView>) -> Self {
        let docking_layout = Arc::new(RwLock::new(DockingLayout::from_settings()));

        let view: DockRootViewModel = DockRootViewModel {
            view_binding: view_binding.clone(),
            _docking_layout: docking_layout.clone(),
            manual_scan_view_model: Arc::new(ManualScanViewModel::new(view_binding.clone())),
            memory_settings_view_model: Arc::new(MemorySettingsViewModel::new(view_binding.clone())),
            output_view_model: Arc::new(OutputViewModel::new(view_binding.clone())),
            process_selector_view_model: Arc::new(ProcessSelectorViewModel::new(view_binding.clone())),
            scan_settings_view_model: Arc::new(ScanSettingsViewModel::new(view_binding.clone())),
        };

        // Initialize the dock root size
        let docking_layout_clone = docking_layout.clone();
        view_binding.execute_on_ui_thread(move |main_window_view, _| {
            if let Ok(mut docking_layout) = docking_layout_clone.write() {
                let dock_root_bindings = main_window_view.global::<DockRootViewModelBindings>();
                docking_layout.set_available_size(
                    dock_root_bindings.get_initial_dock_root_width(),
                    dock_root_bindings.get_initial_dock_root_height(),
                );
            }
        });

        create_view_bindings!(view_binding, {
            WindowViewModelBindings => {
                on_minimize() -> [view_binding] -> Self::on_minimize,
                on_maximize() -> [view_binding] -> Self::on_maximize,
                on_close() -> [] -> Self::on_close,
                on_double_clicked() -> [view_binding] -> Self::on_double_clicked,
                on_drag(delta_x: i32, delta_y: i32) -> [view_binding] -> Self::on_drag
            },
            DockRootViewModelBindings => {
                on_update_dock_root_size(width: f32, height: f32) -> [view_binding, docking_layout] -> Self::on_update_dock_root_size,
                on_update_dock_root_width(width: f32) -> [view_binding, docking_layout] -> Self::on_update_dock_root_width,
                on_update_dock_root_height(height: f32) -> [view_binding, docking_layout] -> Self::on_update_dock_root_height,
                on_update_active_tab_id(identifier: SharedString) -> [view_binding, docking_layout] -> Self::on_update_active_tab_id,
                on_get_tab_text(identifier: SharedString) -> [] -> Self::on_get_tab_text,
                on_hide(identifier: SharedString) -> [view_binding, docking_layout] -> Self::on_hide,
                on_drag_left(dockable_window_id: SharedString, delta_x: i32, delta_y: i32) -> [] -> Self::on_drag_left,
                on_drag_right(dockable_window_id: SharedString, delta_x: i32, delta_y: i32) -> [] -> Self::on_drag_right,
                on_drag_top(dockable_window_id: SharedString, delta_x: i32, delta_y: i32) -> [] -> Self::on_drag_top,
                on_drag_bottom(dockable_window_id: SharedString, delta_x: i32, delta_y: i32) -> [] -> Self::on_drag_bottom,
            }
        });

        view
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
        view_binding: ViewBinding<MainWindowView>,
        docking_layout: Arc<RwLock<DockingLayout>>,
        width: f32,
        height: f32,
    ) -> f32 {
        if let Ok(mut layout_guard) = docking_layout.write() {
            layout_guard.set_available_size(width, height);
        } else {
            log::error!("Could not acquire docking_layout write lock in on_update_dock_root_size");
        }

        Self::propagate_layout(view_binding, docking_layout);

        // Return 0 as part of a UI hack to get responsive UI resizing.
        0.0
    }

    fn on_update_dock_root_width(
        view_binding: ViewBinding<MainWindowView>,
        docking_layout: Arc<RwLock<DockingLayout>>,
        width: f32,
    ) {
        if let Ok(mut layout_guard) = docking_layout.write() {
            layout_guard.set_available_width(width);
        } else {
            log::error!("Could not acquire docking_layout write lock in on_update_dock_root_width");
            return;
        }

        Self::propagate_layout(view_binding, docking_layout);
    }

    fn on_update_dock_root_height(
        view_binding: ViewBinding<MainWindowView>,
        docking_layout: Arc<RwLock<DockingLayout>>,
        height: f32,
    ) {
        if let Ok(mut layout_guard) = docking_layout.write() {
            layout_guard.set_available_height(height);
        } else {
            log::error!("Could not acquire docking_layout write lock in on_update_dock_root_height");
            return;
        }

        Self::propagate_layout(view_binding, docking_layout);
    }

    fn on_update_active_tab_id(
        view_binding: ViewBinding<MainWindowView>,
        docking_layout: Arc<RwLock<DockingLayout>>,
        identifier: SharedString,
    ) {
        if let Ok(mut docking_layout) = docking_layout.write() {
            docking_layout.select_tab_by_leaf_id(identifier.as_str());
        }
        Self::propagate_layout(view_binding, docking_layout);
    }

    fn on_get_tab_text(identifier: SharedString) -> SharedString {
        match identifier.as_str() {
            "settings" => "Settings".into(),
            "scan-results" => "Scan Results".into(),
            "output" => "Output".into(),
            "process-selector" => "Process Selector".into(),
            "property-viewer" => "Property Viewer".into(),
            "project-explorer" => "Project Explorer".into(),
            _ => identifier,
        }
    }

    fn on_hide(
        view_binding: ViewBinding<MainWindowView>,
        docking_layout: Arc<RwLock<DockingLayout>>,
        dockable_window_id: SharedString,
    ) {
        if let Ok(mut docking_layout) = docking_layout.write() {
            if let Some(node) = docking_layout.get_node_by_id_mut(&dockable_window_id) {
                node.set_visible(false);
            }
        }
        Self::propagate_layout(view_binding, docking_layout);
    }

    fn on_drag_left(
        _dockable_window_id: SharedString,
        _delta_x: i32,
        _delta_y: i32,
    ) {
        // TODO: Implement me.
    }

    fn on_drag_right(
        _dockable_window_id: SharedString,
        _delta_x: i32,
        _delta_y: i32,
    ) {
        // TODO: Implement me.
    }

    fn on_drag_top(
        _dockable_window_id: SharedString,
        _delta_x: i32,
        _delta_y: i32,
    ) {
        // TODO: Implement me.
    }

    fn on_drag_bottom(
        _dockable_window_id: SharedString,
        _delta_x: i32,
        _delta_y: i32,
    ) {
        // TODO: Implement me.
    }

    fn propagate_layout(
        view_binding: ViewBinding<MainWindowView>,
        docking_layout: Arc<RwLock<DockingLayout>>,
    ) {
        view_binding.execute_on_ui_thread(move |main_window_view, _view_binding| {
            // Resolve any potential malformed data before attempting to convert to renderable form.
            if let Ok(mut docking_layout) = docking_layout.write() {
                docking_layout.prepare_for_presentation();
            }

            let dock_root_bindings = main_window_view.global::<DockRootViewModelBindings>();
            let converter = DockPanelConverter::new(docking_layout.clone());

            // Acquire the read lock once for all operations.
            let layout_guard = match docking_layout.read() {
                Ok(guard) => guard,
                Err(e) => {
                    log::error!("Failed to acquire read lock on docking_layout: {}", e);
                    return;
                }
            };

            let identifiers = layout_guard.get_all_leaves();
            let default = DockNode::default();

            for identifier in identifiers {
                let node = layout_guard.get_node_by_id(&identifier).unwrap_or(&default);
                let view_data = converter.convert_to_view_data(node);

                match identifier.as_str() {
                    "settings" => {
                        dock_root_bindings.set_settings_panel(view_data);
                    }
                    "scan-results" => {
                        dock_root_bindings.set_scan_results_panel(view_data);
                    }
                    "output" => {
                        dock_root_bindings.set_output_panel(view_data);
                    }
                    "process-selector" => {
                        dock_root_bindings.set_process_selector_panel(view_data);
                    }
                    "property-viewer" => {
                        dock_root_bindings.set_property_viewer_panel(view_data);
                    }
                    "project-explorer" => {
                        dock_root_bindings.set_project_explorer_panel(view_data);
                    }
                    _ => {
                        log::warn!("Unknown window identifier: {}", identifier);
                    }
                }
            }
        });
    }
}
