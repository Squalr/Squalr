use crate::ui::dock_root::dock_root_view::DockRootView;
use crate::ui::main_window::toolbar_view::ToolbarView;
use crate::ui::main_window::{footer_view::FooterView, title_bar_view::TitleBarView};
use crate::ui::theme::Theme;
use eframe::egui::{Align, Context, Layout, Response, Sense, Ui, Widget};
use std::rc::Rc;

#[derive(Clone)]
pub struct MainWindowView {
    pub context: Context,
    pub theme: Rc<Theme>,
    pub title_bar_view: TitleBarView,
    pub toolbar_view: ToolbarView,
    pub dock_root_view: DockRootView,
    pub footer_view: FooterView,
}

impl MainWindowView {
    pub fn new(
        context: Context,
        theme: Rc<Theme>,
    ) -> Self {
        let title_bar_view = TitleBarView {
            context: context.clone(),
            theme: theme.clone(),
            title: "Squalr".to_string(),
            height: 32.0,
        };
        let toolbar_view = ToolbarView {
            context: context.clone(),
            theme: theme.clone(),
            height: 32.0,
        };
        let dock_root_view = DockRootView {
            context: context.clone(),
            theme: theme.clone(),
        };
        let footer_view = FooterView {
            context: context.clone(),
            theme: theme.clone(),
            height: 32.0,
        };

        Self {
            context,
            theme,
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
                user_interface.add(self.dock_root_view);
                user_interface.add(self.footer_view);
            })
            .response;

        response
    }
}
