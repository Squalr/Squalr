#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ValidatedPointerCandidateState {
    Invalid,
    Valid { current_pointer_value: u64 },
}
