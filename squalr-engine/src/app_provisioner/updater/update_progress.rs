use crate::app_provisioner::updater::update_status::UpdateStatus;

#[derive(Clone)]
pub struct UpdateProgress {
    pub status: UpdateStatus,
    pub bytes_downloaded: u64,
    pub total_bytes: u64,
}
