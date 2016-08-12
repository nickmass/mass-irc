#[macro_use]
extern crate nom;
#[macro_use]
extern crate log;
extern crate env_logger;

mod command_type;
mod parse;
use parse::*;

mod client;
use client::Client;

fn main() {
    env_logger::LogBuilder::new().parse("debug").init().unwrap();
    Client::start();

    return;

    let i = b"@asdad=asdad;123123=qqqq :nick!user@example.com PRIVMSG #channel :https://example.com/a-news-story\r\n".to_vec();
    println!("Input:  {:?}", String::from_utf8(i.clone()).unwrap());
    let parser = CommandParser::new(i);
    let r = parser.parse();

    println!("Output: {:?}", r.to_cmd());
    println!("Output: {:?}", r);
}
