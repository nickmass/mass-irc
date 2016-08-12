use super::termion::input::{TermRead};
use super::termion::raw::{IntoRawMode, RawTerminal};
use std::io::{Read, Write};
use std::io;


use super::tokio::io::{Readiness, Transport};
use super::tokio::proto::pipeline;

use super::irc::{Command, UserCommand};

pub struct Terminal<I: Read + TermRead, O: IntoRawMode> {
    in_stream: I,
    out_stream: RawTerminal<O>,
    read_buf: Vec<u8>,
}



impl<I: Read + TermRead, O: IntoRawMode> Terminal<I, O> {
    pub fn new(in_stream: I, out_stream: O) -> Terminal<I, O> {
        Terminal {
            in_stream: in_stream,
            out_stream: out_stream.into_raw_mode().unwrap(),
            read_buf: Vec::new(),
        }
    }
}


impl<I: Read, O: Write> Readiness for Terminal<I, O> {
    fn is_readable(&self) -> bool {
        true
    }

    fn is_writable(&self) -> bool {
        true
    }
}

pub type InFrame = pipeline::Frame<Command, io::Error>;
pub type OutFrame = pipeline::Frame<UserCommand, io::Error>;

impl<I: Read + TermRead, O: Write> Transport for Terminal<I, O> {
    type In = InFrame;
    type Out = OutFrame;

    fn read(&mut self) -> io::Result<Option<Self::Out>> {
        let mut buf = Vec::new();
        if let Ok(bytes) = self.in_stream.read(&mut buf) {
            if bytes > 0 {
                buf.truncate(bytes);
                let mut count = 0;
                for i in &*buf {
                    count += 1;
                    if *i == 0x10 {
                        break;
                    }
                }

                let remainder = buf.split_off(count);
                self.read_buf.append(&mut buf);
                if self.read_buf.len() > 0 {
                    let out_buf = String::from_utf8(self.read_buf.clone()).unwrap();

                    return Ok(Some(pipeline::Frame::Message(
                               UserCommand::Nick("MALIGO".to_string())
                           )));
                }
                self.read_buf = remainder;
            }
        }
        Ok(None)
    }

    fn write(&mut self, req: Self::In) -> io::Result<Option<()>>{
        match req {
            pipeline::Frame::Message(cmd) => {
                let _ = write!(self.out_stream, "{}", cmd.to_cmd());
            },
            _ => {}
        }
        Ok(None)
    }

    fn flush(&mut self) -> io::Result<Option<()>>{
        Ok(None)
    }
}
