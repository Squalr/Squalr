use crate::models::docking::docking_manager::DockingManager;
use crate::models::docking::settings::dockable_window_settings::DockableWindowSettings;
use crate::ui::theme::Theme;
use crate::ui::widgets::docking::dock_root_view::DockRootView;
use crate::ui::widgets::docking::docked_window_view::DockedWindowView;
use crate::ui::widgets::main_window::main_footer_view::MainFooterView;
use crate::ui::widgets::main_window::main_title_bar_view::MainTitleBarView;
use crate::ui::widgets::main_window::main_toolbar_view::MainToolbarView;
use crate::ui::widgets::settings::settings_view::SettingsView;
use eframe::egui::{Align, Context, Id, Layout, ResizeDirection, Response, Sense, Ui, ViewportCommand, Widget};
use epaint::CornerRadius;
use epaint::{Rect, pos2};
use squalr_engine_api::engine::engine_execution_context::EngineExecutionContext;
use std::sync::RwLock;
use std::{rc::Rc, sync::Arc};

#[derive(Clone)]
pub struct MainWindowView {
    _engine_execution_context: Arc<EngineExecutionContext>,
    context: Context,
    _theme: Rc<Theme>,
    main_title_bar_view: MainTitleBarView,
    main_toolbar_view: MainToolbarView,
    dock_root_view: DockRootView,
    main_footer_view: MainFooterView,
    resize_thickness: f32,
}

impl MainWindowView {
    pub fn new(
        engine_execution_context: Arc<EngineExecutionContext>,
        context: Context,
        theme: Rc<Theme>,
        title: String,
        corner_radius: CornerRadius,
    ) -> Self {
        let main_title_bar_view = MainTitleBarView::new(context.clone(), theme.clone(), corner_radius, 32.0, title);
        let main_toolbar_view = MainToolbarView::new(context.clone(), theme.clone(), 32.0);

        // Create built in docked windows.
        let main_dock_root = DockableWindowSettings::get_dock_layout_settings();
        let docking_manager = Arc::new(RwLock::new(DockingManager::new(main_dock_root)));

        let engine_execution_context_for_settings = engine_execution_context.clone();
        let context_for_settings = context.clone();
        let theme_for_settings = theme.clone();

        let settings = DockedWindowView::new(
            engine_execution_context.clone(),
            context.clone(),
            theme.clone(),
            docking_manager.clone(),
            Arc::new(move |user_interface| {
                SettingsView::new(
                    engine_execution_context_for_settings.clone(),
                    context_for_settings.clone(),
                    theme_for_settings.clone(),
                )
                .ui(user_interface)
            }),
            "settings".to_string(),
        );

        let dock_root_view = DockRootView::new(
            engine_execution_context.clone(),
            context.clone(),
            theme.clone(),
            docking_manager,
            vec![settings],
        );

        let main_footer_view = MainFooterView::new(context.clone(), theme.clone(), corner_radius, 28.0);
        let resize_thickness = 4.0;

        Self {
            _engine_execution_context: engine_execution_context,
            context,
            _theme: theme,
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

        Self::add_resize_handles(&self.context, user_interface, self.resize_thickness);

        response
    }
}
