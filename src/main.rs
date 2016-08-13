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
use term::TermStream;

use std::thread;
use std::time::Duration;
use std::io::Write;

fn main() {
    env_logger::LogBuilder::new().parse("mass_irc=debug").init().unwrap();
    let client = Client::new();
    let mut term = TermStream::new().unwrap();
    let (tun_tx, tun_rx) = client.connect("127.0.0.1:6667".parse().unwrap());

    let mut read_buf = Vec::new();
    loop {
        match tun_rx.try_recv() {
            Ok(d) => { term.write(&*d); },
            _ => ()
        }
        
        if let Some(index) = read_buf.iter().position(|x| *x == 13) {
            let mut remainder = read_buf.split_off(index + 1);
            let mut out_buf = Vec::new();
            out_buf.append(&mut read_buf);
            read_buf.append(&mut remainder);
            tun_tx.send(out_buf);
        } else {
            let mut buf = [0;128];
            if let Ok(bytes) = term.read(&mut buf) {
                term.write_all(&buf).unwrap();
                if bytes > 0 {
                    if buf[0] == 3 {
                        break;
                    }
                    read_buf.extend_from_slice(&mut buf[0..bytes]);
                }
            }
        }
        
        thread::sleep(Duration::from_millis(16));
    }
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
