use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub enum ProjectCommand {
    /// List all projects
    List,
    // Add other project commands here
}
