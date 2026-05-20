/// Controls which project refresh mechanisms are active for a session.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ProjectRefreshConfig {
    /// Emits lightweight events for project mutations performed through Squalr commands.
    pub emit_internal_project_events: bool,

    /// Watches project folders for external file-system changes.
    pub watch_file_system: bool,
}

impl Default for ProjectRefreshConfig {
    fn default() -> Self {
        Self {
            emit_internal_project_events: true,
            watch_file_system: false,
        }
    }
}
