use crate::ui::widgets::controls::toolbar_menu::toolbar_menu_item_data::ToolbarMenuItemData;

#[derive(Clone)]
pub struct ToolbarMenuData {
    pub header: String,
    pub items: Vec<ToolbarMenuItemData>,
}
