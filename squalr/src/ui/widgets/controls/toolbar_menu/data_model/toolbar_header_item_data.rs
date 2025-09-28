use crate::ui::widgets::controls::toolbar_menu::data_model::toolbar_menu_item_data::ToolbarMenuItemData;

#[derive(Clone)]
pub struct ToolbarHeaderItemData {
    pub header: String,
    pub items: Vec<ToolbarMenuItemData>,
}
