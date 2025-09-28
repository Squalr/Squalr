use crate::ui::widgets::controls::toolbar_menu::data_model::toolbar_header_item_data::ToolbarHeaderItemData;

#[derive(Clone)]
pub struct ToolbarData {
    pub active_menu: String,
    pub menus: Vec<ToolbarHeaderItemData>,
}
