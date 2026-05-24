use crate::commands::privileged_command::PrivilegedCommand;
use crate::commands::unprivileged_command::UnprivilegedCommand;

#[derive(Clone, Debug)]
pub enum CommandLineCommand {
    Privileged(PrivilegedCommand),
    Unprivileged(UnprivilegedCommand),
}
