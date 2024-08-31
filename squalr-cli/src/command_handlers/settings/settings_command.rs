use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub enum SettingsCommand {
    List {},
    Set {},
}
