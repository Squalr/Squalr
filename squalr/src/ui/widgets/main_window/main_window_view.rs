use crate::app_context::AppContext;
use crate::ui::widgets::docking::dock_root_view::DockRootView;
use crate::ui::widgets::docking::docked_window_view::DockedWindowView;
use crate::ui::widgets::element_scanner::element_scanner::ElementScannerView;
use crate::ui::widgets::main_window::main_footer_view::MainFooterView;
use crate::ui::widgets::main_window::main_title_bar_view::MainTitleBarView;
use crate::ui::widgets::main_window::main_toolbar_view::MainToolbarView;
use crate::ui::widgets::output::output_view::OutputView;
use crate::ui::widgets::pointer_scanner::pointer_scanner_view::PointerScannerView;
use crate::ui::widgets::process_selector::process_selector_view::ProcessSelectorView;
use crate::ui::widgets::project_explorer::project_explorer_view::ProjectExplorerView;
use crate::ui::widgets::settings::settings_view::SettingsView;
use crate::ui::widgets::struct_viewer::struct_viewer_view::StructViewerView;
use eframe::egui::{Align, Context, Id, Layout, ResizeDirection, Response, Sense, Ui, ViewportCommand, Widget};
use epaint::CornerRadius;
use epaint::{Rect, pos2};
use std::{rc::Rc, sync::Arc};

#[derive(Clone)]
pub struct MainWindowView {
    app_context: Rc<AppContext>,
    main_title_bar_view: MainTitleBarView,
    main_toolbar_view: MainToolbarView,
    dock_root_view: DockRootView,
    main_footer_view: MainFooterView,
    resize_thickness: f32,
}

impl MainWindowView {
    pub fn new(
        app_context: Rc<AppContext>,
        title: String,
        corner_radius: CornerRadius,
    ) -> Self {
        let main_title_bar_view = MainTitleBarView::new(app_context.clone(), corner_radius, 32.0, title);
        let main_toolbar_view = MainToolbarView::new(app_context.clone(), 32.0);

        let app_context_for_output = app_context.clone();
        let output_view = DockedWindowView::new(
            app_context_for_output.clone(),
            Arc::new(move |user_interface| OutputView::new(app_context_for_output.clone()).ui(user_interface)),
            "Output".to_string(),
            "output".to_string(),
        );

        let app_context_for_settings = app_context.clone();
        let settings_view = DockedWindowView::new(
            app_context_for_settings.clone(),
            Arc::new(move |user_interface| SettingsView::new(app_context_for_settings.clone()).ui(user_interface)),
            "Settings".to_string(),
            "settings".to_string(),
        );

        let app_context_for_struct_viewer = app_context.clone();
        let struct_viewer_view = DockedWindowView::new(
            app_context_for_struct_viewer.clone(),
            Arc::new(move |user_interface| StructViewerView::new(app_context_for_struct_viewer.clone()).ui(user_interface)),
            "Struct Viewer".to_string(),
            "struct_viewer".to_string(),
        );

        let app_context_for_project_explorer = app_context.clone();
        let project_explorer_view = DockedWindowView::new(
            app_context_for_project_explorer.clone(),
            Arc::new(move |user_interface| ProjectExplorerView::new(app_context_for_project_explorer.clone()).ui(user_interface)),
            "Project Explorer".to_string(),
            "project_explorer".to_string(),
        );

        let app_context_for_process_selector = app_context.clone();
        let process_selector_view = DockedWindowView::new(
            app_context_for_process_selector.clone(),
            Arc::new(move |user_interface| ProcessSelectorView::new(app_context_for_process_selector.clone()).ui(user_interface)),
            "Process Selector".to_string(),
            "process_selector".to_string(),
        );

        let app_context_for_element_scanner = app_context.clone();
        let element_scanner_view = DockedWindowView::new(
            app_context_for_element_scanner.clone(),
            Arc::new(move |user_interface| ElementScannerView::new(app_context_for_element_scanner.clone()).ui(user_interface)),
            "Element Scanner".to_string(),
            "element_scanner".to_string(),
        );

        let app_context_for_pointer_scanner = app_context.clone();
        let pointer_scanner_view = DockedWindowView::new(
            app_context_for_pointer_scanner.clone(),
            Arc::new(move |user_interface| PointerScannerView::new(app_context_for_pointer_scanner.clone()).ui(user_interface)),
            "Pointer Scanner".to_string(),
            "pointer_scanner".to_string(),
        );

        let dock_root_view = DockRootView::new(
            app_context.clone(),
            vec![
                output_view,
                settings_view,
                struct_viewer_view,
                project_explorer_view,
                process_selector_view,
                element_scanner_view,
                pointer_scanner_view,
            ],
        );

        let main_footer_view = MainFooterView::new(app_context.clone(), corner_radius, 28.0);
        let resize_thickness = 4.0;

        Self {
            app_context,
            main_title_bar_view,
            main_toolbar_view,
            dock_root_view,
            main_footer_view,
            resize_thickness,
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
        dir: ResizeDirection,
    ) {
        use eframe::egui::CursorIcon;

        let response: Response = user_interface.interact(rect, Id::new(id), Sense::click_and_drag());
        let drag_started = response.drag_started();

        // Show the appropriate cursor when hovering
        match dir {
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
            context.send_viewport_cmd(ViewportCommand::BeginResize(dir));
        }
    }
}

impl Widget for MainWindowView {
    fn ui(
        self,
        user_interface: &mut Ui,
    ) -> Response {
        let response = user_interface
            .allocate_ui_with_layout(user_interface.available_size(), Layout::top_down(Align::Min), |user_interface| {
                user_interface.add(self.main_title_bar_view);
                user_interface.add(self.main_toolbar_view);
                user_interface.add_sized(
                    [
                        user_interface.available_width(),
                        user_interface.available_height() - self.main_footer_view.get_height(),
                    ],
                    self.dock_root_view,
                );
                user_interface.add(self.main_footer_view);
            })
            .response;

        Self::add_resize_handles(&self.app_context.context, user_interface, self.resize_thickness);

        response
    }
}
