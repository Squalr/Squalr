use crate::ui::widgets::controls::toolbar_menu::toolbar_menu_check_state::ToolbarMenuCheckState;

#[derive(Clone)]
pub struct ToolbarMenuItemData {
    pub id: String,
    pub text: String,
    pub has_separator: bool,
    pub check_state: ToolbarMenuCheckState,
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
            check_state: ToolbarMenuCheckState::None,
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
                ToolbarMenuCheckState::Checked
            } else {
                ToolbarMenuCheckState::Unchecked
            },
        }
    }

    pub fn with_separator(mut self) -> Self {
        self.has_separator = true;
        self
    }
}
