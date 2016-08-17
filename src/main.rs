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

#[cfg(test)]
mod tests;

fn main() {
    env_logger::LogBuilder::new().parse("mass_irc=debug").init().unwrap();
    let client = Client::new();
    let tunnel = client.connect("127.0.0.1:6667".parse().unwrap());
    let mut terminal = Terminal::new(tunnel);
    terminal.run();
}
