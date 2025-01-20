use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub enum SettingsCommand {
    List {
        #[structopt(short = "s", long)]
        scan: bool,
        #[structopt(short = "m", long)]
        memory: bool,
        #[structopt(short = "a", long)]
        list_all: bool,
    },
    Set {
        #[structopt(name = "setting")]
        setting_command: String,
    },
}
