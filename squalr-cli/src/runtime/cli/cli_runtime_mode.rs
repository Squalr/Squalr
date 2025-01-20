use crate::runtime::runtime_mode::RuntimeMode;
use squalr_engine::cli::Cli;
use squalr_engine::command_handlers::handle_commands;
use squalr_engine_common::logging::log_level::LogLevel;
use squalr_engine_common::logging::logger::Logger;
use std::io;
use std::io::Write;
use structopt::StructOpt;

pub struct CliRuntimeMode {
    stdout: io::Stdout,
}

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

        let mut command = match shlex::split(input) {
            Some(command) => command,
            None => {
                Logger::get_instance().log(LogLevel::Error, "Error parsing input", None);
                return true;
            }
        };

        if command.is_empty() {
            return true;
        }

        // Little bit of a hack, but our command system seems to require the first command to be typed twice so just insert it.
        // We could structopt(flatten) our commands to avoid this, but then this creates even stranger command conflict issues.
        command.insert(0, command[0].clone());

        let mut cli = match Cli::from_iter_safe(&command) {
            Ok(cli) => cli,
            Err(e) => {
                Logger::get_instance().log(LogLevel::Error, &format!("{}", e), None);
                return true;
            }
        };

        handle_commands(&mut cli.command);
        true
    }
}

impl RuntimeMode for CliRuntimeMode {
    fn run(&mut self) -> io::Result<()> {
        let stdin = io::stdin();

        loop {
            self.stdout.flush()?;

            let mut input = String::new();
            stdin.read_line(&mut input)?;

            if !self.handle_input(input.trim()) {
                break;
            }
        }
        Ok(())
    }

    fn shutdown(&mut self) {}
}
