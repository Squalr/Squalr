mod cli;
use cli::Cli;
mod command;
mod command_handlers;
mod log_listener; // Add this line to import the LogListener
use command_handlers::handle_commands;
use shlex;
use squalr_engine::engine_function;
use squalr_engine_memory::normalized_flags::{MemoryProtectionEnum, MemoryTypeEnum};
use std::io::{self, Write};
use structopt::StructOpt;
use squalr_engine_common::logging::logger::Logger;
use log_listener::LogListener;
use std::sync::Arc;

fn main() {
    // Initialize logger
    let logger = Logger::new();
    let log_listener = LogListener::new();
    logger.subscribe(log_listener);

    // Log a message to show that it works
    logger.log(squalr_engine_common::logging::log_level::LogLevel::Info, "Logger initialized", None);

    // Temp
    let protection = MemoryProtectionEnum::WRITE | MemoryProtectionEnum::EXECUTE;
    let memory_type = MemoryTypeEnum::PRIVATE | MemoryTypeEnum::IMAGE;
    
    println!("Memory Protection: {:?}", protection);
    println!("Memory Type: {:?}", memory_type);
    println!("{}", engine_function());
    // Temp

    let mut stdout = io::stdout();
    let stdin = io::stdin();

    loop {
        print!("(Sqlr) ");
        stdout.flush().unwrap();

        let mut input = String::new();
        stdin.read_line(&mut input).unwrap();
        let input = input.trim();

        if input.eq_ignore_ascii_case("exit") || input.eq_ignore_ascii_case("close") || input.eq_ignore_ascii_case("quit") {
            break;
        }

        let args = match shlex::split(input) {
            Some(args) => args,
            None => {
                eprintln!("Error parsing input");
                continue;
            }
        };

        let cli = match Cli::from_iter_safe(&args) {
            Ok(cli) => cli,
            Err(e) => {
                eprintln!("{}", e);
                continue;
            }
        };

        handle_commands(cli.command);
    }
}
