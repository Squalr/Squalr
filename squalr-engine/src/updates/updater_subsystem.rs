use super::updater::check_for_updates_task::CheckForUpdatesTask;
use super::updater::perform_update_task::PerformUpdateTask;
use super::updater::update_status::UpdateStatus;
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

        /*
        CheckForUpdatesTask::run(move |status| {
            if let Ok(subscribers) = subscribers.read() {
                for sender in subscribers.iter() {
                    let _ = sender.send(status.clone());
                }
            }

            if let UpdateStatus::UpdateAvailable(_) = status {
                Self::perform_update(subscribers.clone());
            }
        });*/
    }

    fn perform_update(subscribers: Arc<RwLock<Vec<Sender<UpdateStatus>>>>) {
        /*
        PerformUpdateTask::run(move |status| {
            if let Ok(subscribers) = subscribers.read() {
                for sender in subscribers.iter() {
                    let _ = sender.send(status.clone());
                }
            }
        });*/
    }

    pub fn subscribe(&self) -> Receiver<UpdateStatus> {
        let (sender, receiver) = mpsc::channel::<UpdateStatus>();

        if let Ok(mut senders) = self.subscriber_senders.write() {
            senders.push(sender);
        }

        receiver
    }
}
