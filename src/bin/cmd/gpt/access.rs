use super::{Command, SubCommand};
use crate::utils::AccessMode;

pub fn get_access_mode(command: &Command) -> AccessMode {
    match command.cmd {
        SubCommand::Create(_)
        | SubCommand::Add(_)
        | SubCommand::Delete(_)
        | SubCommand::Modify(_) => AccessMode::ReadWrite,
        SubCommand::Dump | SubCommand::Info => AccessMode::ReadOnly,
    }
}
