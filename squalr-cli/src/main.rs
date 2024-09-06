mod cli;
use cli::Cli;
mod cli_log_listener;
mod command;
mod command_handlers;

use cli_log_listener::CliLogListener;
use command_handlers::handle_commands;
use shlex;
use squalr_engine_architecture::vectors::vectors;
use squalr_engine_common::logging::log_level::LogLevel;
use squalr_engine_common::logging::logger::Logger;
use std::io::{self, Write};
use structopt::StructOpt;

fn main() {
    // Initialize cli log listener to route log output to command line
    let cli_log_listener = CliLogListener::new();

    Logger::get_instance().subscribe(cli_log_listener);
    Logger::get_instance().log(LogLevel::Info, "Logger initialized", None);
    vectors::log_vector_architecture();

    let mut stdout = io::stdout();
    let stdin = io::stdin();

    loop {
        stdout.flush().unwrap();

        let mut input = String::new();
        stdin.read_line(&mut input).unwrap();
        let input = input.trim();

        if input.eq_ignore_ascii_case("exit") || input.eq_ignore_ascii_case("close") || input.eq_ignore_ascii_case("quit") {
            break;
        }

        let mut args = match shlex::split(input) {
            Some(args) => args,
            None => {
                Logger::get_instance().log(LogLevel::Error, "Error parsing input", None);
                continue;
            }
        };

        if args.is_empty() {
            continue;
        }

        // Little bit of a hack, but our command system seems to require the first command to be typed twice so just insert it.
        // We could structopt(flatten), our commands to avoid this, but then this creates even stranger command conflict issues.
        args.insert(0, args[0].clone());

        let mut cli = match Cli::from_iter_safe(&args) {
            Ok(cli) => cli,
            Err(e) => {
                Logger::get_instance().log(LogLevel::Error, &format!("{}", e), None);
                continue;
            }
        };

        handle_commands(&mut cli.command);
    }
}
