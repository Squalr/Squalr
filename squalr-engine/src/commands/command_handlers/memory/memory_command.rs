use squalr_engine_common::conversions::parse_hex_or_int;
use squalr_engine_common::dynamic_struct::dynamic_struct::DynamicStruct;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub enum MemoryCommand {
    Read {
        #[structopt(short = "a", long, parse(try_from_str = parse_hex_or_int))]
        address: u64,
        #[structopt(short = "v", long)]
        value: DynamicStruct,
    },
    Write {
        #[structopt(short = "a", long, parse(try_from_str = parse_hex_or_int))]
        address: u64,
        #[structopt(short = "v", long)]
        value: DynamicStruct,
    },
}
