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
        Irc { stream: stream, read_buf: Vec::new(), write_buf: Vec::new()}
    }
}

impl<T> Readiness for Irc<T> {
    fn is_readable(&self) -> bool {
        true
    }

    fn is_writable(&self) -> bool {
        true
    }
}

type Frame = pipeline::Frame<Command, io::Error>;
type OutFrame = pipeline::Frame<Vec<Command>, io::Error>;

impl<T: TryRead + TryWrite + Readiness> Transport for Irc<T> {
    type In = Frame;
    type Out = OutFrame;

    fn read(&mut self) -> io::Result<Option<Self::Out>> {
        let mut res = Vec::new();
        loop {
            if let Some(index) = self.read_buf.iter().position(|x| *x == b'\n') {
                let mut remainder = self.read_buf.split_off(index + 1);
                let mut out_buf = Vec::new();
                out_buf.append(&mut self.read_buf);
                self.read_buf.append(&mut remainder);
                res.push(CommandParser::new(out_buf).parse());
            } else {
                if res.len() > 0 {
                    return Ok(Some(pipeline::Frame::Message(res)));
                }
                break;
            }
        }
        let mut buf = [0;0x4096];
        if let Some(bytes) = try!(self.stream.try_read(&mut buf)) {
            self.read_buf.extend_from_slice(&mut buf[0..bytes]);
        }
        Ok(None)
    }

    fn write(&mut self, req: Self::In) -> io::Result<Option<()>>{
        match req {
            pipeline::Frame::Message(cmd) => {
                self.write_buf.append(&mut cmd.to_cmd().into_bytes());
                if self.write_buf.len() > 0 {
                    if let Some(bytes) = try!(self.stream.try_write(&*self.write_buf)) {
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
        if self.write_buf.len() > 0 {
            try!(self.stream.write_all(&*self.write_buf));
            try!(self.stream.flush());
            self.write_buf = Vec::new();
        }
        Ok(Some(()))
    }
}
