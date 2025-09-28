use crate::ui::widgets::controls::toolbar_menu::data_model::toolbar_menu_data::ToolbarMenuData;

#[derive(Clone)]
pub struct ToolbarData {
    pub active_menu: String,
    pub menus: Vec<ToolbarMenuData>,
}
