use crate::app_context::AppContext;
use crate::ui::widgets::docking::dock_root_view::DockRootView;
use crate::ui::widgets::docking::dock_root_view_data::DockRootViewData;
use crate::ui::widgets::docking::dock_tab_attention_state::DockTabAttentionKind;
use crate::ui::widgets::docking::docked_window_view::DockedWindowView;
use crate::views::code_viewer::code_viewer_view::CodeViewerView;
use crate::views::element_scanner::scanner::element_scanner_view::ElementScannerView;
use crate::views::main_window::main_footer_view::MainFooterView;
use crate::views::main_window::main_shortcut_bar_view::MainShortcutBarView;
use crate::views::main_window::main_title_bar_view::MainTitleBarView;
use crate::views::main_window::main_toolbar_view::MainToolbarView;
use crate::views::main_window::{about_take_over_view::AboutTakeOverView, main_window_take_over_state::MainWindowTakeOverState};
use crate::views::memory_viewer::memory_viewer_view::MemoryViewerView;
use crate::views::output::output_view::OutputView;
use crate::views::plugins::{plugins_view::PluginsView, view_data::plugin_list_view_data::PluginListViewData};
use crate::views::pointer_scanner::pointer_scanner_view::PointerScannerView;
use crate::views::process_selector::process_selector_view::ProcessSelectorView;
use crate::views::process_selector::view_data::process_selector_view_data::ProcessSelectorViewData;
use crate::views::project_explorer::project_explorer_view::ProjectExplorerView;
use crate::views::settings::settings_view::SettingsView;
use crate::views::struct_editor::struct_editor_view::StructEditorView;
use crate::views::struct_viewer::struct_viewer_view::StructViewerView;
use crate::views::symbol_explorer::symbol_explorer_view::SymbolExplorerView;
use crate::views::symbol_table::symbol_table_view::SymbolTableView;
use eframe::egui::{Align, Context, Id, Layout, ResizeDirection, Response, Sense, Ui, ViewportCommand, Widget, vec2};
use epaint::CornerRadius;
use epaint::{Rect, pos2};
use log::Level;
use squalr_engine_api::engine::engine_execution_context::EngineExecutionContext;
use std::rc::Rc;
use std::sync::{Arc, RwLock};

#[derive(Clone)]
pub struct MainWindowView {
    app_context: Arc<AppContext>,
    main_title_bar_view: MainTitleBarView,
    main_toolbar_view: MainToolbarView,
    main_shortcut_bar_view: MainShortcutBarView,
    dock_view_data: Arc<DockRootViewData>,
    dock_root_view: DockRootView,
    main_footer_view: MainFooterView,
    main_window_take_over_state: Arc<RwLock<MainWindowTakeOverState>>,
    output_log_history_len_cursor: Arc<RwLock<usize>>,
    resize_thickness: f32,
}

