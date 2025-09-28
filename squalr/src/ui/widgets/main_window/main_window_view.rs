use crate::ui::theme::Theme;
use crate::ui::widgets::dock_root::dock_root_view::DockRootView;
use crate::ui::widgets::main_window::footer_view::FooterView;
use crate::ui::widgets::main_window::title_bar_view::TitleBarView;
use crate::ui::widgets::main_window::toolbar_view::ToolbarView;
use eframe::egui::{Align, Context, Layout, Response, Ui, Widget};
use epaint::CornerRadius;
use std::rc::Rc;

#[derive(Clone)]
pub struct MainWindowView {
    _context: Context,
    _theme: Rc<Theme>,
    title_bar_view: TitleBarView,
    toolbar_view: ToolbarView,
    dock_root_view: DockRootView,
    footer_view: FooterView,
}

impl MainWindowView {
    pub fn new(
        context: Context,
        theme: Rc<Theme>,
        corner_radius: CornerRadius,
    ) -> Self {
        let title_bar_view = TitleBarView::new(context.clone(), theme.clone(), corner_radius, 32.0, "Squalr".to_string());
        let toolbar_view = ToolbarView::new(context.clone(), theme.clone(), 32.0);
        let dock_root_view = DockRootView::new(context.clone(), theme.clone());
        let footer_view = FooterView::new(context.clone(), theme.clone(), corner_radius, 28.0);

        Self {
            _context: context,
            _theme: theme,
            title_bar_view,
            toolbar_view,
            dock_root_view,
            footer_view,
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
                user_interface.add(self.title_bar_view);
                user_interface.add(self.toolbar_view);
                user_interface.add_sized(
                    [
                        user_interface.available_width(),
                        user_interface.available_height() - self.footer_view.get_height(),
                    ],
                    self.dock_root_view,
                );
                user_interface.add(self.footer_view);
            })
            .response;

        response
    }
}
