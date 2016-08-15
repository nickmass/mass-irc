use super::UserCommand;

pub enum ParseError {
    InputRequired,
    InvalidCommand,
}

#[derive(Debug, Clone, Default)]
struct State {
    active_window: String,
}

pub struct UserInputParser {
}

impl UserInputParser {
    pub fn parse(input: String) -> Result<UserCommand, ParseError> {
        let state = State::default();
        if input.len() == 0 { return Err(ParseError::InputRequired); }
        if !input.starts_with('/') {
            return Ok(UserCommand::PrivMsg(state.active_window.clone(),
                                        input));
        }

        let split: Vec<&str> =  input.splitn(2, ' ').collect();

        let parts = if split.len() == 1 { (split[0], "") } else { (split[0], split[1]) };

        let command = match parts.0 {
            "/nick" => UserCommand::Nick(parts.1.to_string()),
            "/join" | "/j" => UserCommand::Join(parts.1.to_string()),
            "/part" => {
                if parts.1.len() == 0 {
                    UserCommand::Part(state.active_window.clone())
                } else {
                    UserCommand::Part(parts.1.to_string())
                }
            },
            "/msg" => {
                let msg_parts: Vec<&str> = parts.1.splitn(2, ' ').collect();
                UserCommand::PrivMsg(msg_parts.get(0).unwrap().to_string(), msg_parts.get(1).unwrap_or(&"").to_string())
            },
            "/away" => {
                UserCommand::Away(parts.1.to_string())
            },
            "/back" | "/noaway" => {
                UserCommand::Away("".to_string())
            },
            "/whois" => {
                UserCommand::WhoIs(parts.1.to_string())
            },
            _ => {
                println!("{:?}", parts);
                return Err(ParseError::InvalidCommand);
            }

        };

        Ok(command)
    }
}
