use std::thread;

use crossbeam_channel::Receiver;
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
