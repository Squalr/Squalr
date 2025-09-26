use super::install_phase::InstallPhase;

#[derive(Clone, Copy)]
pub struct InstallProgress {
    pub phase: InstallPhase,
    pub progress_percent: f32,
    pub bytes_processed: u64,
    pub total_bytes: u64,
}
