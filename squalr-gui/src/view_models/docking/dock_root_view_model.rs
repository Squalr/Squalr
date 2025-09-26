use crate::DockRootViewModelBindings;
use crate::MainWindowView;
use crate::RedockTarget;
use crate::WindowViewModelBindings;
use crate::converters::dock_target_converter::DocktargetConverter;
use crate::converters::dock_window_converter::DockWindowConverter;
use crate::models::docking::docking_manager::DockingManager;
use crate::models::docking::hierarchy::dock_node::DockNode;
use crate::models::docking::hierarchy::types::dock_splitter_drag_direction::DockSplitterDragDirection;
use crate::models::docking::settings::dockable_window_settings::DockSettingsConfig;
use crate::models::docking::settings::dockable_window_settings::DockableWindowSettings;
use squalr_engine_api::dependency_injection::dependency_container::DependencyContainer;
use squalr_engine_api::engine::engine_execution_context::EngineExecutionContext;
use slint::ComponentHandle;
use slint::SharedString;
use slint_mvvm::convert_from_view_data::ConvertFromViewData;
use slint_mvvm::convert_to_view_data::ConvertToViewData;
use slint_mvvm::view_binding::ViewBinding;
use slint_mvvm_macros::create_view_bindings;
use std::sync::Arc;
use std::sync::RwLock;

pub struct DockRootViewModel {
    view_binding: Arc<ViewBinding<MainWindowView>>,
    _engine_execution_context: Arc<EngineExecutionContext>,
    docking_manager: Arc<RwLock<DockingManager>>,
}

impl DockRootViewModel {
    pub fn register(dependency_container: &DependencyContainer) {
        dependency_container.resolve_all(Self::on_dependencies_resolved);
    }

    fn on_dependencies_resolved(
        dependency_container: DependencyContainer,
        (view_binding, engine_execution_context): (Arc<ViewBinding<MainWindowView>>, Arc<EngineExecutionContext>),
    ) {
        let main_dock_root = DockableWindowSettings::get_dock_layout_settings();
        let docking_manager = Arc::new(RwLock::new(DockingManager::new(main_dock_root)));

        let view_model = Arc::new(DockRootViewModel {
            view_binding: view_binding.clone(),
            _engine_execution_context: engine_execution_context,
            docking_manager: docking_manager.clone(),
        });

        {
            let view_model = view_model.clone();

            create_view_bindings!(view_binding, {
                WindowViewModelBindings => {
                    on_minimize() -> [view_model] -> Self::on_minimize,
                    on_maximize() -> [view_model] -> Self::on_maximize,
                    on_close() -> [] -> Self::on_close,
                    on_double_clicked() -> [view_model] -> Self::on_double_clicked,
                    on_drag(delta_x: i32, delta_y: i32) -> [view_model] -> Self::on_drag
                },
                DockRootViewModelBindings => {
                    on_update_dock_root_size(width: f32, height: f32) -> [view_model] -> Self::on_update_dock_root_size,
                    on_update_dock_root_width(width: f32) -> [view_model] -> Self::on_update_dock_root_width,
                    on_update_dock_root_height(height: f32) -> [view_model] -> Self::on_update_dock_root_height,
                    on_update_active_tab_id(identifier: SharedString) -> [view_model] -> Self::on_update_active_tab_id,
                    on_get_tab_text(identifier: SharedString) -> [] -> Self::on_get_tab_text,
                    on_is_window_visible(identifier: SharedString) -> [view_model] -> Self::on_is_window_visible,
                    on_try_redock_window(identifier: SharedString, target_identifier: SharedString, redock_target: RedockTarget) -> [view_model] -> Self::on_try_redock_window,
                    on_reset_layout() -> [view_model] -> Self::on_reset_layout,
                    on_show(identifier: SharedString) -> [view_model] -> Self::on_show,
                    on_hide(identifier: SharedString) -> [view_model] -> Self::on_hide,
                    on_toggle_visibility(identifier: SharedString) -> [view_model] -> Self::on_toggle_visibility,
                    on_drag_left(dockable_window_id: SharedString, delta_x: i32, delta_y: i32) -> [view_model] -> Self::on_drag_left,
                    on_drag_right(dockable_window_id: SharedString, delta_x: i32, delta_y: i32) -> [view_model] -> Self::on_drag_right,
                    on_drag_top(dockable_window_id: SharedString, delta_x: i32, delta_y: i32) -> [view_model] -> Self::on_drag_top,
                    on_drag_bottom(dockable_window_id: SharedString, delta_x: i32, delta_y: i32) -> [view_model] -> Self::on_drag_bottom,
                }
            });
        }

        dependency_container.register::<DockRootViewModel>(view_model);
    }

