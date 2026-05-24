#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub enum MainWindowTakeOverState {
    #[default]
    None,
    About,
}

impl MainWindowTakeOverState {
    pub fn is_active(&self) -> bool {
        !matches!(self, Self::None)
    }
}
