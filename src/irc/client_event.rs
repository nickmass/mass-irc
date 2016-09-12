use irc::{Sender, Command, CommandType};

pub enum ClientEvent {
    ChannelMessage(String, Option<String>, String),
    PrivateMessage(Option<String>, String),
    JoinChannel(String, Option<String>),
    LeaveChannel(String, Option<String>),
    NoticeMessage(String, Option<String>, String),
    Topic(String, String),
    Names(String, Vec<String>),
    NamesEnd(String),
    Command(Command),
    Connected,
}

impl ClientEvent {
    pub fn from_command(command: &Command) -> Option<ClientEvent> {
        let sender = match command.prefix {
            Some(Sender::Server(ref name)) => Some(name.to_string()),
            Some(Sender::User(ref nick, _, _)) => Some(nick.to_string()),
            _ => None
        };

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
            CommandType::Rpl_Topic => {
                let target = command.get_param(1).unwrap_or("ERROR").to_string();
                let message = command.get_param(2).unwrap_or("ERROR").to_string();
                Some(ClientEvent::Topic(target, message))
            },
            CommandType::Rpl_NoTopic => {
                let target = command.get_param(1).unwrap_or("ERROR").to_string();
                Some(ClientEvent::Topic(target, "".to_string()))
            },
            CommandType::Rpl_NamReply => {
                let target = command.get_param(2).unwrap_or("ERROR").to_string();
                let last = command.get_param(3).unwrap_or("ERROR");
                let names = last.split(' ').map(|x|x.to_string()).collect();
                Some(ClientEvent::Names(target, names))
            },
            CommandType::Rpl_EndOfNames => {
                let target = command.get_param(1).unwrap_or("ERROR").to_string();
                Some(ClientEvent::NamesEnd(target))
            },
            _ => None
        }
    }
}