    fn on_minimize(view_model: Arc<DockRootViewModel>) {
        let view_binding = &view_model.view_binding;

        view_binding.execute_on_ui_thread(move |main_window_view, _view_binding| {
            let window = main_window_view.window();
            window.set_minimized(true);
        });
    }

    fn on_maximize(view_model: Arc<DockRootViewModel>) {
        let view_binding = &view_model.view_binding;

        view_binding.execute_on_ui_thread(move |main_window_view, _view_binding| {
            let window = main_window_view.window();
            window.set_maximized(!window.is_maximized());
        });
    }

    fn on_close() {
        if let Err(error) = slint::quit_event_loop() {
            log::error!("Failed to quit event loop: {}", error);
        }
    }

    fn on_double_clicked(view_model: Arc<DockRootViewModel>) {
        let view_binding = &view_model.view_binding;

        view_binding.execute_on_ui_thread(move |main_window_view, _view_binding| {
            let window = main_window_view.window();
            window.set_maximized(!window.is_maximized());
        });
    }

    fn on_drag(
        view_model: Arc<DockRootViewModel>,
        delta_x: i32,
        delta_y: i32,
    ) {
        let view_binding = &view_model.view_binding;

        view_binding.execute_on_ui_thread(move |main_window_view, _view_binding| {
            let window = main_window_view.window();
            let mut position = window.position();
            position.x += delta_x;
            position.y += delta_y;
            window.set_position(position);
        });
    }

    fn on_update_dock_root_size(
        view_model: Arc<DockRootViewModel>,
        width: f32,
        height: f32,
    ) -> f32 {
        Self::mutate_layout(view_model, false, move |docking_manager| {
            docking_manager
                .get_main_window_layout_mut()
                .set_available_size(width, height);
        });

        // Return 0 as part of a UI hack to get responsive UI resizing.
        0.0
    }

    fn on_update_dock_root_width(
        view_model: Arc<DockRootViewModel>,
        width: f32,
    ) {
        Self::mutate_layout(view_model.clone(), false, move |docking_manager| {
            docking_manager
                .get_main_window_layout_mut()
                .set_available_width(width);
        });

        Self::propagate_layout(view_model);
    }

    fn on_update_dock_root_height(
        view_model: Arc<DockRootViewModel>,
        height: f32,
    ) {
        Self::mutate_layout(view_model.clone(), false, move |docking_manager| {
            docking_manager
                .get_main_window_layout_mut()
                .set_available_height(height);
        });

        Self::propagate_layout(view_model);
    }

    fn on_update_active_tab_id(
        view_model: Arc<DockRootViewModel>,
        identifier: SharedString,
    ) {
        Self::mutate_layout(view_model, true, move |docking_manager| {
            docking_manager.select_tab_by_window_id(identifier.as_str());
        });
    }

    fn on_get_tab_text(identifier: SharedString) -> SharedString {
        match identifier.as_str() {
            "settings" => "Settings".into(),
            "element_scanner" => "Element Scanner".into(),
            "struct_scanner" => "Struct Scanner".into(),
            "pointer_scanner" => "Pointer Scanner".into(),
            "output" => "Output".into(),
            "process_selector" => "Process Selector".into(),
            "struct_viewer" => "Struct Viewer".into(),
            "project_explorer" => "Project Explorer".into(),
            _ => identifier,
        }
    }

    fn on_is_window_visible(
        view_model: Arc<DockRootViewModel>,
        dockable_window_id: SharedString,
    ) -> bool {
        let docking_manager = &view_model.docking_manager;

        if let Ok(docking_manager) = docking_manager.read() {
            if let Some(node) = docking_manager.get_node_by_id(&dockable_window_id) {
                return node.is_visible();
            }
        }

        false
    }

    fn on_try_redock_window(
        view_model: Arc<DockRootViewModel>,
        identifier: SharedString,
        target_identifier: SharedString,
        redock_target: RedockTarget,
    ) {
        Self::mutate_layout(view_model, true, move |docking_manager| {
            docking_manager.reparent_window(
                &identifier,
                &target_identifier,
                DocktargetConverter::new().convert_from_view_data(&redock_target),
            );
        });
    }

