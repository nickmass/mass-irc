use super::{Command,CommandType as CT,CommandBuilder};

pub enum UserCommand {
    Nick(String),
    User(String, String, String, String),
}

impl UserCommand {
    pub fn to_command(self) -> Result<Command, String> {
        let b = CommandBuilder::new();
        match self {
            UserCommand::Nick(nick) => Ok(b.command(CT::Nick)
                                       .add_param(nick)
                                       .build().unwrap()),
            UserCommand::User(nick, p, p1, name) => Ok(b.command(CT::User)
                                       .add_param(nick)
                                       .add_param(p)
                                       .add_param(p1)
                                       .add_param(name)
                                       .build().unwrap()),
        }

    }
}
