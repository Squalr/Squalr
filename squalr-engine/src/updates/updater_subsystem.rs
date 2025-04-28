use crate::updates::updater::perform_update_task::PerformUpdateTask;
use crate::updates::updater::update_status::UpdateStatus;
use crate::updates::version_checker::version_checker_status::VersionCheckerStatus;
use crate::updates::version_checker::version_checker_task::VersionCheckerTask;
use semver::Version;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, RwLock};

pub struct UpdaterSubsystem {
    subscriber_senders: Arc<RwLock<Vec<Sender<UpdateStatus>>>>,
}

impl UpdaterSubsystem {
    pub fn new() -> Self {
        Self {
            subscriber_senders: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub fn initialize(&mut self) {
        let subscribers = self.subscriber_senders.clone();

        VersionCheckerTask::run(move |status| {
            if let Ok(subscribers) = subscribers.read() {
                for sender in subscribers.iter() {
                    let _ = sender.send(UpdateStatus::CheckingVersion(status.clone()));
                }
            }

            if let VersionCheckerStatus::LatestVersionFound(latest_version) = status {
                let current_version = Version::parse(env!("CARGO_PKG_VERSION")).unwrap();

                /*
                if latest_version > current_version {
                    log::info!("An update is available.");
                    Self::perform_update(subscribers.clone());
                } else {
                    log::info!("App is up to date, no update is required.");
                }*/
            }
        });
    }

    fn perform_update(subscribers: Arc<RwLock<Vec<Sender<UpdateStatus>>>>) {
        PerformUpdateTask::run(move |status| {
            if let Ok(subscribers) = subscribers.read() {
                for sender in subscribers.iter() {
                    let _ = sender.send(status.clone());
                }
            }
        });
    }

    pub fn subscribe(&self) -> Receiver<UpdateStatus> {
        let (sender, receiver) = mpsc::channel::<UpdateStatus>();

        if let Ok(mut senders) = self.subscriber_senders.write() {
            senders.push(sender);
        }

        receiver
    }
}
