pub mod stream;
pub use self::stream::TermStream;

use std::io::{Write};
use std::io;


use super::tokio::io::{Readiness, Transport};
use super::tokio::proto::pipeline;

use super::irc::{Command, UserCommand};

pub struct Terminal {
    stream: TermStream,
    read_buf: Vec<u8>,
}



impl Terminal {
    pub fn new(stream: TermStream) -> Terminal {
        Terminal {
            stream: stream,
            read_buf: Vec::new(),
        }
    }
}


impl Readiness for Terminal {
    fn is_readable(&self) -> bool {
        self.stream.is_readable()
    }

    fn is_writable(&self) -> bool {
        self.stream.is_writable()
    }
}

pub type InFrame = pipeline::Frame<Command, io::Error>;
pub type OutFrame = pipeline::Frame<UserCommand, io::Error>;

impl Transport for Terminal {
    type In = InFrame;
    type Out = OutFrame;

    fn read(&mut self) -> io::Result<Option<Self::Out>> {
        if let Some(index) = self.read_buf.iter().position(|x| *x == b'\n') {
            let mut remainder = self.read_buf.split_off(index + 1);
            let mut out_buf = Vec::new();
            out_buf.append(&mut self.read_buf);
            self.read_buf.append(&mut remainder);
            return Ok(Some(pipeline::Frame::Message(
                        UserCommand::Nick(String::from_utf8(out_buf).unwrap())
                        )));
        }

        let mut buf = [0;512];
        if let Ok(bytes) = self.stream.read(&mut buf) {
            try!(self.stream.write_all(&buf));
            self.read_buf.extend_from_slice(&mut buf[0..bytes]);
        }
        Ok(None)
    }

    fn write(&mut self, req: Self::In) -> io::Result<Option<()>>{
        match req {
            pipeline::Frame::Message(cmd) => {
                try!(self.stream.write_all(cmd.to_cmd().as_bytes()));
            },
            _ => {}
        }
        Ok(None)
    }

    fn flush(&mut self) -> io::Result<Option<()>>{
        Ok(None)
    }
}
