#[macro_use]
extern crate nom;
#[macro_use]
extern crate log;
extern crate mio;
extern crate termion;

#[macro_use]
extern crate clap;
use clap::{App, Arg};

mod irc;
use irc::Client;

mod term;
use term::Terminal;

#[cfg(test)]
mod tests;

fn main() {
    let matches = App::new("Mass-IRC")
                .version(crate_version!())
                .author(crate_authors!())
                .about("Simple terminal based IRC client")
                .arg(Arg::with_name("server")
                     .short("s")
                     .long("server")
                     .help("Sets the IRC server to connect to")
                     .takes_value(true)
                     .default_value("localhost"))
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
    let port: u16 = matches.value_of("port").unwrap().parse().unwrap();
    let nick = matches.value_of("nick").unwrap();
    let realname = matches.value_of("realname").unwrap();

    let client = Client::connect((server, port));

    let mut terminal = Terminal::new(client, nick.to_string(), realname.to_string());
    let _ = terminal.init_log();
    terminal.run();
}
