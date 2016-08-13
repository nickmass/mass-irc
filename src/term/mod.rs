use super::termion::input::{TermRead};
use super::termion::raw::{IntoRawMode, RawTerminal};
use std::io::{Read, Write};
use std::io;


use super::tokio::io::{Readiness, Transport};
use super::tokio::proto::pipeline;

use super::irc::{Command, UserCommand};

pub struct Terminal<I: Read + TermRead, O: Write> {
    in_stream: I,
    out_stream: O,
    read_buf: Vec<u8>,
}



impl<I: Read + TermRead, O: Write> Terminal<I, O> {
    pub fn new(in_stream: I, out_stream: O) -> Terminal<I, O> {
        Terminal {
            in_stream: in_stream,
            out_stream: out_stream,
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

                return Ok(Some(pipeline::Frame::Message(
                            UserCommand::Nick("MALIGO".to_string())
                            )));
        if let Some(index) = self.read_buf.iter().position(|x| *x == b'\n') {
            let remainder = self.read_buf.split_off(index);
            debug!("Got Term");
            if self.read_buf.len() > 0 {
                let out_buf = String::from_utf8(self.read_buf.clone()).unwrap();

                return Ok(Some(pipeline::Frame::Message(
                            UserCommand::Nick("MALIGO".to_string())
                            )));
            }
            self.read_buf = remainder;
        }

        let mut buf = Vec::new();
        if let Ok(bytes) = self.in_stream.read_to_end(&mut buf) {
            if bytes > 0 {
                debug!("Got Term BYtes");
                buf.truncate(bytes);
                self.read_buf.append(&mut buf);
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