impl MainWindowView {
    pub fn new(
        app_context: Arc<AppContext>,
        title: Rc<String>,
        corner_radius: CornerRadius,
    ) -> Self {
        app_context
            .dependency_container
            .register(ProcessSelectorViewData::new());
        app_context
            .dependency_container
            .register(PluginListViewData::new());
        let main_window_take_over_state = Arc::new(RwLock::new(MainWindowTakeOverState::None));

        let main_title_bar_view = MainTitleBarView::new(app_context.clone(), corner_radius, 32.0, title);
        let main_toolbar_view = MainToolbarView::new(app_context.clone(), main_window_take_over_state.clone());
        let main_shortcut_bar_view = MainShortcutBarView::new(app_context.clone());
        let dock_view_data = Arc::new(DockRootViewData::new());
        let output_log_history_len_cursor = Arc::new(RwLock::new(
            app_context
                .engine_unprivileged_state
                .get_logger()
                .get_log_history()
                .read()
                .map(|log_history| log_history.len())
                .unwrap_or(0),
        ));

        let app_context_for_output = app_context.clone();
        let output_view = DockedWindowView::new(
            app_context_for_output.clone(),
            dock_view_data.clone(),
            OutputView::new(app_context_for_output.clone()),
            Rc::new("Output".to_string()),
            Rc::new("window_output".to_string()),
        );

        let app_context_for_settings = app_context.clone();
        let settings_view = DockedWindowView::new(
            app_context_for_settings.clone(),
            dock_view_data.clone(),
            SettingsView::new(app_context_for_settings.clone()),
            Rc::new("Settings".to_string()),
            Rc::new("window_settings".to_string()),
        );

        let app_context_for_struct_viewer = app_context.clone();
        let struct_viewer_view = DockedWindowView::new(
            app_context_for_struct_viewer.clone(),
            dock_view_data.clone(),
            StructViewerView::new(app_context_for_struct_viewer.clone()),
            Rc::new("Details Viewer".to_string()),
            Rc::new(StructViewerView::WINDOW_ID.to_string()),
        );

        let app_context_for_struct_editor = app_context.clone();
        let struct_editor_view = DockedWindowView::new(
            app_context_for_struct_editor.clone(),
            dock_view_data.clone(),
            StructEditorView::new(app_context_for_struct_editor.clone()),
            Rc::new("Symbol Structs".to_string()),
            Rc::new(StructEditorView::WINDOW_ID.to_string()),
        );

        let app_context_for_memory_viewer = app_context.clone();
        let memory_viewer_view = DockedWindowView::new(
            app_context_for_memory_viewer.clone(),
            dock_view_data.clone(),
            MemoryViewerView::new(app_context_for_memory_viewer.clone()),
            Rc::new("Memory Viewer".to_string()),
            Rc::new(MemoryViewerView::WINDOW_ID.to_string()),
        );

        let app_context_for_code_viewer = app_context.clone();
        let code_viewer_view = DockedWindowView::new(
            app_context_for_code_viewer.clone(),
            dock_view_data.clone(),
            CodeViewerView::new(app_context_for_code_viewer.clone()),
            Rc::new("Code Viewer".to_string()),
            Rc::new(CodeViewerView::WINDOW_ID.to_string()),
        );

        let app_context_for_project_explorer = app_context.clone();
        let project_explorer_view = DockedWindowView::new(
            app_context_for_project_explorer.clone(),
            dock_view_data.clone(),
            ProjectExplorerView::new(app_context_for_project_explorer.clone()),
            Rc::new("Project Explorer".to_string()),
            Rc::new("window_project_explorer".to_string()),
        );

        let app_context_for_symbol_explorer = app_context.clone();
        let symbol_explorer_view = DockedWindowView::new(
            app_context_for_symbol_explorer.clone(),
            dock_view_data.clone(),
            SymbolExplorerView::new(app_context_for_symbol_explorer.clone()),
            Rc::new("Symbol Tree".to_string()),
            Rc::new(SymbolExplorerView::WINDOW_ID.to_string()),
        );

        let app_context_for_symbol_table = app_context.clone();
        let symbol_table_view = DockedWindowView::new(
            app_context_for_symbol_table.clone(),
            dock_view_data.clone(),
            SymbolTableView::new(app_context_for_symbol_table.clone()),
            Rc::new("Symbol Table".to_string()),
            Rc::new(SymbolTableView::WINDOW_ID.to_string()),
        );

        let app_context_for_process_selector = app_context.clone();
        let process_selector_view = DockedWindowView::new(
            app_context_for_process_selector.clone(),
            dock_view_data.clone(),
            ProcessSelectorView::new(app_context_for_process_selector.clone()),
            Rc::new("Process Selector".to_string()),
            Rc::new("window_process_selector".to_string()),
        );

        let app_context_for_element_scanner = app_context.clone();
        let element_scanner_view = DockedWindowView::new(
            app_context_for_element_scanner.clone(),
            dock_view_data.clone(),
            ElementScannerView::new(app_context_for_element_scanner.clone()),
            Rc::new("Element Scanner".to_string()),
            Rc::new("window_element_scanner".to_string()),
        );

        let app_context_for_pointer_scanner = app_context.clone();
        let pointer_scanner_view = DockedWindowView::new(
            app_context_for_pointer_scanner.clone(),
            dock_view_data.clone(),
            PointerScannerView::new(app_context_for_pointer_scanner.clone()),
            Rc::new("Pointer Scanner".to_string()),
            Rc::new("window_pointer_scanner".to_string()),
        );

        let app_context_for_plugins = app_context.clone();
        let plugins_view = DockedWindowView::new(
            app_context_for_plugins.clone(),
            dock_view_data.clone(),
            PluginsView::new(app_context_for_plugins.clone()),
            Rc::new("Plugins".to_string()),
            Rc::new(PluginsView::WINDOW_ID.to_string()),
        );

        dock_view_data.set_windows(vec![
            Box::new(output_view),
            Box::new(settings_view),
            Box::new(struct_viewer_view),
            Box::new(struct_editor_view),
            Box::new(memory_viewer_view),
            Box::new(code_viewer_view),
            Box::new(project_explorer_view),
            Box::new(symbol_explorer_view),
            Box::new(symbol_table_view),
            Box::new(process_selector_view),
            Box::new(element_scanner_view),
            Box::new(pointer_scanner_view),
            Box::new(plugins_view),
        ]);
        let dock_root_view = DockRootView::new(app_context.clone(), dock_view_data.clone());
        let main_footer_view = MainFooterView::new(app_context.clone(), corner_radius, 24.0);
        let resize_thickness = 4.0;

        Self {
            app_context,
            main_title_bar_view,
            main_toolbar_view,
            main_shortcut_bar_view,
            dock_view_data,
            dock_root_view,
            main_footer_view,
            main_window_take_over_state,
            output_log_history_len_cursor,
            resize_thickness,
        }
    }

