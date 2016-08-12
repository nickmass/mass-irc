use super::{Command,CommandType as CT,CommandBuilder};

pub enum UserCommand {
    Nick(String)
}

impl UserCommand {
    pub fn to_command(self) -> Result<Command, String> {
        let b = CommandBuilder::new();
        match self {
            UserCommand::Nick(p) => Ok(b.command(CT::Nick).add_param(p).build().unwrap()),
        }

    }
}
