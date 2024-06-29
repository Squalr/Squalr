use structopt::StructOpt;
use crate::command::Command;

#[derive(StructOpt)]
pub struct Cli {
    #[structopt(flatten)]
    pub command: Command
}
