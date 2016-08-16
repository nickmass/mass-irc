use irc::{Sender, Command, CommandType};

pub enum ClientEvent {
    ChannelMessage(String, String, String),
    PrivateMessage(String, String),
    JoinChannel(String, String),
    LeaveChannel(String, String),
    NoticeMessage(String, String, String),
    Topic(String, String),
    Command(Command),
}

impl ClientEvent {
    pub fn from_command(command: &Command) -> Option<ClientEvent> {
        let sender = match command.prefix {
            Some(Sender::Server(ref name)) => name,
            Some(Sender::User(ref nick, _, _)) => nick,
            _ => ""
        };

        let sender = sender.to_string();

        match command.command {
            CommandType::PrivMsg => {
                let target = command.get_param(0).unwrap_or("ERROR").to_string();
                let message = command.get_param(1).unwrap_or("ERROR").to_string();

                if target.starts_with("#") {
                    Some(ClientEvent::ChannelMessage(target, sender, message))
                } else {
                    Some(ClientEvent::PrivateMessage(sender, message))
                }
            },
            CommandType::Join => {
                let target = command.get_param(0).unwrap_or("ERROR").to_string();
                Some(ClientEvent::JoinChannel(target, sender))
            },
            CommandType::Part => {
                let target = command.get_param(0).unwrap_or("ERROR").to_string();
                Some(ClientEvent::LeaveChannel(target, sender))
            },
            CommandType::Notice => {
                let target = command.get_param(0).unwrap_or("ERROR").to_string();
                let message = command.get_param(1).unwrap_or("ERROR").to_string();

                Some(ClientEvent::NoticeMessage(target, sender, message))
            },
            CommandType::Topic => {
                let target = command.get_param(0).unwrap_or("ERROR").to_string();
                let message = command.get_param(1).unwrap_or("ERROR").to_string();

                Some(ClientEvent::Topic(target, message))
            },
            _ => None
        }
    }
}
