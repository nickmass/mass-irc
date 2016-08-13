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
pub use term::Terminal;

fn main() {
    env_logger::LogBuilder::new().parse("mass_irc=debug").init().unwrap();
    let client = Client::new();
    client.connect("127.0.0.1:6667".parse().unwrap());

    loop{}
}


#[cfg(test)]
mod tests {

    use irc::{CommandParser, CommandBuilder, CommandType};
    #[test]
    fn parserone() {
        let i = b":irc.example.net NOTICE nickmass :Connection.\r\n";
        let c1 = CommandParser::new(i.to_vec()).parse();
        let c2 = CommandBuilder::new()
            .server_sender("irc.example.net".to_string())
            .command(CommandType::Notice)
            .add_param("nickmass".to_string())
            .add_param("Connection.".to_string())
            .build().unwrap();

        assert_eq!(String::from_utf8(i.to_vec()).unwrap(), c2.to_cmd());

    }

    #[test]
    fn parser() {
        let i = b":irc.example.net NOTICE nickmass :Connection statistics: client 0.0 kb, server 1.3 kb.\r\n";
        let c1 = CommandParser::new(i.to_vec()).parse();
        let c2 = CommandBuilder::new()
            .server_sender("irc.example.net".to_string())
            .command(CommandType::Notice)
            .add_param("nickmass".to_string())
            .add_param("Connection statistics: client 0.0 kb, server 1.3 kb.".to_string())
            .build().unwrap();

        assert_eq!(String::from_utf8(i.to_vec()).unwrap(), c2.to_cmd());

    }
}