    fn get_take_over_state(&self) -> MainWindowTakeOverState {
        match self.main_window_take_over_state.read() {
            Ok(main_window_take_over_state) => main_window_take_over_state.clone(),
            Err(error) => {
                log::error!("Failed to acquire main window take over state for read: {}", error);
                MainWindowTakeOverState::None
            }
        }
    }

    fn clear_take_over_state_shared(main_window_take_over_state: &Arc<RwLock<MainWindowTakeOverState>>) {
        match main_window_take_over_state.write() {
            Ok(mut take_over_state) => {
                *take_over_state = MainWindowTakeOverState::None;
            }
            Err(error) => {
                log::error!("Failed to acquire main window take over state for write: {}", error);
            }
        }
    }

    fn is_window_active_tab(
        app_context: &AppContext,
        window_identifier: &str,
    ) -> bool {
        match app_context.docking_manager.read() {
            Ok(docking_manager) => docking_manager.get_active_tab(window_identifier) == window_identifier,
            Err(error) => {
                log::error!("Failed to acquire docking manager lock while checking active tab state: {}.", error);

                false
            }
        }
    }

    fn poll_output_tab_attention(&self) {
        let log_history = self
            .app_context
            .engine_unprivileged_state
            .get_logger()
            .get_log_history();
        let previous_log_history_len = match self.output_log_history_len_cursor.read() {
            Ok(previous_log_history_len) => *previous_log_history_len,
            Err(error) => {
                log::error!("Failed to acquire output log history cursor for read: {}.", error);

                return;
            }
        };

        let requested_attention_kind = match log_history.read() {
            Ok(log_history) => {
                let current_log_history_len = log_history.len();
                let requested_attention_kind = if current_log_history_len >= previous_log_history_len {
                    let mut requested_attention_kind = None;

                    for log_event in log_history.iter().skip(previous_log_history_len) {
                        match log_event.level {
                            Level::Error => {
                                requested_attention_kind = Some(DockTabAttentionKind::Danger);
                                break;
                            }
                            Level::Warn => {
                                requested_attention_kind = Some(DockTabAttentionKind::Warning);
                            }
                            _ => {}
                        }
                    }

                    requested_attention_kind
                } else {
                    None
                };

                match self.output_log_history_len_cursor.write() {
                    Ok(mut output_log_history_len_cursor) => {
                        *output_log_history_len_cursor = current_log_history_len;
                    }
                    Err(error) => {
                        log::error!("Failed to acquire output log history cursor for write: {}.", error);
                    }
                }

                requested_attention_kind
            }
            Err(error) => {
                log::error!("Failed to acquire output log history for read: {}.", error);

                None
            }
        };

        if let Some(requested_attention_kind) = requested_attention_kind {
            let is_output_tab_active = Self::is_window_active_tab(&self.app_context, OutputView::WINDOW_ID);

            if !is_output_tab_active {
                self.dock_view_data
                    .request_tab_attention(OutputView::WINDOW_ID, requested_attention_kind, false);
            }
        }
    }

    fn poll_project_explorer_tab_attention(&self) {
        let has_opened_project = match self
            .app_context
            .engine_unprivileged_state
            .get_project_manager()
            .get_opened_project()
            .read()
        {
            Ok(opened_project) => opened_project.is_some(),
            Err(error) => {
                log::error!("Failed to acquire opened project lock while polling project explorer tab attention: {}.", error);

                false
            }
        };

        if has_opened_project {
            self.dock_view_data
                .clear_tab_attention(ProjectExplorerView::WINDOW_ID);
        } else {
            self.dock_view_data
                .request_tab_attention(ProjectExplorerView::WINDOW_ID, DockTabAttentionKind::Warning, true);
        }
    }

