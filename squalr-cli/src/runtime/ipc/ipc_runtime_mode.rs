use squalr_engine::commands::command_handlers::memory::MemoryCommand;
use squalr_engine::commands::engine_command::EngineCommand;
use squalr_engine::squalr_engine::SqualrEngine;
use squalr_engine_common::dynamic_struct::dynamic_struct::DynamicStruct;

use crate::runtime::runtime_mode::RuntimeMode;
use std::sync::mpsc::channel;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;

pub struct IpcRuntimeMode {
    receiver: Receiver<String>,
    _sender: Sender<String>,
    running: bool,
}

/// Implements an inter-process communication runtime mode that listens for remote commands to control the engine.
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
    fn run_loop(&mut self) {
        while self.running {
            match self.receiver.try_recv() {
                Ok(command) => {
                    // TODO: Get the command from IPC channel
                    let mut command = EngineCommand::Memory(MemoryCommand::Read {
                        address: 0,
                        value: DynamicStruct::new(),
                    });

                    SqualrEngine::dispatch_command(&mut command);
                }
                Err(_) => {
                    thread::sleep(std::time::Duration::from_millis(100));
                }
            }
        }
    }

    fn shutdown(&mut self) {
        self.running = false;
    }
}
