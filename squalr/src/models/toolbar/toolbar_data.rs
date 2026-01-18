use crate::models::toolbar::toolbar_header_item_data::ToolbarHeaderItemData;
use smallvec::SmallVec;

pub struct ToolbarData {
    pub active_menu: String,
    pub menus: SmallVec<[ToolbarHeaderItemData; 16]>,
}