    fn add_resize_handles(
        context: &Context,
        user_interface: &mut Ui,
        resize_thickness: f32,
    ) {
        let rect = user_interface.max_rect();

        // Top-left corner.
        Self::handle_resize(
            context,
            user_interface,
            Rect::from_min_max(rect.min, pos2(rect.min.x + resize_thickness, rect.min.y + resize_thickness)),
            "resize_top_left",
            ResizeDirection::NorthWest,
        );

        // Top-right corner.
        Self::handle_resize(
            context,
            user_interface,
            Rect::from_min_max(pos2(rect.max.x - resize_thickness, rect.min.y), pos2(rect.max.x, rect.min.y + resize_thickness)),
            "resize_top_right",
            ResizeDirection::NorthEast,
        );

        // Bottom-left corner.
        Self::handle_resize(
            context,
            user_interface,
            Rect::from_min_max(pos2(rect.min.x, rect.max.y - resize_thickness), pos2(rect.min.x + resize_thickness, rect.max.y)),
            "resize_bottom_left",
            ResizeDirection::SouthWest,
        );

        // Bottom-right corner.
        Self::handle_resize(
            context,
            user_interface,
            Rect::from_min_max(pos2(rect.max.x - resize_thickness, rect.max.y - resize_thickness), rect.max),
            "resize_bottom_right",
            ResizeDirection::SouthEast,
        );

        // Left side (skip corners).
        Self::handle_resize(
            context,
            user_interface,
            Rect::from_min_max(
                pos2(rect.min.x, rect.min.y + resize_thickness),
                pos2(rect.min.x + resize_thickness, rect.max.y - resize_thickness),
            ),
            "resize_left",
            ResizeDirection::West,
        );

        // Right side (skip corners).
        Self::handle_resize(
            context,
            user_interface,
            Rect::from_min_max(
                pos2(rect.max.x - resize_thickness, rect.min.y + resize_thickness),
                pos2(rect.max.x, rect.max.y - resize_thickness),
            ),
            "resize_right",
            ResizeDirection::East,
        );

        // Top side (skip corners).
        Self::handle_resize(
            context,
            user_interface,
            Rect::from_min_max(
                pos2(rect.min.x + resize_thickness, rect.min.y),
                pos2(rect.max.x - resize_thickness, rect.min.y + resize_thickness),
            ),
            "resize_top",
            ResizeDirection::North,
        );

        // Bottom side (skip corners).
        Self::handle_resize(
            context,
            user_interface,
            Rect::from_min_max(
                pos2(rect.min.x + resize_thickness, rect.max.y - resize_thickness),
                pos2(rect.max.x - resize_thickness, rect.max.y),
            ),
            "resize_bottom",
            ResizeDirection::South,
        );
    }

    fn handle_resize(
        context: &Context,
        user_interface: &mut Ui,
        rect: Rect,
        id: &str,
        resize_direction: ResizeDirection,
    ) {
        use eframe::egui::CursorIcon;

        let response: Response = user_interface.interact(rect, Id::new(id), Sense::click_and_drag());
        let drag_started = response.drag_started();

        // Show the appropriate cursor when hovering
        match resize_direction {
            ResizeDirection::North | ResizeDirection::South => {
                response.on_hover_cursor(CursorIcon::ResizeVertical);
            }
            ResizeDirection::East | ResizeDirection::West => {
                response.on_hover_cursor(CursorIcon::ResizeHorizontal);
            }
            ResizeDirection::NorthEast | ResizeDirection::SouthWest => {
                response.on_hover_cursor(CursorIcon::ResizeNeSw);
            }
            ResizeDirection::NorthWest | ResizeDirection::SouthEast => {
                response.on_hover_cursor(CursorIcon::ResizeNwSe);
            }
        }

        if drag_started {
            context.send_viewport_cmd(ViewportCommand::BeginResize(resize_direction));
        }
    }
}

impl Widget for MainWindowView {
    fn ui(
        self,
        user_interface: &mut Ui,
    ) -> Response {
        self.poll_output_tab_attention();
        self.poll_project_explorer_tab_attention();
        let take_over_state = self.get_take_over_state();
        let app_context = self.app_context.clone();
        let resize_thickness = self.resize_thickness;
        let main_window_take_over_state = self.main_window_take_over_state.clone();
        let main_title_bar_view = self.main_title_bar_view;
        let main_toolbar_view = self.main_toolbar_view;
        let main_shortcut_bar_view = self.main_shortcut_bar_view;
        let dock_root_view = self.dock_root_view;
        let main_footer_view = self.main_footer_view;
        let response = user_interface
            .allocate_ui_with_layout(user_interface.available_size(), Layout::top_down(Align::Min), |user_interface| {
                user_interface.add(main_title_bar_view);
                user_interface.add(main_toolbar_view);
                user_interface.add(main_shortcut_bar_view);

                if user_interface.available_rect_before_wrap().is_positive() {
                    let content_size = [
                        user_interface.available_width(),
                        user_interface.available_height() - main_footer_view.get_height(),
                    ];

                    match take_over_state {
                        MainWindowTakeOverState::None => {
                            user_interface.add_sized(content_size, dock_root_view);
                        }
                        MainWindowTakeOverState::About => {
                            user_interface.allocate_ui_with_layout(vec2(content_size[0], content_size[1]), Layout::top_down(Align::Min), |user_interface| {
                                let about_take_over_response = AboutTakeOverView::new(app_context.clone()).show(user_interface);

                                if about_take_over_response.should_close {
                                    Self::clear_take_over_state_shared(&main_window_take_over_state);
                                }
                            });
                        }
                    }
                }

                user_interface.add(main_footer_view);
            })
            .response;

        Self::add_resize_handles(&app_context.context, user_interface, resize_thickness);

        response
    }
}
