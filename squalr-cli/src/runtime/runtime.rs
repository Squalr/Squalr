use crate::runtime::cli::cli_runtime_mode::CliRuntimeMode;
use crate::runtime::ipc::ipc_runtime_mode::IpcRuntimeMode;
use crate::runtime::runtime_mode::RuntimeMode;

pub struct Runtime {
    mode: Box<dyn RuntimeMode>,
}

impl Runtime {
    pub fn new(args: Vec<String>) -> Self {
        let mode: Box<dyn RuntimeMode> = if args.len() > 1 && args[1] == "--ipc" {
            Box::new(IpcRuntimeMode::new())
        } else {
            Box::new(CliRuntimeMode::new())
        };

        Self { mode }
    }

    pub fn run_loop(&mut self) {
        self.mode.run_loop()
    }

    pub fn shutdown(&mut self) {
        self.mode.shutdown()
    }
}
