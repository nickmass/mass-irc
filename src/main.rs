extern crate mio;
#[macro_use]
extern crate nom;
#[macro_use]
extern crate log;
extern crate env_logger;
extern crate tokio;
extern crate termion;

use termion::{color, cursor, terminal_size, clear};

mod irc;
use irc::client::Client;

mod term;
use term::TermStream;

use std::collections::VecDeque;
use std::thread;
use std::time::Duration;
use std::io::Write;

fn main() {
    env_logger::LogBuilder::new().parse("mass_irc=debug").init().unwrap();
    let client = Client::new();
    let mut term = TermStream::new().unwrap();
    term.write_all(format!("{}", clear::All).as_bytes());
    let (tun_tx, tun_rx) = client.connect("127.0.0.1:6667".parse().unwrap());

    let mut message_history = VecDeque::new();
    let mut read_buf = Vec::new();
    let mut line_buf = Vec::new();
    loop {
        loop {
            match tun_rx.try_recv() {
                Ok(d) => { message_history.push_front(d); },
                _ => break
            }
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
                if bytes > 0 {
                    if buf[0] == 3 {
                        break;
                    }
                    line_buf.extend_from_slice(&mut buf[0..bytes]);
                    read_buf.extend_from_slice(&mut buf[0..bytes]);
                }
            }
        }

        fn render(message_history: &mut VecDeque<Vec<u8>>, term: &mut TermStream, line: &mut Vec<u8>) {
            let size = terminal_size().unwrap();

            let mut messages_out = Vec::new();
            let mut rows = (size.1 - 1) as usize;
            let width = size.0 as usize;

            let iter = message_history.as_slices().0.iter().rev();

            let mut msgs = 0;
            for msg in iter {
                rows -= (msg.len()-2) as usize / width + 1;
                if rows <= 0 {
                    break;
                }
                msgs += 1;
            }

            while rows > 0 {
                rows -= 1;
                messages_out.push(b'\n');
            }
            let spaces = [b' '; 2000];
            let mut iter: Vec<&Vec<u8>> = message_history.as_slices().0.iter().take(msgs).collect();
            iter.reverse();
            for msg in iter {
                let space = width - ((msg.len()-2) % width);
                messages_out.extend_from_slice(&msg[0..msg.len()-2]);
                messages_out.extend_from_slice(&spaces[0..space]);
                messages_out.append(&mut b"\r\n".to_vec());
            }

            let mut vis_line = Vec::new();
            for c in &*line {
                match c {
                    &127 => { if vis_line.len() != 0 { vis_line.pop(); }},
                    &13 => { vis_line.clear(); },
                    _ => { vis_line.push(*c); },

                }
            }

            let vis_end = (vis_line.len()+1) as u16;
            while vis_line.len() < width {
                vis_line.push(b' ');
            }

            term.write_all(&*format!("{}{}{}{}{}{}{}",
                                     cursor::Goto(1,1),
                                     color::Fg(color::White),
                                     String::from_utf8(messages_out).unwrap(),
                                     cursor::Goto(1,size.1),
                                     color::Fg(color::LightWhite),
                                     String::from_utf8(vis_line).unwrap(),
                                     cursor::Goto(vis_end ,size.1)
                                    ).into_bytes());
            term.flush();
        }

        render(&mut message_history, &mut term, &mut line_buf);
        thread::sleep(Duration::from_millis(16));
    }
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
