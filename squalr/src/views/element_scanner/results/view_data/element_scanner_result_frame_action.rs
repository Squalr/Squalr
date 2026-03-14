use crate::ui::widgets::controls::check_state::CheckState;
use squalr_engine_api::structures::data_values::anonymous_value_string::AnonymousValueString;

#[derive(Clone, PartialEq)]
pub enum ElementScannerResultFrameAction {
    None,
    SetSelectionStart(Option<i32>),
    SetSelectionEnd(Option<i32>),
    FreezeIndex(i32, bool),
    ToggleFreezeSelection(bool),
    AddSelection,
    AddScanResult(i32),
    DeleteSelection,
    CommitValueToSelection(AnonymousValueString),
}

impl ElementScannerResultFrameAction {
    pub fn from_selection_freeze_checkstate(selection_freeze_checkstate: CheckState) -> Self {
        match selection_freeze_checkstate {
            CheckState::False => Self::ToggleFreezeSelection(true),
            CheckState::Mixed | CheckState::True => Self::ToggleFreezeSelection(false),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ElementScannerResultFrameAction;
    use crate::ui::widgets::controls::check_state::CheckState;

    #[test]
    fn from_selection_freeze_checkstate_freezes_unfrozen_selection() {
        assert!(
            ElementScannerResultFrameAction::from_selection_freeze_checkstate(CheckState::False)
                == ElementScannerResultFrameAction::ToggleFreezeSelection(true)
        );
    }

    #[test]
    fn from_selection_freeze_checkstate_unfreezes_mixed_selection() {
        assert!(
            ElementScannerResultFrameAction::from_selection_freeze_checkstate(CheckState::Mixed)
                == ElementScannerResultFrameAction::ToggleFreezeSelection(false)
        );
    }

    #[test]
    fn from_selection_freeze_checkstate_unfreezes_frozen_selection() {
        assert!(
            ElementScannerResultFrameAction::from_selection_freeze_checkstate(CheckState::True)
                == ElementScannerResultFrameAction::ToggleFreezeSelection(false)
        );
    }
}
