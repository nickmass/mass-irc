extern crate mio;
#[macro_use]
extern crate nom;
#[macro_use]
extern crate log;
extern crate env_logger;
extern crate tokio;
extern crate termion;


mod irc;
use irc::client::Client;

mod term;
use term::Terminal;

fn main() {
    env_logger::LogBuilder::new().parse("mass_irc=debug").init().unwrap();
    let client = Client::new();
    let tunnel = client.connect("127.0.0.1:6667".parse().unwrap());
    let mut terminal = Terminal::new(tunnel);
    terminal.run();
}


#[cfg(test)]
mod tests {
    use irc::{CommandParser, CommandBuilder, CommandType};
    #[test]
    fn parser_full() {
        let i = b":irc.example.net NOTICE nickmass :Helloo There Nick\r\n".to_vec();
        let c1 = CommandParser::new().parse(&i);
        let i = b":irc.example.net NOTICE nickmass asdad\r\n".to_vec();
        let c1 = CommandParser::new().parse(&i);
        let i = b":irc.example.net NOTICE nickmass :\r\n".to_vec();
        let c1 = CommandParser::new().parse(&i);
        let i = b"NOTICE nickmass one two th:ree: :Trailing\r\n".to_vec();
        let c1 = CommandParser::new().parse(&i);
        let i = b"NOTICE one two three\r\n".to_vec();
        let c1 = CommandParser::new().parse(&i);
        let i = b":irc.example.net 001 something\r\n".to_vec();
        let c1 = CommandParser::new().parse(&i);
        let i = b"NOTICE :Trailing\r\n".to_vec();
        let c1 = CommandParser::new().parse(&i);
    }
}
