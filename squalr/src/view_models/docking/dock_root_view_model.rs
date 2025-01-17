use crate::DockRootViewModelBindings;
use crate::MainWindowView;
use crate::RedockTarget;
use crate::WindowViewModelBindings;
use crate::models::docking::docking_manager::DockingManager;
use crate::models::docking::hierarchy::dock_node::DockNode;
use crate::models::docking::hierarchy::types::dock_splitter_drag_direction::DockSplitterDragDirection;
use crate::models::docking::settings::dockable_window_settings::DockSettingsConfig;
use crate::models::docking::settings::dockable_window_settings::DockableWindowSettings;
use crate::view_models::docking::dock_target_converter::DocktargetConverter;
use crate::view_models::docking::dock_window_converter::DockWindowConverter;
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
    _docking_manager: Arc<RwLock<DockingManager>>,
    manual_scan_view_model: Arc<ManualScanViewModel>,
    memory_settings_view_model: Arc<MemorySettingsViewModel>,
    output_view_model: Arc<OutputViewModel>,
    process_selector_view_model: Arc<ProcessSelectorViewModel>,
    scan_settings_view_model: Arc<ScanSettingsViewModel>,
}

impl DockRootViewModel {
    pub fn new(view_binding: ViewBinding<MainWindowView>) -> Self {
        let main_dock_root = DockableWindowSettings::get_instance().get_dock_layout_settings();
        let docking_manager = Arc::new(RwLock::new(DockingManager::new(main_dock_root)));

        let view: DockRootViewModel = DockRootViewModel {
            view_binding: view_binding.clone(),
            _docking_manager: docking_manager.clone(),
            manual_scan_view_model: Arc::new(ManualScanViewModel::new(view_binding.clone())),
            memory_settings_view_model: Arc::new(MemorySettingsViewModel::new(view_binding.clone())),
            output_view_model: Arc::new(OutputViewModel::new(view_binding.clone())),
            process_selector_view_model: Arc::new(ProcessSelectorViewModel::new(view_binding.clone())),
            scan_settings_view_model: Arc::new(ScanSettingsViewModel::new(view_binding.clone())),
        };

        // Initialize the dock root size.
        let docking_manager_clone = docking_manager.clone();
        view_binding.execute_on_ui_thread(move |main_window_view, _| {
            if let Ok(mut docking_manager) = docking_manager_clone.write() {
                let dock_root_bindings = main_window_view.global::<DockRootViewModelBindings>();
                docking_manager.get_layout_mut().set_available_size(
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
                on_update_dock_root_size(width: f32, height: f32) -> [view_binding, docking_manager] -> Self::on_update_dock_root_size,
                on_update_dock_root_width(width: f32) -> [view_binding, docking_manager] -> Self::on_update_dock_root_width,
                on_update_dock_root_height(height: f32) -> [view_binding, docking_manager] -> Self::on_update_dock_root_height,
                on_update_active_tab_id(identifier: SharedString) -> [view_binding, docking_manager] -> Self::on_update_active_tab_id,
                on_get_tab_text(identifier: SharedString) -> [] -> Self::on_get_tab_text,
                on_try_redock_window(identifier: SharedString, target_identifier: SharedString, redock_target: RedockTarget) -> [view_binding, docking_manager] -> Self::on_try_redock_window,
                on_reset_layout() -> [view_binding, docking_manager] -> Self::on_reset_layout,
                on_hide(identifier: SharedString) -> [view_binding, docking_manager] -> Self::on_hide,
                on_drag_left(dockable_window_id: SharedString, delta_x: i32, delta_y: i32) -> [view_binding, docking_manager] -> Self::on_drag_left,
                on_drag_right(dockable_window_id: SharedString, delta_x: i32, delta_y: i32) -> [view_binding, docking_manager] -> Self::on_drag_right,
                on_drag_top(dockable_window_id: SharedString, delta_x: i32, delta_y: i32) -> [view_binding, docking_manager] -> Self::on_drag_top,
                on_drag_bottom(dockable_window_id: SharedString, delta_x: i32, delta_y: i32) -> [view_binding, docking_manager] -> Self::on_drag_bottom,
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
        docking_manager: Arc<RwLock<DockingManager>>,
        width: f32,
        height: f32,
    ) -> f32 {
        Self::mutate_layout(&view_binding, &docking_manager, false, move |docking_manager| {
            docking_manager
                .get_layout_mut()
                .set_available_size(width, height);
        });

        // Return 0 as part of a UI hack to get responsive UI resizing.
        0.0
    }

    fn on_update_dock_root_width(
        view_binding: ViewBinding<MainWindowView>,
        docking_manager: Arc<RwLock<DockingManager>>,
        width: f32,
    ) {
        Self::mutate_layout(&view_binding, &docking_manager, false, move |docking_manager| {
            docking_manager.get_layout_mut().set_available_width(width);
        });

        Self::propagate_layout(view_binding, docking_manager);
    }

    fn on_update_dock_root_height(
        view_binding: ViewBinding<MainWindowView>,
        docking_manager: Arc<RwLock<DockingManager>>,
        height: f32,
    ) {
        Self::mutate_layout(&view_binding, &docking_manager, false, move |docking_manager| {
            docking_manager.get_layout_mut().set_available_height(height);
        });

        Self::propagate_layout(view_binding, docking_manager);
    }

    fn on_update_active_tab_id(
        view_binding: ViewBinding<MainWindowView>,
        docking_manager: Arc<RwLock<DockingManager>>,
        identifier: SharedString,
    ) {
        Self::mutate_layout(&view_binding, &docking_manager, true, move |docking_manager| {
            docking_manager.select_tab_by_window_id(identifier.as_str());
        });
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

    fn on_try_redock_window(
        view_binding: ViewBinding<MainWindowView>,
        docking_manager: Arc<RwLock<DockingManager>>,
        identifier: SharedString,
        target_identifier: SharedString,
        redock_target: RedockTarget,
    ) {
        Self::mutate_layout(&view_binding, &docking_manager, true, move |docking_manager| {
            docking_manager.reparent_window(
                &identifier,
                &target_identifier,
                DocktargetConverter::new().convert_from_view_data(&redock_target),
            );
        });
    }

    fn on_reset_layout(
        view_binding: ViewBinding<MainWindowView>,
        docking_manager: Arc<RwLock<DockingManager>>,
    ) {
        Self::mutate_layout(&view_binding, &docking_manager, true, move |docking_manager| {
            docking_manager.set_root(DockSettingsConfig::get_default_layout());
        });
    }

    fn on_hide(
        view_binding: ViewBinding<MainWindowView>,
        docking_manager: Arc<RwLock<DockingManager>>,
        dockable_window_id: SharedString,
    ) {
        Self::mutate_layout(&view_binding, &docking_manager, true, move |docking_manager| {
            if let Some(node) = docking_manager.get_node_by_id_mut(&dockable_window_id) {
                node.set_visible(false);
            }
        });
    }

    fn on_drag_left(
        view_binding: ViewBinding<MainWindowView>,
        docking_manager: Arc<RwLock<DockingManager>>,
        dockable_window_id: SharedString,
        delta_x: i32,
        delta_y: i32,
    ) {
        Self::mutate_layout(&view_binding, &docking_manager, false, move |manager| {
            manager.adjust_window_size(dockable_window_id.as_str(), &DockSplitterDragDirection::Left, delta_x, delta_y);
        });
    }

    fn on_drag_right(
        view_binding: ViewBinding<MainWindowView>,
        docking_manager: Arc<RwLock<DockingManager>>,
        dockable_window_id: SharedString,
        delta_x: i32,
        delta_y: i32,
    ) {
        Self::mutate_layout(&view_binding, &docking_manager, false, move |manager| {
            manager.adjust_window_size(dockable_window_id.as_str(), &DockSplitterDragDirection::Right, delta_x, delta_y);
        });
    }

    fn on_drag_top(
        view_binding: ViewBinding<MainWindowView>,
        docking_manager: Arc<RwLock<DockingManager>>,
        dockable_window_id: SharedString,
        delta_x: i32,
        delta_y: i32,
    ) {
        Self::mutate_layout(&view_binding, &docking_manager, false, move |manager| {
            manager.adjust_window_size(dockable_window_id.as_str(), &DockSplitterDragDirection::Top, delta_x, delta_y);
        });
    }

    fn on_drag_bottom(
        view_binding: ViewBinding<MainWindowView>,
        docking_manager: Arc<RwLock<DockingManager>>,
        dockable_window_id: SharedString,
        delta_x: i32,
        delta_y: i32,
    ) {
        Self::mutate_layout(&view_binding, &docking_manager, false, move |manager| {
            manager.adjust_window_size(dockable_window_id.as_str(), &DockSplitterDragDirection::Bottom, delta_x, delta_y);
        });
    }

    fn mutate_layout<F>(
        view_binding: &ViewBinding<MainWindowView>,
        docking_manager: &Arc<RwLock<DockingManager>>,
        save_layout: bool,
        f: F,
    ) where
        F: FnOnce(&mut DockingManager),
    {
        let mut layout_guard = match docking_manager.write() {
            Ok(guard) => guard,
            Err(err) => {
                log::error!("Could not acquire docking_manager write lock: {err}");
                return;
            }
        };

        f(&mut layout_guard);

        // Optionally save changes.
        if save_layout {
            DockableWindowSettings::get_instance().set_dock_layout_settings(layout_guard.get_root());
        }

        drop(layout_guard);

        Self::propagate_layout(view_binding.clone(), docking_manager.clone());
    }

    fn propagate_layout(
        view_binding: ViewBinding<MainWindowView>,
        docking_manager: Arc<RwLock<DockingManager>>,
    ) {
        view_binding.execute_on_ui_thread(move |main_window_view, _view_binding| {
            // Resolve any potential malformed data before attempting to convert to renderable form.
            if let Ok(mut docking_manager) = docking_manager.write() {
                docking_manager.prepare_for_presentation();
            }

            let dock_root_bindings = main_window_view.global::<DockRootViewModelBindings>();
            let converter = DockWindowConverter::new(docking_manager.clone());

            // Acquire the read lock once for all operations.
            let layout_guard = match docking_manager.read() {
                Ok(guard) => guard,
                Err(e) => {
                    log::error!("Failed to acquire read lock on docking_manager: {}", e);
                    return;
                }
            };

            let identifiers = layout_guard.get_all_child_window_ids();
            let default = DockNode::default();

            for identifier in identifiers {
                let node = layout_guard.get_node_by_id(&identifier).unwrap_or(&default);
                let view_data = converter.convert_to_view_data(node);

                match identifier.as_str() {
                    "settings" => {
                        dock_root_bindings.set_settings_window(view_data);
                    }
                    "scan-results" => {
                        dock_root_bindings.set_scan_results_window(view_data);
                    }
                    "output" => {
                        dock_root_bindings.set_output_window(view_data);
                    }
                    "process-selector" => {
                        dock_root_bindings.set_process_selector_window(view_data);
                    }
                    "property-viewer" => {
                        dock_root_bindings.set_property_viewer_window(view_data);
                    }
                    "project-explorer" => {
                        dock_root_bindings.set_project_explorer_window(view_data);
                    }
                    _ => {
                        log::warn!("Unknown window identifier: {}", identifier);
                    }
                }
            }
        });
    }
}
