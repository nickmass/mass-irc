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
        let i = b":irc.example.net NOTICE nickmass :Connection statistics: client 0.0 kb, server 1.3 kb.\r\n".to_vec();
        let c1 = CommandParser::new().parse(&i);
        let c2 = CommandBuilder::new()
            .server_sender("irc.example.net".to_string())
            .command(CommandType::Notice)
            .add_param("nickmass".to_string())
            .add_param("Connection statistics: client 0.0 kb, server 1.3 kb.".to_string())
            .build().unwrap();

        assert_eq!(String::from_utf8(i).unwrap(), c2.to_string());
        assert_eq!(c1.to_string(), c2.to_string());
    }
}
