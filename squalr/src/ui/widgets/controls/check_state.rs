#[derive(Clone, Copy, PartialEq, Eq)]
pub enum CheckState {
    True,
    False,
    Mixed,
}

impl Default for CheckState {
    fn default() -> Self {
        CheckState::False
    }
}

impl CheckState {
    pub fn from_bool(is_checked: bool) -> Self {
        match is_checked {
            true => CheckState::True,
            false => CheckState::False,
        }
    }
}
