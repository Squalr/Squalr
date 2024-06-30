
use squalr_engine_common::dynamic_struct::dynamic_struct::DynamicStruct;

use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub enum MemoryCommand {
    Read {
        #[structopt(short = "a", long)]
        address: u64,
        #[structopt(short = "v", long)]
        value: DynamicStruct,
    },
    Write {
        #[structopt(short = "a", long)]
        address: u64,
        #[structopt(short = "v", long)]
        value: DynamicStruct,
    },
}
