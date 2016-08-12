#[macro_use]
extern crate nom;
#[macro_use]
extern crate log;
extern crate env_logger;
extern crate tokio;
extern crate termion;

use std::io::{stdout, stdin};

mod irc;
use irc::client::Client;

mod term;
use term::Terminal;

fn main() {
    env_logger::LogBuilder::new().parse("debug").init().unwrap();
    let term = Terminal::new(stdin(), stdout());
    let client = Client::new(term, "127.0.0.1:6667".parse().unwrap());
    client.connect();
}
