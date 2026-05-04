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

#[cfg(test)]
mod tests {
    use super::MainWindowTakeOverState;

    #[test]
    fn default_take_over_state_is_none() {
        assert_eq!(MainWindowTakeOverState::default(), MainWindowTakeOverState::None);
    }

    #[test]
    fn about_take_over_state_is_active() {
        assert!(MainWindowTakeOverState::About.is_active());
    }
}
