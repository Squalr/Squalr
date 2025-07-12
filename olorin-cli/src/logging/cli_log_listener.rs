use crossbeam_channel::Receiver;
use std::thread;

pub struct CliLogListener {}

impl CliLogListener {
    pub fn new(log_receiver: Receiver<String>) -> Self {
        let cli_log_listener = Self {};

        thread::spawn(move || {
            while let Ok(log_message) = log_receiver.recv() {
                println!("{}", log_message);
            }
        });

        cli_log_listener
    }
}
