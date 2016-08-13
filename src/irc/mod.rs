mod command_type;
pub mod client;
pub mod server;
mod command_parser;
mod command_builder;
mod user_command;
mod command;

pub use super::tokio;
pub use self::user_command::UserCommand;
pub use self::command_parser::CommandParser;
pub use self::command_builder::CommandBuilder;
pub use self::command::Command;
pub use self::command_type::CommandType;
use self::tokio::io::{TryWrite, TryRead, Readiness, Transport};
use self::tokio::proto::pipeline;
use std::io;

struct Irc<T> {
    stream: T,
    read_buf: Vec<u8>,
    write_buf: Vec<u8>,
}

impl<T: TryRead + TryWrite + Readiness> Irc<T> {
    fn new(stream: T) -> Irc<T> {
        let mut write_buf = Vec::new();
        write_buf.append(&mut b"USER nickmass 8 * : Nick Massey\r\n".to_vec());
        write_buf.append(&mut b"NICK nickmass\r\n".to_vec());
        Irc { stream: stream, read_buf: Vec::new(), write_buf: write_buf}
    }
}

impl<T> Readiness for Irc<T> {
    fn is_readable(&self) -> bool {
        self.read_buf.len() > 0
    }

    fn is_writable(&self) -> bool {
        true
    }
}

type Frame = pipeline::Frame<Command, io::Error>;

impl<T: TryRead + TryWrite + Readiness> Transport for Irc<T> {
    type In = Frame;
    type Out = Frame;

    fn read(&mut self) -> io::Result<Option<Self::Out>> {
        if let Some(index) = self.read_buf.iter().position(|x| *x == b'\n') {
            let mut remainder = self.read_buf.split_off(index + 1);

            if self.read_buf.len() > 0 {
                let mut out_buf = Vec::new();
                out_buf.append(&mut self.read_buf);
                self.read_buf.append(&mut remainder);
                return Ok(Some(pipeline::Frame::Message(
                            CommandParser::new(out_buf).parse()
                            )));
            }
            self.read_buf.append(&mut remainder);
        }
        
        if self.stream.is_readable() {
            let mut buf = [0;10];
            if let Ok(()) = self.stream.read_exact(&mut buf) {
                self.read_buf.extend_from_slice(&mut buf);
            }
        }
        Ok(None)
    }

    fn write(&mut self, req: Self::In) -> io::Result<Option<()>>{
        match req {
            pipeline::Frame::Message(cmd) => {
                //self.write_buf.append(&mut cmd.to_cmd().into_bytes());
                if self.write_buf.len() > 0 {
                    if let Some(bytes) = try!(self.stream.try_write(&*self.write_buf)) {
                        println!("{} - {}", bytes, ::std::str::from_utf8(&*self.write_buf).unwrap());
                        self.write_buf = self.write_buf.split_off(bytes);
                        return Ok(Some(()));
                    }
                }
            },
            _ => {}
        }
        Ok(None)
    }

    fn flush(&mut self) -> io::Result<Option<()>>{
        Ok(None)
    }
}
