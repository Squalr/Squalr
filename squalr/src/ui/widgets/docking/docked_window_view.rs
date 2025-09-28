use crate::ui::widgets::docking::docked_window_content_view::DockedWindowContentView;
use crate::ui::widgets::docking::docked_window_footer_view::DockedWindowFooterView;
use crate::ui::{theme::Theme, widgets::docking::docked_window_title_bar_view::DockedWindowTitleBarView};
use eframe::egui::{Align, Context, Layout, Response, Ui, Widget};
use epaint::CornerRadius;
use std::rc::Rc;

#[derive(Clone)]
pub struct DockedWindowView {
    _context: Context,
    _theme: Rc<Theme>,
    docked_window_title_bar_view: DockedWindowTitleBarView,
    docked_window_content_view: DockedWindowContentView,
    docked_window_footer_view: DockedWindowFooterView,
}

impl DockedWindowView {
    pub fn new(
        context: Context,
        theme: Rc<Theme>,
        title: String,
        corner_radius: CornerRadius,
    ) -> Self {
        let docked_window_title_bar_view = DockedWindowTitleBarView::new(context.clone(), theme.clone(), corner_radius, 32.0, title);
        let docked_window_content_view = DockedWindowContentView::new(context.clone(), theme.clone());
        let docked_window_footer_view = DockedWindowFooterView::new(context.clone(), theme.clone(), corner_radius, 28.0);

        Self {
            _context: context,
            _theme: theme,
            docked_window_title_bar_view,
            docked_window_content_view,
            docked_window_footer_view,
        }
    }
}

impl Widget for DockedWindowView {
    fn ui(
        self,
        user_interface: &mut Ui,
    ) -> Response {
        let response = user_interface
            .allocate_ui_with_layout(user_interface.available_size(), Layout::top_down(Align::Min), |user_interface| {
                user_interface.add(self.docked_window_title_bar_view);
                user_interface.add_sized(
                    [
                        user_interface.available_width(),
                        user_interface.available_height() - self.docked_window_footer_view.get_height(),
                    ],
                    self.docked_window_content_view,
                );
                user_interface.add(self.docked_window_footer_view);
            })
            .response;

        response
    }
}
