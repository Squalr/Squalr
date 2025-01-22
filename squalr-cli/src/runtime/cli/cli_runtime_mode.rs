use crate::runtime::runtime_mode::RuntimeMode;
use squalr_engine::commands::engine_command::EngineCommand;
use squalr_engine::squalr_engine::SqualrEngine;
use squalr_engine_common::logging::log_level::LogLevel;
use squalr_engine_common::logging::logger::Logger;
use std::io;
use std::io::Write;
use structopt::StructOpt;

pub struct CliRuntimeMode {
    stdout: io::Stdout,
}

/// Implements a command line runtime mode that listens for text input commands to control the engine.
impl CliRuntimeMode {
    pub fn new() -> Self {
        Self { stdout: io::stdout() }
    }

    fn handle_input(
        &self,
        input: &str,
    ) -> bool {
        if input.eq_ignore_ascii_case("exit") || input.eq_ignore_ascii_case("close") || input.eq_ignore_ascii_case("quit") {
            return false;
        }

        let mut cli_command = match shlex::split(input) {
            Some(cli_command) => cli_command,
            None => {
                Logger::get_instance().log(LogLevel::Error, "Error parsing input", None);
                return true;
            }
        };

        if cli_command.is_empty() {
            return true;
        }

        // Little bit of a hack, but our command system seems to require the first command to be typed twice so just insert it.
        // We could structopt(flatten) our commands to avoid this, but then this creates even stranger command conflict issues.
        cli_command.insert(0, cli_command[0].clone());

        let mut engine_command = match EngineCommand::from_iter_safe(&cli_command) {
            Ok(engine_command) => engine_command,
            Err(e) => {
                Logger::get_instance().log(LogLevel::Error, &format!("{}", e), None);
                return true;
            }
        };

        SqualrEngine::dispatch_command(&mut engine_command);
        true
    }
}

impl RuntimeMode for CliRuntimeMode {
    fn run_loop(&mut self) {
        let stdin = io::stdin();

        loop {
            if let Err(err) = self.stdout.flush() {
                Logger::get_instance().log(LogLevel::Error, &format!("Error flushing stdout {}", err), None);
                break;
            }

            let mut input = String::new();
            if let Err(err) = stdin.read_line(&mut input) {
                Logger::get_instance().log(LogLevel::Error, &format!("Error reading input {}", err), None);
                break;
            }

            if !self.handle_input(input.trim()) {
                break;
            }
        }
    }

    fn shutdown(&mut self) {}
}
