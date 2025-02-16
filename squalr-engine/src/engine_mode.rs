/// Defines the mode of operation for Squalr's engine.
#[derive(Clone, Copy, PartialEq)]
pub enum EngineMode {
    /// Standalone mode grants full functionality. This is the most common mode.
    Standalone,

    /// In Unprivileged Host mode, we only send and receive engine commands from the privileged shell.
    /// This is necessary on some platforms like Android, where the main process may be unprivileged.
    UnprivilegedHost,

    /// The privileged shell does heavy lifting (scanning, debugging, etc) and sends responses to the host.
    PrivilegedShell,
}
