use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub enum ScanCommand {
    /// Scan for a specific value
    Value {
        value: i32,
    },
    // Add other scan commands here
}
