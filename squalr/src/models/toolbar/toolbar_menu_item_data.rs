pub struct ToolbarMenuItemData {
    pub id: String,
    pub text: String,
    pub has_separator: bool,
    pub check_state: Option<Box<dyn Fn() -> Option<bool> + Send + Sync>>,
}

impl ToolbarMenuItemData {
    pub fn new(
        id: impl Into<String>,
        text: impl Into<String>,
        check_state: Option<Box<dyn Fn() -> Option<bool> + Send + Sync>>,
    ) -> Self {
        Self {
            id: id.into(),
            text: text.into(),
            has_separator: false,
            check_state,
        }
    }

    pub fn with_separator(mut self) -> Self {
        self.has_separator = true;

        self
    }
}
