use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub enum ScanCommand {
    /// Scan for a specific value
    Value {
        value: i32,
    },
    /// Collect values for the current scan if one exist, otherwise collect for a new scan
    Collect,
}
