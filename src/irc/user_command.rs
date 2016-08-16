use irc::{Command,CommandType as CT,CommandBuilder};

pub enum UserCommand {
    Nick(String),
    User(String, String, String, String),
    Join(String),
    PrivMsg(String, String),
    WhoIs(String),
    Away(String),
    Part(String),
    Quit(String),
    GetTopic(String),
    SetTopic(String, String),
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
            UserCommand::Join(chan) => Ok(b.command(CT::Join)
                                       .add_param(chan)
                                       .build().unwrap()),
            UserCommand::PrivMsg(targ, msg) => Ok(b.command(CT::PrivMsg)
                                       .add_param(targ)
                                       .add_param(msg)
                                       .build().unwrap()),
            UserCommand::WhoIs(targ) => Ok(b.command(CT::WhoIs)
                                       .add_param(targ)
                                       .build().unwrap()),
            UserCommand::Away(msg) => Ok(b.command(CT::Away)
                                       .add_param(msg)
                                       .build().unwrap()),
            UserCommand::Part(targ) => Ok(b.command(CT::Part)
                                       .add_param(targ)
                                       .build().unwrap()),
            UserCommand::Quit(msg) => Ok(b.command(CT::Quit)
                                       .add_param(msg)
                                       .build().unwrap()),
            UserCommand::GetTopic(chan) => Ok(b.command(CT::Topic)
                                            .add_param(chan)
                                            .build().unwrap()),
            UserCommand::SetTopic(chan, topic) => Ok(b.command(CT::Topic)
                                                     .add_param(chan)
                                                     .add_param(topic)
                                                     .build().unwrap()),
        }

    }
}
