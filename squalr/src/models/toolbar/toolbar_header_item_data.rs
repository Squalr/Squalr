use crate::models::toolbar::toolbar_menu_item_data::ToolbarMenuItemData;
use smallvec::SmallVec;

#[derive(Clone)]
pub struct ToolbarHeaderItemData {
    pub header: String,
    pub items: SmallVec<[ToolbarMenuItemData; 24]>,
}
