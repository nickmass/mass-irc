#![feature(lookup_host)]

extern crate mio;
#[macro_use]
extern crate nom;
#[macro_use]
extern crate log;
extern crate env_logger;
extern crate tokio;
extern crate termion;

extern crate clap;
use clap::{App, Arg};

mod irc;
use irc::client::Client;

use std::net;

mod term;
use term::Terminal;

#[cfg(test)]
mod tests;

fn main() {
    env_logger::LogBuilder::new().parse("mass_irc=debug").init().unwrap();

    let matches = App::new("Mass-IRC")
                .version("0.0.1")
                .author("Nick Massey <nickmass@nickmass.com>")
                .about("Simple terminal based IRC client")
                .arg(Arg::with_name("server")
                     .short("s")
                     .long("server")
                     .help("Sets the IRC server to connect to")
                     .takes_value(true)
                     .default_value("127.0.0.1"))
                .arg(Arg::with_name("port")
                     .short("p")
                     .long("port")
                     .help("Sets the server port to connect to")
                     .takes_value(true)
                     .default_value("6667"))
                .arg(Arg::with_name("nick")
                     .short("n")
                     .long("nick")
                     .help("Sets your default nickname")
                     .takes_value(true)
                     .default_value("NickMass"))
                .arg(Arg::with_name("realname")
                     .short("r")
                     .long("realname")
                     .help("Sets your default real name")
                     .takes_value(true)
                     .default_value("Nick Massey"))
                .get_matches();

    let server = matches.value_of("server").unwrap();
    let port = matches.value_of("port").unwrap();
    let nick = matches.value_of("nick").unwrap();
    let realname = matches.value_of("realname").unwrap();

    let ip = match format!("{}:{}", server, port).parse::<net::SocketAddrV4>() {
        Ok(addr) => Some(addr),
        Err(_) => {
            match net::lookup_host(server) {
                Ok(r) => {
                    let mut found_ip = None;
                    let mut r = r;
                    loop {
                        match r.next() {
                            Some(net::SocketAddr::V4(ip)) => {
                                found_ip = Some(ip);
                                break;
                            },
                            Some(net::SocketAddr::V6(_)) => continue,
                            None => break,
                        }
                    }
                    found_ip
                }
                Err(_) => None
            }
        }
    };

    if ip.is_none() {
        panic!("Invalid Server");    
    }

    let ip = ip.unwrap();

    let client = Client::new();
    let tunnel = client.connect(format!("{}:{}", ip.ip(), port).parse().unwrap());
    let mut terminal = Terminal::new(tunnel, nick.to_string(), realname.to_string());
    terminal.run();
}
