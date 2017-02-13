mod command_type;
pub mod client;
pub mod server;
mod command_parser;
mod command_builder;
mod input_parser;
mod user_command;
mod command;
mod client_event;

pub use self::input_parser::UserInputParser;
pub use self::user_command::UserCommand;
pub use self::command_parser::CommandParser;
pub use self::command_builder::CommandBuilder;
pub use self::command::{Command, Sender};
pub use self::command_type::CommandType;
pub use self::client_event::{ClientEvent};
//pub use self::client::Client;
pub use self::client::tokio::Client;

use std::io::{self, Read, Write};

use mio::{Evented, Poll, PollOpt, Ready, Token};

pub const CLIENT: Token = Token(0);
pub const CLIENT_MSG: Token = Token(1);

pub struct Irc<'a, T> {
    stream: T,
    poll: &'a Poll,
    read_buf: Vec<u8>,
    write_buf: Vec<u8>,
    parser: CommandParser,
}

impl<'a, T: Read + Write + Evented> Irc<'a, T> {
    fn new(stream: T, poll: &'a Poll) -> Irc<'a, T> {
        poll.register(&stream,
                      CLIENT,
                      Ready::readable(),
                      PollOpt::edge()).unwrap();
        Irc {
            stream: stream,
            poll: poll,
            read_buf: Vec::new(),
            write_buf: Vec::new(),
            parser: CommandParser::new(),
        }
    }

    fn read(&mut self) -> Result<Option<Command>, io::Error> {
        self.register();

        let mut buf = [0;0x4096];
        if let Ok(bytes) = self.stream.read(&mut buf) {
            if bytes > 0 {
                self.read_buf.extend_from_slice(&mut buf[0..bytes]);
            }
        }

        if let Some(index) = self.read_buf.iter().position(|x| *x == b'\n') {
            let mut remainder = self.read_buf.split_off(index + 1);
            let res = self.parser.parse(&self.read_buf);
            self.read_buf.clear();
            self.read_buf.append(&mut remainder);
            return Ok(Some(res));
        }

        Ok(None)
    }

    fn buf_write(&mut self, req: Command) {
        self.write_buf.append(&mut req.to_string().into_bytes());
        self.register();
    }

    fn register(&self) {
        if self.write_buf.len() > 0 {
            self.poll.reregister(&self.stream,
                                 CLIENT,
                                 Ready::readable() | Ready::writable(),
                                 PollOpt::edge()).unwrap();
        } else {
            self.poll.reregister(&self.stream,
                                 CLIENT,
                                 Ready::readable(),
                                 PollOpt::edge()).unwrap();
        }
    }

    fn write(&mut self) -> Result<(), io::Error> {
        if self.write_buf.len() > 0 {
            let bytes = try!(self.stream.write(&*self.write_buf));
            if bytes > 0 {
                self.write_buf = self.write_buf.split_off(bytes);
                self.register();
                return Ok(());
            }
        }
        Ok(())
    }

    fn flush(&mut self) -> Result<(), io::Error> {
        if self.write_buf.len() > 0 {
            try!(self.stream.write_all(&*self.write_buf));
            try!(self.stream.flush());
            self.write_buf = Vec::new();
        }
        Ok(())
    }
}
