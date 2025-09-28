use crate::models::toolbar::toolbar_menu_item_check_state::ToolbarMenuItemCheckState;

#[derive(Clone)]
pub struct ToolbarMenuItemData {
    pub id: String,
    pub text: String,
    pub has_separator: bool,
    pub check_state: ToolbarMenuItemCheckState,
}

impl ToolbarMenuItemData {
    pub fn action(
        id: impl Into<String>,
        text: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            text: text.into(),
            has_separator: false,
            check_state: ToolbarMenuItemCheckState::None,
        }
    }

    pub fn checkable(
        id: impl Into<String>,
        text: impl Into<String>,
        checked: bool,
    ) -> Self {
        Self {
            id: id.into(),
            text: text.into(),
            has_separator: false,
            check_state: if checked {
                ToolbarMenuItemCheckState::Checked
            } else {
                ToolbarMenuItemCheckState::Unchecked
            },
        }
    }

    pub fn with_separator(mut self) -> Self {
        self.has_separator = true;
        self
    }
}
