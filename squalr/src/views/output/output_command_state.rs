use std::collections::VecDeque;

#[derive(Clone, Debug)]
pub struct OutputCommandState {
    command_text: String,
    command_history: VecDeque<String>,
    history_cursor: Option<usize>,
    max_history_len: usize,
}

impl OutputCommandState {
    pub fn new() -> Self {
        Self {
            command_text: String::new(),
            command_history: VecDeque::new(),
            history_cursor: None,
            max_history_len: 128,
        }
    }

    pub fn command_text_mut(&mut self) -> &mut String {
        &mut self.command_text
    }

    pub fn command_text(&self) -> &str {
        &self.command_text
    }

    pub fn has_pending_command(&self) -> bool {
        !self.command_text.trim().is_empty()
    }

    pub fn submit_command(&mut self) -> Option<String> {
        let command_text = self.command_text.trim().to_string();

        if command_text.is_empty() {
            return None;
        }

        self.push_history(command_text.clone());
        self.command_text.clear();
        self.history_cursor = None;

        Some(command_text)
    }

    pub fn navigate_previous(&mut self) {
        if self.command_history.is_empty() {
            return;
        }

        let previous_cursor = match self.history_cursor {
            Some(history_cursor) => history_cursor.saturating_sub(1),
            None => self.command_history.len().saturating_sub(1),
        };

        self.history_cursor = Some(previous_cursor);

        if let Some(history_command) = self.command_history.get(previous_cursor) {
            self.command_text.clone_from(history_command);
        }
    }

    pub fn navigate_next(&mut self) {
        let Some(history_cursor) = self.history_cursor else {
            return;
        };

        let next_cursor = history_cursor + 1;

        if next_cursor >= self.command_history.len() {
            self.history_cursor = None;
            self.command_text.clear();
            return;
        }

        self.history_cursor = Some(next_cursor);

        if let Some(history_command) = self.command_history.get(next_cursor) {
            self.command_text.clone_from(history_command);
        }
    }

    fn push_history(
        &mut self,
        command_text: String,
    ) {
        if self
            .command_history
            .back()
            .is_some_and(|previous_command| previous_command == &command_text)
        {
            return;
        }

        while self.command_history.len() >= self.max_history_len {
            self.command_history.pop_front();
        }

        self.command_history.push_back(command_text);
    }
}

impl Default for OutputCommandState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::OutputCommandState;

    #[test]
    fn submit_command_trims_and_clears_text() {
        let mut state = OutputCommandState::new();
        *state.command_text_mut() = String::from("  process list  ");

        let command_text = state.submit_command();

        assert_eq!(command_text.as_deref(), Some("process list"));
        assert!(!state.has_pending_command());
    }

    #[test]
    fn submit_command_ignores_empty_text() {
        let mut state = OutputCommandState::new();
        *state.command_text_mut() = String::from("   ");

        assert!(state.submit_command().is_none());
    }

    #[test]
    fn history_navigation_walks_back_and_forward() {
        let mut state = OutputCommandState::new();
        *state.command_text_mut() = String::from("process list");
        state.submit_command();
        *state.command_text_mut() = String::from("project list");
        state.submit_command();

        state.navigate_previous();
        assert_eq!(state.command_text_mut(), "project list");

        state.navigate_previous();
        assert_eq!(state.command_text_mut(), "process list");

        state.navigate_next();
        assert_eq!(state.command_text_mut(), "project list");

        state.navigate_next();
        assert_eq!(state.command_text_mut(), "");
    }
}
