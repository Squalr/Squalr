use crate::runtime::runtime_mode::RuntimeMode;
use std::io;
use std::sync::mpsc::channel;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;

pub struct IpcRuntimeMode {
    receiver: Receiver<String>,
    _sender: Sender<String>, // Kept for future use
    running: bool,
}

impl IpcRuntimeMode {
    pub fn new() -> Self {
        let (tx, rx) = channel();
        Self {
            receiver: rx,
            _sender: tx,
            running: true,
        }
    }
}

impl RuntimeMode for IpcRuntimeMode {
    fn run(&mut self) -> io::Result<()> {
        while self.running {
            match self.receiver.try_recv() {
                Ok(command) => {
                    // Handle IPC command
                    println!("Received IPC command: {}", command);
                    // TODO: Implement actual IPC command handling
                }
                Err(_) => {
                    thread::sleep(std::time::Duration::from_millis(100));
                }
            }
        }
        Ok(())
    }

    fn shutdown(&mut self) {
        self.running = false;
    }
}