    fn on_reset_layout(view_model: Arc<DockRootViewModel>) {
        Self::mutate_layout(view_model, true, move |docking_manager| {
            docking_manager.set_root(DockSettingsConfig::get_default_layout());
        });
    }

    fn on_show(
        view_model: Arc<DockRootViewModel>,
        dockable_window_id: SharedString,
    ) {
        Self::mutate_layout(view_model, true, move |docking_manager| {
            if let Some(node) = docking_manager.get_node_by_id_mut(&dockable_window_id) {
                node.set_visible(true);
            }
            docking_manager
                .get_root_mut()
                .select_tab_by_window_id(&dockable_window_id);
        });
    }

    fn on_hide(
        view_model: Arc<DockRootViewModel>,
        dockable_window_id: SharedString,
    ) {
        Self::mutate_layout(view_model, true, move |docking_manager| {
            if let Some(node) = docking_manager.get_node_by_id_mut(&dockable_window_id) {
                node.set_visible(false);
            }
        });
    }

    fn on_toggle_visibility(
        view_model: Arc<DockRootViewModel>,
        dockable_window_id: SharedString,
    ) {
        Self::mutate_layout(view_model, true, move |docking_manager| {
            if let Some(node) = docking_manager.get_node_by_id_mut(&dockable_window_id) {
                node.set_visible(!node.is_visible());

                if node.is_visible() {
                    docking_manager
                        .get_root_mut()
                        .select_tab_by_window_id(&dockable_window_id);
                }
            }
        });
    }

    fn on_drag_left(
        view_model: Arc<DockRootViewModel>,
        dockable_window_id: SharedString,
        delta_x: i32,
        delta_y: i32,
    ) {
        Self::mutate_layout(view_model, false, move |manager| {
            manager.adjust_window_size(dockable_window_id.as_str(), &DockSplitterDragDirection::Left, delta_x, delta_y);
        });
    }

    fn on_drag_right(
        view_model: Arc<DockRootViewModel>,
        dockable_window_id: SharedString,
        delta_x: i32,
        delta_y: i32,
    ) {
        Self::mutate_layout(view_model, false, move |manager| {
            manager.adjust_window_size(dockable_window_id.as_str(), &DockSplitterDragDirection::Right, delta_x, delta_y);
        });
    }

    fn on_drag_top(
        view_model: Arc<DockRootViewModel>,
        dockable_window_id: SharedString,
        delta_x: i32,
        delta_y: i32,
    ) {
        Self::mutate_layout(view_model, false, move |manager| {
            manager.adjust_window_size(dockable_window_id.as_str(), &DockSplitterDragDirection::Top, delta_x, delta_y);
        });
    }

    fn on_drag_bottom(
        view_model: Arc<DockRootViewModel>,
        dockable_window_id: SharedString,
        delta_x: i32,
        delta_y: i32,
    ) {
        Self::mutate_layout(view_model, false, move |manager| {
            manager.adjust_window_size(dockable_window_id.as_str(), &DockSplitterDragDirection::Bottom, delta_x, delta_y);
        });
    }

    fn mutate_layout<F>(
        view_model: Arc<DockRootViewModel>,
        save_layout: bool,
        callback: F,
    ) where
        F: FnOnce(&mut DockingManager),
    {
        let docking_manager = view_model.docking_manager.clone();

        let mut layout_guard = match docking_manager.write() {
            Ok(guard) => guard,
            Err(error) => {
                log::error!("Could not acquire docking_manager write lock: {}", error);
                return;
            }
        };

        callback(&mut layout_guard);

        // Optionally save changes.
        if save_layout {
            DockableWindowSettings::set_dock_layout_settings(layout_guard.get_root());
        }

        drop(layout_guard);

        Self::propagate_layout(view_model);
    }

    fn propagate_layout(view_model: Arc<DockRootViewModel>) {
        let view_binding = &view_model.view_binding;
        let docking_manager = view_model.docking_manager.clone();

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
                Err(error) => {
                    log::error!("Failed to acquire read lock on docking_manager: {}", error);
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
                    "element_scanner" => {
                        dock_root_bindings.set_element_scanner_window(view_data);
                    }
                    "pointer_scanner" => {
                        dock_root_bindings.set_pointer_scanner_window(view_data);
                    }
                    "output" => {
                        dock_root_bindings.set_output_window(view_data);
                    }
                    "process_selector" => {
                        dock_root_bindings.set_process_selector_window(view_data);
                    }
                    "struct_viewer" => {
                        dock_root_bindings.set_struct_viewer_window(view_data);
                    }
                    "project_explorer" => {
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
