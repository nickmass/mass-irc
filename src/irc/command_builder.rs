use super::{CommandType};
use super::command::*;

pub struct CommandBuilder {
    sender: Option<Sender>,
    command_type: Option<CommandType>,
    params: Vec<String>,
    tags: Vec<Tag>,
}


impl CommandBuilder {
    pub fn new() -> CommandBuilder {
        CommandBuilder {
            sender: None,
            command_type: None,
            params: Vec::new(),
            tags: Vec::new(),
        }
    }

    pub fn user_sender(mut self, nick: String, user: Option<String>, host: Option<String>) -> Self {
        self.sender = Some(Sender::User(nick, user, host));

        self
    }

    pub fn server_sender(mut self, host: String) -> Self {
        self.sender = Some(Sender::Server(host));

        self
    }

    pub fn command(mut self, cmd: CommandType) -> Self {
        self.command_type = Some(cmd);

        self
    }

    pub fn add_param(mut self, param: String) -> Self {
        self.params.push(param);

        self
    }

    pub fn add_params(mut self, params: Vec<String>) -> Self {
        let mut params = params;
        self.params.append(&mut params);

        self
    }

    pub fn add_tag(mut self, key: String, value: String) -> Self {
        self.tags.push(Tag { key: key, value: value});

        self
    }

    pub fn build(self) -> Option<Command> { //TODO make result
        if self.command_type.is_none() {
            return None;
        }
        
        let params = Params { data: self.params };

        let tags = if self.tags.len() > 0 {
            Some(Tags { data: self.tags})
        } else {
            None
        };

        Some(Command {
            tags: tags,
            prefix: self.sender,
            command: self.command_type.unwrap(),
            params: params
        })
    }
}
