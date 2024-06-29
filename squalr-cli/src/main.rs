mod cli;
use cli::Cli;
mod command;
mod command_handlers;
use command_handlers::handle_commands;
use shlex;
use squalr_engine::engine_function;
use squalr_engine_memory::normalized_flags::{MemoryProtectionEnum, MemoryTypeEnum};
use std::io::{self, Write};
use structopt::StructOpt;

fn main() {
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
